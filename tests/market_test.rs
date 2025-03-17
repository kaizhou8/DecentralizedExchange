// Market test module
// 市场测试模块

#[cfg(test)]
mod market_tests {
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
        instruction::DexInstruction,
        state::Market,
    };

    #[tokio::test]
    async fn test_initialize_market() {
        // Create program test environment
        // 创建程序测试环境
        let program_id = Pubkey::new_unique();
        let mut program_test = ProgramTest::new(
            "solana_rust_dex",
            program_id,
            processor!(solana_rust_dex::entrypoint::process_instruction),
        );

        // Create accounts for the test
        // 为测试创建账户
        let market_authority = Keypair::new();
        let market_account = Keypair::new();
        let base_mint = Pubkey::new_unique();
        let quote_mint = Pubkey::new_unique();

        // Add market account to the test environment
        // 将市场账户添加到测试环境
        program_test.add_account(
            market_account.pubkey(),
            solana_sdk::account::Account {
                lamports: 1000000000,
                data: vec![0; Market::LEN],
                owner: program_id,
                executable: false,
                rent_epoch: 0,
            },
        );

        // Start the test environment
        // 启动测试环境
        let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

        // Create initialize market instruction
        // 创建初始化市场指令
        let min_base_order_size = 100;
        let tick_size = 10;
        let fee_rate_bps = 25; // 0.25%

        let init_market_ix = DexInstruction::initialize_market(
            &program_id,
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
        transaction.sign(&[&payer, &market_authority], recent_blockhash);

        // Process transaction
        // 处理交易
        banks_client.process_transaction(transaction).await.unwrap();

        // Verify market state
        // 验证市场状态
        let market_account = banks_client
            .get_account(market_account.pubkey())
            .await
            .unwrap()
            .unwrap();

        let market = Market::unpack_from_slice(&market_account.data).unwrap();
        assert!(market.is_initialized);
        assert_eq!(market.authority, market_authority.pubkey());
        assert_eq!(market.base_mint, base_mint);
        assert_eq!(market.quote_mint, quote_mint);
        assert_eq!(market.min_base_order_size, min_base_order_size);
        assert_eq!(market.tick_size, tick_size);
        assert_eq!(market.fee_rate_bps, fee_rate_bps);
        assert_eq!(market.next_order_id, 1);
        assert_eq!(market.num_bids, 0);
        assert_eq!(market.num_asks, 0);
    }
} 