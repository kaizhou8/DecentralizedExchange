// Solana Rust DEX CLI

use clap::{App, Arg, SubCommand};
use solana_clap_utils::{
    input_parsers::{keypair_of, pubkey_of},
    input_validators::{is_keypair, is_pubkey, is_url},
};
use solana_program::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_rust_dex_client::DexClient;
use solana_rust_dex::instruction::SelfTradeBehavior;
use std::{error::Error, str::FromStr};

fn main() -> Result<(), Box<dyn Error>> {
    let matches = App::new("Solana Rust DEX CLI")
        .version("0.1.0")
        .author("Your Name <your.email@example.com>")
        .about("Command line interface for the Solana Rust DEX")
        .arg(
            Arg::with_name("url")
                .short("u")
                .long("url")
                .value_name("URL")
                .takes_value(true)
                .validator(is_url)
                .default_value("https://api.devnet.solana.com")
                .help("RPC URL to Solana cluster"),
        )
        .arg(
            Arg::with_name("program_id")
                .short("p")
                .long("program-id")
                .value_name("PUBKEY")
                .takes_value(true)
                .validator(is_pubkey)
                .default_value("DEX1111111111111111111111111111111111111111")
                .help("DEX program ID"),
        )
        .arg(
            Arg::with_name("fee_payer")
                .short("f")
                .long("fee-payer")
                .value_name("KEYPAIR")
                .takes_value(true)
                .validator(is_keypair)
                .help("Fee payer keypair"),
        )
        .subcommand(
            SubCommand::with_name("init-market")
                .about("Initialize a new market")
                .arg(
                    Arg::with_name("authority")
                        .long("authority")
                        .value_name("KEYPAIR")
                        .takes_value(true)
                        .validator(is_keypair)
                        .help("Market authority keypair"),
                )
                .arg(
                    Arg::with_name("market")
                        .long("market")
                        .value_name("KEYPAIR")
                        .takes_value(true)
                        .validator(is_keypair)
                        .help("Market account keypair"),
                )
                .arg(
                    Arg::with_name("base_mint")
                        .long("base-mint")
                        .value_name("PUBKEY")
                        .takes_value(true)
                        .validator(is_pubkey)
                        .help("Base token mint"),
                )
                .arg(
                    Arg::with_name("quote_mint")
                        .long("quote-mint")
                        .value_name("PUBKEY")
                        .takes_value(true)
                        .validator(is_pubkey)
                        .help("Quote token mint"),
                )
                .arg(
                    Arg::with_name("min_base_order_size")
                        .long("min-base-order-size")
                        .value_name("AMOUNT")
                        .takes_value(true)
                        .help("Minimum base token order size"),
                )
                .arg(
                    Arg::with_name("tick_size")
                        .long("tick-size")
                        .value_name("AMOUNT")
                        .takes_value(true)
                        .help("Minimum price increment in quote tokens"),
                )
                .arg(
                    Arg::with_name("fee_rate_bps")
                        .long("fee-rate-bps")
                        .value_name("BPS")
                        .takes_value(true)
                        .help("Fee rate in basis points (1/100 of 1%)"),
                ),
        )
        .subcommand(
            SubCommand::with_name("place-order")
                .about("Place a limit order")
                .arg(
                    Arg::with_name("owner")
                        .long("owner")
                        .value_name("KEYPAIR")
                        .takes_value(true)
                        .validator(is_keypair)
                        .help("Order owner keypair"),
                )
                .arg(
                    Arg::with_name("market")
                        .long("market")
                        .value_name("PUBKEY")
                        .takes_value(true)
                        .validator(is_pubkey)
                        .help("Market account pubkey"),
                )
                .arg(
                    Arg::with_name("order")
                        .long("order")
                        .value_name("KEYPAIR")
                        .takes_value(true)
                        .validator(is_keypair)
                        .help("Order account keypair"),
                )
                .arg(
                    Arg::with_name("token_account")
                        .long("token-account")
                        .value_name("PUBKEY")
                        .takes_value(true)
                        .validator(is_pubkey)
                        .help("Owner's token account pubkey"),
                )
                .arg(
                    Arg::with_name("token_program")
                        .long("token-program")
                        .value_name("PUBKEY")
                        .takes_value(true)
                        .validator(is_pubkey)
                        .default_value("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")
                        .help("Token program ID"),
                )
                .arg(
                    Arg::with_name("side")
                        .long("side")
                        .value_name("SIDE")
                        .takes_value(true)
                        .possible_values(&["buy", "sell"])
                        .help("Order side (buy or sell)"),
                )
                .arg(
                    Arg::with_name("price")
                        .long("price")
                        .value_name("PRICE")
                        .takes_value(true)
                        .help("Limit price in quote tokens"),
                )
                .arg(
                    Arg::with_name("quantity")
                        .long("quantity")
                        .value_name("QUANTITY")
                        .takes_value(true)
                        .help("Order quantity in base tokens"),
                )
                .arg(
                    Arg::with_name("self_trade_behavior")
                        .long("self-trade-behavior")
                        .value_name("BEHAVIOR")
                        .takes_value(true)
                        .possible_values(&["decrement-take", "cancel-provide", "abort"])
                        .default_value("decrement-take")
                        .help("Self-trade behavior"),
                ),
        )
        .subcommand(
            SubCommand::with_name("cancel-order")
                .about("Cancel an order")
                .arg(
                    Arg::with_name("owner")
                        .long("owner")
                        .value_name("KEYPAIR")
                        .takes_value(true)
                        .validator(is_keypair)
                        .help("Order owner keypair"),
                )
                .arg(
                    Arg::with_name("market")
                        .long("market")
                        .value_name("PUBKEY")
                        .takes_value(true)
                        .validator(is_pubkey)
                        .help("Market account pubkey"),
                )
                .arg(
                    Arg::with_name("order")
                        .long("order")
                        .value_name("PUBKEY")
                        .takes_value(true)
                        .validator(is_pubkey)
                        .help("Order account pubkey"),
                )
                .arg(
                    Arg::with_name("token_account")
                        .long("token-account")
                        .value_name("PUBKEY")
                        .takes_value(true)
                        .validator(is_pubkey)
                        .help("Owner's token account pubkey"),
                )
                .arg(
                    Arg::with_name("token_program")
                        .long("token-program")
                        .value_name("PUBKEY")
                        .takes_value(true)
                        .validator(is_pubkey)
                        .default_value("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")
                        .help("Token program ID"),
                ),
        )
        .subcommand(
            SubCommand::with_name("get-market")
                .about("Get market information")
                .arg(
                    Arg::with_name("market")
                        .long("market")
                        .value_name("PUBKEY")
                        .takes_value(true)
                        .validator(is_pubkey)
                        .help("Market account pubkey"),
                ),
        )
        .subcommand(
            SubCommand::with_name("get-order")
                .about("Get order information")
                .arg(
                    Arg::with_name("order")
                        .long("order")
                        .value_name("PUBKEY")
                        .takes_value(true)
                        .validator(is_pubkey)
                        .help("Order account pubkey"),
                ),
        )
        .get_matches();

    // Get common parameters
    let url = matches.value_of("url").unwrap();
    let program_id = Pubkey::from_str(matches.value_of("program_id").unwrap())?;
    let fee_payer = keypair_of(&matches, "fee_payer").unwrap_or_else(|| {
        Keypair::new() // Use a new keypair if not provided
    });

    // Create DEX client
    let client = DexClient::new(url, program_id);

    // Process subcommands
    match matches.subcommand() {
        ("init-market", Some(sub_matches)) => {
            let authority = keypair_of(sub_matches, "authority").expect("Authority keypair required");
            let market = keypair_of(sub_matches, "market").expect("Market keypair required");
            let base_mint = pubkey_of(sub_matches, "base_mint").expect("Base mint required");
            let quote_mint = pubkey_of(sub_matches, "quote_mint").expect("Quote mint required");
            let min_base_order_size = sub_matches
                .value_of("min_base_order_size")
                .expect("Minimum base order size required")
                .parse::<u64>()?;
            let tick_size = sub_matches
                .value_of("tick_size")
                .expect("Tick size required")
                .parse::<u64>()?;
            let fee_rate_bps = sub_matches
                .value_of("fee_rate_bps")
                .expect("Fee rate required")
                .parse::<u16>()?;

            let signature = client.initialize_market(
                &fee_payer,
                &authority,
                &market,
                &base_mint,
                &quote_mint,
                min_base_order_size,
                tick_size,
                fee_rate_bps,
            )?;

            println!("Market initialized successfully");
            println!("Market ID: {}", market.pubkey());
            println!("Transaction signature: {}", signature);
        }
        ("place-order", Some(sub_matches)) => {
            let owner = keypair_of(sub_matches, "owner").expect("Owner keypair required");
            let market = pubkey_of(sub_matches, "market").expect("Market pubkey required");
            let order = keypair_of(sub_matches, "order").expect("Order keypair required");
            let token_account = pubkey_of(sub_matches, "token_account").expect("Token account required");
            let token_program = pubkey_of(sub_matches, "token_program").unwrap();
            
            let side = sub_matches.value_of("side").expect("Side required");
            let is_buy = match side {
                "buy" => true,
                "sell" => false,
                _ => panic!("Invalid side"),
            };
            
            let price = sub_matches
                .value_of("price")
                .expect("Price required")
                .parse::<u64>()?;
                
            let quantity = sub_matches
                .value_of("quantity")
                .expect("Quantity required")
                .parse::<u64>()?;
                
            let self_trade_behavior = match sub_matches.value_of("self_trade_behavior").unwrap() {
                "decrement-take" => SelfTradeBehavior::DecrementTake,
                "cancel-provide" => SelfTradeBehavior::CancelProvide,
                "abort" => SelfTradeBehavior::AbortTransaction,
                _ => panic!("Invalid self-trade behavior"),
            };

            let signature = client.place_limit_order(
                &fee_payer,
                &owner,
                &market,
                &order,
                &token_account,
                &token_program,
                is_buy,
                price,
                quantity,
                self_trade_behavior,
            )?;

            println!("Order placed successfully");
            println!("Order ID: {}", order.pubkey());
            println!("Transaction signature: {}", signature);
        }
        ("cancel-order", Some(sub_matches)) => {
            let owner = keypair_of(sub_matches, "owner").expect("Owner keypair required");
            let market = pubkey_of(sub_matches, "market").expect("Market pubkey required");
            let order = pubkey_of(sub_matches, "order").expect("Order pubkey required");
            let token_account = pubkey_of(sub_matches, "token_account").expect("Token account required");
            let token_program = pubkey_of(sub_matches, "token_program").unwrap();

            let signature = client.cancel_order(
                &fee_payer,
                &owner,
                &market,
                &order,
                &token_account,
                &token_program,
            )?;

            println!("Order cancelled successfully");
            println!("Transaction signature: {}", signature);
        }
        ("get-market", Some(sub_matches)) => {
            let market_pubkey = pubkey_of(sub_matches, "market").expect("Market pubkey required");
            let market = client.get_market(&market_pubkey)?;

            println!("Market Information:");
            println!("  Authority: {}", market.authority);
            println!("  Base Mint: {}", market.base_mint);
            println!("  Quote Mint: {}", market.quote_mint);
            println!("  Min Base Order Size: {}", market.min_base_order_size);
            println!("  Tick Size: {}", market.tick_size);
            println!("  Fee Rate (bps): {}", market.fee_rate_bps);
            println!("  Next Order ID: {}", market.next_order_id);
            println!("  Number of Bids: {}", market.num_bids);
            println!("  Number of Asks: {}", market.num_asks);
        }
        ("get-order", Some(sub_matches)) => {
            let order_pubkey = pubkey_of(sub_matches, "order").expect("Order pubkey required");
            let order = client.get_order(&order_pubkey)?;

            println!("Order Information:");
            println!("  Order ID: {}", order.order_id);
            println!("  Owner: {}", order.owner);
            println!("  Market: {}", order.market);
            println!("  Side: {}", if order.is_buy { "Buy" } else { "Sell" });
            println!("  Limit Price: {}", order.limit_price);
            println!("  Original Quantity: {}", order.original_quantity);
            println!("  Remaining Quantity: {}", order.remaining_quantity);
            println!("  Creation Timestamp: {}", order.creation_timestamp);
        }
        _ => {
            println!("No command specified. Use --help for usage information.");
        }
    }

    Ok(())
} 