use anchor_lang::{prelude::*, AnchorDeserialize};

use crate::{constants::SEED_WORKER, errors::*};

/// Worker
#[account]
#[derive(Debug, InitSpace)]
pub struct Worker {
    /// The worker's authority (owner).
    pub authority: Pubkey,
    /// The number of lamports claimable by the authority as commission for running the worker.
    pub commission_balance: u64,
    /// Integer between 0 and 100 determining the percentage of fees worker will keep as commission.
    pub commission_rate: u64,
    /// The worker's id.
    pub id: u64,
    /// The worker's signatory address (used to sign txs).
    pub signatory: Pubkey,
    /// The number delegations allocated to this worker.
    pub total_delegations: u64,
    pub bump: u8,
}

impl Worker {
    pub fn pubkey(id: u64) -> Pubkey {
        Pubkey::find_program_address(&[SEED_WORKER, id.to_be_bytes().as_ref()], &crate::ID).0
    }
}

/// WorkerSettings
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct WorkerSettings {
    pub commission_rate: u64,
    pub signatory: Pubkey,
}

/// WorkerAccount
pub trait WorkerAccount {
    fn pubkey(&self) -> Pubkey;

    fn init(&mut self, authority: &mut Signer, id: u64, signatory: &Signer, bump: u8)
        -> Result<()>;

    fn update(&mut self, settings: WorkerSettings) -> Result<()>;
}

impl WorkerAccount for Account<'_, Worker> {
    fn pubkey(&self) -> Pubkey {
        Worker::pubkey(self.id)
    }

    fn init(
        &mut self,
        authority: &mut Signer,
        id: u64,
        signatory: &Signer,
        bump: u8,
    ) -> Result<()> {
        self.authority = authority.key();
        self.commission_balance = 0;
        self.commission_rate = 0;
        self.id = id;
        self.signatory = signatory.key();
        self.total_delegations = 0;
        self.bump = bump;
        Ok(())
    }

    fn update(&mut self, settings: WorkerSettings) -> Result<()> {
        require!(
            settings.commission_rate > 0 && settings.commission_rate <= 100,
            SablierError::InvalidCommissionRate
        );
        self.commission_rate = settings.commission_rate;

        require!(
            settings.signatory.ne(&self.authority),
            SablierError::InvalidSignatory
        );
        self.signatory = settings.signatory;
        Ok(())
    }
}
