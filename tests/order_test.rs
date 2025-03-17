// Order test module
// 订单测试模块

#[cfg(test)]
mod order_tests {
    use solana_program::{
        program_pack::Pack,
        pubkey::Pubkey,
    };
    use solana_program_test::*;
    use solana_sdk::{
        signature::{Keypair, Signer},
        transaction::Transaction,
    };
    use solana_rust_dex::{
        instruction::{DexInstruction, SelfTradeBehavior},
        state::{Market, Order},
    };

    async fn setup_market(
        program_id: &Pubkey,
        banks_client: &mut BanksClient,
        payer: &Keypair,
        recent_blockhash: &solana_sdk::hash::Hash,
    ) -> (Keypair, Keypair, Pubkey, Pubkey) {
        // Create accounts for the test
        // 为测试创建账户
        let market_authority = Keypair::new();
        let market_account = Keypair::new();
        let base_mint = Pubkey::new_unique();
        let quote_mint = Pubkey::new_unique();

        // Create initialize market instruction
        // 创建初始化市场指令
        let min_base_order_size = 100;
        let tick_size = 10;
        let fee_rate_bps = 25; // 0.25%

        let init_market_ix = DexInstruction::initialize_market(
            program_id,
            &market_authority.pubkey(),
            &market_account.pubkey(),
            &base_mint,
            &quote_mint,
            min_base_order_size,
            tick_size,
            fee_rate_bps,
        )
        .unwrap();

        // Create and sign transaction
        // 创建并签名交易
        let mut transaction = Transaction::new_with_payer(
            &[init_market_ix],
            Some(&payer.pubkey()),
        );
        transaction.sign(&[payer, &market_authority], *recent_blockhash);

        // Process transaction
        // 处理交易
        banks_client.process_transaction(transaction).await.unwrap();

        (market_authority, market_account, base_mint, quote_mint)
    }

    #[tokio::test]
    async fn test_place_limit_order() {
        // Create program test environment
        // 创建程序测试环境
        let program_id = Pubkey::new_unique();
        let mut program_test = ProgramTest::new(
            "solana_rust_dex",
            program_id,
            processor!(solana_rust_dex::entrypoint::process_instruction),
        );

        // Start the test environment
        // 启动测试环境
        let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

        // Setup market
        // 设置市场
        let (_, market_account, _, _) = setup_market(
            &program_id,
            &mut banks_client,
            &payer,
            &recent_blockhash,
        ).await;

        // Create order accounts
        // 创建订单账户
        let order_owner = Keypair::new();
        let order_account = Keypair::new();
        let owner_token_account = Pubkey::new_unique();
        let token_program = Pubkey::new_unique();

        // Add order account to the test environment
        // 将订单账户添加到测试环境
        program_test.add_account(
            order_account.pubkey(),
            solana_sdk::account::Account {
                lamports: 1000000000,
                data: vec![0; Order::LEN],
                owner: program_id,
                executable: false,
                rent_epoch: 0,
            },
        );

        // Create place limit order instruction
        // 创建下限价单指令
        let is_buy = true;
        let limit_price = 1000;
        let quantity = 500;
        let self_trade_behavior = SelfTradeBehavior::DecrementTake;

        let place_order_ix = DexInstruction::place_limit_order(
            &program_id,
            &order_owner.pubkey(),
            &market_account.pubkey(),
            &order_account.pubkey(),
            &owner_token_account,
            &token_program,
            is_buy,
            limit_price,
            quantity,
            self_trade_behavior,
        )
        .unwrap();

        // Create and sign transaction
        // 创建并签名交易
        let mut transaction = Transaction::new_with_payer(
            &[place_order_ix],
            Some(&payer.pubkey()),
        );
        transaction.sign(&[&payer, &order_owner], recent_blockhash);

        // Process transaction
        // 处理交易
        banks_client.process_transaction(transaction).await.unwrap();

        // Verify order state
        // 验证订单状态
        let order_account_data = banks_client
            .get_account(order_account.pubkey())
            .await
            .unwrap()
            .unwrap();

        let order = Order::unpack_from_slice(&order_account_data.data).unwrap();
        assert!(order.is_initialized);
        assert_eq!(order.order_id, 1);
        assert_eq!(order.owner, order_owner.pubkey());
        assert_eq!(order.market, market_account.pubkey());
        assert_eq!(order.is_buy, is_buy);
        assert_eq!(order.limit_price, limit_price);
        assert_eq!(order.original_quantity, quantity);
        assert_eq!(order.remaining_quantity, quantity);
    }

    #[tokio::test]
    async fn test_cancel_order() {
        // Create program test environment
        // 创建程序测试环境
        let program_id = Pubkey::new_unique();
        let mut program_test = ProgramTest::new(
            "solana_rust_dex",
            program_id,
            processor!(solana_rust_dex::entrypoint::process_instruction),
        );

        // Start the test environment
        // 启动测试环境
        let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

        // Setup market
        // 设置市场
        let (_, market_account, _, _) = setup_market(
            &program_id,
            &mut banks_client,
            &payer,
            &recent_blockhash,
        ).await;

        // Create order accounts
        // 创建订单账户
        let order_owner = Keypair::new();
        let order_account = Keypair::new();
        let owner_token_account = Pubkey::new_unique();
        let token_program = Pubkey::new_unique();

        // Add order account to the test environment with pre-initialized data
        // 将带有预初始化数据的订单账户添加到测试环境
        let order = Order {
            is_initialized: true,
            order_id: 1,
            owner: order_owner.pubkey(),
            market: market_account.pubkey(),
            is_buy: true,
            limit_price: 1000,
            original_quantity: 500,
            remaining_quantity: 500,
            creation_timestamp: 0,
        };

        let mut order_data = vec![0; Order::LEN];
        order.pack_into_slice(&mut order_data);

        program_test.add_account(
            order_account.pubkey(),
            solana_sdk::account::Account {
                lamports: 1000000000,
                data: order_data,
                owner: program_id,
                executable: false,
                rent_epoch: 0,
            },
        );

        // Create cancel order instruction
        // 创建取消订单指令
        let cancel_order_ix = DexInstruction::cancel_order(
            &program_id,
            &order_owner.pubkey(),
            &market_account.pubkey(),
            &order_account.pubkey(),
            &owner_token_account,
            &token_program,
        )
        .unwrap();

        // Create and sign transaction
        // 创建并签名交易
        let mut transaction = Transaction::new_with_payer(
            &[cancel_order_ix],
            Some(&payer.pubkey()),
        );
        transaction.sign(&[&payer, &order_owner], recent_blockhash);

        // Process transaction
        // 处理交易
        banks_client.process_transaction(transaction).await.unwrap();

        // Verify market state (bid count should be decremented)
        // 验证市场状态（买单数量应该减少）
        let market_account_data = banks_client
            .get_account(market_account.pubkey())
            .await
            .unwrap()
            .unwrap();

        let market = Market::unpack_from_slice(&market_account_data.data).unwrap();
        assert_eq!(market.num_bids, 0);
    }
} 