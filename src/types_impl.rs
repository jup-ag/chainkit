//! Impl for the types in `types`. Separate to simplify the udl definitions
use std::str::FromStr;

use crate::types::*;

impl MnemonicWords {
    /// Join all words in the mnenomic vec with `space`
    pub fn joined(&self) -> String {
        self.words.join(" ")
    }
}

impl FromStr for MnemonicWords {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            words: s.split(' ').map(str::to_owned).collect(),
        })
    }
}

impl Derivation {
    pub fn iter(&self) -> impl Iterator<Item = u32> {
        self.start..(self.start + self.count)
    }
}

impl From<&str> for DecimalNumber {
    fn from(value: &str) -> Self {
        Self {
            value: value.to_string(),
        }
    }
}

impl From<String> for DecimalNumber {
    fn from(value: String) -> Self {
        Self { value }
    }
}

impl From<usize> for DecimalNumber {
    fn from(value: usize) -> Self {
        Self {
            value: format!("{value}"),
        }
    }
}

impl DecimalNumber {
    pub fn new(value: impl AsRef<str>) -> Self {
        Self {
            value: value.as_ref().to_string(),
        }
    }

    pub fn zero() -> Self {
        Self {
            value: "0".to_string(),
        }
    }
}

impl ChainPublicKey {
    pub fn new(address: impl AsRef<str>, chain: Blockchain) -> Self {
        Self {
            contents: address.as_ref().to_string(),
            chain,
        }
    }
}

impl ChainPrivateKey {
    pub fn new(private: impl AsRef<str>, address: impl AsRef<str>, chain: Blockchain) -> Self {
        Self {
            contents: private.as_ref().to_string(),
            public_key: ChainPublicKey::new(address, chain),
        }
    }
}
