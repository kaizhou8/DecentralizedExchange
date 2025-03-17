// State module for the DEX program

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};
use std::convert::TryFrom;

/// Market state
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct Market {
    /// Is this market initialized
    pub is_initialized: bool,
    
    /// Market authority
    pub authority: Pubkey,
    
    /// Base token mint
    pub base_mint: Pubkey,
    
    /// Quote token mint
    pub quote_mint: Pubkey,
    
    /// Minimum base order size
    pub min_base_order_size: u64,
    
    /// Tick size (minimum price increment)
    pub tick_size: u64,
    
    /// Fee rate in basis points (1/100 of 1%)
    pub fee_rate_bps: u16,
    
    /// Next order ID
    pub next_order_id: u64,
    
    /// Number of bids in the order book
    pub num_bids: u64,
    
    /// Number of asks in the order book
    pub num_asks: u64,
}

impl Market {
    /// Calculate fee for a trade
    pub fn calculate_fee(&self, trade_value: u64) -> Result<u64, ProgramError> {
        // Calculate fee based on fee rate
        let fee = trade_value
            .checked_mul(self.fee_rate_bps as u64)
            .ok_or(ProgramError::ArithmeticOverflow)?
            .checked_div(10000)
            .ok_or(ProgramError::ArithmeticOverflow)?;
        
        Ok(fee)
    }
}

impl Sealed for Market {}

impl IsInitialized for Market {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Pack for Market {
    const LEN: usize = 1 + 32 + 32 + 32 + 8 + 8 + 2 + 8 + 8 + 8;
    
    fn pack_into_slice(&self, dst: &mut [u8]) {
        let mut slice = dst;
        self.serialize(&mut slice).unwrap();
    }
    
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        Self::try_from_slice(src).map_err(|_| ProgramError::InvalidAccountData)
    }
}

/// Order state
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct Order {
    /// Is this order initialized
    pub is_initialized: bool,
    
    /// Order ID
    pub order_id: u64,
    
    /// Owner of the order
    pub owner: Pubkey,
    
    /// Market this order belongs to
    pub market: Pubkey,
    
    /// Is this a buy order
    pub is_buy: bool,
    
    /// Limit price
    pub limit_price: u64,
    
    /// Original quantity
    pub original_quantity: u64,
    
    /// Remaining quantity
    pub remaining_quantity: u64,
    
    /// Creation timestamp
    pub creation_timestamp: u64,
}

impl Sealed for Order {}

impl IsInitialized for Order {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Pack for Order {
    const LEN: usize = 1 + 8 + 32 + 32 + 1 + 8 + 8 + 8 + 8;
    
    fn pack_into_slice(&self, dst: &mut [u8]) {
        let mut slice = dst;
        self.serialize(&mut slice).unwrap();
    }
    
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        Self::try_from_slice(src).map_err(|_| ProgramError::InvalidAccountData)
    }
}

/// Order book side enum
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub enum OrderBookSide {
    /// Bids (buy orders)
    Bids,
    /// Asks (sell orders)
    Asks,
}

/// Trade information
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct Trade {
    /// Maker order ID
    pub maker_order_id: u64,
    
    /// Taker order ID
    pub taker_order_id: u64,
    
    /// Maker
    pub maker: Pubkey,
    
    /// Taker
    pub taker: Pubkey,
    
    /// Trade price
    pub price: u64,
    
    /// Trade quantity
    pub quantity: u64,
    
    /// Trade side (true if taker is buyer)
    pub taker_side: bool,
    
    /// Timestamp
    pub timestamp: u64,
}
