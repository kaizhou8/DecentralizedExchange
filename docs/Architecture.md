# Solana Rust DEX - Architecture Document

This document outlines the architecture of the Solana Rust DEX (Decentralized Exchange) project, including its components, data structures, and interactions.

## System Overview

The Solana Rust DEX is a decentralized exchange built on the Solana blockchain. It enables users to trade tokens in a trustless, non-custodial manner with high throughput and low transaction costs.

### Key Features

- Order book based trading system
- Support for SPL tokens
- Limit orders with price-time priority
- Fee system with configurable rates
- Client library and CLI for easy interaction

## High-Level Architecture

The system consists of the following main components:

1. **On-chain Program (Smart Contract)**
   - Core DEX logic implemented as a Solana program
   - Handles all trading operations and state management

2. **Client Library**
   - Provides an API for interacting with the DEX program
   - Handles transaction creation and submission

3. **Command Line Interface (CLI)**
   - User-friendly interface for DEX operations
   - Built on top of the client library

## On-chain Program Architecture

The on-chain program follows a modular design with the following components:

### Entrypoint Module

- Entry point for all program invocations
- Receives and routes instructions to the processor

### Instruction Module

- Defines all supported instructions
- Handles instruction data serialization/deserialization
- Provides helper functions for creating instructions

### State Module

- Defines data structures for program state
- Implements serialization/deserialization for state data
- Includes:
  - Market structure
  - Order structure
  - Trade structure

### Processor Module

- Contains the business logic for processing instructions
- Implements handlers for each instruction type
- Validates inputs and manages state transitions

### Error Module

- Defines custom error types
- Provides error handling utilities

## Data Structures

### Market

```rust
pub struct Market {
    pub is_initialized: bool,
    pub authority: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub min_base_order_size: u64,
    pub tick_size: u64,
    pub fee_rate_bps: u16,
    pub next_order_id: u64,
    pub num_bids: u64,
    pub num_asks: u64,
}
```

The Market structure represents a trading pair (e.g., SOL/USDC) and contains:

- Authority: The account that has administrative privileges
- Base and quote token mints
- Trading parameters (minimum order size, tick size)
- Fee configuration
- Order book statistics

### Order

```rust
pub struct Order {
    pub is_initialized: bool,
    pub order_id: u64,
    pub owner: Pubkey,
    pub market: Pubkey,
    pub is_buy: bool,
    pub limit_price: u64,
    pub original_quantity: u64,
    pub remaining_quantity: u64,
    pub creation_timestamp: u64,
}
```

The Order structure represents a limit order and contains:

- Order ID and owner information
- Market reference
- Order type (buy/sell)
- Price and quantity information
- Timestamp for order prioritization

### Trade

```rust
pub struct Trade {
    pub maker_order_id: u64,
    pub taker_order_id: u64,
    pub maker: Pubkey,
    pub taker: Pubkey,
    pub price: u64,
    pub quantity: u64,
    pub taker_side: bool,
    pub timestamp: u64,
}
```

The Trade structure represents a completed trade between two orders and contains:

- Maker and taker information
- Order references
- Trade details (price, quantity)
- Timestamp

## Instruction Flow

### Initialize Market

1. Client creates a new market account
2. Client submits InitializeMarket instruction
3. Program validates inputs and initializes market state

### Place Limit Order

1. Client creates a new order account
2. Client submits PlaceLimitOrder instruction
3. Program validates inputs and creates the order
4. Program attempts to match the order with existing orders
5. If matches are found, trades are executed
6. Remaining order quantity is placed on the order book

### Cancel Order

1. Client submits CancelOrder instruction
2. Program validates that the caller is the order owner
3. Program removes the order from the order book
4. Program returns locked funds to the owner

### Settle Funds

1. Program executes this internally after trades
2. Transfers base tokens from seller to buyer
3. Transfers quote tokens from buyer to seller (minus fees)
4. Collects fees in the fee recipient account

## Client Architecture

### Client Library

The client library provides a high-level API for interacting with the DEX program:

- DexClient class with methods for all DEX operations
- Handles account creation and transaction building
- Provides utilities for querying market and order information

### CLI Tool

The CLI tool provides a command-line interface for:

- Market initialization
- Order placement and cancellation
- Market and order information queries

## Security Considerations

The DEX implements several security measures:

1. **Ownership Validation**
   - Orders can only be cancelled by their owners
   - Market parameters can only be modified by the authority

2. **Input Validation**
   - All instruction parameters are validated
   - Price and quantity constraints are enforced

3. **Arithmetic Safety**
   - All arithmetic operations use checked math to prevent overflows
   - Fee calculations handle potential precision loss

## Future Enhancements

Planned architectural improvements include:

1. **Advanced Order Types**
   - Market orders
   - Stop-limit orders
   - Fill-or-kill orders

2. **Optimized Order Matching**
   - More efficient data structures for the order book
   - Improved matching algorithm

3. **Enhanced Fee System**
   - Tiered fee structure
   - Fee discounts for token holders

4. **Oracle Integration**
   - Price feeds for advanced order types
   - Circuit breakers for market volatility

## Conclusion

The Solana Rust DEX architecture provides a solid foundation for a high-performance decentralized exchange. Its modular design allows for easy maintenance and future enhancements, while the client library and CLI tool provide convenient access for users and developers. 