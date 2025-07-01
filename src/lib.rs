uniffi::include_scaffolding!("interface");

pub mod solana;
pub mod encryption;
pub mod errors;
pub mod types;
pub mod types_impl;
mod utils;

use crate::solana::types::*;
use errors::*;
use types::*;

pub use encryption::{decrypt_ciphertext, encrypt_plaintext};

// UtilsFactory

pub fn generate_mnemonic(
    length: u32
) -> Result<MnemonicWords, KeyError> {
    solana::Factory
        .generate_mnemonic(length)
}

// PrivateKeyFactory

pub fn derive(
    chain: Blockchain,
    mnemonic: MnemonicWords,
    passphrase: Option<String>,
    derivation: Derivation,
) -> Result<Vec<DerivedPrivateKey>, KeyError> {
    chain
        .key_factory()
        .derive(mnemonic, passphrase.as_deref(), derivation)
}

pub fn derive_from_data(
    chain: Blockchain,
    data: String,
) -> Result<DerivedPrivateKey, KeyError> {
    chain.key_factory().derive_from_data(&data)
}

pub fn raw_private_key(chain: Blockchain, key: String) -> Result<ChainPrivateKey, KeyError> {
    chain.key_factory().raw_private_key(&key)
}

pub fn is_valid(chain: Blockchain, address: String) -> bool {
    chain.key_factory().is_valid(&address)
}

/// Tries to parse any string and return a private key for the correct
/// blockchain
pub fn parse_public_key(address: String) -> Option<ChainPublicKey> {
    let chain = if solana::Factory.is_valid(&address) {
        Blockchain::Solana
    } else {
        return None;
    };
    Some(ChainPublicKey {
        contents: address,
        chain,
    })
}

/// Tries to parse any data into a private key for a given blockchain
pub fn parse_private_key(key: impl AsRef<str>) -> Option<ChainPrivateKey> {
    let key = key.as_ref();
    if let Ok(valid) = solana::Factory.raw_private_key(key) {
        Some(valid)
    } else {
        None
    }
}

// TransactionFactory

pub fn send_transaction(
    chain: Blockchain,
    sender: ChainPublicKey,
    receiver: ChainPublicKey,
    amount: DecimalNumber,
    parameters: TransactionParameters,
) -> Result<String, TransactionError> {
    chain
        .tx_factory()
        .send_transaction(sender, receiver, amount, parameters)
}

pub fn token_transaction(
    chain: Blockchain,
    destination: TokenDestination,
    owner: ChainPublicKey,
    token: ChainPublicKey,
    kind: TransactionKind,
    parameters: TransactionParameters,
) -> Result<String, TransactionError> {
    chain
        .tx_factory()
        .token_transaction(destination, owner, token, kind, parameters)
}

pub fn sign_transaction(
    chain: Blockchain,
    transaction: String,
    signers: Vec<ChainPrivateKey>,
    parameters: Option<TransactionParameters>,
) -> Result<ChainTransaction, TransactionError> {
    chain
        .tx_factory()
        .sign_transaction(transaction, signers, parameters)
}

pub fn sign_message(
    chain: Blockchain,
    message: String,
    signers: Vec<ChainPrivateKey>,
) -> Result<String, TransactionError> {
    chain.tx_factory().sign_message(message, signers)
}

fn sign_typed_data(
    chain: Blockchain,
    typed_data: String,
    signers: Vec<ChainPrivateKey>,
) -> Result<String, TransactionError> {
    chain.tx_factory().sign_typed_data(typed_data, signers)
}

/// Parse the `transaction`, update it with the given `parameters`,
/// sign it again, and return the base64 encoded String
pub fn modify_transaction(
    chain: Blockchain,
    transaction: String,
    owner: ChainPrivateKey,
    parameters: TransactionParameters,
) -> Result<String, TransactionError> {
    chain
        .tx_factory()
        .modify_transaction(transaction, owner, parameters)
}

/// Parse the transaction and return the RLP data as base64
pub fn parse_transaction(
    chain: Blockchain,
    transaction: String,
) -> Result<ChainTransaction, TransactionError> {
    chain.tx_factory().parse_transaction(transaction)
}

pub fn get_associated_token_address(
    wallet_address: String,
    owner_program: String,
    token_mint_address: String,
) -> Result<ChainPublicKey, TransactionError> {
    solana::Factory.get_associated_token_address(wallet_address, owner_program, token_mint_address)
}

pub fn get_program_address(
    seeds: Vec<String>,
    program: String,
) -> Result<ChainPublicKey, TransactionError> {
    solana::Factory.get_program_address(seeds, program)
}

pub fn get_message(transaction: String) -> Result<String, TransactionError> {
    solana::Factory.get_message(transaction)
}

pub fn append_signature_to_transaction(signer: String, signature: String, transaction: String) -> Result<String, TransactionError> {
    solana::Factory.append_signature_to_transaction(signer, signature, transaction)
}

impl Blockchain {
    fn key_factory(&self) -> Box<dyn PrivateKeyFactory> {
        match self {
            Blockchain::Solana => Box::new(solana::Factory),
        }
    }

    fn tx_factory(&self) -> Box<dyn TransactionFactory> {
        match self {
            Blockchain::Solana => Box::new(solana::Factory),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_parse_public_keys() {
        assert_eq!(
            parse_public_key("HnXJX1Bvps8piQwDYEYC6oea9GEkvQvahvRj3c97X9xr".to_string())
                .unwrap()
                .chain,
            Blockchain::Solana
        );
    }

    #[test]
    fn test_invalid_public_keys() {
        assert!(parse_public_key("s".to_string()).is_none());
        assert!(parse_public_key("sh".to_string()).is_none());
        assert!(parse_public_key("sha".to_string()).is_none());
        assert!(parse_public_key("shaq".to_string()).is_none());
        assert!(parse_public_key("shaq.".to_string()).is_none());
        assert!(parse_public_key("shaq.s".to_string()).is_none());
        assert!(parse_public_key("shaq.so".to_string()).is_none());
        assert!(parse_public_key("shaq.sol".to_string()).is_none());
    }

    #[test]
    fn test_parse_private_key() {
        fn parse_key(key: &str) -> Option<Blockchain> {
            Some(parse_private_key(key)?.public_key.chain)
        }
        assert_eq!(
            parse_key("[27,153,159,181,6,1,91,15,197,226,231,97,95,7,137,92,71,179,37,198,230,114,5,253,107,33,44,63,48,96,131,124,8,144,230,241,171,179,101,73,241,150,248,129,215,137,57,221,119,238,150,90,248,94,202,188,207,238,162,84,174,209,99,96]").unwrap(),
            Blockchain::Solana
        );
        assert_eq!(
            parse_key("Z1JavLZ6voTNSNzunLw9TvtQroNnGb7ivfYur4iiJsM1TmAoWePYXNhXzkzLk95fBf6ZFj3jb461qeXWyMNdQUP").unwrap(),
            Blockchain::Solana
        );
        assert_eq!(
            parse_key("4Z7cXSyeFR8wNGMVXUE1TwtKn5D5Vu7FzEv69dokLv7KrQk7h6pu4LF8ZRR9yQBhc7uSM6RTTZtU1fmaxiNrxXrs").unwrap(),
            Blockchain::Solana
        );
        assert_eq!(parse_key("a a a a a a a a a a a a a a a a "), None);
    }
}
