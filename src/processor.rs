// Processor module for the DEX program

use crate::{
    error::{return_dex_error, DexError},
    instruction::DexInstruction,
    state::{Market, Order},
};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack},
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};
use spl_token::instruction as token_instruction;

// Processor struct for handling instructions
pub struct Processor {}

impl Processor {
    // Process instruction entrypoint
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        // Deserialize instruction data
        let instruction = DexInstruction::try_from_slice(instruction_data)
            .map_err(|_| return_dex_error(DexError::InvalidInstructionData, "Failed to deserialize instruction"))?;

        // Process the appropriate instruction
        match instruction {
            DexInstruction::InitializeMarket {
                min_base_order_size,
                tick_size,
                fee_rate_bps,
            } => {
                msg!("Instruction: Initialize Market");
                Self::process_initialize_market(
                    program_id,
                    accounts,
                    min_base_order_size,
                    tick_size,
                    fee_rate_bps,
                )
            }
            DexInstruction::PlaceLimitOrder {
                is_buy,
                limit_price,
                quantity,
                self_trade_behavior,
            } => {
                msg!("Instruction: Place Limit Order");
                Self::process_place_limit_order(
                    program_id,
                    accounts,
                    is_buy,
                    limit_price,
                    quantity,
                    self_trade_behavior,
                )
            }
            DexInstruction::CancelOrder => {
                msg!("Instruction: Cancel Order");
                Self::process_cancel_order(program_id, accounts)
            }
            DexInstruction::SettleFunds {
                base_amount,
                quote_amount,
            } => {
                msg!("Instruction: Settle Funds");
                Self::process_settle_funds(program_id, accounts, base_amount, quote_amount)
            }
        }
    }

    // Process initialize market instruction
    fn process_initialize_market(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        min_base_order_size: u64,
        tick_size: u64,
        fee_rate_bps: u16,
    ) -> ProgramResult {
        // Get accounts
        let account_info_iter = &mut accounts.iter();
        let market_authority = next_account_info(account_info_iter)?;
        let market_account = next_account_info(account_info_iter)?;
        let base_mint = next_account_info(account_info_iter)?;
        let quote_mint = next_account_info(account_info_iter)?;
        let rent_account = next_account_info(account_info_iter)?;
        let system_program_account = next_account_info(account_info_iter)?;

        // Verify accounts
        if !market_authority.is_signer {
            return Err(return_dex_error(
                DexError::AccountNotAuthorized,
                "Market authority must sign",
            ));
        }

        // Verify program ownership
        if market_account.owner != program_id {
            // Create market account if it doesn't exist
            let rent = Rent::from_account_info(rent_account)?;
            let space = Market::LEN;
            let lamports = rent.minimum_balance(space);

            // Create account
            invoke(
                &system_instruction::create_account(
                    market_authority.key,
                    market_account.key,
                    lamports,
                    space as u64,
                    program_id,
                ),
                &[
                    market_authority.clone(),
                    market_account.clone(),
                    system_program_account.clone(),
                ],
            )?;
        }

        // Initialize market state
        let market = Market {
            is_initialized: true,
            authority: *market_authority.key,
            base_mint: *base_mint.key,
            quote_mint: *quote_mint.key,
            min_base_order_size,
            tick_size,
            fee_rate_bps,
            next_order_id: 1,
            num_bids: 0,
            num_asks: 0,
        };

        // Save market state
        market.pack_into_slice(&mut market_account.data.borrow_mut());

        msg!("Market initialized successfully");
        Ok(())
    }

    // Process place limit order instruction
    fn process_place_limit_order(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        is_buy: bool,
        limit_price: u64,
        quantity: u64,
        _self_trade_behavior: crate::instruction::SelfTradeBehavior,
    ) -> ProgramResult {
        // Get accounts
        let account_info_iter = &mut accounts.iter();
        let owner = next_account_info(account_info_iter)?;
        let market_account = next_account_info(account_info_iter)?;
        let order_account = next_account_info(account_info_iter)?;
        let owner_token_account = next_account_info(account_info_iter)?;
        let token_program = next_account_info(account_info_iter)?;
        let system_program_account = next_account_info(account_info_iter)?;

        // Verify accounts
        if !owner.is_signer {
            return Err(return_dex_error(
                DexError::AccountNotAuthorized,
                "Order owner must sign",
            ));
        }

        // Load market
        let mut market = Market::unpack_from_slice(&market_account.data.borrow())?;
        if !market.is_initialized {
            return Err(return_dex_error(
                DexError::InvalidAccountData,
                "Market not initialized",
            ));
        }

        // Validate order parameters
        if quantity < market.min_base_order_size {
            return Err(return_dex_error(
                DexError::InvalidOrderSize,
                "Order size below minimum",
            ));
        }

        if limit_price % market.tick_size != 0 {
            return Err(return_dex_error(
                DexError::InvalidOrderPrice,
                "Price not a multiple of tick size",
            ));
        }

        // Create order account if needed
        if order_account.owner != program_id {
            let rent = Rent::get()?;
            let space = Order::LEN;
            let lamports = rent.minimum_balance(space);

            // Create account
            invoke(
                &system_instruction::create_account(
                    owner.key,
                    order_account.key,
                    lamports,
                    space as u64,
                    program_id,
                ),
                &[
                    owner.clone(),
                    order_account.clone(),
                    system_program_account.clone(),
                ],
            )?;
        }

        // Get current timestamp
        let clock = Clock::get()?;
        let timestamp = clock.unix_timestamp as u64;

        // Create order
        let order = Order {
            is_initialized: true,
            order_id: market.next_order_id,
            owner: *owner.key,
            market: *market_account.key,
            is_buy,
            limit_price,
            original_quantity: quantity,
            remaining_quantity: quantity,
            creation_timestamp: timestamp,
        };

        // Save order
        order.pack_into_slice(&mut order_account.data.borrow_mut());

        // Update market
        market.next_order_id += 1;
        if is_buy {
            market.num_bids += 1;
        } else {
            market.num_asks += 1;
        }
        market.pack_into_slice(&mut market_account.data.borrow_mut());

        // Lock funds for the order
        if is_buy {
            // For buy orders, lock quote tokens (price * quantity)
            let amount = limit_price
                .checked_mul(quantity)
                .ok_or(ProgramError::ArithmeticOverflow)?;

            // Transfer tokens to program account
            invoke(
                &token_instruction::transfer(
                    token_program.key,
                    owner_token_account.key,
                    order_account.key,
                    owner.key,
                    &[],
                    amount,
                )?,
                &[
                    owner_token_account.clone(),
                    order_account.clone(),
                    owner.clone(),
                    token_program.clone(),
                ],
            )?;
        } else {
            // For sell orders, lock base tokens (quantity)
            // Transfer tokens to program account
            invoke(
                &token_instruction::transfer(
                    token_program.key,
                    owner_token_account.key,
                    order_account.key,
                    owner.key,
                    &[],
                    quantity,
                )?,
                &[
                    owner_token_account.clone(),
                    order_account.clone(),
                    owner.clone(),
                    token_program.clone(),
                ],
            )?;
        }

        msg!("Order placed successfully");
        Ok(())
    }

    // Process cancel order instruction
    fn process_cancel_order(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        // Get accounts
        let account_info_iter = &mut accounts.iter();
        let owner = next_account_info(account_info_iter)?;
        let market_account = next_account_info(account_info_iter)?;
        let order_account = next_account_info(account_info_iter)?;
        let owner_token_account = next_account_info(account_info_iter)?;
        let token_program = next_account_info(account_info_iter)?;

        // Verify accounts
        if !owner.is_signer {
            return Err(return_dex_error(
                DexError::AccountNotAuthorized,
                "Order owner must sign",
            ));
        }

        // Load order
        let order = Order::unpack_from_slice(&order_account.data.borrow())?;
        if !order.is_initialized {
            return Err(return_dex_error(
                DexError::InvalidAccountData,
                "Order not initialized",
            ));
        }

        // Verify owner
        if order.owner != *owner.key {
            return Err(return_dex_error(
                DexError::AccountNotAuthorized,
                "Not order owner",
            ));
        }

        // Load market
        let mut market = Market::unpack_from_slice(&market_account.data.borrow())?;
        if !market.is_initialized {
            return Err(return_dex_error(
                DexError::InvalidAccountData,
                "Market not initialized",
            ));
        }

        // Verify market
        if order.market != *market_account.key {
            return Err(return_dex_error(
                DexError::InvalidAccountData,
                "Order does not belong to this market",
            ));
        }

        // Return locked funds
        if order.is_buy {
            // For buy orders, return quote tokens (price * remaining quantity)
            let amount = order
                .limit_price
                .checked_mul(order.remaining_quantity)
                .ok_or(ProgramError::ArithmeticOverflow)?;

            // Transfer tokens back to owner
            invoke_signed(
                &token_instruction::transfer(
                    token_program.key,
                    order_account.key,
                    owner_token_account.key,
                    order_account.key,
                    &[],
                    amount,
                )?,
                &[
                    order_account.clone(),
                    owner_token_account.clone(),
                    order_account.clone(),
                    token_program.clone(),
                ],
                &[&[&order.order_id.to_le_bytes()]],
            )?;
        } else {
            // For sell orders, return base tokens (remaining quantity)
            // Transfer tokens back to owner
            invoke_signed(
                &token_instruction::transfer(
                    token_program.key,
                    order_account.key,
                    owner_token_account.key,
                    order_account.key,
                    &[],
                    order.remaining_quantity,
                )?,
                &[
                    order_account.clone(),
                    owner_token_account.clone(),
                    order_account.clone(),
                    token_program.clone(),
                ],
                &[&[&order.order_id.to_le_bytes()]],
            )?;
        }

        // Update market
        if order.is_buy {
            market.num_bids = market.num_bids.saturating_sub(1);
        } else {
            market.num_asks = market.num_asks.saturating_sub(1);
        }
        market.pack_into_slice(&mut market_account.data.borrow_mut());

        // Close order account
        // Zero out the data
        let mut data = order_account.data.borrow_mut();
        for byte in data.iter_mut() {
            *byte = 0;
        }

        msg!("Order cancelled successfully");
        Ok(())
    }

    // Process settle funds instruction
    fn process_settle_funds(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        base_amount: u64,
        quote_amount: u64,
    ) -> ProgramResult {
        // Get accounts
        let account_info_iter = &mut accounts.iter();
        let authority = next_account_info(account_info_iter)?;
        let market_account = next_account_info(account_info_iter)?;
        let taker_account = next_account_info(account_info_iter)?;
        let maker_account = next_account_info(account_info_iter)?;
        let taker_base_account = next_account_info(account_info_iter)?;
        let taker_quote_account = next_account_info(account_info_iter)?;
        let maker_base_account = next_account_info(account_info_iter)?;
        let maker_quote_account = next_account_info(account_info_iter)?;
        let fee_recipient_account = next_account_info(account_info_iter)?;
        let token_program = next_account_info(account_info_iter)?;

        // Verify accounts
        if !authority.is_signer {
            return Err(return_dex_error(
                DexError::AccountNotAuthorized,
                "Authority must sign",
            ));
        }

        // Load market
        let market = Market::unpack_from_slice(&market_account.data.borrow())?;
        if !market.is_initialized {
            return Err(return_dex_error(
                DexError::InvalidAccountData,
                "Market not initialized",
            ));
        }

        // Verify authority
        if market.authority != *authority.key {
            return Err(return_dex_error(
                DexError::AccountNotAuthorized,
                "Not market authority",
            ));
        }

        // Calculate fee
        let fee = market.calculate_fee(quote_amount)?;
        let quote_amount_after_fee = quote_amount.checked_sub(fee).ok_or(ProgramError::ArithmeticOverflow)?;

        // Transfer base tokens from seller to buyer
        invoke_signed(
            &token_instruction::transfer(
                token_program.key,
                maker_base_account.key,
                taker_base_account.key,
                market_account.key,
                &[],
                base_amount,
            )?,
            &[
                maker_base_account.clone(),
                taker_base_account.clone(),
                market_account.clone(),
                token_program.clone(),
            ],
            &[&[&market.authority.to_bytes()]],
        )?;

        // Transfer quote tokens from buyer to seller (minus fee)
        invoke_signed(
            &token_instruction::transfer(
                token_program.key,
                taker_quote_account.key,
                maker_quote_account.key,
                market_account.key,
                &[],
                quote_amount_after_fee,
            )?,
            &[
                taker_quote_account.clone(),
                maker_quote_account.clone(),
                market_account.clone(),
                token_program.clone(),
            ],
            &[&[&market.authority.to_bytes()]],
        )?;

        // Transfer fee to fee recipient
        if fee > 0 {
            invoke_signed(
                &token_instruction::transfer(
                    token_program.key,
                    taker_quote_account.key,
                    fee_recipient_account.key,
                    market_account.key,
                    &[],
                    fee,
                )?,
                &[
                    taker_quote_account.clone(),
                    fee_recipient_account.clone(),
                    market_account.clone(),
                    token_program.clone(),
                ],
                &[&[&market.authority.to_bytes()]],
            )?;
        }

        msg!("Funds settled successfully");
        Ok(())
    }
}
