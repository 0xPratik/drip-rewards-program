use anchor_lang::prelude::{
    borsh::{BorshDeserialize, BorshSerialize},
    *,
};
pub const BITARRAY_BITS: usize = 64;
pub const MAX_TICKETS: usize = 7000;
pub const MAX_COLLECTIONS: usize = 3;

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone, Hash, PartialOrd)]
pub enum RewardType {
    MPLNFT,
    SPLTOKEN,
    SPLTOKEN22,
    CNFT,
    CoreAsset,
}

// for now let's keep weight as 1 2 3
#[derive(Default, Debug, Clone, AnchorSerialize, AnchorDeserialize, PartialEq, Eq)]
pub struct Collection {
    pub mint_address: Pubkey,
    pub weight: u8,
}

#[derive(Default, Debug, Clone, AnchorSerialize, AnchorDeserialize, PartialEq, Eq)]
pub struct RaffleReward {
    // asset Id
    pub reward: Pubkey,
    // random no from RNG
    pub random_no: u16,
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub enum RaffleRewardType {
    MPLNFT(RaffleReward),
    SPLTOKEN(RaffleReward),
    SPLTOKEN22(RaffleReward),
    CNFT(RaffleReward),
    CoreAsset(RaffleReward),
}

impl Default for RaffleRewardType {
    fn default() -> Self {
        RaffleRewardType::MPLNFT(RaffleReward::default())
    }
}

#[account]
#[derive(Default)]
pub struct Raffle {
    pub owner: Pubkey,
    pub bump: u8,
    pub entries: u8,
    pub end_date: i64,
    pub max_entries: u8,
    pub sold_tickets: u16,
    pub collections: Vec<Collection>, // 4 + 32 +
    pub reward: RaffleRewardType,
}

impl Raffle {
    pub const BASE_LEN: usize = 8 + std::mem::size_of::<Self>() + 4 + (MAX_COLLECTIONS * 36);
    pub const PREFIX: &'static str = "raffle";
}

#[account]
pub struct Entry {
    pub owner: Pubkey,
    pub tickets: BitArray,
}

impl Entry {
    pub const LEN: usize = 1024;

    pub const PREFIX: &'static str = "entry";
}

#[derive(Default, Debug, Clone, AnchorSerialize, AnchorDeserialize, PartialEq, Eq)]
pub struct BitArray {
    pub tickets: Vec<u64>, // 8
    pub amount: u16,       // 2
}

impl BitArray {
    pub fn init(&mut self) {
        self.amount = 0;
        // msg!("LL {:?}", MAX_TICKETS / BITARRAY_BITS);
        self.tickets = vec![0; (MAX_TICKETS / BITARRAY_BITS) + 1]; // 117 elements 936 bytes
                                                                   // msg!("L {:?}", self.tickets.len());
    }

    pub fn check_ticket(&self, i: usize) -> u64 {
        let r = ((i - 1) & (BITARRAY_BITS - 1)) + 1;
        (self.tickets[(i / BITARRAY_BITS) as usize] / (1 << (BITARRAY_BITS - r))) & 1
    }

    pub fn set_ticket(&mut self, i: usize) {
        let r = ((i - 1) & (BITARRAY_BITS - 1)) + 1;
        if (self.tickets[(i / BITARRAY_BITS) as usize] / (1 << (BITARRAY_BITS - r))) & 1 == 0 {
            self.tickets[(i / BITARRAY_BITS) as usize] += 1 << (BITARRAY_BITS - r);
        }
    }

    pub fn adquire_tickets(&mut self, _first_ticket: u16, _last_ticket: u16) {
        self.amount += (_last_ticket - _first_ticket) + 1;
        msg!("Amount is {}", self.amount);
        for i in _first_ticket as u32.._last_ticket as u32 + 1 {
            self.set_ticket(i as usize);
        }
    }
}
