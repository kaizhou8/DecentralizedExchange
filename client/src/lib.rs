// Solana Rust DEX Client Library

use solana_client::rpc_client::RpcClient;
use solana_program::{
    instruction::Instruction,
    program_pack::Pack,
    pubkey::Pubkey,
    system_instruction,
};
use solana_sdk::{
    commitment_config::CommitmentConfig,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use solana_rust_dex::{
    instruction::{DexInstruction, SelfTradeBehavior},
    state::{Market, Order},
};
use spl_token::state::Account as TokenAccount;
use spl_associated_token_account::get_associated_token_address;
use std::error::Error;

/// DEX client for interacting with the DEX program
pub struct DexClient {
    /// RPC client for communicating with the Solana cluster
    pub rpc_client: RpcClient,
    
    /// Program ID of the DEX program
    pub program_id: Pubkey,
}

impl DexClient {
    /// Create a new DEX client
    pub fn new(rpc_url: &str, program_id: Pubkey) -> Self {
        let rpc_client = RpcClient::new_with_commitment(
            rpc_url.to_string(),
            CommitmentConfig::confirmed(),
        );
        
        Self {
            rpc_client,
            program_id,
        }
    }
    
    /// Initialize a new market
    pub fn initialize_market(
        &self,
        payer: &Keypair,
        market_authority: &Keypair,
        market_account: &Keypair,
        base_mint: &Pubkey,
        quote_mint: &Pubkey,
        min_base_order_size: u64,
        tick_size: u64,
        fee_rate_bps: u16,
    ) -> Result<String, Box<dyn Error>> {
        // Create initialize market instruction
        let instruction = DexInstruction::initialize_market(
            &self.program_id,
            &market_authority.pubkey(),
            &market_account.pubkey(),
            base_mint,
            quote_mint,
            min_base_order_size,
            tick_size,
            fee_rate_bps,
        )?;
        
        // Create and send transaction
        let recent_blockhash = self.rpc_client.get_latest_blockhash()?;
        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&payer.pubkey()),
            &[payer, market_authority, market_account],
            recent_blockhash,
        );
        
        let signature = self.rpc_client.send_and_confirm_transaction(&transaction)?;
        Ok(signature.to_string())
    }
    
    /// Place a limit order
    pub fn place_limit_order(
        &self,
        payer: &Keypair,
        owner: &Keypair,
        market_pubkey: &Pubkey,
        order_account: &Keypair,
        owner_token_account: &Pubkey,
        token_program: &Pubkey,
        is_buy: bool,
        limit_price: u64,
        quantity: u64,
        self_trade_behavior: SelfTradeBehavior,
    ) -> Result<String, Box<dyn Error>> {
        // Create place limit order instruction
        let instruction = DexInstruction::place_limit_order(
            &self.program_id,
            &owner.pubkey(),
            market_pubkey,
            &order_account.pubkey(),
            owner_token_account,
            token_program,
            is_buy,
            limit_price,
            quantity,
            self_trade_behavior,
        )?;
        
        // Create and send transaction
        let recent_blockhash = self.rpc_client.get_latest_blockhash()?;
        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&payer.pubkey()),
            &[payer, owner, order_account],
            recent_blockhash,
        );
        
        let signature = self.rpc_client.send_and_confirm_transaction(&transaction)?;
        Ok(signature.to_string())
    }
    
    /// Cancel an order
    pub fn cancel_order(
        &self,
        payer: &Keypair,
        owner: &Keypair,
        market_pubkey: &Pubkey,
        order_account: &Pubkey,
        owner_token_account: &Pubkey,
        token_program: &Pubkey,
    ) -> Result<String, Box<dyn Error>> {
        // Create cancel order instruction
        let instruction = DexInstruction::cancel_order(
            &self.program_id,
            &owner.pubkey(),
            market_pubkey,
            order_account,
            owner_token_account,
            token_program,
        )?;
        
        // Create and send transaction
        let recent_blockhash = self.rpc_client.get_latest_blockhash()?;
        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&payer.pubkey()),
            &[payer, owner],
            recent_blockhash,
        );
        
        let signature = self.rpc_client.send_and_confirm_transaction(&transaction)?;
        Ok(signature.to_string())
    }
    
    /// Get market information
    pub fn get_market(&self, market_pubkey: &Pubkey) -> Result<Market, Box<dyn Error>> {
        let account = self.rpc_client.get_account(market_pubkey)?;
        let market = Market::unpack_from_slice(&account.data)?;
        Ok(market)
    }
    
    /// Get order information
    pub fn get_order(&self, order_pubkey: &Pubkey) -> Result<Order, Box<dyn Error>> {
        let account = self.rpc_client.get_account(order_pubkey)?;
        let order = Order::unpack_from_slice(&account.data)?;
        Ok(order)
    }
    
    /// Get token account information
    pub fn get_token_account(&self, token_account_pubkey: &Pubkey) -> Result<TokenAccount, Box<dyn Error>> {
        let account = self.rpc_client.get_account(token_account_pubkey)?;
        let token_account = TokenAccount::unpack_from_slice(&account.data)?;
        Ok(token_account)
    }
    
    /// Get associated token account address
    pub fn get_associated_token_account(&self, wallet_pubkey: &Pubkey, token_mint: &Pubkey) -> Pubkey {
        get_associated_token_address(wallet_pubkey, token_mint)
    }
} 