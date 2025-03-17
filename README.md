# Solana Rust DEX

A decentralized exchange (DEX) built with Rust on the Solana blockchain.

## Features

- Order book based trading
- Support for SPL tokens
- Low transaction fees
- High throughput
- Secure and decentralized

## Architecture

The DEX is built as a Solana program (smart contract) with the following components:

1. **Order Book**: Manages buy and sell orders
2. **Matching Engine**: Matches compatible buy and sell orders
3. **Settlement**: Handles token transfers between traders
4. **Fee System**: Collects and distributes trading fees

## Getting Started

### Prerequisites

- Rust and Cargo
- Solana CLI
- Node.js and npm (for client applications)

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/solana-rust-dex.git
cd solana-rust-dex

# Build the program
cargo build-sbf
```

### Deployment

```bash
# Deploy to Solana devnet
solana program deploy target/deploy/solana_rust_dex.so --keypair path/to/keypair.json
```

## Development Roadmap

- [x] Project setup
- [ ] Basic order book implementation
- [ ] Token deposit and withdrawal
- [ ] Order matching engine
- [ ] Fee system
- [ ] Client SDK
- [ ] Web interface

## License

This project is licensed under the MIT License - see the LICENSE file for details.
