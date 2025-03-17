// Error module for the DEX program

use solana_program::{
    program_error::ProgramError,
    msg,
};
use thiserror::Error;

// Custom error enum for the DEX program
#[derive(Error, Debug, Copy, Clone)]
pub enum DexError {
    // Invalid instruction data provided
    #[error("Invalid instruction data")]
    InvalidInstructionData,

    // Invalid account data provided
    #[error("Invalid account data")]
    InvalidAccountData,

    // Account not authorized for this operation
    #[error("Account not authorized")]
    AccountNotAuthorized,

    // Insufficient funds for the operation
    #[error("Insufficient funds")]
    InsufficientFunds,

    // Order not found
    #[error("Order not found")]
    OrderNotFound,

    // Invalid order price
    #[error("Invalid order price")]
    InvalidOrderPrice,

    // Invalid order size
    #[error("Invalid order size")]
    InvalidOrderSize,

    // Order book is full
    #[error("Order book is full")]
    OrderBookFull,

    // Invalid token account
    #[error("Invalid token account")]
    InvalidTokenAccount,

    // Arithmetic operation overflow
    #[error("Arithmetic overflow")]
    ArithmeticOverflow,
}

// Implement From trait to convert DexError to ProgramError
impl From<DexError> for ProgramError {
    fn from(e: DexError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

// Helper function to log and return errors
pub fn return_dex_error(error: DexError, msg_str: &str) -> ProgramError {
    msg!("Error: {}: {}", msg_str, error.to_string());
    error.into()
}
