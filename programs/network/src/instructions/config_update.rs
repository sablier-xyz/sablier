use {
    crate::{constants::*, state::*},
    anchor_lang::prelude::*,
};

#[derive(Accounts)]
#[instruction(settings: ConfigSettings)]
pub struct ConfigUpdate<'info> {
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [SEED_CONFIG],
        bump,
        has_one = admin
    )]
    pub config: AccountLoader<'info, Config>,
}

pub fn handler(ctx: Context<ConfigUpdate>, settings: ConfigSettings) -> Result<()> {
    let config = &mut ctx.accounts.config;
    config.update(settings)
}
