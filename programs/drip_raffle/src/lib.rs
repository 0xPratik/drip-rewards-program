use anchor_lang::prelude::*;
pub mod errors;
pub mod instructions;
pub mod state;

pub use errors::ErrorCode;
pub use instructions::*;
pub use state::*;

declare_id!("GtuWk2HjbKSNCn3t15jeyXD4qCjd6hXHN38RYetgkwAd");

#[program]
pub mod drip_raffle {

    use super::*;

    pub fn create_raffle(
        ctx: Context<CreateRaffle>,
        end_date: i64,
        max_entries: u8,
        collections: Vec<Collection>,
    ) -> Result<()> {
        create_raffle::handler(ctx, end_date, max_entries, collections)
    }

    pub fn deposit_nft(ctx: Context<DepositSplReward>) -> Result<()> {
        deposit_nft_reward::handler(ctx)
    }

    pub fn create_entry<'info>(
        ctx: Context<'_, '_, '_, 'info, CreateEntry<'info>>,
        root: [u8; 32],
        data_hash: [u8; 32],
        creator_hash: [u8; 32],
        nonce: u64,
        index: u32,
    ) -> Result<()> {
        create_entry::handler(ctx, root, data_hash, creator_hash, nonce, index)
    }

    pub fn declare_winner(ctx: Context<DeclareWinner>) -> Result<()> {
        declare_winner::handler(ctx)
    }
}

#[derive(Accounts)]
pub struct Initialize {}
