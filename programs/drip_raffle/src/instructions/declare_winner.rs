use crate::errors::ErrorCode;
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::sysvar;
use arrayref::array_ref;

#[derive(Accounts)]
pub struct DeclareWinner<'info> {
    #[account(mut)]
    pub raffle_account: Account<'info, Raffle>,
    /// CHECK: ITS RECENT SLOTHASHES
    #[account(address = sysvar::slot_hashes::id())]
    pub recent_slothashes: UncheckedAccount<'info>,
}

pub fn handler(ctx: Context<DeclareWinner>) -> Result<()> {
    // msg!("end_date is {}", ctx.accounts.raffle_account.end_date);
    // msg!("current time is {}", Clock::get().unwrap().unix_timestamp);
    // if ctx.accounts.raffle_account.end_date < Clock::get().unwrap().unix_timestamp {
    //     return Err(error!(ErrorCode::NotAllowedToRequestWinner));
    // }

    if ctx.accounts.raffle_account.sold_tickets == 0 {
        return Err(error!(ErrorCode::NoTicketsSold));
    }

    let reward_type = RewardType::MPLNFT;
    // let reward = ctx.accounts.raffle_account.reward;
    let reward = match ctx.accounts.raffle_account.reward {
        RaffleRewardType::MPLNFT(ref reward) => reward,
        _ => return Err(error!(ErrorCode::NoTicketsSold)), // Add this error to your ErrorCode enum
    };
    // for reward in ctx.accounts.raffle_account.rewards.iter() {
    //     if reward.random_no == 0 {
    //         is_all_drawn = false;
    //         break;
    //     }
    // }

    if reward.random_no != 0 {
        return Err(error!(ErrorCode::AllWinnerDeclared));
    }

    let recent_slothashes = &ctx.accounts.recent_slothashes;
    let data = recent_slothashes.data.borrow();
    let most_recent = array_ref![data, 12, 8];

    let clock = Clock::get()?;
    let seed = u64::from_le_bytes(*most_recent).saturating_sub(clock.unix_timestamp as u64);
    msg!("seed is {:?}", seed);
    let max_result = ctx.accounts.raffle_account.sold_tickets;
    let result = seed as u16 % max_result + 1;
    msg!("RESULT IS {}", result);
    let mut raffle_account = ctx.accounts.raffle_account.clone();

    raffle_account.reward = RaffleRewardType::MPLNFT(RaffleReward {
        reward: reward.reward,
        random_no: result,
    });

    msg!("RAFFLE REWARDS {:?}", raffle_account.reward);

    Ok(())
}
