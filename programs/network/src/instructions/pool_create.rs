use {
    crate::{constants::*, errors::*, state::*},
    anchor_lang::prelude::*,
    std::mem::size_of,
};

#[derive(Accounts)]
pub struct PoolCreate<'info> {
    #[account(address = config.load()?.admin)]
    pub admin: Signer<'info>,

    #[account(
        address = Config::pubkey(),
        has_one = admin
    )]
    pub config: AccountLoader<'info, Config>,

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        seeds = [
            SEED_POOL,
            registry.total_pools.to_be_bytes().as_ref(),
        ],
        bump,
        payer = payer,
        space = 8 + size_of::<Pool>() + size_of::<Pubkey>(),
    )]
    pub pool: Account<'info, Pool>,

    #[account(
        mut,
        seeds = [SEED_REGISTRY],
        bump,
        constraint = !registry.locked @ SablierError::RegistryLocked
    )]
    pub registry: Box<Account<'info, Registry>>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<PoolCreate>) -> Result<()> {
    // Get accounts
    let pool = &mut ctx.accounts.pool;
    let registry = &mut ctx.accounts.registry;

    // Initialize the pool account.
    pool.init(registry.total_pools)?;

    // Increment the registry's pool counter.
    registry.total_pools += 1;

    Ok(())
}
