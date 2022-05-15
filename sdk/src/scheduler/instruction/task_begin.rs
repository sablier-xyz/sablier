use anchor_lang::{
    solana_program::{
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
        sysvar,
    },
    InstructionData,
};

pub fn task_begin(delegate: Pubkey, queue: Pubkey, task: Pubkey) -> Instruction {
    Instruction {
        program_id: cronos_scheduler::ID,
        accounts: vec![
            AccountMeta::new_readonly(sysvar::clock::ID, false),
            AccountMeta::new(delegate, true),
            AccountMeta::new(queue, false),
            AccountMeta::new(task, false),
        ],
        data: cronos_scheduler::instruction::TaskBegin {}.data(),
    }
}