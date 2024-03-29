use {
    crate::{constants::*, errors::*, state::*},
    anchor_lang::prelude::*,
};

#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct UnstakeCreate<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        seeds = [
            SEED_DELEGATION,
            delegation.worker.as_ref(),
            delegation.id.to_be_bytes().as_ref(),
        ],
        bump,
        has_one = authority,
        has_one = worker,
    )]
    pub delegation: Account<'info, Delegation>,

    #[account(
        mut,
        seeds = [SEED_REGISTRY],
        bump,
        constraint = !registry.locked @ SablierError::RegistryLocked
    )]
    pub registry: Account<'info, Registry>,

    pub system_program: Program<'info, System>,

    #[account(
        init,
        seeds = [
            SEED_UNSTAKE,
            registry.total_unstakes.to_be_bytes().as_ref(),
        ],
        bump,
        payer = authority,
        space = 8 + Unstake::INIT_SPACE,
    )]
    pub unstake: Account<'info, Unstake>,

    #[account(address = worker.pubkey())]
    pub worker: Account<'info, Worker>,
}

pub fn handler(ctx: Context<UnstakeCreate>, amount: u64) -> Result<()> {
    // Get accounts.
    let authority = &ctx.accounts.authority;
    let delegation = &ctx.accounts.delegation;
    let registry = &mut ctx.accounts.registry;
    let unstake = &mut ctx.accounts.unstake;
    let worker = &ctx.accounts.worker;

    // Validate the request is valid.
    require!(
        amount <= delegation.stake_amount,
        SablierError::InvalidUnstakeAmount
    );

    // Initialize the unstake account.
    unstake.init(
        amount,
        authority.key(),
        delegation.key(),
        registry.total_unstakes,
        worker.key(),
    )?;

    // Increment the registry's unstake counter.
    registry.total_unstakes += 1;

    Ok(())
}
