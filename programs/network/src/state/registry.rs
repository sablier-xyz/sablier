use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use anchor_lang::{prelude::*, AnchorDeserialize};

use crate::constants::SEED_REGISTRY;

/// Registry

#[account]
#[derive(Debug, InitSpace)]
pub struct Registry {
    pub current_epoch: u64,
    pub locked: bool,
    pub nonce: u64,
    pub total_pools: u64,
    pub total_unstakes: u64,
    pub total_workers: u64,
    pub bump: u8,
}

impl Registry {
    pub fn pubkey() -> Pubkey {
        Pubkey::find_program_address(&[SEED_REGISTRY], &crate::ID).0
    }
}

/**
 * RegistryAccount
 */

pub trait RegistryAccount {
    fn init(&mut self, bump: u8) -> Result<()>;

    fn hash_nonce(&mut self) -> Result<()>;
}

impl RegistryAccount for Account<'_, Registry> {
    fn init(&mut self, bump: u8) -> Result<()> {
        self.current_epoch = 0;
        self.locked = false;
        self.total_workers = 0;
        self.bump = bump;
        Ok(())
    }

    fn hash_nonce(&mut self) -> Result<()> {
        let mut hasher = DefaultHasher::new();
        Clock::get()?.slot.hash(&mut hasher);
        self.nonce.hash(&mut hasher);
        self.nonce = hasher.finish();
        Ok(())
    }
}
