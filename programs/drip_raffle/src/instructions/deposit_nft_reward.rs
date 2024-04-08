use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, Token, TokenAccount, Transfer},
};

#[derive(Accounts)]
pub struct DepositSplReward<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(mut,has_one = owner )]
    pub raffle_account: Account<'info, Raffle>,
    pub reward_mint: Account<'info, Mint>,
    #[
        account(
            init,
            associated_token::mint = reward_mint,
            payer = owner,
            associated_token::authority = raffle_account,
    )]
    pub reward_raffle_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::authority =  owner,
        associated_token::mint = reward_mint,
    )]
    pub reward_owner_account: Account<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn handler(ctx: Context<DepositSplReward>) -> Result<()> {
    let raffle_account = &mut ctx.accounts.raffle_account;
    let transfer_accounts = Transfer {
        authority: ctx.accounts.owner.to_account_info(),
        from: ctx.accounts.reward_owner_account.to_account_info(),
        to: ctx.accounts.reward_raffle_account.to_account_info(),
    };
    let token_program = ctx.accounts.token_program.to_account_info();
    let transfer_context = CpiContext::new(token_program, transfer_accounts);
    token::transfer(transfer_context, 1)?;

    raffle_account.reward = RaffleRewardType::MPLNFT(RaffleReward {
        reward: raffle_account.key(),
        random_no: 0,
    });

    msg!("REWARD {:?}", raffle_account.reward);
    Ok(())
}
