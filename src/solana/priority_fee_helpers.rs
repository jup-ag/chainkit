use solana_sdk::{
    compute_budget::{ ComputeBudgetInstruction, self },
    instruction::Instruction,
    instruction::CompiledInstruction,
    message::VersionedMessage,
    pubkey::Pubkey,
    transaction::VersionedTransaction,
};
use bincode::serialize;

pub(crate) fn add_compute_unit_limit(transaction: &mut VersionedTransaction, units: u32) {
    add(ComputeBudgetInstruction::SetComputeUnitLimit(units), transaction);
}

pub(crate) fn add_compute_unit_price(transaction: &mut VersionedTransaction, price: u64) {
    add(ComputeBudgetInstruction::SetComputeUnitPrice(price), transaction);
}

fn add(compute_budget_instruction: ComputeBudgetInstruction, transaction: &mut VersionedTransaction) {
    // Convert ComputeBudgetInstruction to Instruction
    let instruction = Instruction::new_with_bincode(
        compute_budget::id(),
        &compute_budget_instruction,
        vec![],
    );

    // Extract account keys and find or add the program ID index
    let (account_keys, program_id_index) = match &mut transaction.message {
        VersionedMessage::Legacy(legacy_message) => {
            if let Some(index) = find_account_key_index(&legacy_message.account_keys, &compute_budget::id()) {
                (&legacy_message.account_keys, index)
            } else {
                legacy_message.account_keys.push(compute_budget::id());
                let index = (legacy_message.account_keys.len() - 1) as u8;
                (&legacy_message.account_keys, index)
            }
        }
        VersionedMessage::V0(v0_message) => {
            if let Some(index) = find_account_key_index(&v0_message.account_keys, &compute_budget::id()) {
                (&v0_message.account_keys, index)
            } else {
                v0_message.account_keys.push(compute_budget::id());
                let index = (v0_message.account_keys.len() - 1) as u8;
                (&v0_message.account_keys, index)
            }
        }
    };

    // Manually convert Instruction to CompiledInstruction
    let compiled_instruction = CompiledInstruction {
        program_id_index,
        accounts: instruction
            .accounts
            .iter()
            .map(|meta| {
                find_account_key_index(account_keys, &meta.pubkey)
                    .expect("Account key not found in account keys")
            })
            .collect(),
        data: serialize(&instruction.data).unwrap(),
    };

    match &mut transaction.message {
        VersionedMessage::Legacy(legacy_message) => {
            legacy_message.instructions.insert(0, compiled_instruction);
        }
        VersionedMessage::V0(v0_message) => {
            v0_message.instructions.insert(0, compiled_instruction);
        }
    }
}

// Helper function to find the index of a public key in account keys
fn find_account_key_index(account_keys: &[Pubkey], key: &Pubkey) -> Option<u8> {
    account_keys.iter().position(|&k| k == *key).map(|i| i as u8)
}