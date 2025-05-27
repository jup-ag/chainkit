use bip39::{Language, Mnemonic, Seed, MnemonicType};
use bs58;
use hex;
use rust_decimal::{prelude::FromPrimitive, Decimal, Error};
use sha2::{Digest, Sha256};
use solana_program::{instruction::Instruction, native_token::LAMPORTS_PER_SOL};
use solana_sdk::{
    compute_budget::ComputeBudgetInstruction,
    hash,
    instruction::AccountMeta,
    message::VersionedMessage,
    pubkey::Pubkey,
    signature::{keypair_from_seed, keypair_from_seed_and_derivation_path, Keypair, Signature, Signer},
    signer::SignerError,
    transaction::{Transaction, VersionedTransaction},
    system_instruction,
    message::{Message}
};
use spl_memo::build_memo;
use spl_token::instruction::TokenInstruction;
use types::ExternalAddress;
use std::str::FromStr;

use super::types::*;
use crate::errors::*;
use crate::utils::{from_base64, to_base64};

mod jupiter_helpers;
use jupiter_helpers::mutate_transaction_slippage_bps;

mod priority_fee_helpers;
use priority_fee_helpers::{add_compute_unit_limit, add_compute_unit_price};

pub mod types;

pub struct Factory;

const TOKEN_2022_PROGRAM: &str = "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb";
const SPL_PROGRAM: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
const INVITE_ESCROW_PROGRAM: &str = "inv1tEtSwRMtM44tbvJGNiTxMvDfPVnX9StyqXfDfks";
const ALLOWED_PROGRAMS: [&str; 3] = [TOKEN_2022_PROGRAM, SPL_PROGRAM, INVITE_ESCROW_PROGRAM];

impl UtilsFactory for Factory {
    fn generate_mnemonic(
            &self,
            length: u32
        ) -> Result<MnemonicWords, KeyError> {

        // Ensure the mnemonic length is valid, for deriving we support only 12 and 24 words,
        // therefore for creating also gonna support only that
        let mnemonic_type = match length {
            12 => MnemonicType::Words12,
            24 => MnemonicType::Words24,
            _ => return Err(KeyError::InvalidMnemonic(
                "Only 12 or 24 word mnemonics are supported".to_string(),
             )),
        };

        // Generate the mnemonic
        let mnemonic = Mnemonic::new(mnemonic_type, Language::English);

        // Check if mnemonic generation succeeded
        if mnemonic.phrase().is_empty() {
            return Err(KeyError::Generic("Cannot create mnemonic".to_string()));
        }

        // Convert mnemonic phrase to Vec<String>
        let words: Vec<String> = mnemonic.phrase()
            .split_whitespace()
            .map(String::from)
            .collect();

        Ok(MnemonicWords { words })
    }
}

impl PrivateKeyFactory for Factory {
    fn derive(
        &self,
        mnemonic: MnemonicWords,
        passphrase: Option<&str>,
        derivation: Derivation,
    ) -> Result<Vec<DerivedPrivateKey>, KeyError> {
        if mnemonic.words.len() != 12 && mnemonic.words.len() != 24 {
            return Err(KeyError::InvalidMnemonic(
                "Only 12 or 24 word mnemonics are supported".to_string(),
            ));
        }
        let mnemonic_phrase = Mnemonic::from_phrase(&mnemonic.joined(), Language::English)
            .map_err(|e| KeyError::InvalidMnemonic(format!("Invalid Mnemonic: {e:?}")))?;

        let mut keys = Vec::new();

        let seed = Seed::new(&mnemonic_phrase, passphrase.unwrap_or(""));

        for (path_index, path_string) in derivation.paths_with_index().iter() {
            let path =
                solana_sdk::derivation_path::DerivationPath::from_absolute_path_str(path_string)
                    .map_err(KeyError::derivation)?;

            let keypair =
                keypair_from_seed_and_derivation_path(seed.as_bytes(), Some(path))
                    .map_err(|e| KeyError::Generic(format!("Invalid Keypair: {e:?}")))?;
            let key = DerivedPrivateKey {
                contents: keypair.to_base58_string(),
                public_key: ChainPublicKey {
                    contents: keypair.pubkey().to_string(),
                    chain: Blockchain::Solana,
                },
                index: *path_index,
                path: Some(path_string.clone()),
                path_type: Some(derivation.path)
            };

            keys.push(key)
        }

        Ok(keys)
    }

    fn derive_from_data(
        &self,
        data: &str,
    ) -> Result<DerivedPrivateKey, KeyError> {
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        let hashed_data = hasher.finalize();
        let seed: &[u8] = &hashed_data;

        let keypair =
                keypair_from_seed(seed)
                    .map_err(|e| KeyError::Generic(format!("Invalid Keypair: {e:?}")))?;
        let key = DerivedPrivateKey {
            contents: keypair.to_base58_string(),
            public_key: ChainPublicKey {
                contents: keypair.pubkey().to_string(),
                chain: Blockchain::Solana,
            },
            index: 0,
            path: None,
            path_type: None
        };

        Ok(key)
    }

    fn raw_private_key(&self, key: &str) -> Result<ChainPrivateKey, KeyError> {
        let key_bytes = if let Some(data) = crate::utils::parse_string_as_byte_array(key) {
            data
        } else if let Ok(data) = bs58::decode(&key).into_vec() {
            data
        } else if let Ok(data) = hex::decode(&key) {
            data
        } else {
            return Err(KeyError::PrivateKey(
                "Not a valid Base58, HEX encoded or array encoded key".to_string(),
            ));
        };
        let keypair = Keypair::from_bytes(&key_bytes).map_err(KeyError::keypair)?;

        // validate keypair. If a user accidentally mutates a PK (e.g. replaces a char)
        // then the key can be imported, and it can sign, but it can't send tx.
        // The validation of the signature below makes sure we know whether the key
        // is valid.
        let msg = &[0, 1, 2, 3, 4, 5, 6, 7];
        let sig = keypair.try_sign_message(msg).map_err(KeyError::keypair)?;
        let pk_bytes = keypair.try_pubkey().map_err(KeyError::keypair)?.to_bytes();
        let sig_valid = sig.verify(&pk_bytes, msg);

        if !sig_valid {
            return Err(KeyError::PrivateKey("Broken Private Key".into()));
        }

        Ok(ChainPrivateKey {
            contents: keypair.to_base58_string(),
            public_key: ChainPublicKey {
                contents: keypair.pubkey().to_string(),
                chain: Blockchain::Solana,
            },
        })
    }

    fn is_valid(&self, address: &str) -> bool {
        Pubkey::from_str(address).is_ok()
    }
}

impl TransactionFactory for Factory {
    fn send_transaction(
        &self,
        sender: ChainPublicKey,
        receiver: ChainPublicKey,
        amount: DecimalNumber,
        parameters: TransactionParameters,
    ) -> Result<String, TransactionError> {
        let from_pubkey: Pubkey =
            Pubkey::from_str(sender.contents.as_str()).map_err(TransactionError::keypair)?;

        let to_pubkey = &receiver.to_solana_pubkey()?;
        let amount = Decimal::from_str_exact(&amount.value)
            .map_err(|e| TransactionError::decimal(e, &amount))?;

        let decimal_lamports_per_sol = Decimal::from_u64(LAMPORTS_PER_SOL).ok_or_else(|| {
            TransactionError::parsing_failure(Error::ErrorString(
                "failed to parse Decimal from LAMPORTS_PER_SOL".to_string(),
            ))
        })?;

        let amount_lamports = amount
            .checked_mul(decimal_lamports_per_sol)
            .ok_or_else(|| {
                TransactionError::Generic("Failed to calculate amount in LAMPORTS".to_string())
            })?;

        let mut instructions: Vec<Instruction> = Vec::new();

        if let Some(unit_limit) = parameters.compute_budget_unit_limit() {
            let compute_budget_instruction =
                ComputeBudgetInstruction::set_compute_unit_limit(unit_limit);
            instructions.push(compute_budget_instruction);
        }
        if let Some(unit_price) = parameters.compute_budget_unit_price() {
            let compute_budget_instruction =
                ComputeBudgetInstruction::set_compute_unit_price(unit_price);
            instructions.push(compute_budget_instruction);
        }

        if let Some(memo) = parameters.memo() {
            instructions.push(build_memo(memo.as_bytes(), &[&from_pubkey]));
        };

        let mut instruction = solana_sdk::system_instruction::transfer(
            &from_pubkey,
            to_pubkey,
            amount_lamports
                .try_into()
                .map_err(TransactionError::parsing_failure)?,
        );
        for reference in parameters.references().iter() {
            let pubkey = Pubkey::from_str(reference).map_err(TransactionError::public_key)?;
            instruction
                .accounts
                .push(AccountMeta::new_readonly(pubkey, false));
        }
        instructions.push(instruction);

        let transaction = Transaction::new_with_payer(&instructions, Some(&from_pubkey));
        let mut versioned_transaction = VersionedTransaction::from(transaction);

        if let Some(external_address) = parameters.external_address() {
            let recent_blockhash = external_address
                .recent_blockhash
                .parse::<hash::Hash>()
                .map_err(TransactionError::parsing_failure)?;

            versioned_transaction.message.set_recent_blockhash(recent_blockhash);
        }

        let serialized_tx =
            bincode::serialize(&versioned_transaction).map_err(TransactionError::parsing_failure)?;

        Ok(to_base64(serialized_tx))
    }

    fn token_transaction(
        &self,
        destination: TokenDestination,
        owner: ChainPublicKey,
        token: ChainPublicKey,
        kind: TransactionKind,
        parameters: TransactionParameters,
    ) -> Result<String, TransactionError> {
        let owner_program = if let Some(program) = parameters.owner_program() {
            program.parse().map_err(TransactionError::public_key)?
        } else {
            spl_token::id()
        };
        let decimals = parameters.decimals().unwrap_or(0);

        let owner_pubkey =
            Pubkey::from_str(&owner.contents.as_str()).map_err(TransactionError::public_key)?;
        let mint_pubkey = &token.to_solana_pubkey()?;
        let source_token_account =
            associated_token_address_2022(&owner_pubkey, &owner_program, mint_pubkey);
        let references: Result<Vec<_>, _> = parameters
            .references()
            .iter()
            .map(|r| Pubkey::from_str(r).map_err(TransactionError::public_key))
            .collect();
        // we get the correct source token account because source might be inaccurate

        match kind {
            TransactionKind::Token {
                amount,
                close_account,
            } => match destination {
                TokenDestination::Account {
                    transfer_destination,
                } => {
                    let destination = Pubkey::from_str(&transfer_destination)
                        .map_err(TransactionError::public_key)?;

                    let mut instructions: Vec<Instruction> = Vec::new();

                    if let Some(unit_limit) = parameters.compute_budget_unit_limit() {
                        let compute_budget_instruction =
                            ComputeBudgetInstruction::set_compute_unit_limit(unit_limit);
                        instructions.push(compute_budget_instruction);
                    }
                    if let Some(unit_price) = parameters.compute_budget_unit_price() {
                        let compute_budget_instruction =
                            ComputeBudgetInstruction::set_compute_unit_price(unit_price);
                        instructions.push(compute_budget_instruction);
                    }

                    if let Some(memo) = parameters.memo() {
                        instructions.push(build_memo(memo.as_bytes(), &[&owner_pubkey]));
                    };
                    let transfer_instruction = transfer_2022(
                        &owner_program,
                        &source_token_account,
                        &destination,
                        &owner_pubkey,
                        &[&owner_pubkey],
                        &references?,
                        amount.to_u64()?,
                        decimals,
                        mint_pubkey,
                    )
                    .map_err(TransactionError::instruction_error)?;
                    instructions.push(transfer_instruction);

                    if close_account {
                        instructions.push(
                            close_token_account(
                                &owner_program,
                                &source_token_account,
                                &owner_pubkey,
                                &owner_pubkey,
                                &[],
                            )
                            .map_err(TransactionError::instruction_error)?,
                        );
                    }

                    let transaction = Transaction::new_with_payer(&instructions, Some(&owner_pubkey));
                    let mut versioned_transaction = VersionedTransaction::from(transaction);

                    if let Some(external_address) = parameters.external_address() {
                        let recent_blockhash = external_address
                            .recent_blockhash
                            .parse::<hash::Hash>()
                            .map_err(TransactionError::parsing_failure)?;

                        versioned_transaction.message.set_recent_blockhash(recent_blockhash);
                    }

                    let serialized_tx = bincode::serialize(&versioned_transaction)
                        .map_err(TransactionError::parsing_failure)?;
                    Ok(to_base64(serialized_tx))
                }
                TokenDestination::Wallet { public_key } => {
                    let receiver_pubkey = &public_key.to_solana_pubkey()?;
                    let destination =
                        associated_token_address_2022(receiver_pubkey, &owner_program, mint_pubkey);

                    let mut instructions: Vec<Instruction> = Vec::new();

                    if let Some(unit_limit) = parameters.compute_budget_unit_limit() {
                        let compute_budget_instruction =
                            ComputeBudgetInstruction::set_compute_unit_limit(unit_limit);
                        instructions.push(compute_budget_instruction);
                    }
                    if let Some(unit_price) = parameters.compute_budget_unit_price() {
                        let compute_budget_instruction =
                            ComputeBudgetInstruction::set_compute_unit_price(unit_price);
                        instructions.push(compute_budget_instruction);
                    }

                    let create_instruction = create_associated_token_account_2022(
                        &owner_pubkey,
                        receiver_pubkey,
                        mint_pubkey,
                        owner_program,
                    );
                    instructions.push(create_instruction);

                    if let Some(memo) = parameters.memo() {
                        instructions.push(build_memo(memo.as_bytes(), &[&owner_pubkey]));
                    };

                    let transfer_instruction = transfer_2022(
                        &owner_program,
                        &source_token_account,
                        &destination,
                        &owner_pubkey,
                        &[&owner_pubkey],
                        &references?,
                        amount.to_u64()?,
                        decimals,
                        mint_pubkey,
                    )
                    .map_err(TransactionError::instruction_error)?;
                    instructions.push(transfer_instruction);

                    if close_account {
                        instructions.push(
                            close_token_account(
                                &owner_program,
                                &source_token_account,
                                &owner_pubkey,
                                &owner_pubkey,
                                &[],
                            )
                            .map_err(TransactionError::instruction_error)?,
                        );
                    }

                    let transaction = Transaction::new_with_payer(&instructions, Some(&owner_pubkey));
                    let mut versioned_transaction = VersionedTransaction::from(transaction);

                    if let Some(external_address) = parameters.external_address() {
                        let recent_blockhash = external_address
                            .recent_blockhash
                            .parse::<hash::Hash>()
                            .map_err(TransactionError::parsing_failure)?;

                        versioned_transaction.message.set_recent_blockhash(recent_blockhash);
                    }

                    let serialized_tx = bincode::serialize(&versioned_transaction)
                        .map_err(TransactionError::parsing_failure)?;
                    Ok(to_base64(serialized_tx))
                }
            },
            _ => Err(TransactionError::Generic(
                "Unsupported Transaction kind on Solana".to_string(),
            )),
        }
    }

    fn sign_transaction(
        &self,
        transaction: String,
        signers: Vec<ChainPrivateKey>,
        parameters: Option<TransactionParameters>,
    ) -> Result<ChainTransaction, TransactionError> {
        if signers.is_empty() {
            return Err(TransactionError::SignerMissing);
        }

        let transaction_bytes =
            from_base64(&transaction).map_err(TransactionError::parsing_failure)?;

        let (
            external_address,
            transaction_type,
            swap_slippage_bps,
            compute_budget_unit_limit,
            compute_budget_unit_price,
        ) = match parameters {
            Some(TransactionParameters::Solana {
                external_address,
                transaction_type,
                owner_program: _,
                decimals: _,
                memo: _,
                references: _,
                swap_slippage_bps,
                compute_budget_unit_limit,
                compute_budget_unit_price,
            }) => (
                external_address,
                transaction_type,
                swap_slippage_bps,
                compute_budget_unit_limit,
                compute_budget_unit_price,
            ),
            None => return Err(TransactionError::parameters("No parameters were provided")),
        };

        let signer_keypairs = signers
            .iter()
            .map(|signer| signer.signer_keypair().map_err(TransactionError::keypair))
            .collect::<Result<Vec<Keypair>, TransactionError>>()?;
        let signer_keypairs: Vec<&Keypair> = signer_keypairs.iter().collect();

        let mut transaction: VersionedTransaction = match transaction_type {
            SolanaTransactionType::Legacy => {
                let transaction: Transaction = bincode::deserialize(&transaction_bytes)
                    .map_err(TransactionError::parsing_failure)?;

                VersionedTransaction::try_from(transaction)
                    .map_err(TransactionError::parsing_failure)?
            }
            SolanaTransactionType::Versioned => bincode::deserialize(&transaction_bytes)
                .map_err(TransactionError::parsing_failure)?,
        };

        if let Some(swap_slippage_bps) = swap_slippage_bps {
            let _ = mutate_transaction_slippage_bps(&mut transaction, swap_slippage_bps);
        }

        if let Some(compute_budget_unit_limit) = compute_budget_unit_limit {
            let _ = add_compute_unit_limit(&mut transaction, compute_budget_unit_limit);
        }

        if let Some(compute_budget_unit_price) = compute_budget_unit_price {
            let _ = add_compute_unit_price(&mut transaction, compute_budget_unit_price);
        }

        if let Some(external_address) = &external_address {
            let recent_blockhash = external_address
                .recent_blockhash
                .parse::<hash::Hash>()
                .map_err(TransactionError::parsing_failure)?;

            if transaction
                .signatures
                .iter()
                .all(|&sig| sig == Signature::from([0u8; 64]) || sig == Signature::from([1u8; 64]))
            {
                transaction.message.set_recent_blockhash(recent_blockhash);
            }
        }

        let result: ChainTransaction =
            match VersionedTransaction::try_new(transaction.message, &signer_keypairs) {
                Ok(versioned_tx) => match bincode::serialize(&versioned_tx) {
                    Ok(serialized_tx) => ChainTransaction {
                        tx: to_base64(serialized_tx),
                        signers: signers
                            .iter()
                            .map(|signer| signer.public_key.clone())
                            .collect(),
                            accounts: versioned_tx
                                .message
                                .static_account_keys()
                                .iter()
                                .map(|pubkey| ChainPublicKey {
                                    contents: bs58::encode(pubkey).into_string(),
                                    chain: Blockchain::Solana,
                                })
                                .collect(),
                        full_signature: calculate_signature(&versioned_tx.signatures),
                        signatures: signatures_to_base58(&versioned_tx.signatures),
                        instruction_programs: get_instruction_programs(versioned_tx.message),
                    },
                    Err(error) => return Err(TransactionError::parsing_failure(error)),
                },
                Err(error) => match error {
                    // Some Dapps pass TXs that are partially signed
                    // Other Dapps expect partially signed TXs (which they sign upon receiving)
                    // in those cases the solana-sdk wants to throw SignerError::NotEnoughSigners
                    // we catch those errors and do our own partial signing
                    SignerError::NotEnoughSigners => {
                        let mut transaction: VersionedTransaction =
                            bincode::deserialize(&transaction_bytes)
                                .map_err(TransactionError::parsing_failure)?;

                        let account_keys = transaction.message.static_account_keys();
                        let signer_pubkeys = signer_keypairs
                            .iter()
                            .map(|kp| kp.pubkey())
                            .collect::<Vec<Pubkey>>();
                        let signer_position_and_signature = account_keys
                            .iter()
                            .enumerate()
                            .filter(|(_, acc)| signer_pubkeys.contains(acc))
                            .filter_map(|(i, acc)| {
                                if let Some(signer_keypair) = signer_keypairs
                                    .clone()
                                    .into_iter()
                                    .find(|kp| kp.pubkey() == *acc)
                                {
                                    // index is important for inserting sig at expected position
                                    let signature = signer_keypair
                                        .sign_message(&transaction.message.serialize());
                                    Some((i, signature))
                                } else {
                                    None
                                }
                            })
                            .collect::<Vec<(usize, Signature)>>();

                        for (signature_index, signature) in signer_position_and_signature {
                            if let Some(current_signature) =
                                transaction.signatures.get(signature_index)
                            {
                                // default signature = not signed
                                let is_default_signature = current_signature.to_string()
                                    == Signature::default().to_string();
                                if is_default_signature {
                                    transaction.signatures[signature_index] = signature;
                                }
                            }
                        }

                        let serialized_tx = bincode::serialize(&transaction);

                        match serialized_tx {
                            Ok(serialized_tx) => ChainTransaction {
                                tx: to_base64(serialized_tx),
                                signers: signers
                                    .iter()
                                    .map(|signer| signer.public_key.clone())
                                    .collect(),
                                    accounts: transaction
                                        .message
                                        .static_account_keys()
                                        .iter()
                                        .map(|pubkey| ChainPublicKey {
                                            contents: bs58::encode(pubkey).into_string(),
                                            chain: Blockchain::Solana,
                                        })
                                        .collect(),
                                full_signature: calculate_signature(&transaction.signatures),
                                signatures: signatures_to_base58(&transaction.signatures),
                                instruction_programs: get_instruction_programs(transaction.message),
                            },
                            Err(error) => return Err(TransactionError::parsing_failure(error)),
                        }
                    }
                    _ => return Err(TransactionError::parsing_failure(error)),
                },
            };

        Ok(result)
    }

    fn sign_message(
        &self,
        message: String,
        signers: Vec<ChainPrivateKey>,
    ) -> Result<String, TransactionError> {
        match signers.len() {
            1 => (),
            0 => return Err(TransactionError::SignerMissing),
            _ => return Err(TransactionError::MultipleSigners),
        }
        let message_bytes = from_base64(&message).map_err(TransactionError::parsing_failure)?;

        // Check if the message is a Solana transaction or transaction message
        if bincode::deserialize::<Transaction>(&message_bytes).is_ok()
            || bincode::deserialize::<VersionedTransaction>(&message_bytes).is_ok()
            || bincode::deserialize::<Message>(&message_bytes).is_ok()
            || bincode::deserialize::<VersionedMessage>(&message_bytes).is_ok()
        {
            return Err(TransactionError::SignMsgError(
                "You cannot sign solana transactions using sign_message".to_string(),
            ));
        }

        let signer_keypair = &signers[0].signer_keypair()?;
        let signature = signer_keypair
            .try_sign_message(&message_bytes)
            .map_err(TransactionError::instruction_error)?;

        let signature_bytes =
            bincode::serialize(&signature).map_err(TransactionError::parsing_failure)?;

        Ok(to_base64(signature_bytes))
    }

    fn sign_typed_data(
        &self,
        _typed_data: String,
        _signers: Vec<ChainPrivateKey>,
    ) -> Result<String, TransactionError> {
        Err(TransactionError::Generic("Not applicable".into()))
    }

    fn modify_transaction(
        &self,
        _transaction: String,
        _owner: ChainPrivateKey,
        _parameters: TransactionParameters,
    ) -> Result<String, TransactionError> {
        todo!()
    }

    fn parse_transaction(
        &self,
        _transaction: String,
    ) -> Result<ParsedTransaction, TransactionError> {
        todo!()
    }

    fn get_associated_token_address(
        &self,
        wallet_address: String,
        owner_program: String,
        token_mint_address: String,
    ) -> Result<ChainPublicKey, TransactionError> {
        let wallet_address_pubkey = Pubkey::from_str(&wallet_address).map_err(TransactionError::public_key)?;
        let owner_program_pubkey = Pubkey::from_str(&owner_program).map_err(TransactionError::public_key)?;
        let token_mint_address_pubkey = Pubkey::from_str(&token_mint_address).map_err(TransactionError::public_key)?;

        if !is_program_allowed(&owner_program_pubkey) {
            return Err(TransactionError::InstructionError(
                "wrong token program".to_string(),
            ));
        }

        let generated_associated_token_account = associated_token_address_2022(
            &wallet_address_pubkey,
            &owner_program_pubkey,
            &token_mint_address_pubkey
        );

        Ok(ChainPublicKey {
            contents: generated_associated_token_account.to_owned().to_string(),
            chain: Blockchain::Solana,
        })
    }

    fn get_program_address(
        &self,
        seeds: Vec<String>,
        program: String,
    ) -> Result<ChainPublicKey, TransactionError> {
        // Convert the program (String) into a Solana PublicKey
        let program_pub_key = Pubkey::from_str(&program).map_err(TransactionError::public_key)?;

        if (!is_program_allowed(&program_pub_key)) {
            return Err(TransactionError::InstructionError(
                "wrong token program".to_string(),
            ));
        }

        // We will need to work with owned `Vec<u8>` instead of references to slices `&[u8]`
        let mut seeds_u8: Vec<Vec<u8>> = Vec::new();

        // Iterate over the seeds, converting valid Solana public keys to `Pubkey`, and other strings to `Vec<u8>`
        for seed in seeds {
            // Try to parse the seed as a Solana public key
            if let Ok(pubkey) = Pubkey::from_str(&seed) {
                // If it is a valid Solana public key, push its bytes (owned `Vec<u8>`)
                seeds_u8.push(pubkey.to_bytes().to_vec()); // `to_vec()` converts the `[u8; 32]` to `Vec<u8>`
            } else {
                // If it's not a valid `Pubkey`, treat it as a string and convert it to `Vec<u8>`
                seeds_u8.push(seed.as_bytes().to_vec());
            }
        }

        // Convert `Vec<Vec<u8>>` into `Vec<&[u8]>` for the `find_program_address` call
        let seeds_u8_refs: Vec<&[u8]> = seeds_u8.iter().map(|v| v.as_slice()).collect();

        // Generate the program address (PDA)
        let program_address = Pubkey::find_program_address(
            &seeds_u8_refs,
            &program_pub_key,
        ).0;

        Ok(ChainPublicKey {
            contents: program_address.to_owned().to_string(),
            chain: Blockchain::Solana,
        })
    }

    fn get_message(&self, transaction: String) -> Result<String, TransactionError> {
        let transaction_bytes = from_base64(&transaction).map_err(TransactionError::parsing_failure)?;

        if let Ok(versioned_tx) = bincode::deserialize::<VersionedTransaction>(&transaction_bytes) {
            let message_bytes = bincode::serialize(&versioned_tx.message).map_err(TransactionError::parsing_failure)?;
            return Ok(to_base64(message_bytes));
        }

        if let Ok(tx) = bincode::deserialize::<Transaction>(&transaction_bytes) {
            let message_bytes = bincode::serialize(&tx.message).map_err(TransactionError::parsing_failure)?;
            return Ok(to_base64(message_bytes));
        }

        Err(TransactionError::parsing_failure(TransactionError::parsing_failure(Error::ErrorString(
            "Failed to parse transaction".to_string(),
        ))))
    }

    fn get_transaction(&self, message: String) -> Result<String, TransactionError> {
        let message_bytes = from_base64(&message).map_err(TransactionError::parsing_failure)?;

        // Try VersionedMessage first
        if let Ok(versioned_msg) = bincode::deserialize::<VersionedMessage>(&message_bytes) {
            let tx = VersionedTransaction {
                signatures: vec![Signature::default()], // Placeholder signature
                message: versioned_msg,
            };
            let tx_bytes = bincode::serialize(&tx).map_err(TransactionError::parsing_failure)?;
            return Ok(to_base64(tx_bytes));
        }

        // Try Legacy Message
        if let Ok(message) = bincode::deserialize::<Message>(&message_bytes) {
            let tx = Transaction {
                signatures: vec![Signature::default()], // Placeholder signature
                message,
            };
            let tx_bytes = bincode::serialize(&tx).map_err(TransactionError::parsing_failure)?;
            return Ok(to_base64(tx_bytes));
        }

        Err(TransactionError::parsing_failure(Error::ErrorString(
            "Failed to parse message".to_string(),
        )))
    }

    fn append_signature_to_transaction(&self, signer: String, signature: String, transaction: String) -> Result<String, TransactionError> {
        let transaction_bytes = from_base64(&transaction).map_err(TransactionError::parsing_failure)?;
        let sig_bytes = bs58::decode(&signature).into_vec().map_err(TransactionError::parsing_failure)?;
        let signature = Signature::try_from(sig_bytes.as_slice()).map_err(TransactionError::parsing_failure)?;
        let pubkey = Pubkey::from_str(&signer).map_err(TransactionError::public_key)?;

        if let Ok(mut versioned_tx) = bincode::deserialize::<VersionedTransaction>(&transaction_bytes) {
            // Find position of signer in account keys
            let account_keys = versioned_tx.message.static_account_keys();
            let signer_position = account_keys.iter().position(|key| *key == pubkey).ok_or_else(|| TransactionError::Generic("Signer not found in account keys".to_string()))?;

            // Insert signature at correct position
            if signer_position < versioned_tx.signatures.len() {
                versioned_tx.signatures[signer_position] = signature;
                let serialized_tx = bincode::serialize(&versioned_tx).map_err(TransactionError::parsing_failure)?;
                return Ok(to_base64(serialized_tx));
            }
        }

        if let Ok(mut tx) = bincode::deserialize::<Transaction>(&transaction_bytes) {
            // Find position of signer in account keys
            let account_keys = &tx.message.account_keys;
            let signer_position = account_keys.iter().position(|key| *key == pubkey).ok_or_else(|| TransactionError::Generic("Signer not found in account keys".to_string()))?;

            // Insert signature at correct position
            if signer_position < tx.signatures.len() {
                tx.signatures[signer_position] = signature;
                let serialized_tx = bincode::serialize(&tx).map_err(TransactionError::parsing_failure)?;
                return Ok(to_base64(serialized_tx));
            }
        }

        Err(TransactionError::Generic("Failed to append signature to transaction".to_string()))
    }
}


fn calculate_signature(signatures: &Vec<Signature>) -> Option<String> {
    if signatures.is_empty() {
        return None;
    }

    let concatenated: Vec<u8> = signatures
        .iter()
        .flat_map(|sig| sig.as_ref().to_vec())
        .collect();

    let encoded = bs58::encode(concatenated).into_string();

    Some(encoded)
}

fn signatures_to_base58(signatures: &Vec<Signature>) -> Option<Vec<String>> {
    if signatures.is_empty() {
        return None;
    }

    let encoded_signatures = signatures
        .into_iter()
        .map(|sig| bs58::encode(sig.as_ref().to_vec()).into_string()) // Convert each Signature to a Base58 String
        .collect();

    Some(encoded_signatures)
}

fn get_instruction_programs(message: VersionedMessage) -> Vec<String> {
    let program_id_indexes: Vec<u8> = message
        .instructions()
        .iter()
        .map(|i| i.program_id_index)
        .collect();

    let account_keys = message.static_account_keys();

    program_id_indexes
        .iter()
        .map(|index| account_keys[*index as usize])
        .map(|pubkey| bs58::encode(pubkey).into_string())
        .collect()
}

impl TransactionParameters {
    fn decimals(&self) -> Option<u8> {
        let TransactionParameters::Solana { decimals, .. } = &self;
        *decimals
    }
    fn owner_program(&self) -> Option<String> {
        let TransactionParameters::Solana { owner_program, .. } = &self;
        owner_program.clone()
    }
    fn memo(&self) -> Option<String> {
        let TransactionParameters::Solana { memo, .. } = &self;
        memo.clone()
    }
    fn references(&self) -> Vec<String> {
        let TransactionParameters::Solana { references, .. } = &self;
        references.clone()
    }
    fn compute_budget_unit_price(&self) -> Option<u64> {
        let TransactionParameters::Solana {
            compute_budget_unit_price,
            ..
        } = &self;
        *compute_budget_unit_price
    }
    fn compute_budget_unit_limit(&self) -> Option<u32> {
        let TransactionParameters::Solana {
            compute_budget_unit_limit,
            ..
        } = &self;
        *compute_budget_unit_limit
    }
    fn external_address(&self) -> Option<ExternalAddress> {
        let TransactionParameters::Solana { external_address, .. } = &self;
        external_address.clone()
    }
}

impl ChainPublicKey {
    fn to_solana_pubkey(&self) -> Result<Pubkey, TransactionError> {
        match Pubkey::from_str(&self.contents) {
            Ok(pubkey) => Ok(pubkey),
            Err(e) => Err(TransactionError::public_key(e)),
        }
    }
}

impl ChainPrivateKey {
    fn signer_keypair(&self) -> Result<Keypair, TransactionError> {
        let keypair_bytes: Vec<u8> = bs58::decode(&self.contents)
            .into_vec()
            .map_err(TransactionError::parsing_failure)?;
        let from_keypair =
            Keypair::from_bytes(&keypair_bytes).map_err(TransactionError::keypair)?;

        Ok(from_keypair)
    }
}

impl DecimalNumber {
    fn to_u64(&self) -> Result<u64, TransactionError> {
        let amount =
            Decimal::from_str_exact(&self.value).map_err(|e| TransactionError::decimal(e, self))?;

        let u64: u64 = amount
            .try_into()
            .map_err(TransactionError::parsing_failure)?;

        Ok(u64)
    }
}

fn associated_token_address_2022(
    wallet_address: &Pubkey,
    owner_program: &Pubkey,
    spl_token_mint_address: &Pubkey,
) -> Pubkey {
    let associated_program_id = &spl_associated_token_account::id();
    Pubkey::find_program_address(
        &[
            &wallet_address.to_bytes(),
            &owner_program.to_bytes(),
            &spl_token_mint_address.to_bytes(),
        ],
        associated_program_id,
    )
    .0
}

fn create_associated_token_account_2022(
    funding_address: &Pubkey,
    wallet_address: &Pubkey,
    spl_token_mint_address: &Pubkey,
    owner_program: Pubkey,
) -> Instruction {
    let associated_account_address =
        associated_token_address_2022(wallet_address, &owner_program, spl_token_mint_address);

    Instruction {
        program_id: spl_associated_token_account::id(),
        accounts: vec![
            AccountMeta::new(*funding_address, true),
            AccountMeta::new(associated_account_address, false),
            AccountMeta::new_readonly(*wallet_address, false),
            AccountMeta::new_readonly(*spl_token_mint_address, false),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
            AccountMeta::new_readonly(owner_program, false),
        ],
        data: vec![0], // would like to use instruction_data.try_to_vec() here, but the trait is private and this is valid
    }
}

/// Creates a `Transfer` with new tokenz 2022 program also as option instruction.
#[allow(clippy::too_many_arguments)]
pub fn transfer_2022(
    token_program_id: &Pubkey,
    source_pubkey: &Pubkey,
    destination_pubkey: &Pubkey,
    authority_pubkey: &Pubkey,
    signer_pubkeys: &[&Pubkey],
    references: &[Pubkey],
    amount: u64,
    decimals: u8,
    mint_pubkey: &Pubkey,
) -> Result<Instruction, TransactionError> {
    if !is_program_allowed(token_program_id) {
        return Err(TransactionError::InstructionError(
            "wrong token program".to_string(),
        ));
    }

    let data = TokenInstruction::TransferChecked { amount, decimals }.pack();

    let mut accounts = Vec::with_capacity(4 + signer_pubkeys.len() + references.len());
    accounts.push(AccountMeta::new(*source_pubkey, false));
    accounts.push(AccountMeta::new_readonly(*mint_pubkey, false));
    accounts.push(AccountMeta::new(*destination_pubkey, false));
    accounts.push(AccountMeta::new_readonly(
        *authority_pubkey,
        signer_pubkeys.is_empty(),
    ));
    for signer_pubkey in signer_pubkeys.iter() {
        accounts.push(AccountMeta::new_readonly(**signer_pubkey, true));
    }
    for reference in references.iter() {
        accounts.push(AccountMeta::new_readonly(*reference, false));
    }

    Ok(Instruction {
        program_id: *token_program_id,
        accounts,
        data,
    })
}

pub fn close_token_account(
    token_program_id: &Pubkey,
    account_pubkey: &Pubkey,
    owner_pubkey: &Pubkey,
    destination_pubkey: &Pubkey,
    signer_pubkeys: &[&Pubkey],
) -> Result<Instruction, TransactionError> {
    if !is_program_allowed(token_program_id) {
        return Err(TransactionError::InstructionError(
            "wrong token program".to_string(),
        ));
    }

    let data = TokenInstruction::CloseAccount.pack();

    let mut accounts = Vec::with_capacity(3 + signer_pubkeys.len());
    accounts.push(AccountMeta::new(*account_pubkey, false));
    accounts.push(AccountMeta::new(*destination_pubkey, false));
    accounts.push(AccountMeta::new_readonly(
        *owner_pubkey,
        signer_pubkeys.is_empty(),
    ));

    for signer_pubkey in signer_pubkeys.iter() {
        accounts.push(AccountMeta::new_readonly(**signer_pubkey, true));
    }

    Ok(Instruction {
        program_id: *token_program_id,
        accounts: accounts,
        data,
    })
}

/// Checks that the supplied program ID is the correct one for SPL-token / Token2022
fn is_program_allowed(spl_token_program_id: &Pubkey) -> bool {
    let to_compare = spl_token_program_id.to_string();
    ALLOWED_PROGRAMS.contains(&to_compare.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solana::types::ExternalAddress;
    use spl_associated_token_account::instruction::create_associated_token_account;
    use std::collections::HashSet;

    #[test]
    fn test_generate_mnemonic_12_words() {
        let result = Factory.generate_mnemonic(12);

        assert!(result.is_ok(), "Expected mnemonic generation to succeed");

        let mnemonic = result.unwrap();
        assert_eq!(mnemonic.words.len(), 12, "Expected 12 words in mnemonic");
    }

    #[test]
    fn test_generate_mnemonic_24_words() {
        let result = Factory.generate_mnemonic(24);

        assert!(result.is_ok(), "Expected mnemonic generation to succeed");

        let mnemonic = result.unwrap();
        assert_eq!(mnemonic.words.len(), 24, "Expected 24 words in mnemonic");
    }

    #[test]
    fn test_generate_mnemonic_invalid_length() {
        let result = Factory.generate_mnemonic(18);

        assert!(result.is_err(), "Expected error for invalid mnemonic length");

        if let Err(KeyError::InvalidMnemonic(msg)) = result {
            assert_eq!(msg, "Only 12 or 24 word mnemonics are supported");
        } else {
            panic!("Expected InvalidMnemonic error");
        }
    }

    #[test]
    fn test_generate_mnemonic_uniqueness() {
        let mut mnemonics = HashSet::new();

        for _ in 0..10 {
            let result = Factory.generate_mnemonic(12).unwrap();
            let phrase = result.words.join(" ");
            mnemonics.insert(phrase);
        }

        assert_eq!(mnemonics.len(), 10, "Expected 10 unique mnemonics");
    }

    #[test]
    fn test_bip44_root_derivation() {
        let mnemonic = MnemonicWords::from_str(
            "miracle pizza supply useful steak border same again youth silver access hundred",
        )
        .unwrap();

        let derivation = Derivation {
            start: 0,
            count: 1,
            path: DerivationPath::Bip44Root
        };

        let sender = Factory.derive(mnemonic, None, derivation).unwrap();

        let key = ChainPrivateKey {
            contents: sender[0].contents.to_owned(),
            public_key: ChainPublicKey {
                contents: sender[0].public_key.contents.to_owned(),
                chain: Blockchain::Solana,
            },
        };

        assert_eq!(
            key.public_key.contents,
            "9nNwJNeJnQmduBZZzYP717LRF8ExHT4GAa5Y6TktWgQq"
        );
    }

    #[test]
    fn create_2022_is_valid() {
        let m = create_associated_token_account(
            &spl_associated_token_account::id(),
            &spl_associated_token_account::id(),
            &spl_associated_token_account::id(),
        );

        let n = create_associated_token_account_2022(
            &spl_associated_token_account::id(),
            &spl_associated_token_account::id(),
            &spl_associated_token_account::id(),
            spl_token::id(),
        );
        assert_eq!(m, n);
    }

    fn generate_key_from_mnemonic(mnemonic: &str) -> ChainPrivateKey {
        let mnemonic = MnemonicWords::from_str(mnemonic).unwrap();

        let derivation = Derivation {
            start: 0,
            count: 1,
            path: DerivationPath::Bip44Change
        };

        let sender = Factory.derive(mnemonic, None, derivation).unwrap();

        ChainPrivateKey {
            contents: sender[0].contents.to_owned(),
            public_key: ChainPublicKey {
                contents: sender[0].public_key.contents.to_owned(),
                chain: Blockchain::Solana,
            },
        }
    }
    #[test]
    fn test_valid_pubkey() {
        assert!(Factory.is_valid("ASFCGfuYZPrxFCcYRfwkQACv73dsfG9JLnA3eUuDg1bL"))
    }

    #[test]
    fn test_derivation_12() {
        let mnemonic = MnemonicWords::from_str(
            "miracle pizza supply useful steak border same again youth silver access hundred",
        )
        .unwrap();

        let derivation = Derivation {
            start: 0,
            count: 2,
            path: DerivationPath::Bip44Change
        };

        let output = Factory.derive(mnemonic, None, derivation).unwrap();

        assert_eq!(
            &output[0].public_key.contents,
            "HnXJX1Bvps8piQwDYEYC6oea9GEkvQvahvRj3c97X9xr"
        );

        // Tests against swap-and-earn wallet
        let mnemonic = MnemonicWords::from_str(
            "ski seven shuffle amazing tooth net useful asthma drive crystal solar glare",
        )
        .unwrap();

        let derivation = Derivation {
            start: 0,
            count: 5,
            path: DerivationPath::Bip44Change
        };

        let output = Factory.derive(mnemonic, None, derivation).unwrap();

        let derived_pubkeys: Vec<(u32, String)> = output
            .iter()
            .map(|kp| (kp.index, kp.public_key.contents.to_owned()))
            .collect();

        assert_eq!(
            derived_pubkeys,
            vec![
                (0 as u32, "F7xVyQuLzvyUKbMQyrBHaqYGCzHWpmsocn8b7oRUyeC5".to_string()),
                (1 as u32, "DdSeC77Fih7CeVmJLw6FPv8pVPzyjpMfUPGFXP5RZ7uF".to_string()),
                (2 as u32, "HH5kWPVZXZSQPDSTb5TWrfnn3hbCuz3wHHgQ9Snrs5Hj".to_string()),
                (3 as u32, "54YcaDwtMN2grT2qrHKHbs6eFKY4mA7uof6zqq6YKAuY".to_string()),
                (4 as u32, "FCXWQT4Yx5AxV2Y5zr5PNdaK5Fi54RTT1cD1Z1nsQNoa".to_string())
            ]
        );

        // Tests deriving specific wallet
        let mnemonic = MnemonicWords::from_str(
            "ski seven shuffle amazing tooth net useful asthma drive crystal solar glare",
        )
        .unwrap();

        let derivation = Derivation {
            start: 4,
            count: 1,
            path: DerivationPath::Bip44Change
        };

        let output = Factory.derive(mnemonic, None, derivation).unwrap();

        let derived_pubkeys: Vec<(u32, String)> = output
            .iter()
            .map(|kp| (kp.index, kp.public_key.contents.to_owned()))
            .collect();

        assert_eq!(
            derived_pubkeys,
            vec![
                (4 as u32, "FCXWQT4Yx5AxV2Y5zr5PNdaK5Fi54RTT1cD1Z1nsQNoa".to_string())
            ]
        )
    }

    #[test]
    fn test_derivation_24_sollet_phantom() {
        let mnemonic = MnemonicWords::from_str(
           "avoid cement buddy stay nasty erosion parade fog limb marine season media staff lady torch trust sunny pattern odor harsh lamp bounce van glue"
        )
        .unwrap();

        let derivation = Derivation {
            start: 0,
            count: 2,
            path: DerivationPath::Bip44Change
        };

        let output = Factory.derive(mnemonic, None, derivation).unwrap();

        assert_eq!(
            &output[0].contents,
            "5bxuASQJNxBHicjXBvYmu1VfaAiydsdBxur8MdbBrgjbzWRY2u5PBoFRb3yR85eLr3nafvkb5xQKDxC64ow2vyBP"
        );
        assert_eq!(
            &output[0].public_key.contents,
            "BnYdjb9nS4N4TRkbW984G82pL8FuW5LYLGqTD737T8cy"
        )
    }

    #[test]
    fn test_derivation_24() {
        let mnemonic = MnemonicWords::from_str(
            "budget resource fluid mutual ankle salt demise long burst sting doctor ozone risk magic wrap clap post pole jungle great update air interest abandon"
        )
        .unwrap();

        let derivation = Derivation {
            start: 0,
            count: 2,
            path: DerivationPath::Bip44Change
        };

        let output = Factory.derive(mnemonic, None, derivation).unwrap();

        assert_eq!(
            &output[0].contents,
            "4yVmRRXjMaJB7CBCMFTipraVKhyheGNmyEXkYnw7QXbPDNDj3WkgU1bmknXynGKVHWE9LArczq42CDqv4mXk2es8"
        );

        assert_eq!(
            &output[0].public_key.contents,
            "9avcmC97zLPwHKXiDz6GpXyjvPn9VcN3ggqM5gsRnjvv"
        )
    }

    #[test]

    fn test_raw_private_key() {
        let raw_private_key = "[
            89, 35, 184, 142, 158, 93, 59, 17, 149, 153, 229, 56, 52, 46, 14, 247, 169, 118, 131,
            84, 126, 3, 25, 239, 20, 216, 231, 172, 96, 201, 69, 128, 249, 100, 99, 204, 73, 189,
            89, 199, 229, 125, 53, 183, 99, 173, 199, 6, 168, 196, 163, 19, 148, 94, 155, 180, 28,
            151, 216, 49, 22, 145, 143, 151
        ]";

        let output = Factory.raw_private_key(raw_private_key).unwrap();

        assert_eq!(
            output.public_key.contents,
            "HnXJX1Bvps8piQwDYEYC6oea9GEkvQvahvRj3c97X9xr"
        );

        let raw_private_key2 = "[
            126, 249, 31, 59, 155, 202, 96, 175, 18, 63, 64, 2, 141, 163, 23, 239, 139, 142, 6, 65,
            10, 29, 228, 121, 237, 108, 13, 232, 198, 238, 207, 47, 75, 83, 65, 81, 121, 108, 47,
            242, 135, 76, 161, 239, 42, 160, 141, 176, 93, 218, 100, 34, 112, 34, 73, 44, 136, 224,
            18, 240, 43, 121, 14, 232
        ]";
        let output2 = Factory.raw_private_key(raw_private_key2).unwrap();
        assert_eq!(
            output2.public_key.contents,
            "653D6kzCj8JjsErCgwpYS8TF4tJhtUGB7NYi4VVdwEns"
        );

        let raw_private_key3 = "685dfe9f42a5ae73e9ca75776a37d7e7a84a8e16df67bd9fa4bbe7e1daea4713c2618f316bc0ec0d36bf0284189c7e43549958be927e91f746b434a1d0979046";
        let output3 = Factory.raw_private_key(raw_private_key3).unwrap();
        assert_eq!(
            output3.public_key.contents,
            "E5nNmfoMkc86poF7F85Lxpb2VETvDV6X6C9FPRBArwvu"
        )
    }

    #[test]
    fn test_send_transaction_smallest_amount() {
        let sender_key = generate_key_from_mnemonic(
            "elegant flat lumber sibling peace convince manage logic crunch pair impact bench",
        );

        assert_eq!(
            sender_key.public_key.contents,
            "sDaZSSKL8BPeAduGRcTudB6Brz5EdxfqUDyVJHr5EAB"
        );
        assert_eq!(sender_key.contents, "4oMuZhcmihwvCpZXGbPbyXVgBzeo9FTvJRBDrkY52EyR1cupGPB1tzgokZV1b983F9c5NmW1v64xPsy59g1N86zK");

        let receiver = ChainPublicKey {
            contents: "9biD1JVeWCPQpWSAGdxGZaNd6VeUm5QYQu9hp2EMnfnp".to_string(),
            chain: Blockchain::Solana,
        };

        let payload = TransactionParameters::Solana {
            transaction_type: SolanaTransactionType::Legacy,

            external_address: Some(ExternalAddress {
                recent_blockhash: "8ccgXYvhnTaqz2uTcurv9x9PshA714QzqPSxCesyMgng".to_string(),
            }),
            owner_program: None,
            decimals: None,
            memo: None,
            references: Vec::new(),
            swap_slippage_bps: None,
            compute_budget_unit_limit: None,
            compute_budget_unit_price: None,
        };

        let signed_tx = Factory
            .send_transaction(
                sender_key.public_key,
                receiver,
                DecimalNumber {
                    value: "0.000000001".to_string(),
                },
                payload,
            )
            .unwrap();

        assert_eq!(
            signed_tx,
            "AQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAAEDDN1DXJdZtFaZvlIgEqCo94Hfe4zs/k+7zpFgpos4O9h/wdX3dkomxgJlLY6GGcOFWkAa84Cr5zlHAzBgnBfY4QAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAcSF7ZDDOSxTErZg5qfh/hsKRZlwnN81vnzITudOZjnUBAgIAAQwCAAAAAQAAAAAAAAA="
        )
    }

    #[test]
    fn test_send_transaction_non_smallest_amount() {
        let sender_key = generate_key_from_mnemonic(
            "elegant flat lumber sibling peace convince manage logic crunch pair impact bench",
        );

        assert_eq!(
            sender_key.public_key.contents,
            "sDaZSSKL8BPeAduGRcTudB6Brz5EdxfqUDyVJHr5EAB"
        );
        assert_eq!(sender_key.contents, "4oMuZhcmihwvCpZXGbPbyXVgBzeo9FTvJRBDrkY52EyR1cupGPB1tzgokZV1b983F9c5NmW1v64xPsy59g1N86zK");

        let receiver = ChainPublicKey {
            contents: "9biD1JVeWCPQpWSAGdxGZaNd6VeUm5QYQu9hp2EMnfnp".to_string(),
            chain: Blockchain::Solana,
        };

        let payload = TransactionParameters::Solana {
            transaction_type: SolanaTransactionType::Legacy,
            external_address: Some(ExternalAddress {
                recent_blockhash: "8ccgXYvhnTaqz2uTcurv9x9PshA714QzqPSxCesyMgng".to_string(),
            }),
            owner_program: None,
            decimals: None,
            memo: None,
            references: Vec::new(),
            swap_slippage_bps: None,
            compute_budget_unit_limit: None,
            compute_budget_unit_price: None,
        };

        let signed_tx = Factory
            .send_transaction(
                sender_key.public_key,
                receiver,
                DecimalNumber {
                    value: "0.000001001".to_string(),
                },
                payload,
            )
            .unwrap();

        assert_eq!(
            signed_tx,
            "AQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAAEDDN1DXJdZtFaZvlIgEqCo94Hfe4zs/k+7zpFgpos4O9h/wdX3dkomxgJlLY6GGcOFWkAa84Cr5zlHAzBgnBfY4QAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAcSF7ZDDOSxTErZg5qfh/hsKRZlwnN81vnzITudOZjnUBAgIAAQwCAAAA6QMAAAAAAAA="
        )
    }

    #[test]
    fn test_send_transaction_amount_one() {
        let sender_key = generate_key_from_mnemonic(
            "elegant flat lumber sibling peace convince manage logic crunch pair impact bench",
        );

        assert_eq!(
            sender_key.public_key.contents,
            "sDaZSSKL8BPeAduGRcTudB6Brz5EdxfqUDyVJHr5EAB"
        );
        assert_eq!(sender_key.contents, "4oMuZhcmihwvCpZXGbPbyXVgBzeo9FTvJRBDrkY52EyR1cupGPB1tzgokZV1b983F9c5NmW1v64xPsy59g1N86zK");

        let receiver = ChainPublicKey {
            contents: "9biD1JVeWCPQpWSAGdxGZaNd6VeUm5QYQu9hp2EMnfnp".to_string(),
            chain: Blockchain::Solana,
        };

        let parameters = TransactionParameters::Solana {
            transaction_type: SolanaTransactionType::Legacy,
            external_address: Some(ExternalAddress {
                recent_blockhash: "8ccgXYvhnTaqz2uTcurv9x9PshA714QzqPSxCesyMgng".to_string(),
            }),
            owner_program: None,
            decimals: None,
            memo: None,
            references: Vec::new(),
            swap_slippage_bps: None,
            compute_budget_unit_limit: None,
            compute_budget_unit_price: None,
        };

        let signed_tx = Factory
            .send_transaction(
                sender_key.public_key,
                receiver,
                DecimalNumber {
                    value: "1".to_string(),
                },
                parameters,
            )
            .unwrap();

        assert_eq!(
            signed_tx,
            "AQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAAEDDN1DXJdZtFaZvlIgEqCo94Hfe4zs/k+7zpFgpos4O9h/wdX3dkomxgJlLY6GGcOFWkAa84Cr5zlHAzBgnBfY4QAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAcSF7ZDDOSxTErZg5qfh/hsKRZlwnN81vnzITudOZjnUBAgIAAQwCAAAAAMqaOwAAAAA="
        )
    }

    #[test]
    fn test_token_transaction_wallet_destination() {
        let owner_key = generate_key_from_mnemonic(
            "coffee double wise share bridge bird raw light area exact spray dial",
        );
        assert_eq!(
            owner_key.public_key.contents,
            "9biD1JVeWCPQpWSAGdxGZaNd6VeUm5QYQu9hp2EMnfnp"
        );
        assert_eq!(owner_key.contents, "2PsFPYS5k1EXfMikpaSgHkeVw1z3pgeQnQRwgCyzXpyAxNbfvbKps3yGbVfkcJ6R5efMFxDtUN7eaXbh13VhffEg");

        let receiver_wallet = TokenDestination::Wallet {
            public_key: ChainPublicKey {
                contents: "sDaZSSKL8BPeAduGRcTudB6Brz5EdxfqUDyVJHr5EAB".to_string(),
                chain: Blockchain::Solana,
            },
        };

        let token_mint = ChainPublicKey {
            contents: "MERt85fc5boKw3BW1eYdxonEuJNvXbiMbs6hvheau5K".to_string(),
            chain: Blockchain::Solana,
        };

        let transaction_kind = TransactionKind::Token {
            amount: "1".into(),
            close_account: false,
        };

        let parameters = TransactionParameters::Solana {
            transaction_type: SolanaTransactionType::Legacy,
            external_address: Some(ExternalAddress {
                recent_blockhash: "CP9EYNGDbYL8BJLyaSzgWJUo18xAVTN4Skgih1CNMkPr".to_string(),
            }),
            owner_program: None,
            decimals: Some(0),
            memo: None,
            references: Vec::new(),
            swap_slippage_bps: None,
            compute_budget_unit_limit: None,
            compute_budget_unit_price: None,
        };

        let signed_tx = Factory
            .token_transaction(
                receiver_wallet,
                owner_key.public_key,
                token_mint,
                transaction_kind,
                parameters,
            )
            .unwrap();

        // TODO
        // The output does differ but upon inspection still generates a valid signed TX
        // with the correct instrutions, my gut tells me the way metaplex's lib creates the TX differs slightly
        // from the spl lib, but however remains functional
        // confirmed in both our simulate endpoint + solana explorer
        //         assert_eq!(
        //             signed_tx,
        // "ASpFqGbE6QCPyORIzrXoAPeNqImqZbMuEr9xjl28edyDxKo1dwvME4h4+kYZNT0DjjZu8H6ykRrFwRQTDLXLgAgBAAYJf8HV93ZKJsYCZS2OhhnDhVpAGvOAq+c5RwMwYJwX2OHYwlbgG1W3PhF+t7bT+eF63FS+MFT8wfnf5PQkZLoo2eBnCBuplHILX5svG7vBAH64rRZxRrLeWp00k7XmAa4LAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACMlyWPTiSJ8bs9ECkUjg2DC1oTmdr/EIQEjnvY2+n4WQUuzOZH5G7v7plkJy7+2KXm4r81EQE/tbzjHsmNqpACDN1DXJdZtFaZvlIgEqCo94Hfe4zs/k+7zpFgpos4O9gGp9UXGSxcUSGMyUw9SvF/WNruCJuh/UTj29mKAAAAAAbd9uHXZaGT2cvhRs7reawctIXtX1s3kTqM9YV+/wCpqRykwqGGABuwLizWh1DJZiX4LOlZsGEUqTzUAHGqjIkCBAcAAgYFAwgHAAgDAQIACQMBAAAAAAAAAA=="
        //         )

        assert_eq!(
            signed_tx,
            "AQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAAUIf8HV93ZKJsYCZS2OhhnDhVpAGvOAq+c5RwMwYJwX2OHYwlbgG1W3PhF+t7bT+eF63FS+MFT8wfnf5PQkZLoo2eBnCBuplHILX5svG7vBAH64rRZxRrLeWp00k7XmAa4LAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAFLszmR+Ru7+6ZZCcu/til5uK/NREBP7W84x7JjaqQAgbd9uHXZaGT2cvhRs7reawctIXtX1s3kTqM9YV+/wCpDN1DXJdZtFaZvlIgEqCo94Hfe4zs/k+7zpFgpos4O9iMlyWPTiSJ8bs9ECkUjg2DC1oTmdr/EIQEjnvY2+n4WakcpMKhhgAbsC4s1odQyWYl+CzpWbBhFKk81ABxqoyJAgcGAAIGBAMFAQAFBQEEAgAACgwBAAAAAAAAAAA="
        )
    }

    #[test]
    fn test_token_transaction_account_destination() {
        let owner_key = generate_key_from_mnemonic(
            "coffee double wise share bridge bird raw light area exact spray dial",
        );
        assert_eq!(
            owner_key.public_key.contents,
            "9biD1JVeWCPQpWSAGdxGZaNd6VeUm5QYQu9hp2EMnfnp"
        );
        assert_eq!(owner_key.contents, "2PsFPYS5k1EXfMikpaSgHkeVw1z3pgeQnQRwgCyzXpyAxNbfvbKps3yGbVfkcJ6R5efMFxDtUN7eaXbh13VhffEg");

        let receiver_token_account = TokenDestination::Account {
            transfer_destination: "dquDQsAZRT2D8E9mpvhpA67jZDwKNRHrE2eGJGca6Lz".to_string(),
        };

        let token_mint = ChainPublicKey {
            contents: "MERt85fc5boKw3BW1eYdxonEuJNvXbiMbs6hvheau5K".to_string(),
            chain: Blockchain::Solana,
        };

        let transaction_kind = TransactionKind::Token {
            amount: "1".into(),
            close_account: false,
        };

        let parameters = TransactionParameters::Solana {
            transaction_type: SolanaTransactionType::Legacy,
            external_address: Some(ExternalAddress {
                recent_blockhash: "HDcARt8PjyBHwGxopZxfp6Sr2DMamh8Hnm7oeo1467o4".to_string(),
            }),
            owner_program: None,
            decimals: Some(0),
            memo: None,
            references: Vec::new(),
            swap_slippage_bps: None,
            compute_budget_unit_limit: None,
            compute_budget_unit_price: None,
        };

        let signed_tx = Factory
            .token_transaction(
                receiver_token_account,
                owner_key.public_key,
                token_mint,
                transaction_kind,
                parameters,
            )
            .unwrap();

        // TODO
        // The output does differ but upon inspection still generates a valid signed TX
        // with the correct instrutions, my gut tells me the way metaplex's lib creates the TX differs slightly
        // from the spl lib, but however remains functional
        // confirmed in both our simulate endpoint + solana explorer
        // assert_eq!(
        //     signed_tx,
        //     "AaE1C7yMLfOQcb5vqnjaZzgzJmnf4lKcocn+UB6XDBDnqdhZdyFRYDfM/TFxg4BOrMej8O8OqDJWwRuyWEBXRAABAAEEf8HV93ZKJsYCZS2OhhnDhVpAGvOAq+c5RwMwYJwX2OEJcDNoAPdW+2JOGibqlZYtKGTkbigRwvKXyy1B2nLVs/shO0A5AOuoJ9bgsT4uiM64ZPHo3MIFAJNqpODmnsb5Bt324ddloZPZy+FGzut5rBy0he1fWzeROoz1hX7/AKnw9bgOq6cjMBtgv7kOXUDdlyy2DQV1e9GAzFfD4GwpPwEDAwIBAAkDAQAAAAAAAAA="
        // )

        assert_eq!(
            signed_tx,
            "AQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAAIFf8HV93ZKJsYCZS2OhhnDhVpAGvOAq+c5RwMwYJwX2OEJcDNoAPdW+2JOGibqlZYtKGTkbigRwvKXyy1B2nLVs9jCVuAbVbc+EX63ttP54XrcVL4wVPzB+d/k9CRkuijZBS7M5kfkbu/umWQnLv7YpebivzURAT+1vOMeyY2qkAIG3fbh12Whk9nL4UbO63msHLSF7V9bN5E6jPWFfv8AqfD1uA6rpyMwG2C/uQ5dQN2XLLYNBXV70YDMV8PgbCk/AQQFAgMBAAAKDAEAAAAAAAAAAA=="
        )
    }

    #[test]
    fn test_sign_legacy_tx() {
        let signer = generate_key_from_mnemonic(
            "ski seven shuffle amazing tooth net useful asthma drive crystal solar glare",
        );
        assert_eq!(
            signer.public_key.contents,
            "F7xVyQuLzvyUKbMQyrBHaqYGCzHWpmsocn8b7oRUyeC5"
        );
        assert_eq!(signer.contents, "3NpEaVRpJAQjCxa6RM9zVrBpC7B61ypQJxt8FQM41jKZLi5bDdUnn3yXHyyLcDuFcoQrbECDH7SEiPi2z4j9w9PT");

        let base_64_unsigned_tx = "AQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAAoT0cva94tdeUJahExxXl5yoYrk1nsxlCWaSqvYqqppa6YRmJQz2BPHWeODDKN8Qtx9iPTXJqZBIdulNKq1NcTf7jOzHsTv+PoomuqMlUwBYy4tdkkIzlRNaGW97xEb/2ErRbSgDpkuuIlAxLykw4mGod2nd6ziifou4usSCxcEWihty/B1SeAdNCE/corYRxO1txeHL6w4E8q5Y68xLI3bzW8pOWiySsYpNA+F1v5c0yUnlsvwURGDQhu0yesZsdXTkF98aN834lf+Eb3lP/B6CzO3Amsn1E2Gfu77h7qH/CGUn5HzpNhL87Ljrhp5ZwCQzOu6oNRrlR0hBxy7aLDv167AdbKQTIZo4sepWTNMK23MjtaNesDAMU08VgSaH3F4AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACMlyWPTiSJ8bs9ECkUjg2DC1oTmdr/EIQEjnvY2+n4WQMGRm/lIRcy/+ytunLDm+e8jOW7xfcSayxDmzpAAAAAu6nN/XmuJp8Db7aXm0uYp3KSexdu9rcZZXaHIfHx8uTsgRBREqJX1h30z18T7gobAZGXyMU0O08qfsiEauIsGu8Ni2/aLOukHaFdQJXR2jkqDS+O0MbHvA9M+sjCgLVtBpuIV/6rgYT7aH9jRhjANdrEOdwa6ztVmKDwAAAAAAEGm4uYWqtTKkUJDehVf83cvmy378c6CmWwb5IDXbc+7Aan1RcZLFxRIYzJTD1K8X9Y2u4Im6H9ROPb2YoAAAAABt324ddloZPZy+FGzut5rBy0he1fWzeROoz1hX7/AKlmcm3QJD5/XiE4nDDqNootQizJhe85xH/7f7PQFMjncAkLAAUC4JMEAAsACQOVdQAAAAAAAAkCAAF8AwAAANHL2veLXXlCWoRMcV5ecqGK5NZ7MZQlmkqr2KqqaWumIAAAAAAAAAA0VXBEMmZoN3hIM1ZQOVFRYVh0c1MxWVkzYnh6V2h0ZsCnlwAAAAAAFAUAAAAAAAAGm4uYWqtTKkUJDehVf83cvmy378c6CmWwb5IDXbc+7BAFAQIAERIBBgkCAAgMAgAAAJCkIAAAAAAACgcACAAPCRIRAAoHAAYAAwkSEQAQDggGBAUDAgwHAQAODQASCQ6ghgEAAAAAABIDCAAAAQk=".to_string();

        let expected_base64_signed_tx = "AYTk34Oql2cYQmaF+V5kRmhk3snfBwWCsSaFrpUKojPDG0tseRVrPy4mDPBf7W2dP+ipfw4mm6eubsMT17cKdwUBAAoT0cva94tdeUJahExxXl5yoYrk1nsxlCWaSqvYqqppa6YRmJQz2BPHWeODDKN8Qtx9iPTXJqZBIdulNKq1NcTf7jOzHsTv+PoomuqMlUwBYy4tdkkIzlRNaGW97xEb/2ErRbSgDpkuuIlAxLykw4mGod2nd6ziifou4usSCxcEWihty/B1SeAdNCE/corYRxO1txeHL6w4E8q5Y68xLI3bzW8pOWiySsYpNA+F1v5c0yUnlsvwURGDQhu0yesZsdXTkF98aN834lf+Eb3lP/B6CzO3Amsn1E2Gfu77h7qH/CGUn5HzpNhL87Ljrhp5ZwCQzOu6oNRrlR0hBxy7aLDv167AdbKQTIZo4sepWTNMK23MjtaNesDAMU08VgSaH3F4AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACMlyWPTiSJ8bs9ECkUjg2DC1oTmdr/EIQEjnvY2+n4WQMGRm/lIRcy/+ytunLDm+e8jOW7xfcSayxDmzpAAAAAu6nN/XmuJp8Db7aXm0uYp3KSexdu9rcZZXaHIfHx8uTsgRBREqJX1h30z18T7gobAZGXyMU0O08qfsiEauIsGu8Ni2/aLOukHaFdQJXR2jkqDS+O0MbHvA9M+sjCgLVtBpuIV/6rgYT7aH9jRhjANdrEOdwa6ztVmKDwAAAAAAEGm4uYWqtTKkUJDehVf83cvmy378c6CmWwb5IDXbc+7Aan1RcZLFxRIYzJTD1K8X9Y2u4Im6H9ROPb2YoAAAAABt324ddloZPZy+FGzut5rBy0he1fWzeROoz1hX7/AKlmcm3QJD5/XiE4nDDqNootQizJhe85xH/7f7PQFMjncAkLAAUC4JMEAAsACQOVdQAAAAAAAAkCAAF8AwAAANHL2veLXXlCWoRMcV5ecqGK5NZ7MZQlmkqr2KqqaWumIAAAAAAAAAA0VXBEMmZoN3hIM1ZQOVFRYVh0c1MxWVkzYnh6V2h0ZsCnlwAAAAAAFAUAAAAAAAAGm4uYWqtTKkUJDehVf83cvmy378c6CmWwb5IDXbc+7BAFAQIAERIBBgkCAAgMAgAAAJCkIAAAAAAACgcACAAPCRIRAAoHAAYAAwkSEQAQDggGBAUDAgwHAQAODQASCQ6ghgEAAAAAABIDCAAAAQk=".to_string();

        let parameters = TransactionParameters::Solana {
            transaction_type: SolanaTransactionType::Legacy,
            external_address: Some(ExternalAddress {
                recent_blockhash: "7tundXorVXYY2cjaBq8WDDLAz3X7AhNJsZpyGMPAbU7h".to_string(),
            }),
            owner_program: None,
            decimals: Some(0),
            memo: None,
            references: Vec::new(),
            swap_slippage_bps: None,
            compute_budget_unit_limit: None,
            compute_budget_unit_price: None,
        };

        let signed_base64_tx = Factory
            .sign_transaction(base_64_unsigned_tx, vec![signer], Some(parameters))
            .unwrap()
            .tx;

        assert_eq!(signed_base64_tx, expected_base64_signed_tx);
    }

    #[test]
    fn test_sign_versioned_tx() {
        let signer = generate_key_from_mnemonic(
            "ski seven shuffle amazing tooth net useful asthma drive crystal solar glare",
        );
        assert_eq!(
            signer.public_key.contents,
            "F7xVyQuLzvyUKbMQyrBHaqYGCzHWpmsocn8b7oRUyeC5"
        );
        assert_eq!(signer.contents, "3NpEaVRpJAQjCxa6RM9zVrBpC7B61ypQJxt8FQM41jKZLi5bDdUnn3yXHyyLcDuFcoQrbECDH7SEiPi2z4j9w9PT");

        let base_64_unsigned_tx = "AQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACAAQADBtHL2veLXXlCWoRMcV5ecqGK5NZ7MZQlmkqr2KqqaWumAsdJ+MLHwJwkLcM/DVenjI6q9RQhnxKyWVUXQWa4BSh9+vRWPcRHjSHT5FiJaeHhaUDdkoTv/ibHdZHMHpbkYQMGRm/lIRcy/+ytunLDm+e8jOW7xfcSayxDmzpAAAAAMdjhfd4PWcGOB1uYyp1rZcj6JO1QbSBsXr48GA8CN39bF8fIam5zn68XUYGDY+lPkIvzcATObT+88Ze90vUfHHrR344h8Qz+PiG3R/+v65WYQ2dwCPNMzokGqsnOrpwWAgMACQNQwwAAAAAAAAQLBQEABgcIAgAJBggRdf+aR/U6X1kQJwAAAAAAAAABxzn1RUDrpZ6nv3JQ3k8eedEJfsImTqq3Px255rI83OgCAwUCBCY=".to_string();

        let expected_base64_signed_tx = "AXEne4zpxgHsa4kvOzhmSxK4oh2RpAKrpbhXANH07gIBqlOxa3ldvPWCgAybYIuAIWzkLY7y6hYLGbxdw4MRNAaAAQADBtHL2veLXXlCWoRMcV5ecqGK5NZ7MZQlmkqr2KqqaWumAsdJ+MLHwJwkLcM/DVenjI6q9RQhnxKyWVUXQWa4BSh9+vRWPcRHjSHT5FiJaeHhaUDdkoTv/ibHdZHMHpbkYQMGRm/lIRcy/+ytunLDm+e8jOW7xfcSayxDmzpAAAAAMdjhfd4PWcGOB1uYyp1rZcj6JO1QbSBsXr48GA8CN39bF8fIam5zn68XUYGDY+lPkIvzcATObT+88Ze90vUfHHrR344h8Qz+PiG3R/+v65WYQ2dwCPNMzokGqsnOrpwWAgMACQNQwwAAAAAAAAQLBQEABgcIAgAJBggRdf+aR/U6X1kQJwAAAAAAAAABxzn1RUDrpZ6nv3JQ3k8eedEJfsImTqq3Px255rI83OgCAwUCBCY=".to_string();

        let parameters = TransactionParameters::Solana {
            transaction_type: SolanaTransactionType::Versioned,
            external_address: Some(ExternalAddress {
                recent_blockhash: "9GSMRUkUAJYXU9Xz4XgGUMY9c1ndqUk4929WTPaVCZP3".to_string(),
            }),
            owner_program: None,
            decimals: None,
            memo: None,
            references: Vec::new(),
            swap_slippage_bps: None,
            compute_budget_unit_limit: None,
            compute_budget_unit_price: None,
        };

        let signed_base64_tx = Factory
            .sign_transaction(base_64_unsigned_tx, vec![signer], Some(parameters))
            .unwrap()
            .tx;

        assert_eq!(signed_base64_tx, expected_base64_signed_tx);
    }

    #[test]
    fn test_partially_signed_by_receiver_versioned_tx() {
        let signer = generate_key_from_mnemonic(
            "ski seven shuffle amazing tooth net useful asthma drive crystal solar glare",
        );
        assert_eq!(
            signer.public_key.contents,
            "F7xVyQuLzvyUKbMQyrBHaqYGCzHWpmsocn8b7oRUyeC5"
        );
        assert_eq!(signer.contents, "3NpEaVRpJAQjCxa6RM9zVrBpC7B61ypQJxt8FQM41jKZLi5bDdUnn3yXHyyLcDuFcoQrbECDH7SEiPi2z4j9w9PT");

        let base_64_unsigned_tx = "AgAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAgAHE9HL2veLXXlCWoRMcV5ecqGK5NZ7MZQlmkqr2KqqaWumTbN4X+WyPEKmz78gUw7kPfJ34GDbmN0M4dBnbI7Pl6oROS/6uomKs1GUntZaCNOynoyjJ2kB9EBx9SZlpld+yTcupyMWhev/jm806YQiUQh8olPQsaooX014ecIG/rQRWULT4sVshXcKatT1WLCJwf7YCp3vT7uUhMV9AJbAktF4uHnwlmWTP7i+YDLj4DLfYY7WuoP6ysbgf33fp2sJQ3369FY9xEeNIdPkWIlp4eFpQN2ShO/+Jsd1kcweluRhiYvr57A0tT1PTgLkQvb3+hBw3Ww08mC6UmkqwSJtjA62Ao8MrgKvcKTga+Z6q+/rlkq4vpLGNizNGBSgLpyUh7auMGL4kkJbZoqV68a+yV6pUyhBDDP5mIaOJrfqBkDwywKD1IYsYCTn8dvguormrOz3uyCHQWI3q883baADVC/tmCcgLAYIi9oQSjVpIzlazKwEC/xgz3x8f3UH3ygj7wAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAYhnpBCJt8bHy/4ogw1J7KOnj3PZwzCT1lg0LNtN5Bi+F/gItgg96L89o2JXC/COCZ6dbAiQ020h+0Nzd3oIXEQabiFf+q4GE+2h/Y0YYwDXaxDncGus7VZig8AAAAAABBqfVFxksXFEhjMlMPUrxf1ja7gibof1E49vZigAAAAAG3fbh12Whk9nL4UbO63msHLSF7V9bN5E6jPWFfv8AqQ4DaF+OkJBT5FgSHGb1p2rtx3BqoRyC+KqVKo8reHmp9aQP2xHW5ToEY0rvLwQDSWm/UPFv5nqMDU5wfHYbURAEDAIAATQAAAAA8B0fAAAAAAClAAAAAAAAAAbd9uHXZaGT2cvhRs7reawctIXtX1s3kTqM9YV+/wCpEQQBDwAQAQESFBEACAIBCwYHAQMKBAkJCQUFBQ4NO8Ng7WxEotvmCgAAAAAAAACTAQAAAAAAAAEAAa8zG6gyf7s1scT+/wAAAABQOwEAAQAAAAAAAAAAAAAAEQMBAAABCQ==".to_string();

        let expected_base64_signed_tx = "AiUkCo2fSFiCBBCgSZ4F8XR3u2gGzzHPU1eNyvz0Ey1+cqQlq/y+hh1xx+Hjn6HVOd7JvmmJnrjrjO9hrFcZZQIAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAgAHE9HL2veLXXlCWoRMcV5ecqGK5NZ7MZQlmkqr2KqqaWumTbN4X+WyPEKmz78gUw7kPfJ34GDbmN0M4dBnbI7Pl6oROS/6uomKs1GUntZaCNOynoyjJ2kB9EBx9SZlpld+yTcupyMWhev/jm806YQiUQh8olPQsaooX014ecIG/rQRWULT4sVshXcKatT1WLCJwf7YCp3vT7uUhMV9AJbAktF4uHnwlmWTP7i+YDLj4DLfYY7WuoP6ysbgf33fp2sJQ3369FY9xEeNIdPkWIlp4eFpQN2ShO/+Jsd1kcweluRhiYvr57A0tT1PTgLkQvb3+hBw3Ww08mC6UmkqwSJtjA62Ao8MrgKvcKTga+Z6q+/rlkq4vpLGNizNGBSgLpyUh7auMGL4kkJbZoqV68a+yV6pUyhBDDP5mIaOJrfqBkDwywKD1IYsYCTn8dvguormrOz3uyCHQWI3q883baADVC/tmCcgLAYIi9oQSjVpIzlazKwEC/xgz3x8f3UH3ygj7wAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAYhnpBCJt8bHy/4ogw1J7KOnj3PZwzCT1lg0LNtN5Bi+F/gItgg96L89o2JXC/COCZ6dbAiQ020h+0Nzd3oIXEQabiFf+q4GE+2h/Y0YYwDXaxDncGus7VZig8AAAAAABBqfVFxksXFEhjMlMPUrxf1ja7gibof1E49vZigAAAAAG3fbh12Whk9nL4UbO63msHLSF7V9bN5E6jPWFfv8AqQ4DaF+OkJBT5FgSHGb1p2rtx3BqoRyC+KqVKo8reHmp9aQP2xHW5ToEY0rvLwQDSWm/UPFv5nqMDU5wfHYbURAEDAIAATQAAAAA8B0fAAAAAAClAAAAAAAAAAbd9uHXZaGT2cvhRs7reawctIXtX1s3kTqM9YV+/wCpEQQBDwAQAQESFBEACAIBCwYHAQMKBAkJCQUFBQ4NO8Ng7WxEotvmCgAAAAAAAACTAQAAAAAAAAEAAa8zG6gyf7s1scT+/wAAAABQOwEAAQAAAAAAAAAAAAAAEQMBAAABCQ==".to_string();

        let parameters = TransactionParameters::Solana {
            transaction_type: SolanaTransactionType::Versioned,
            external_address: Some(ExternalAddress {
                recent_blockhash: "HXsz2MHGFu5VqSJZ7KMyboDZJLfMcN2sa2jDnpuTzSFR".to_string(),
            }),
            owner_program: None,
            decimals: None,
            memo: None,
            references: Vec::new(),
            swap_slippage_bps: None,
            compute_budget_unit_limit: None,
            compute_budget_unit_price: None,
        };

        let signed_base64_tx = Factory
            .sign_transaction(base_64_unsigned_tx, vec![signer], Some(parameters))
            .unwrap()
            .tx;

        assert_eq!(signed_base64_tx, expected_base64_signed_tx);

        let transaction_bytes = from_base64(&signed_base64_tx).unwrap();
        let versioned_tx: VersionedTransaction = bincode::deserialize(&transaction_bytes).unwrap();

        assert_eq!(versioned_tx.signatures.len(), 2);
        assert_eq!(versioned_tx.signatures[0].to_string(), "k4z1JxCSjBKDEbMCQj6aZJJgJgtoxUnEGZJNRYiGiTzTyAvB1Tzaamfwxdzsms9iSrFPKpDRVS5Yh1ScJFPFLKs".to_string());
        assert_eq!(
            versioned_tx.signatures[1].to_string(),
            Signature::default().to_string()
        )
    }

    #[test]
    fn test_partially_signed_by_sender_versioned_tx() {
        let signer = generate_key_from_mnemonic(
            "ski seven shuffle amazing tooth net useful asthma drive crystal solar glare",
        );
        assert_eq!(
            signer.public_key.contents,
            "F7xVyQuLzvyUKbMQyrBHaqYGCzHWpmsocn8b7oRUyeC5"
        );
        assert_eq!(signer.contents, "3NpEaVRpJAQjCxa6RM9zVrBpC7B61ypQJxt8FQM41jKZLi5bDdUnn3yXHyyLcDuFcoQrbECDH7SEiPi2z4j9w9PT");

        let base_64_unsigned_tx = "Ap6zKWvWlgd0ICJPyuAQLCiRv6RwWOPNOEfF+cffja+rljr/GqJpmUxXaP7Kb81qLmVjUGucmNRZp7sLW7HEWg4AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAgEGC92gIQD99/51qLbWc7d5WojjJE40xdVtvQwQIe8z3kZo0cva94tdeUJahExxXl5yoYrk1nsxlCWaSqvYqqppa6Y3c+dMp9VE6tXsYqZWY5AFARhuy5wbb2YF9VKslqmuYEFazacjR0XZrvgcKDOZnQDIpiKAiJevaKtngDiqTdXRffr0Vj3ER40h0+RYiWnh4WlA3ZKE7/4mx3WRzB6W5GEAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAMGRm/lIRcy/+ytunLDm+e8jOW7xfcSayxDmzpAAAAABt324ddloZPZy+FGzut5rBy0he1fWzeROoz1hX7/AKkWHlK6uJknHZO7juRzrr83NbHhSpB+m2d99aPD6DvLD4yXJY9OJInxuz0QKRSODYMLWhOZ2v8QhASOe9jb6fhZxvp6877brTo9ZfNqq8l0MbG75MLS9uDkfKYCA0UvXWFLwb0qjTubchFpROs4NlqdbSPc3OmOZZGbfC+1Ul9fngUGAAUCQEIPAAYACQMAAAAAAAAAAAkGAAMICgUHAQAHBAQDAQEJA2P/AgAAAAAABwQEAgEBCQMt0QAAAAAAAA==".to_string();

        let expected_base64_signed_tx = "Ap6zKWvWlgd0ICJPyuAQLCiRv6RwWOPNOEfF+cffja+rljr/GqJpmUxXaP7Kb81qLmVjUGucmNRZp7sLW7HEWg7X2sasG5LnyMHHYCTxvEUmypcQEa0qf3OdKmoACaC5INCQPi2hp1fYsfkbxaSFHoLCSW6i5zMV9bIZpsN5m08NAgEGC92gIQD99/51qLbWc7d5WojjJE40xdVtvQwQIe8z3kZo0cva94tdeUJahExxXl5yoYrk1nsxlCWaSqvYqqppa6Y3c+dMp9VE6tXsYqZWY5AFARhuy5wbb2YF9VKslqmuYEFazacjR0XZrvgcKDOZnQDIpiKAiJevaKtngDiqTdXRffr0Vj3ER40h0+RYiWnh4WlA3ZKE7/4mx3WRzB6W5GEAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAMGRm/lIRcy/+ytunLDm+e8jOW7xfcSayxDmzpAAAAABt324ddloZPZy+FGzut5rBy0he1fWzeROoz1hX7/AKkWHlK6uJknHZO7juRzrr83NbHhSpB+m2d99aPD6DvLD4yXJY9OJInxuz0QKRSODYMLWhOZ2v8QhASOe9jb6fhZxvp6877brTo9ZfNqq8l0MbG75MLS9uDkfKYCA0UvXWFLwb0qjTubchFpROs4NlqdbSPc3OmOZZGbfC+1Ul9fngUGAAUCQEIPAAYACQMAAAAAAAAAAAkGAAMICgUHAQAHBAQDAQEJA2P/AgAAAAAABwQEAgEBCQMt0QAAAAAAAA==".to_string();

        let expected_sender_signature = "4B2hSYeSHM5AB5mqQFX7t7Ckpq2CKKbdkzdKnZKqVKMD1PAgo8YdifCthyXLS93HTGHge7sbght8ycZNoPepEFoK".to_string();
        let expected_signer_signature = "5KJmAje8dJJyrG7xpekHKTQEr8t5iHVfaC3MvYrP91NDDgGXt15LX16YASHCzgfbFeEhMfFGNS422jErKqn71Jd2".to_string();

        let parameters = TransactionParameters::Solana {
            transaction_type: SolanaTransactionType::Versioned,
            external_address: Some(ExternalAddress {
                recent_blockhash: "66ivRKuuUcN57EcBqtYMucThhPhRm6ZmH3Kbk2vbJPJ5".to_string(),
            }),
            owner_program: None,
            decimals: None,
            memo: None,
            references: Vec::new(),
            swap_slippage_bps: None,
            compute_budget_unit_limit: None,
            compute_budget_unit_price: None,
        };

        let signed_base64_tx = Factory
            .sign_transaction(base_64_unsigned_tx, vec![signer], Some(parameters))
            .unwrap()
            .tx;

        assert_eq!(signed_base64_tx, expected_base64_signed_tx);

        let transaction_bytes = from_base64(&signed_base64_tx).unwrap();
        let versioned_tx: VersionedTransaction = bincode::deserialize(&transaction_bytes).unwrap();

        assert_eq!(versioned_tx.signatures.len(), 2);
        assert_eq!(
            versioned_tx.signatures[0].to_string(),
            expected_sender_signature
        );
        assert_eq!(
            versioned_tx.signatures[1].to_string(),
            expected_signer_signature
        )
    }

    #[test]
    fn test_partially_signed_by_wallet_versioned_tx() {
        let signer = generate_key_from_mnemonic(
            "ski seven shuffle amazing tooth net useful asthma drive crystal solar glare",
        );
        assert_eq!(
            signer.public_key.contents,
            "F7xVyQuLzvyUKbMQyrBHaqYGCzHWpmsocn8b7oRUyeC5"
        );
        assert_eq!(signer.contents, "3NpEaVRpJAQjCxa6RM9zVrBpC7B61ypQJxt8FQM41jKZLi5bDdUnn3yXHyyLcDuFcoQrbECDH7SEiPi2z4j9w9PT");

        let base_64_unsigned_tx = "AgAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAUfAKGV9uw9h0dhWb/glBW2vnGqXHd+c9lKEd3EVg+dtffBs6Wlvj9svCEjZcXHyIZ3xS4rj8zpQkVXUtjBmsLAgAKENHL2veLXXlCWoRMcV5ecqGK5NZ7MZQlmkqr2KqqaWumi8gFfVLRqGlWQMlIXIAQpdb6pQwbkzq1VPL9rPKjV7kr1lQqaOSFYS9WELcT14N7mJY9eLJbJXlsZ9Z5/AUPNkwLKxPjXqRuX5TZBccJ2WRO0qjuZzPzqFdncg8vWz45fvy+/SUYA+TAXz8ruI65e2kjhp+mg2AawUQN5bX3gP68K5ZficO5VwesMce/cvsBy5AvfQoKym53Aehbqm9wSQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAASj7vSwPIKnFZnqB6Fu5Lz23OMTV9hGCyrBvUw6mGDJ1UX6MOo7w/PClm2otsPf7406t9pXygIypU5KAmT//DwgMGRm/lIRcy/+ytunLDm+e8jOW7xfcSayxDmzpAAAAACVTbvp7JYMmKeik/4hM2lm/hgNFRrkuBeVYfiYVKU/bqoCDGHMR5cSgTRhzhU4lKlqbACyHtDPwnmNH5qenJSu8Ni2/aLOukHaFdQJXR2jkqDS+O0MbHvA9M+sjCgLVtBpuIV/6rgYT7aH9jRhjANdrEOdwa6ztVmKDwAAAAAAEGp9UXGSxcUSGMyUw9SvF/WNruCJuh/UTj29mKAAAAAAbd9uHXZaGT2cvhRs7reawctIXtX1s3kTqM9YV+/wCpkRdIS4Uf1lOIIH03UKufBY2K1atOv3eUwl5+GsBLsRQFCQAFAsAnCQAGAgABNAAAAAAgHZoAAAAAAKUAAAAAAAAABt324ddloZPZy+FGzut5rBy0he1fWzeROoz1hX7/AKkPBAENAA4BAQoLBwQDAAUBDwsMCAIT8iPGiVLh8rYBAKCGAQAAAAAAAA8DAQAAAQk=".to_string();

        let expected_base64_signed_tx = "Aq/evSDXvErhQ3IosEqB0M8iTXlerkvQPA+5lvhIRe9TL7eq4NXXPcpQv4dUMGYdqIqacQTvu4LRzZ8b1PDQ/gwUfAKGV9uw9h0dhWb/glBW2vnGqXHd+c9lKEd3EVg+dtffBs6Wlvj9svCEjZcXHyIZ3xS4rj8zpQkVXUtjBmsLAgAKENHL2veLXXlCWoRMcV5ecqGK5NZ7MZQlmkqr2KqqaWumi8gFfVLRqGlWQMlIXIAQpdb6pQwbkzq1VPL9rPKjV7kr1lQqaOSFYS9WELcT14N7mJY9eLJbJXlsZ9Z5/AUPNkwLKxPjXqRuX5TZBccJ2WRO0qjuZzPzqFdncg8vWz45fvy+/SUYA+TAXz8ruI65e2kjhp+mg2AawUQN5bX3gP68K5ZficO5VwesMce/cvsBy5AvfQoKym53Aehbqm9wSQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAASj7vSwPIKnFZnqB6Fu5Lz23OMTV9hGCyrBvUw6mGDJ1UX6MOo7w/PClm2otsPf7406t9pXygIypU5KAmT//DwgMGRm/lIRcy/+ytunLDm+e8jOW7xfcSayxDmzpAAAAACVTbvp7JYMmKeik/4hM2lm/hgNFRrkuBeVYfiYVKU/bqoCDGHMR5cSgTRhzhU4lKlqbACyHtDPwnmNH5qenJSu8Ni2/aLOukHaFdQJXR2jkqDS+O0MbHvA9M+sjCgLVtBpuIV/6rgYT7aH9jRhjANdrEOdwa6ztVmKDwAAAAAAEGp9UXGSxcUSGMyUw9SvF/WNruCJuh/UTj29mKAAAAAAbd9uHXZaGT2cvhRs7reawctIXtX1s3kTqM9YV+/wCpkRdIS4Uf1lOIIH03UKufBY2K1atOv3eUwl5+GsBLsRQFCQAFAsAnCQAGAgABNAAAAAAgHZoAAAAAAKUAAAAAAAAABt324ddloZPZy+FGzut5rBy0he1fWzeROoz1hX7/AKkPBAENAA4BAQoLBwQDAAUBDwsMCAIT8iPGiVLh8rYBAKCGAQAAAAAAAA8DAQAAAQk=".to_string();

        let parameters = TransactionParameters::Solana {
            transaction_type: SolanaTransactionType::Versioned,
            external_address: Some(ExternalAddress {
                recent_blockhash: "AmNi2ByGKxqmdMmv2uqkAPMpbu2r8VxLb78FFYLpW6Z9".to_string(),
            }),
            owner_program: None,
            decimals: None,
            memo: None,
            references: Vec::new(),
            swap_slippage_bps: None,
            compute_budget_unit_limit: None,
            compute_budget_unit_price: None,
        };

        let signed_base64_tx = Factory
            .sign_transaction(base_64_unsigned_tx, vec![signer], Some(parameters))
            .unwrap()
            .tx;

        assert_eq!(signed_base64_tx, expected_base64_signed_tx);

        let transaction_bytes = from_base64(&signed_base64_tx).unwrap();
        let versioned_tx: VersionedTransaction = bincode::deserialize(&transaction_bytes).unwrap();

        assert_eq!(versioned_tx.signatures.len(), 2);
        assert_eq!(versioned_tx.signatures[0].to_string(), "4WwWzwNKPkWXGytQeoc8L2ndpMjvjbpD89aK7nk3tFdbpHcZAZqkRL9VXvSs3kma4kSLjaS3RFaPHWVcCc3WYjcF".to_string());
        assert_ne!(
            versioned_tx.signatures[1].to_string(),
            Signature::default().to_string()
        )
    }

    #[test]
    fn test_sign_tx_as_msg() {
        let signer = generate_key_from_mnemonic(
            "ski seven shuffle amazing tooth net useful asthma drive crystal solar glare",
        );
        assert_eq!(
            signer.public_key.contents,
            "F7xVyQuLzvyUKbMQyrBHaqYGCzHWpmsocn8b7oRUyeC5"
        );
        assert_eq!(signer.contents, "3NpEaVRpJAQjCxa6RM9zVrBpC7B61ypQJxt8FQM41jKZLi5bDdUnn3yXHyyLcDuFcoQrbECDH7SEiPi2z4j9w9PT");

        let tx = "AQABAvgqkM3VuRGEx9d9Nxkc2mkQbyWkAIKgKm8Kc6kWriuNAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAADw9HByfZnAzxsUu0r8ArmMmQchplDaq1uPaGgpIw/QSgEBAgAADAIAAABAQg8AAAAAAA==";

        let signed_message = Factory.sign_message(tx.to_string(), vec![signer]);

        assert!(signed_message.is_err());
        match signed_message {
            Err(TransactionError::SignMsgError(msg)) => {
                assert_eq!(
                    msg,
                    "You cannot sign solana transactions using sign_message"
                )
            }
            _ => panic!("Expected Generic error with specific message"),
        }
    }

    #[test]
    fn test_sign_message() {
        let signer = generate_key_from_mnemonic(
            "ski seven shuffle amazing tooth net useful asthma drive crystal solar glare",
        );
        assert_eq!(
            signer.public_key.contents,
            "F7xVyQuLzvyUKbMQyrBHaqYGCzHWpmsocn8b7oRUyeC5"
        );
        assert_eq!(signer.contents, "3NpEaVRpJAQjCxa6RM9zVrBpC7B61ypQJxt8FQM41jKZLi5bDdUnn3yXHyyLcDuFcoQrbECDH7SEiPi2z4j9w9PT");

        let raw_message_byte_array: [u8; 541] = [
            109, 97, 103, 105, 99, 101, 100, 101, 110, 46, 105, 111, 32, 119, 97, 110, 116, 115,
            32, 121, 111, 117, 32, 116, 111, 32, 115, 105, 103, 110, 32, 105, 110, 32, 119, 105,
            116, 104, 32, 121, 111, 117, 114, 32, 83, 111, 108, 97, 110, 97, 32, 97, 99, 99, 111,
            117, 110, 116, 58, 10, 70, 55, 120, 86, 121, 81, 117, 76, 122, 118, 121, 85, 75, 98,
            77, 81, 121, 114, 66, 72, 97, 113, 89, 71, 67, 122, 72, 87, 112, 109, 115, 111, 99,
            110, 56, 98, 55, 111, 82, 85, 121, 101, 67, 53, 10, 10, 67, 108, 105, 99, 107, 32, 83,
            105, 103, 110, 32, 111, 114, 32, 65, 112, 112, 114, 111, 118, 101, 32, 111, 110, 108,
            121, 32, 109, 101, 97, 110, 115, 32, 121, 111, 117, 32, 104, 97, 118, 101, 32, 112,
            114, 111, 118, 101, 100, 32, 116, 104, 105, 115, 32, 119, 97, 108, 108, 101, 116, 32,
            105, 115, 32, 111, 119, 110, 101, 100, 32, 98, 121, 32, 121, 111, 117, 46, 32, 84, 104,
            105, 115, 32, 114, 101, 113, 117, 101, 115, 116, 32, 119, 105, 108, 108, 32, 110, 111,
            116, 32, 116, 114, 105, 103, 103, 101, 114, 32, 97, 110, 121, 32, 98, 108, 111, 99,
            107, 99, 104, 97, 105, 110, 32, 116, 114, 97, 110, 115, 97, 99, 116, 105, 111, 110, 32,
            111, 114, 32, 99, 111, 115, 116, 32, 97, 110, 121, 32, 103, 97, 115, 32, 102, 101, 101,
            46, 32, 85, 115, 101, 32, 111, 102, 32, 111, 117, 114, 32, 119, 101, 98, 115, 105, 116,
            101, 32, 97, 110, 100, 32, 115, 101, 114, 118, 105, 99, 101, 32, 97, 114, 101, 32, 115,
            117, 98, 106, 101, 99, 116, 32, 116, 111, 32, 111, 117, 114, 32, 84, 101, 114, 109,
            115, 32, 111, 102, 32, 83, 101, 114, 118, 105, 99, 101, 58, 32, 104, 116, 116, 112,
            115, 58, 47, 47, 109, 97, 103, 105, 99, 101, 100, 101, 110, 46, 105, 111, 47, 116, 101,
            114, 109, 115, 45, 111, 102, 45, 115, 101, 114, 118, 105, 99, 101, 46, 112, 100, 102,
            32, 97, 110, 100, 32, 80, 114, 105, 118, 97, 99, 121, 32, 80, 111, 108, 105, 99, 121,
            58, 32, 104, 116, 116, 112, 115, 58, 47, 47, 109, 97, 103, 105, 99, 101, 100, 101, 110,
            46, 105, 111, 47, 112, 114, 105, 118, 97, 99, 121, 45, 112, 111, 108, 105, 99, 121, 46,
            112, 100, 102, 10, 10, 85, 82, 73, 58, 32, 104, 116, 116, 112, 115, 58, 47, 47, 109,
            97, 103, 105, 99, 101, 100, 101, 110, 46, 105, 111, 10, 86, 101, 114, 115, 105, 111,
            110, 58, 32, 49, 10, 67, 104, 97, 105, 110, 32, 73, 68, 58, 32, 109, 97, 105, 110, 110,
            101, 116, 10, 78, 111, 110, 99, 101, 58, 32, 80, 110, 76, 122, 108, 77, 104, 85, 90,
            120, 10, 73, 115, 115, 117, 101, 100, 32, 65, 116, 58, 32, 50, 48, 50, 51, 45, 48, 52,
            45, 50, 49, 84, 48, 56, 58, 51, 53, 58, 48, 56, 46, 49, 51, 50, 90,
        ];

        assert_eq!(
            String::from_utf8(raw_message_byte_array.into()).unwrap(),
            "magiceden.io wants you to sign in with your Solana account:\nF7xVyQuLzvyUKbMQyrBHaqYGCzHWpmsocn8b7oRUyeC5\n\nClick Sign or Approve only means you have proved this wallet is owned by you. This request will not trigger any blockchain transaction or cost any gas fee. Use of our website and service are subject to our Terms of Service: https://magiceden.io/terms-of-service.pdf and Privacy Policy: https://magiceden.io/privacy-policy.pdf\n\nURI: https://magiceden.io\nVersion: 1\nChain ID: mainnet\nNonce: PnLzlMhUZx\nIssued At: 2023-04-21T08:35:08.132Z"
        );

        let input_as_base64_string = to_base64(raw_message_byte_array);

        let input_back_to_bytes = from_base64(&input_as_base64_string).unwrap();

        assert_eq!(raw_message_byte_array, input_back_to_bytes.as_slice());

        assert_eq!(input_as_base64_string, "bWFnaWNlZGVuLmlvIHdhbnRzIHlvdSB0byBzaWduIGluIHdpdGggeW91ciBTb2xhbmEgYWNjb3VudDoKRjd4VnlRdUx6dnlVS2JNUXlyQkhhcVlHQ3pIV3Btc29jbjhiN29SVXllQzUKCkNsaWNrIFNpZ24gb3IgQXBwcm92ZSBvbmx5IG1lYW5zIHlvdSBoYXZlIHByb3ZlZCB0aGlzIHdhbGxldCBpcyBvd25lZCBieSB5b3UuIFRoaXMgcmVxdWVzdCB3aWxsIG5vdCB0cmlnZ2VyIGFueSBibG9ja2NoYWluIHRyYW5zYWN0aW9uIG9yIGNvc3QgYW55IGdhcyBmZWUuIFVzZSBvZiBvdXIgd2Vic2l0ZSBhbmQgc2VydmljZSBhcmUgc3ViamVjdCB0byBvdXIgVGVybXMgb2YgU2VydmljZTogaHR0cHM6Ly9tYWdpY2VkZW4uaW8vdGVybXMtb2Ytc2VydmljZS5wZGYgYW5kIFByaXZhY3kgUG9saWN5OiBodHRwczovL21hZ2ljZWRlbi5pby9wcml2YWN5LXBvbGljeS5wZGYKClVSSTogaHR0cHM6Ly9tYWdpY2VkZW4uaW8KVmVyc2lvbjogMQpDaGFpbiBJRDogbWFpbm5ldApOb25jZTogUG5MemxNaFVaeApJc3N1ZWQgQXQ6IDIwMjMtMDQtMjFUMDg6MzU6MDguMTMyWg==");

        let signed_message = Factory
            .sign_message(input_as_base64_string, vec![signer])
            .unwrap();

        assert_eq!(signed_message, "z/DaX3zde+wxLHOR8ahRYOIa1GItHyUxuCcbfontSzvtYEq9+wodK75qBKoeqn8rEV6Yg3zJHTWVM7eQKcWlDQ==");

        let expected_byte_array = [
            207, 240, 218, 95, 124, 221, 123, 236, 49, 44, 115, 145, 241, 168, 81, 96, 226, 26,
            212, 98, 45, 31, 37, 49, 184, 39, 27, 126, 137, 237, 75, 59, 237, 96, 74, 189, 251, 10,
            29, 43, 190, 106, 4, 170, 30, 170, 127, 43, 17, 94, 152, 131, 124, 201, 29, 53, 149,
            51, 183, 144, 41, 197, 165, 13,
        ];

        assert_eq!(from_base64(&signed_message).unwrap(), expected_byte_array);
    }

    #[test]
    fn test_broken_private_key() {
        let key = "Gm8YqXq1U5NYyeHt7XmMu9TEMUU8e4dSurhe55e41nFPa7er9oCaMNu1zHBhiprs2F2QnudEfRAmCrRCbb8FDPr";
        let result = Factory.raw_private_key(key);
        if !matches!(result, Err(KeyError::InvalidKeypair(_))) {
            panic!("Broken keys should not be parsed")
        }
    }

    #[test]
    fn test_correct_associated_token_address() {
        let result = Factory.get_associated_token_address(
            "HhjkkWaHbMLLve8mmRsvpVkPQ8hz8Dt5BvXA5y7S92Hz".to_string(),
            "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".to_string(),
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string()
        ).unwrap();

        assert_eq!(
            result.contents,
            "EpUzTPQzX6o3Sb3MZoyXaJXh2G2LRB5sKB1tij5xEnuw"
        )
    }

    #[test]
    fn test_wrong_wallet_address_associated_token_address() {
        let result = Factory.get_associated_token_address(
            "something_wrong".to_string(),
            "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".to_string(),
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string()
        );

        if !matches!(result, Err(TransactionError::PublicKey(_))) {
            panic!("Broken keys should not be parsed")
        }
    }

    #[test]
    fn test_correct_program_address() {
        let result = Factory.get_program_address(
            [
                "invite".to_string(),
                "c8Zhu3498MhJ98PBc7CmPj3oCRJ1HZaB6gPZU3r58kJ".to_string()
            ].to_vec(),
            "inv1tEtSwRMtM44tbvJGNiTxMvDfPVnX9StyqXfDfks".to_string()
        ).unwrap();

        assert_eq!(
            result.contents,
            "HzdVcCqFUPkr6BetwfPyNEPEtWm5usib9nowpAu58WRw"
        )
    }

    #[test]
    fn test_wrong_program_address() {
        let result = Factory.get_program_address(
            [
                "invite".to_string(),
                "wrong".to_string()
            ].to_vec(),
            "inv1tEtSwRMtM44tbvJGNiTxMvDfPVnX9StyqXfDfks".to_string()
        ).unwrap();

        assert_ne!(
            result.contents,
            "HzdVcCqFUPkr6BetwfPyNEPEtWm5usib9nowpAu58WRw"
        )
    }

    #[test]
    fn test_exception_program_address() {
        let result = Factory.get_program_address(
            [
                "1".to_string(),
                "2".to_string()
            ].to_vec(),
            "3".to_string()
        );

        if !matches!(result, Err(TransactionError::PublicKey(_))) {
            panic!("Broken keys should not be parsed")
        }
    }

    #[test]
    fn test_get_message() {
        let sender_pubkey = Pubkey::from_str("7vEitk7AmNJVJqwtsVsxSJkAhYQ4oHWXQadeDUeD4iMy").unwrap();
        let receiver_pubkey = Pubkey::from_str("HhjkkWaHbMLLve8mmRsvpVkPQ8hz8Dt5BvXA5y7S92Hz").unwrap();

        let amount_in_sol = 0.0001;
        let lamports = (amount_in_sol * 1_000_000_000.0) as u64;

        let instruction = system_instruction::transfer(
            &sender_pubkey,
            &receiver_pubkey,
            lamports,
        );

        let message = Message::new(&[instruction], Some(&sender_pubkey));
        let serialized_message = bincode::serialize(&message).unwrap();
        let base64_legacy_message = base64::encode(serialized_message);

        let transaction = Transaction::new_unsigned(message.clone());
        let serialized_tx = bincode::serialize(&transaction).unwrap();
        let base64_legacy_tx = base64::encode(serialized_tx);

        let versioned_message = VersionedMessage::Legacy(message.clone());
        let serialized_versioned_message = bincode::serialize(&versioned_message).unwrap();
        let base64_versioned_message = base64::encode(serialized_versioned_message);

        let versioned_transaction = VersionedTransaction::from(transaction.clone());
        let serialized_versioned_tx = bincode::serialize(&versioned_transaction).unwrap();
        let base64_versioned_tx = base64::encode(serialized_versioned_tx);

        // Test legacy transaction message extraction
        let converted_legacy_message = Factory.get_message(base64_legacy_tx).unwrap();
        assert_eq!(converted_legacy_message, base64_legacy_message);

        // Test versioned transaction message extraction
        let converted_versioned_message = Factory.get_message(base64_versioned_tx).unwrap();
        assert_eq!(converted_versioned_message, base64_versioned_message);

        // Test invalid transaction message extraction
        let result = Factory.get_message("invalid base64".to_string());
        assert!(result.is_err());
    }
    
    #[test]
    fn test_get_transaction() {
        let sender = Keypair::new();
        let receiver_pubkey = Pubkey::from_str("HhjkkWaHbMLLve8mmRsvpVkPQ8hz8Dt5BvXA5y7S92Hz").unwrap();

        let amount_in_sol = 0.0001;
        let lamports = (amount_in_sol * 1_000_000_000.0) as u64;

        let instruction = system_instruction::transfer(&sender.pubkey(), &receiver_pubkey, lamports);
        let message = Message::new(&[instruction], Some(&sender.pubkey()));

        let legacy_bytes = bincode::serialize(&message).unwrap();
        let legacy_b64 = base64::encode(&legacy_bytes);

        let versioned_msg = VersionedMessage::Legacy(message.clone());
        let versioned_bytes = bincode::serialize(&versioned_msg).unwrap();
        let versioned_b64 = base64::encode(&versioned_bytes);

        // Test legacy message
        let tx_legacy_b64 = Factory.get_transaction(legacy_b64.clone()).unwrap();
        let tx_legacy: Transaction = bincode::deserialize(&base64::decode(&tx_legacy_b64).unwrap()).unwrap();
        let tx_legacy_msg_b64 = base64::encode(bincode::serialize(&tx_legacy.message).unwrap());
        let message_b64 = base64::encode(bincode::serialize(&message).unwrap());
        assert_eq!(tx_legacy_msg_b64, message_b64);

        // Test versioned message
        let tx_versioned_b64 = Factory.get_transaction(versioned_b64.clone()).unwrap();
        let tx_versioned: VersionedTransaction = bincode::deserialize(&base64::decode(&tx_versioned_b64).unwrap()).unwrap();
        let tx_versioned_msg_b64 = base64::encode(bincode::serialize(&tx_versioned.message).unwrap());
        let versioned_msg_b64 = base64::encode(bincode::serialize(&versioned_msg).unwrap());
        assert_eq!(tx_versioned_msg_b64, versioned_msg_b64);

        // Test invalid base64
        let invalid = Factory.get_transaction("not-base64".to_string());
        assert!(invalid.is_err());
    }

    #[test]
    fn test_append_signature_to_transaction() {
        let signer = generate_key_from_mnemonic(
            "ski seven shuffle amazing tooth net useful asthma drive crystal solar glare",
        );
        assert_eq!(
            signer.public_key.contents,
            "F7xVyQuLzvyUKbMQyrBHaqYGCzHWpmsocn8b7oRUyeC5"
        );
        assert_eq!(signer.contents, "3NpEaVRpJAQjCxa6RM9zVrBpC7B61ypQJxt8FQM41jKZLi5bDdUnn3yXHyyLcDuFcoQrbECDH7SEiPi2z4j9w9PT");

        let base_64_unsigned_tx = "AQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAAoT0cva94tdeUJahExxXl5yoYrk1nsxlCWaSqvYqqppa6YRmJQz2BPHWeODDKN8Qtx9iPTXJqZBIdulNKq1NcTf7jOzHsTv+PoomuqMlUwBYy4tdkkIzlRNaGW97xEb/2ErRbSgDpkuuIlAxLykw4mGod2nd6ziifou4usSCxcEWihty/B1SeAdNCE/corYRxO1txeHL6w4E8q5Y68xLI3bzW8pOWiySsYpNA+F1v5c0yUnlsvwURGDQhu0yesZsdXTkF98aN834lf+Eb3lP/B6CzO3Amsn1E2Gfu77h7qH/CGUn5HzpNhL87Ljrhp5ZwCQzOu6oNRrlR0hBxy7aLDv167AdbKQTIZo4sepWTNMK23MjtaNesDAMU08VgSaH3F4AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACMlyWPTiSJ8bs9ECkUjg2DC1oTmdr/EIQEjnvY2+n4WQMGRm/lIRcy/+ytunLDm+e8jOW7xfcSayxDmzpAAAAAu6nN/XmuJp8Db7aXm0uYp3KSexdu9rcZZXaHIfHx8uTsgRBREqJX1h30z18T7gobAZGXyMU0O08qfsiEauIsGu8Ni2/aLOukHaFdQJXR2jkqDS+O0MbHvA9M+sjCgLVtBpuIV/6rgYT7aH9jRhjANdrEOdwa6ztVmKDwAAAAAAEGm4uYWqtTKkUJDehVf83cvmy378c6CmWwb5IDXbc+7Aan1RcZLFxRIYzJTD1K8X9Y2u4Im6H9ROPb2YoAAAAABt324ddloZPZy+FGzut5rBy0he1fWzeROoz1hX7/AKlmcm3QJD5/XiE4nDDqNootQizJhe85xH/7f7PQFMjncAkLAAUC4JMEAAsACQOVdQAAAAAAAAkCAAF8AwAAANHL2veLXXlCWoRMcV5ecqGK5NZ7MZQlmkqr2KqqaWumIAAAAAAAAAA0VXBEMmZoN3hIM1ZQOVFRYVh0c1MxWVkzYnh6V2h0ZsCnlwAAAAAAFAUAAAAAAAAGm4uYWqtTKkUJDehVf83cvmy378c6CmWwb5IDXbc+7BAFAQIAERIBBgkCAAgMAgAAAJCkIAAAAAAACgcACAAPCRIRAAoHAAYAAwkSEQAQDggGBAUDAgwHAQAODQASCQ6ghgEAAAAAABIDCAAAAQk=".to_string();

        let expected_base64_signed_tx = "AYTk34Oql2cYQmaF+V5kRmhk3snfBwWCsSaFrpUKojPDG0tseRVrPy4mDPBf7W2dP+ipfw4mm6eubsMT17cKdwUBAAoT0cva94tdeUJahExxXl5yoYrk1nsxlCWaSqvYqqppa6YRmJQz2BPHWeODDKN8Qtx9iPTXJqZBIdulNKq1NcTf7jOzHsTv+PoomuqMlUwBYy4tdkkIzlRNaGW97xEb/2ErRbSgDpkuuIlAxLykw4mGod2nd6ziifou4usSCxcEWihty/B1SeAdNCE/corYRxO1txeHL6w4E8q5Y68xLI3bzW8pOWiySsYpNA+F1v5c0yUnlsvwURGDQhu0yesZsdXTkF98aN834lf+Eb3lP/B6CzO3Amsn1E2Gfu77h7qH/CGUn5HzpNhL87Ljrhp5ZwCQzOu6oNRrlR0hBxy7aLDv167AdbKQTIZo4sepWTNMK23MjtaNesDAMU08VgSaH3F4AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACMlyWPTiSJ8bs9ECkUjg2DC1oTmdr/EIQEjnvY2+n4WQMGRm/lIRcy/+ytunLDm+e8jOW7xfcSayxDmzpAAAAAu6nN/XmuJp8Db7aXm0uYp3KSexdu9rcZZXaHIfHx8uTsgRBREqJX1h30z18T7gobAZGXyMU0O08qfsiEauIsGu8Ni2/aLOukHaFdQJXR2jkqDS+O0MbHvA9M+sjCgLVtBpuIV/6rgYT7aH9jRhjANdrEOdwa6ztVmKDwAAAAAAEGm4uYWqtTKkUJDehVf83cvmy378c6CmWwb5IDXbc+7Aan1RcZLFxRIYzJTD1K8X9Y2u4Im6H9ROPb2YoAAAAABt324ddloZPZy+FGzut5rBy0he1fWzeROoz1hX7/AKlmcm3QJD5/XiE4nDDqNootQizJhe85xH/7f7PQFMjncAkLAAUC4JMEAAsACQOVdQAAAAAAAAkCAAF8AwAAANHL2veLXXlCWoRMcV5ecqGK5NZ7MZQlmkqr2KqqaWumIAAAAAAAAAA0VXBEMmZoN3hIM1ZQOVFRYVh0c1MxWVkzYnh6V2h0ZsCnlwAAAAAAFAUAAAAAAAAGm4uYWqtTKkUJDehVf83cvmy378c6CmWwb5IDXbc+7BAFAQIAERIBBgkCAAgMAgAAAJCkIAAAAAAACgcACAAPCRIRAAoHAAYAAwkSEQAQDggGBAUDAgwHAQAODQASCQ6ghgEAAAAAABIDCAAAAQk=".to_string();

        let new_signature = "3f75BQ998yqJbEqMo78TSTMJk7phRZha1q298t7FbSUi54kPCrLv4yrBrQdE7tUEmBTLUAswjrMVAGgxpDyXAHzL".to_string();

        // Test successful signature append
        let appended_tx = Factory
            .append_signature_to_transaction(
                signer.public_key.contents.clone(),
                new_signature.clone(),
                base_64_unsigned_tx.clone(),
            )
            .unwrap();

        assert_eq!(appended_tx, expected_base64_signed_tx);

        let transaction_bytes = from_base64(&appended_tx).unwrap();
        let versioned_tx: VersionedTransaction = bincode::deserialize(&transaction_bytes).unwrap();

        assert_eq!(versioned_tx.signatures.len(), 1);
        assert_eq!(versioned_tx.signatures[0].to_string(), new_signature);

        // Test invalid base64 transaction
        let result = Factory.append_signature_to_transaction(
            signer.public_key.contents.clone(),
            new_signature.clone(),
            "invalid base64".to_string(),
        );
        assert!(result.is_err());

        // Test invalid signature
        let result = Factory.append_signature_to_transaction(
            signer.public_key.contents,
            "invalid signature".to_string(),
            base_64_unsigned_tx,
        );
        assert!(result.is_err());
    }
}
