
use solana_sdk::{
    pubkey,
    pubkey::Pubkey,
    transaction::VersionedTransaction,
};

use crate::errors::*;

const JUPITER_V6_PROGRAM_ID: Pubkey = pubkey!("JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4");
const ROUTE_DISCRIMINATOR: &[u8] = &[229, 23, 203, 151, 122, 227, 173, 42];
const SHARED_ACCOUNTS_ROUTE_DISCRIMINATOR: &[u8] = &[193, 32, 155, 51, 65, 214, 156, 129];
const EXACT_OUT_ROUTE_DISCRIMINATOR: &[u8] = &[208, 51, 239, 151, 123, 43, 237, 92];
const SHARED_ACCOUNTS_EXACT_OUT_DISCRIMINATOR: &[u8] = &[176, 209, 105, 168, 154, 125, 69, 62];

pub(crate) fn mutate_transaction_slippage_bps(
    versioned_transaction: &mut VersionedTransaction,
    slippage_bps: u16,
) -> Result<(), TransactionError> {
    let target_program_id_index = versioned_transaction
        .message
        .static_account_keys()
        .iter()
        .position(|key| key == &JUPITER_V6_PROGRAM_ID)
        .ok_or(TransactionError::Generic(
            "Missing jupiter aggregator program id from static keys".into(),
        ))?;

    let mut jupiter_swap_ix_found = false;
    // Iterate through the instructions to find and modify the target instruction

    let compiled_instructions = match &mut versioned_transaction.message {
        solana_sdk::message::VersionedMessage::Legacy(legacy_message) => {
            &mut legacy_message.instructions
        }
        solana_sdk::message::VersionedMessage::V0(v0_message) => &mut v0_message.instructions,
    };

    for instruction in compiled_instructions.iter_mut() {
        if usize::from(instruction.program_id_index) == target_program_id_index
            && instruction.data.len() > 8 + 2 + 1
        // Discriminator length + slippage_bps + platform_bps + ...
        {
            if let Some(discriminator) = instruction.data.get(0..8) {
                match discriminator {
                    ROUTE_DISCRIMINATOR
                    | SHARED_ACCOUNTS_ROUTE_DISCRIMINATOR
                    | SHARED_ACCOUNTS_EXACT_OUT_DISCRIMINATOR
                    | EXACT_OUT_ROUTE_DISCRIMINATOR => {
                        if jupiter_swap_ix_found {
                            return Err(TransactionError::Generic(
                                "Duplicate swap instruction".into(),
                            ));
                        }
                        jupiter_swap_ix_found = true;

                        let data_len = instruction.data.len();
                        match instruction.data.get_mut(data_len - 2 - 1..data_len - 1) {
                            Some(slippage_bps_slice) => slippage_bps_slice
                                .as_mut()
                                .copy_from_slice(&slippage_bps.to_le_bytes()),
                            None => {
                                return Err(TransactionError::Generic(
                                    "Failed to find slippage bps slice".into(),
                                ));
                            }
                        }
                    }
                    _ => (),
                }
            }
        }
    }
    if !jupiter_swap_ix_found {
        return Err(TransactionError::Generic(
            "Could not find swap instruction".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::from_base64;
    
    #[test]
    fn test_mutate_transaction_slippage_bps() {
        let base64_tx = "AQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACAAQAGCmb9xdrDtJYk7SvJmju4CpS8tgk++rcm6zvJ55YhNnkyFyMa9+i/QdXyfkMKzum7vNcYFEYFPWEHOkn7ubmPMy8uy3ly9YjP0u4bWlq58MCtylAkiN9u7LB/14O1R2UKGEtLpKDA2nb16o7DnkNeYpajr8pWfkX5+cYZej/F5CTJAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAADBkZv5SEXMv/srbpyw5vnvIzlu8X3EmssQ5s6QAAAAAR51VvyMcBu7nTFbs5oFQf9sbLeo/SOUQKxzaJWvBOPBt324ddloZPZy+FGzut5rBy0he1fWzeROoz1hX7/AKmMlyWPTiSJ8bs9ECkUjg2DC1oTmdr/EIQEjnvY2+n4WbQ/+if11/ZKdMCbHylYed5LCas238ndUUsyGqezjOXo4AY1NdAvbDuSSJJNK0yR9lJs7g4BkENiJvgeZ7c1JKcHBQAFAm5dAgAFAAkDBgAAAAAAAAAIBgACABEEBwEBBAIAAgwCAAAAAOH1BQAAAAAHAQIBEQYdBwACAwYTBgkGEA4QCwoCAxETDxAABwcSEA0MAQYj5RfLl3rjrSoBAAAAJmQAAQDh9QUAAAAA53bhAAAAAAAsAQAHAwIAAAEJAbaRFM1U56as5v3jHnktfIiBQXM0Thew4qJELNzQaM6RBqnMqM/R0AQlAhXN";
        let transaction_bytes = from_base64(&base64_tx).unwrap();
        let mut transaction: VersionedTransaction =
            bincode::deserialize(&transaction_bytes).unwrap();
        assert_eq!(
            (),
            mutate_transaction_slippage_bps(&mut transaction, 12345).unwrap()
        );
    }
}