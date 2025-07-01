use crate::{errors::*, solana::types::*};
use serde::{Deserialize, Serialize};

pub trait UtilsFactory {
    /// Generates mnemonic for defined length and returns it as the list of words
    fn generate_mnemonic(
        &self,
        length: u32
    ) -> Result<MnemonicWords, KeyError>;
}

pub trait PrivateKeyFactory {
    /// Derives private keys in range
    fn derive(
        &self,
        mnemonic: MnemonicWords,
        passphrase: Option<&str>,
        derivation: Derivation,
    ) -> Result<Vec<DerivedPrivateKey>, KeyError>;

    /// Derives a single private key
    fn derive_from_data(&self, data: &str) -> Result<DerivedPrivateKey, KeyError>;

    /// Creates a private key from data
    /// The `data` is the ascii codepoints of the string making up the private key.
    fn raw_private_key(&self, key: &str) -> Result<ChainPrivateKey, KeyError>;

    /// Validate a given address.
    fn is_valid(&self, address: &str) -> bool;
}

pub trait TransactionFactory {
    fn send_transaction(
        &self,
        sender: ChainPublicKey,
        receiver: ChainPublicKey,
        amount: DecimalNumber,
        parameters: TransactionParameters,
    ) -> Result<String, TransactionError>;

    fn token_transaction(
        &self,
        destination: TokenDestination,
        owner: ChainPublicKey,
        token: ChainPublicKey,
        kind: TransactionKind,
        parameters: TransactionParameters,
    ) -> Result<String, TransactionError>;

    fn sign_transaction(
        &self,
        transaction: String,
        signers: Vec<ChainPrivateKey>,
        parameters: Option<TransactionParameters>,
    ) -> Result<ChainTransaction, TransactionError>;

    fn sign_message(
        &self,
        message: String,
        signers: Vec<ChainPrivateKey>,
    ) -> Result<String, TransactionError>;

    fn sign_typed_data(
        &self,
        typed_data: String,
        signers: Vec<ChainPrivateKey>,
    ) -> Result<String, TransactionError>;

    fn modify_transaction(
        &self,
        transaction: String,
        owner: ChainPrivateKey,
        parameters: TransactionParameters,
    ) -> Result<String, TransactionError>;

    fn parse_transaction(&self, transaction: String)
        -> Result<ChainTransaction, TransactionError>;

    fn get_associated_token_address(
        &self,
        wallet_address: String,
        owner_program: String,
        token_mint_address: String,
    ) -> Result<ChainPublicKey, TransactionError>;

    fn get_program_address(
        &self,
        seeds: Vec<String>,
        program: String,
    ) -> Result<ChainPublicKey, TransactionError>;

    fn get_message(&self, transaction: String) -> Result<String, TransactionError>;

    fn append_signature_to_transaction(&self, signer: String, signature: String, transaction: String) -> Result<String, TransactionError>;
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum Blockchain {
    Solana,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum DerivationPath {
    Bip44Root,
    Bip44,
    Bip44Change,
    Deprecated
}

impl DerivationPath {
    pub fn format(&self) -> &str {
        match self {
            DerivationPath::Bip44Root => "m/44'/501'",
            DerivationPath::Bip44 => "m/44'/501'/{}'",
            DerivationPath::Bip44Change => "m/44'/501'/{}'/0'",
            DerivationPath::Deprecated => "m/501'/{}'/0/0",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Derivation {
    pub start: u32,
    pub count: u32,
    pub path: DerivationPath,
}

impl Derivation {
    pub fn paths_with_index(&self) -> Vec<(u32, String)> {
        let mut paths:Vec<(u32, String)> = Vec::new();

        match self.path {
            DerivationPath::Bip44Root => {
                paths.push((0 as u32, self.path.format().to_string()))
            },
            _ => {
                for i in self.start..(self.start + self.count) {
                    let path = format!("{}", self.path.format().replace("{}", &i.to_string()));
                    paths.push((i, path));
                }
            }
        }

        paths
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MnemonicWords {
    pub words: Vec<String>,
}

/// Representation of a derived private key.
/// - `contents`: Hex representation of the private key data without `0x`
/// - `index`: The index of this derivation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DerivedPrivateKey {
    pub contents: String,
    pub public_key: ChainPublicKey,
    pub index: u32,
    pub path: Option<String>,
    pub path_type: Option<DerivationPath>,
}

/// Representation of a private key.
/// - `contents`:
/// Solana - Base58 representation of private key data
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChainPrivateKey {
    pub contents: String,
    pub public_key: ChainPublicKey,
}

/// Representation of a public key
/// - `contents` Hex representation of a public key **WITH** `0x`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainPublicKey {
    pub contents: String,
    pub chain: Blockchain,
}

/// Expose transaction internals that the client might need to
/// know about
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionData {
    Solana{
        signatures: Vec<String>
    },
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, Default)]
pub struct DecimalNumber {
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionParameters {
    Solana {
        external_address: Option<ExternalAddress>,
        transaction_type: SolanaTransactionType,
        owner_program: Option<String>,
        decimals: Option<u8>,
        memo: Option<String>,
        references: Vec<String>,
        swap_slippage_bps: Option<u16>,
        compute_budget_unit_price: Option<u64>,
        compute_budget_unit_limit: Option<u32>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SolanaTransactionType {
    Legacy,
    Versioned,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionKind {
    Token {
        amount: DecimalNumber,
        close_account: bool,
    },
    Nft {
        amount: u64,
        id: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TokenDestination {
    /// Token account transfer destination
    Account { transfer_destination: String },
    /// In case the destination wallet doesn't have an account to transfer the token,
    /// the transaction will create a new token account owned by a wallet public key
    /// and then the desired token will be transfered to the new account
    Wallet { public_key: ChainPublicKey },
}

/*
- The order of signatures matches the order of signer public keys in the account_keys list of the Message.
- Only the first signatures.len() entries in account_keys correspond to the signers of the transaction.
- Non-signing public keys (e.g., program accounts) appear later in the account_keys list.

   By following this logic, you can accurately determine the public key associated with each signature
   in the Transaction.
*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainTransaction {
    pub tx: String,
    pub signers: Vec<ChainPublicKey>,
    pub accounts: Vec<ChainPublicKey>,
    pub full_signature: Option<String>,
    pub signatures: Option<Vec<String>>,
    pub instruction_programs: Vec<String>,
}
