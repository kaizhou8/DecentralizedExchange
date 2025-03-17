// Instruction module for the DEX program

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    pubkey::Pubkey,
    system_program,
    sysvar::rent,
};
use std::convert::TryInto;

// Instruction enum for the DEX program
#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, PartialEq)]
pub enum DexInstruction {
    /// Initialize a new market
    ///
    /// Accounts expected:
    /// 0. `[signer, writable]` Market authority account
    /// 1. `[writable]` Market account (uninitialized)
    /// 2. `[]` Base token mint
    /// 3. `[]` Quote token mint
    /// 4. `[]` Rent sysvar
    /// 5. `[]` System program
    InitializeMarket {
        /// Minimum order size in base token amount
        min_base_order_size: u64,
        /// Tick size in quote token amount (minimum price increment)
        tick_size: u64,
        /// Transaction fee rate in basis points (1/100 of 1%)
        fee_rate_bps: u16,
    },

    /// Place a limit order
    ///
    /// Accounts expected:
    /// 0. `[signer, writable]` Order owner account
    /// 1. `[writable]` Market account
    /// 2. `[writable]` Order account (uninitialized)
    /// 3. `[writable]` Owner's token account to debit
    /// 4. `[]` Token program
    /// 5. `[]` System program
    PlaceLimitOrder {
        /// Side of the order (true for buy, false for sell)
        is_buy: bool,
        /// Limit price in quote tokens
        limit_price: u64,
        /// Order quantity in base tokens
        quantity: u64,
        /// Self-trade behavior
        self_trade_behavior: SelfTradeBehavior,
    },

    /// Cancel an order
    ///
    /// Accounts expected:
    /// 0. `[signer]` Order owner account
    /// 1. `[writable]` Market account
    /// 2. `[writable]` Order account
    /// 3. `[writable]` Owner's token account to credit
    /// 4. `[]` Token program
    CancelOrder,

    /// Settle funds after a trade
    ///
    /// Accounts expected:
    /// 0. `[signer]` Authority account
    /// 1. `[writable]` Market account
    /// 2. `[writable]` Taker account
    /// 3. `[writable]` Maker account
    /// 4. `[writable]` Taker base token account
    /// 5. `[writable]` Taker quote token account
    /// 6. `[writable]` Maker base token account
    /// 7. `[writable]` Maker quote token account
    /// 8. `[writable]` Fee recipient account
    /// 9. `[]` Token program
    SettleFunds {
        /// Base token amount to settle
        base_amount: u64,
        /// Quote token amount to settle
        quote_amount: u64,
    },
}

/// Self-trade behavior enum
#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, PartialEq)]
pub enum SelfTradeBehavior {
    /// Decrement take (maker gets priority)
    DecrementTake,
    /// Cancel provide (taker gets priority)
    CancelProvide,
    /// Abort transaction
    AbortTransaction,
}

// Implementation of DexInstruction
impl DexInstruction {
    /// Create an initialize market instruction
    pub fn initialize_market(
        program_id: &Pubkey,
        market_authority: &Pubkey,
        market_account: &Pubkey,
        base_mint: &Pubkey,
        quote_mint: &Pubkey,
        min_base_order_size: u64,
        tick_size: u64,
        fee_rate_bps: u16,
    ) -> Result<Instruction, ProgramError> {
        // Create instruction data
        let data = DexInstruction::InitializeMarket {
            min_base_order_size,
            tick_size,
            fee_rate_bps,
        }
        .try_to_vec()?;

        // Create account metas
        let accounts = vec![
            AccountMeta::new(*market_authority, true),
            AccountMeta::new(*market_account, false),
            AccountMeta::new_readonly(*base_mint, false),
            AccountMeta::new_readonly(*quote_mint, false),
            AccountMeta::new_readonly(rent::id(), false),
            AccountMeta::new_readonly(system_program::id(), false),
        ];

        Ok(Instruction {
            program_id: *program_id,
            accounts,
            data,
        })
    }

    /// Create a place limit order instruction
    pub fn place_limit_order(
        program_id: &Pubkey,
        owner: &Pubkey,
        market: &Pubkey,
        order_account: &Pubkey,
        owner_token_account: &Pubkey,
        token_program: &Pubkey,
        is_buy: bool,
        limit_price: u64,
        quantity: u64,
        self_trade_behavior: SelfTradeBehavior,
    ) -> Result<Instruction, ProgramError> {
        // Create instruction data
        let data = DexInstruction::PlaceLimitOrder {
            is_buy,
            limit_price,
            quantity,
            self_trade_behavior,
        }
        .try_to_vec()?;

        // Create account metas
        let accounts = vec![
            AccountMeta::new(*owner, true),
            AccountMeta::new(*market, false),
            AccountMeta::new(*order_account, false),
            AccountMeta::new(*owner_token_account, false),
            AccountMeta::new_readonly(*token_program, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ];

        Ok(Instruction {
            program_id: *program_id,
            accounts,
            data,
        })
    }

    /// Create a cancel order instruction
    pub fn cancel_order(
        program_id: &Pubkey,
        owner: &Pubkey,
        market: &Pubkey,
        order_account: &Pubkey,
        owner_token_account: &Pubkey,
        token_program: &Pubkey,
    ) -> Result<Instruction, ProgramError> {
        // Create instruction data
        let data = DexInstruction::CancelOrder.try_to_vec()?;

        // Create account metas
        let accounts = vec![
            AccountMeta::new_readonly(*owner, true),
            AccountMeta::new(*market, false),
            AccountMeta::new(*order_account, false),
            AccountMeta::new(*owner_token_account, false),
            AccountMeta::new_readonly(*token_program, false),
        ];

        Ok(Instruction {
            program_id: *program_id,
            accounts,
            data,
        })
    }

    /// Create a settle funds instruction
    pub fn settle_funds(
        program_id: &Pubkey,
        authority: &Pubkey,
        market: &Pubkey,
        taker: &Pubkey,
        maker: &Pubkey,
        taker_base_account: &Pubkey,
        taker_quote_account: &Pubkey,
        maker_base_account: &Pubkey,
        maker_quote_account: &Pubkey,
        fee_recipient: &Pubkey,
        token_program: &Pubkey,
        base_amount: u64,
        quote_amount: u64,
    ) -> Result<Instruction, ProgramError> {
        // Create instruction data
        let data = DexInstruction::SettleFunds {
            base_amount,
            quote_amount,
        }
        .try_to_vec()?;

        // Create account metas
        let accounts = vec![
            AccountMeta::new_readonly(*authority, true),
            AccountMeta::new(*market, false),
            AccountMeta::new(*taker, false),
            AccountMeta::new(*maker, false),
            AccountMeta::new(*taker_base_account, false),
            AccountMeta::new(*taker_quote_account, false),
            AccountMeta::new(*maker_base_account, false),
            AccountMeta::new(*maker_quote_account, false),
            AccountMeta::new(*fee_recipient, false),
            AccountMeta::new_readonly(*token_program, false),
        ];

        Ok(Instruction {
            program_id: *program_id,
            accounts,
            data,
        })
    }
}
