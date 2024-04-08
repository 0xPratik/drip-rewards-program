use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Raffle Not Ended")]
    RaffleNotEnded,

    #[msg("Wrong Max Entries")]
    WrongMaxEntries,

    #[msg("Raffle already ended")]
    RaffleEnded,

    #[msg("Invalid VRF Account")]
    InvalidRaffleAccount,

    #[msg("Cannot request Winner Before Time")]
    NotAllowedToRequestWinner,

    #[msg("Not Allowed to Claim Early")]
    NotAllowedToClaimNow,

    #[msg("Random Account not Selected")]
    RandomNoNotSelected,

    #[msg("Not a Winner")]
    NoWinner,

    #[msg("All Tickets Sold")]
    AllSold,

    #[msg("end_date is in past")]
    WrongEndDate,

    #[msg("price cannot be zero")]
    PriceCannotBeZero,

    #[msg("Collection Cannot be more than 3")]
    CannotBeMoreThanThree,

    #[msg("All Winner's Declared")]
    AllWinnerDeclared,

    #[msg("All Winner's not Declared")]
    AllWinnersNotDeclared,

    #[msg("Random No Not Found")]
    RandomNotFound,

    #[msg("No Balance")]
    NoBalance,

    #[msg("No Tickets Sold")]
    NoTicketsSold,

    #[msg("No Eligible Entries")]
    NoEligibleEntries,

    #[msg("No Reward item found")]
    NoRewardItemFound,
}
