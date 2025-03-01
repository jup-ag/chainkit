use crate::types::DecimalNumber;
use std::error::Error;

#[derive(Debug, thiserror::Error)]
pub enum KeyError {
    #[error("Invalid Keypair: {0}")]
    InvalidKeypair(String),
    #[error("Invalid Mnenomic: {0}")]
    InvalidMnemonic(String),
    #[error("Invalid DerivationPath: {0}")]
    DerivationPath(String),
    #[error("Something went wrong: {0}")]
    Generic(String),
    #[error("Invalid Private Key: {0}")]
    PrivateKey(String),
    #[error("Invalid Public Key: {0}")]
    PublicKey(String),
}

/// Some methods to quickly create an error from a given generic error
/// Can be used like
/// ```rust,ignore
/// failing_operation().map_err(KeyError::private_key)
/// ```
impl KeyError {
    pub fn keypair<E: Error>(error: E) -> Self {
        Self::InvalidKeypair(format!("{error:?}"))
    }

    pub fn mnemonic<E: Error>(error: E) -> Self {
        Self::InvalidMnemonic(format!("{error:?}"))
    }

    pub fn derivation<E: Error>(error: E) -> Self {
        Self::DerivationPath(format!("{error:?}"))
    }

    pub fn private_key<E: Error>(error: E) -> Self {
        Self::PrivateKey(format!("{error:?}"))
    }

    pub fn public_key<E: Error>(error: E) -> Self {
        Self::PublicKey(format!("{error:?}"))
    }

    pub fn generic(message: impl AsRef<str>) -> Self {
        Self::Generic(message.as_ref().to_string())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TransactionError {
    #[error("Invalid Keypair: {0}")]
    KeyPair(String),
    #[error("Signer Missing")]
    SignerMissing,
    #[error("Mutliple Signers is not currently supported")]
    MultipleSigners,
    #[error("Invalid PrivateKey: {0}")]
    PrivateKey(String),
    #[error("Invalid Transaction Parameters: {0}")]
    Parameters(String),
    #[error("Invalid PublicKey: {0}")]
    PublicKey(String),
    #[error("Invalid DecimalConversion for {0}: {1}")]
    DecimalConversion(String, String),
    #[error("Parsing Failure: {0}")]
    ParsingFailure(String),
    #[error("Instruction Error: {0}")]
    InstructionError(String),
    #[error("Generic Error: {0}")]
    Generic(String),
}

/// Some methods to quickly create an error from a given generic error
/// Can be used like
/// ```rust,ignore
/// failing_operation().map_err(TransactionError::private_key)
/// ```
impl TransactionError {
    pub fn keypair<E: Error>(error: E) -> Self {
        Self::KeyPair(format!("{error:?}"))
    }

    pub fn private_key<E: Error>(error: E) -> Self {
        Self::PrivateKey(format!("{error:?}"))
    }

    pub fn public_key<E: Error>(error: E) -> Self {
        Self::PublicKey(format!("{error:?}"))
    }

    pub fn parameters(error: impl AsRef<str>) -> Self {
        Self::Parameters(error.as_ref().to_string())
    }

    pub fn decimal<E: Error>(error: E, decimal: &DecimalNumber) -> Self {
        Self::DecimalConversion(decimal.value.to_string(), format!("{error:?}"))
    }

    pub fn parsing_failure<E: Error>(error: E) -> Self {
        Self::ParsingFailure(format!("{error:?}"))
    }

    pub fn instruction_error<E: Error>(error: E) -> Self {
        Self::InstructionError(format!("{error:?}"))
    }

    pub fn generic(message: impl AsRef<str>) -> Self {
        Self::Generic(message.as_ref().to_string())
    }

    pub fn generic_error<E: Error>(error: E) -> Self {
        Self::Generic(format!("{error:?}"))
    }
}

// Small error wrapper to go from box errors to a string formatted
// typed error
pub fn to_err<T, E>(
    a: impl Fn() -> Result<T, Box<dyn std::error::Error>>,
    b: impl Fn(String) -> E,
) -> Result<T, E> {
    a().map_err(|e| b(format!("{e:?}")))
}

#[derive(Debug, thiserror::Error)]
pub enum EncryptionError {
    #[error("Something went wrong: {0}")]
    Generic(String),
}

impl EncryptionError {
    pub fn generic_error<E: Error>(error: E) -> Self {
        Self::Generic(format!("{error:?}"))
    }

    pub fn generic_string(message: impl AsRef<str>) -> Self {
        Self::Generic(message.as_ref().to_string())
    }
}
