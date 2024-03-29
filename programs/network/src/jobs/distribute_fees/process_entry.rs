use anchor_lang::{prelude::*, solana_program::instruction::Instruction, InstructionData};
use sablier_utils::thread::ThreadResponse;

use crate::{constants::*, state::*};

#[derive(Accounts)]
pub struct DistributeFeesProcessEntry<'info> {
    #[account(address = Config::pubkey())]
    pub config: AccountLoader<'info, Config>,

    #[account(
        mut,
        seeds = [
            SEED_DELEGATION,
            delegation.worker.as_ref(),
            delegation.id.to_be_bytes().as_ref(),
        ],
        bump,
        constraint = delegation.id == snapshot_entry.id,
        has_one = worker,
    )]
    pub delegation: Account<'info, Delegation>,

    #[account(
        mut,
        seeds = [
            SEED_FEE,
            fee.worker.as_ref(),
        ],
        bump,
        has_one = worker,
    )]
    pub fee: Account<'info, Fee>,

    #[account(address = Registry::pubkey())]
    pub registry: Account<'info, Registry>,

    #[account(
        address = snapshot.pubkey(),
        constraint = snapshot.id == registry.current_epoch
    )]
    pub snapshot: Account<'info, Snapshot>,

    #[account(
        address = snapshot_entry.pubkey(),
        has_one = snapshot_frame,
    )]
    pub snapshot_entry: Account<'info, SnapshotEntry>,

    #[account(
        address = snapshot_frame.pubkey(),
        has_one = snapshot,
        has_one = worker,
    )]
    pub snapshot_frame: Account<'info, SnapshotFrame>,

    #[account(address = config.load()?.epoch_thread)]
    pub thread: Signer<'info>,

    #[account(address = worker.pubkey())]
    pub worker: Account<'info, Worker>,
}

pub fn handler(ctx: Context<DistributeFeesProcessEntry>) -> Result<ThreadResponse> {
    // Get accounts
    let config = &ctx.accounts.config;
    let delegation = &mut ctx.accounts.delegation;
    let fee = &mut ctx.accounts.fee;
    let registry = &ctx.accounts.registry;
    let snapshot = &ctx.accounts.snapshot;
    let snapshot_entry = &ctx.accounts.snapshot_entry;
    let snapshot_frame = &ctx.accounts.snapshot_frame;
    let thread = &ctx.accounts.thread;
    let worker = &ctx.accounts.worker;

    // Calculate the balance of this particular delegation, based on the weight of its stake with this worker.
    let distribution_balance = if snapshot_frame.stake_amount > 0 {
        fee.distributable_balance * snapshot_entry.stake_amount / snapshot_frame.stake_amount
    } else {
        0
    };

    // Transfer yield to the worker.
    fee.sub_lamports(distribution_balance)?;
    delegation.add_lamports(distribution_balance)?;

    // Increment the delegation's yield balance.
    delegation.yield_balance += distribution_balance;

    // Build the next instruction for the thread.
    let dynamic_instruction = if (snapshot_entry.id + 1) < snapshot_frame.total_entries {
        // This frame has more entries. Move on to the next one.
        let next_delegation_pubkey = Delegation::pubkey(worker.key(), delegation.id + 1);
        let next_snapshot_entry_pubkey =
            SnapshotEntry::pubkey(snapshot_frame.key(), snapshot_entry.id + 1);
        Some(
            Instruction {
                program_id: crate::ID,
                accounts: crate::accounts::DistributeFeesProcessEntry {
                    config: config.key(),
                    delegation: next_delegation_pubkey,
                    fee: fee.key(),
                    registry: registry.key(),
                    snapshot: snapshot.key(),
                    snapshot_entry: next_snapshot_entry_pubkey,
                    snapshot_frame: snapshot_frame.key(),
                    thread: thread.key(),
                    worker: worker.key(),
                }
                .to_account_metas(Some(true)),
                data: crate::instruction::DistributeFeesProcessEntry {}.data(),
            }
            .into(),
        )
    } else if (snapshot_frame.id + 1) < snapshot.total_frames {
        // This frame has no more entries. Move on to the next worker.
        let next_worker_pubkey = Worker::pubkey(worker.id + 1);
        let next_snapshot_frame_pubkey =
            SnapshotFrame::pubkey(snapshot.key(), snapshot_frame.id + 1);
        Some(
            Instruction {
                program_id: crate::ID,
                accounts: crate::accounts::DistributeFeesProcessFrame {
                    config: config.key(),
                    fee: Fee::pubkey(next_worker_pubkey),
                    registry: registry.key(),
                    snapshot: snapshot.key(),
                    snapshot_frame: next_snapshot_frame_pubkey,
                    thread: thread.key(),
                    worker: next_worker_pubkey,
                }
                .to_account_metas(Some(true)),
                data: crate::instruction::DistributeFeesProcessFrame {}.data(),
            }
            .into(),
        )
    } else {
        None
    };

    Ok(ThreadResponse {
        dynamic_instruction,
        close_to: None,
        trigger: None,
    })
}
