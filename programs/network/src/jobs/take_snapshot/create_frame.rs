use anchor_lang::{prelude::*, solana_program::instruction::Instruction, InstructionData};
use anchor_spl::{associated_token::get_associated_token_address, token::TokenAccount};
use sablier_utils::thread::{ThreadResponse, PAYER_PUBKEY};

use crate::{constants::*, state::*};

#[derive(Accounts)]
pub struct TakeSnapshotCreateFrame<'info> {
    #[account(address = Config::pubkey())]
    pub config: AccountLoader<'info, Config>,

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        address = Registry::pubkey(),
        constraint = registry.locked
    )]
    pub registry: Account<'info, Registry>,

    #[account(
        mut,
        seeds = [
            SEED_SNAPSHOT,
            snapshot.id.to_be_bytes().as_ref(),
        ],
        bump,
        constraint = (registry.current_epoch + 1) == snapshot.id,
        constraint = snapshot.total_frames < registry.total_workers,
    )]
    pub snapshot: Account<'info, Snapshot>,

    #[account(
        init,
        seeds = [
            SEED_SNAPSHOT_FRAME,
            snapshot.key().as_ref(),
            snapshot.total_frames.to_be_bytes().as_ref(),
        ],
        bump,
        payer = payer,
        space = 8 + SnapshotFrame::INIT_SPACE,
    )]
    pub snapshot_frame: Account<'info, SnapshotFrame>,

    pub system_program: Program<'info, System>,

    #[account(address = config.load()?.epoch_thread)]
    pub thread: Signer<'info>,

    #[account(
        address = worker.pubkey(),
        constraint = worker.id == snapshot.total_frames,
    )]
    pub worker: Account<'info, Worker>,

    #[account(
        associated_token::authority = worker,
        associated_token::mint = config.load()?.mint,
    )]
    pub worker_stake: Account<'info, TokenAccount>,
}

pub fn handler(ctx: Context<TakeSnapshotCreateFrame>) -> Result<ThreadResponse> {
    // Get accounts.
    let config_key = ctx.accounts.config.key();
    let config = &ctx.accounts.config.load()?;
    let registry = &ctx.accounts.registry;
    let snapshot = &mut ctx.accounts.snapshot;
    let snapshot_frame = &mut ctx.accounts.snapshot_frame;
    let system_program = &ctx.accounts.system_program;
    let thread = &ctx.accounts.thread;
    let worker = &ctx.accounts.worker;
    let worker_stake = &ctx.accounts.worker_stake;

    // Initialize snapshot frame account.
    snapshot_frame.init(
        snapshot.total_frames,
        snapshot.key(),
        worker_stake.amount,
        snapshot.total_stake,
        worker.key(),
    )?;

    // Update snapshot total workers.
    snapshot.total_stake += worker_stake.amount;
    snapshot.total_frames += 1;

    // Build the next instruction for the thread.
    let dynamic_instruction = if worker.total_delegations > 0 {
        // This worker has delegations. Create a snapshot entry for each delegation associated with this worker.
        let zeroth_delegation_pubkey = Delegation::pubkey(worker.pubkey(), 0);
        let zeroth_snapshot_entry_pubkey = SnapshotEntry::pubkey(snapshot_frame.key(), 0);
        Some(
            Instruction {
                program_id: crate::ID,
                accounts: crate::accounts::TakeSnapshotCreateEntry {
                    config: config_key,
                    delegation: zeroth_delegation_pubkey,
                    payer: PAYER_PUBKEY,
                    registry: registry.key(),
                    snapshot: snapshot.key(),
                    snapshot_entry: zeroth_snapshot_entry_pubkey,
                    snapshot_frame: snapshot_frame.key(),
                    system_program: system_program.key(),
                    thread: thread.key(),
                    worker: worker.key(),
                }
                .to_account_metas(Some(true)),
                data: crate::instruction::TakeSnapshotCreateEntry {}.data(),
            }
            .into(),
        )
    } else if snapshot.total_frames.lt(&registry.total_workers) {
        // This worker has no delegations. Create a snapshot frame for the next worker.
        let next_snapshot_frame_pubkey =
            SnapshotFrame::pubkey(snapshot.key(), snapshot_frame.id + 1);
        let next_worker_pubkey = Worker::pubkey(worker.id + 1);
        Some(
            Instruction {
                program_id: crate::ID,
                accounts: crate::accounts::TakeSnapshotCreateFrame {
                    config: config_key,
                    payer: PAYER_PUBKEY,
                    registry: registry.key(),
                    snapshot: snapshot.key(),
                    snapshot_frame: next_snapshot_frame_pubkey,
                    system_program: system_program.key(),
                    thread: thread.key(),
                    worker: next_worker_pubkey,
                    worker_stake: get_associated_token_address(&next_worker_pubkey, &config.mint),
                }
                .to_account_metas(Some(true)),
                data: crate::instruction::TakeSnapshotCreateFrame {}.data(),
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
