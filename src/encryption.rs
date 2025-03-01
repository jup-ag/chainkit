use aes::Aes256;
use eax::{
    aead::{generic_array::GenericArray, Aead, KeyInit},
    Eax,
};
use hmac::digest::generic_array::ArrayLength;
use std::result::Result;

use crate::errors::EncryptionError;

type Aes256Eax = Eax<Aes256>;

/// We need a salt. Generated with `openssl rand -hex 8`
const HASH_SALT: &str = "4e3cefbd9d5831a3";

pub fn encrypt_plaintext(plaintext: String, password: String) -> Result<String, EncryptionError> {
    let State { cipher, nonce } = prepare_state(password.as_str())?;
    let ciphertext = cipher
        .encrypt(&nonce, plaintext.as_ref())
        .map_err(|e| EncryptionError::generic_string(format!("{e:?}")))?;
    Ok(crate::utils::to_base64(ciphertext))
}

pub fn decrypt_ciphertext(ciphertext: String, password: String) -> Result<String, EncryptionError> {
    let State { cipher, nonce } = prepare_state(password.as_str())?;
    let bytes = crate::utils::from_base64(&ciphertext)
        .map_err(|e| EncryptionError::generic_string(format!("Invalid Base64: {e:?}")))?;
    let bytes = cipher
        .decrypt(&nonce, bytes.as_ref())
        .map_err(|e| EncryptionError::generic_string(format!("{e:?}")))?;
    let plaintext = String::from_utf8(bytes).map_err(EncryptionError::generic_error)?;
    Ok(plaintext)
}

struct State<U: ArrayLength<u8>> {
    cipher: Aes256Eax,
    nonce: GenericArray<u8, U>,
}

/// Prepare the state that is required for both encryption and decryptoin
fn prepare_state<U: ArrayLength<u8>>(password: &str) -> Result<State<U>, EncryptionError> {
    let bytes: [u8; 32] = hashed_password(password)?;

    let key = GenericArray::from_slice(&bytes);
    let cipher = Aes256Eax::new(key);

    // We re-use the nonce for all encryptions
    let nonce: [u8; 16] = hashed_password("7a6f1d76af20316ece3016d66de2642e")?;

    let nonce = GenericArray::from_slice(&nonce).to_owned();
    Ok(State { cipher, nonce })
}

/// Hash a password into a certain amount of bytes as is required by Aes
fn hashed_password<const L: usize>(input: &str) -> Result<[u8; L], EncryptionError> {
    use hmac::Hmac;
    use pbkdf2::pbkdf2_array;
    use sha2::Sha256;

    const PBKDF2_ITERATIONS: u32 = 600_000;
    let res = pbkdf2_array::<Hmac<Sha256>, L>(
        input.as_bytes(),
        HASH_SALT.as_bytes(),
        PBKDF2_ITERATIONS
    )
    .map_err(EncryptionError::generic_error)?;
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_decryption() {
        let pw = "hunter123";
        let input = "古池や　蛙飛び込む　水の音. The old pond, a frog jumps in, sound of water. 👪 👨‍👩‍👦 ❤️";
        let ciphertext = encrypt_plaintext(input.to_string(), pw.to_string()).unwrap();
        let plaintext = decrypt_ciphertext(ciphertext.to_owned(), pw.to_string()).unwrap();
        assert_eq!(input, &plaintext);
    }

    #[test]
    fn test_encryption_stable() {
        let pw = "hunter123";
        let input = "Something Short";
        let ciphertext = encrypt_plaintext(input.to_string(), pw.to_string()).unwrap();
        assert_eq!(&ciphertext, "bl9g5SDAUVEg62aJFk/XuPcAtB1cB2ouYu1rfOXFSA==");
        let plaintext = decrypt_ciphertext(ciphertext.to_owned(), pw.to_string()).unwrap();
        assert_eq!(input, &plaintext);
    }
}
