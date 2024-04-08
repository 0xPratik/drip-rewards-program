use crate::state::*;
use crate::ErrorCode;
use anchor_lang::prelude::*;
use anchor_spl::token::Mint;
use mpl_bubblegum::instructions::VerifyLeafCpi;
use mpl_bubblegum::instructions::VerifyLeafCpiAccounts;
use mpl_bubblegum::instructions::VerifyLeafInstructionArgs;
use mpl_bubblegum::types::LeafSchema;
use mpl_bubblegum::utils::get_asset_id;

#[derive(Clone)]
pub struct MplBubblegum;
impl Id for MplBubblegum {
    fn id() -> Pubkey {
        mpl_bubblegum::ID
    }
}



#[derive(Accounts)]
pub struct CreateEntry<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(init,
        seeds = [b"entry".as_ref(),authority.key().as_ref(),raffle_account.key().as_ref()],
        bump,
        payer = authority,
        space = Entry::LEN 
    )]
    pub ticket_account: Account<'info,Entry>,
    pub raffle_account: Account<'info,Raffle>,
    #[account(
        constraint = cnft_collection.supply == 1, constraint = cnft_collection.decimals == 0
    )]
    pub cnft_collection: Account<'info, Mint>,
    ///CHECK: Checked in CPI
    pub merkle_tree: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
     ///CHECK: Checked in CPI
    pub log_wrapper: UncheckedAccount<'info>,
    /// CHECK: This account is checked in the CPI
    pub compression_program: UncheckedAccount<'info>,
    pub bubblegum_program: Program<'info, MplBubblegum>,
}



pub fn handler<'info>(
    ctx: Context<'_, '_, '_, 'info, CreateEntry<'info>>,  
    root: [u8; 32],
    data_hash: [u8; 32],
    creator_hash: [u8; 32],
    nonce: u64,
    index: u32
) -> Result<()> {
    let current_time = Clock::get()?.unix_timestamp;

    
    let raffle_account = &mut ctx.accounts.raffle_account;
    let ticket_account = &mut ctx.accounts.ticket_account;
    let collections = &raffle_account.collections;
    let mut weight: u8 = 1;

    if current_time > raffle_account.end_date {
        return err!(ErrorCode::RaffleEnded);
    }


    // for collection in collections.iter() {
    //     if collection.mint_address == ctx.accounts.cnft_collection.key() {
    //          weight = collection.weight;
    //     }
    // }

    if weight == 0 {
        return Err(error!(ErrorCode::NoEligibleEntries));
    }

    let leaf = LeafSchema::V1 {
        id: get_asset_id(&ctx.accounts.merkle_tree.key(), nonce),
        owner: ctx.accounts.authority.key(), 
        delegate: ctx.accounts.authority.key(),
        nonce: nonce, 
        data_hash: data_hash,
        creator_hash: creator_hash
     };
    

    VerifyLeafCpi::new(&ctx.accounts.bubblegum_program,VerifyLeafCpiAccounts{
        merkle_tree: &ctx.accounts.merkle_tree.to_account_info(),
    } , VerifyLeafInstructionArgs{
        index: index,
        root: root,
        leaf: leaf.hash()
    }).invoke_with_remaining_accounts( ctx.remaining_accounts
        .iter()
        .map(|account| (account, false, false))
        .collect::<Vec<_>>()
        .as_slice())?;






    let current_ticket_number = raffle_account.sold_tickets;
    msg!("no_of_tickets {}", current_ticket_number);
    let no_of_tickets = weight;

    if ticket_account.tickets.amount == 0 {
        msg!("INIT");
        ticket_account.tickets.init();
    }
    if no_of_tickets == 1 {
        let first_ticket = current_ticket_number.checked_add(1).unwrap();
        ticket_account
            .tickets
            .adquire_tickets(first_ticket, first_ticket);
    } else {
        let first_ticket = current_ticket_number.checked_add(1).unwrap();
        let last_ticket = first_ticket.checked_add(no_of_tickets.into()).unwrap() - 1;
        ticket_account
            .tickets
            .adquire_tickets(first_ticket, last_ticket);
    }

    raffle_account.entries = raffle_account.entries.checked_add(1).unwrap();
    raffle_account.sold_tickets = raffle_account.sold_tickets.checked_add(no_of_tickets.into()).unwrap();
    Ok(())
}