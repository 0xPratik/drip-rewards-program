use crate::state::*;
use crate::ErrorCode;
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(end_date: i64, max_entries: u8)]
pub struct CreateRaffle<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(init,
        seeds = [b"raffle".as_ref(),authority.key().as_ref(),&end_date.to_le_bytes(),&max_entries.to_le_bytes()],
        bump,
        payer = authority,
        space = Raffle::BASE_LEN 
    )]
    pub raffle_account: Account<'info, Raffle>,
    pub system_program: Program<'info, System>,
}


pub fn handler(ctx: Context<CreateRaffle>,end_date: i64,max_entries: u8,collections: Vec<Collection>) -> Result<()> {

    if collections.len() > MAX_COLLECTIONS {
        return Err(error!(ErrorCode::CannotBeMoreThanThree));
    }

    if end_date < Clock::get().unwrap().unix_timestamp {
        return Err(error!(ErrorCode::WrongEndDate));
    }

    if max_entries <= 0 {
        return Err(error!(ErrorCode::WrongMaxEntries));
    }

    let raffle_account = &mut ctx.accounts.raffle_account;
    raffle_account.owner = ctx.accounts.authority.key();
    raffle_account.bump = ctx.bumps.raffle_account;
    raffle_account.end_date = end_date;
    raffle_account.max_entries = max_entries;
    raffle_account.collections = collections;
    raffle_account.sold_tickets = 0;



    Ok(())
}