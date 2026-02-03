//! Tests for the `z_getstandardfees` RPC.

use std::{collections::HashMap, sync::Arc};

use futures::FutureExt;
use tower::buffer::Buffer;

use zebra_chain::{
    amount::{Amount, NonNegative},
    block::{tests::generate::block_header, Block, Header, Height},
    chain_sync_status::MockSyncStatus,
    chain_tip::mock::MockChainTip,
    parameters::Network::Mainnet,
    transaction::{LockTime, Transaction},
    transparent,
};
use zebra_network::address_book_peers::MockAddressBookPeers;
use zebra_node_services::BoxError;
use zebra_state::{HashOrHeight, ReadRequest, ReadResponse};
use zebra_test::mock_service::MockService;

use super::super::{calculate_transaction_fee, RpcImpl, RpcServer};
use crate::server::error::LegacyCode;

#[tokio::test(flavor = "multi_thread")]
async fn z_getstandardfees_happy_path() {
    let _init_guard = zebra_test::init();

    let mut mempool: MockService<_, _, _, BoxError> = MockService::build().for_unit_tests();
    let mut state: MockService<_, _, _, BoxError> = MockService::build().for_unit_tests();
    let mut read_state: MockService<_, _, _, BoxError> = MockService::build().for_unit_tests();

    let (chain_tip, chain_tip_sender) = MockChainTip::new();
    chain_tip_sender.send_best_tip_height(Height(55));

    let (_tx, rx) = tokio::sync::watch::channel(None);
    let (rpc, rpc_tx_queue) = RpcImpl::new(
        Mainnet,
        Default::default(),
        Default::default(),
        "0.0.1",
        "RPC test",
        Buffer::new(mempool.clone(), 1),
        Buffer::new(state.clone(), 1),
        Buffer::new(read_state.clone(), 1),
        MockService::build().for_unit_tests(),
        MockSyncStatus::default(),
        chain_tip,
        MockAddressBookPeers::default(),
        rx,
        None,
    );

    let rpc_future = tokio::spawn(async move { rpc.z_getstandardfees().await });

    let header = Arc::new(block_header().0);
    let base_value = 10_000;

    for height in 1u32..=50u32 {
        let fee_zats = i64::from(height) * 2;
        let block = make_block(height, base_value, fee_zats, &header);

        let request = ReadRequest::Block(HashOrHeight::Height(Height(height)));
        read_state
            .expect_request(request)
            .await
            .respond(ReadResponse::Block(Some(block)));
    }

    let response = rpc_future
        .await
        .expect("rpc task should not panic")
        .expect("rpc should succeed");
    assert_eq!(response.standard_fee, 10);
    assert_eq!(response.priority_fee, 100);

    mempool.expect_no_requests().await;
    state.expect_no_requests().await;
    read_state.expect_no_requests().await;

    // The queue task should continue without errors or panics.
    assert!(rpc_tx_queue.now_or_never().is_none());
}

#[tokio::test(flavor = "multi_thread")]
async fn z_getstandardfees_not_enough_blocks_end_underflow() {
    let _init_guard = zebra_test::init();

    let mut mempool: MockService<_, _, _, BoxError> = MockService::build().for_unit_tests();
    let mut state: MockService<_, _, _, BoxError> = MockService::build().for_unit_tests();
    let mut read_state: MockService<_, _, _, BoxError> = MockService::build().for_unit_tests();

    let (chain_tip, chain_tip_sender) = MockChainTip::new();
    chain_tip_sender.send_best_tip_height(Height(4));

    let (_tx, rx) = tokio::sync::watch::channel(None);
    let (rpc, rpc_tx_queue) = RpcImpl::new(
        Mainnet,
        Default::default(),
        Default::default(),
        "0.0.1",
        "RPC test",
        Buffer::new(mempool.clone(), 1),
        Buffer::new(state.clone(), 1),
        Buffer::new(read_state.clone(), 1),
        MockService::build().for_unit_tests(),
        MockSyncStatus::default(),
        chain_tip,
        MockAddressBookPeers::default(),
        rx,
        None,
    );

    let error = rpc
        .z_getstandardfees()
        .await
        .expect_err("expected not enough blocks error");

    assert_eq!(error.code(), i32::from(LegacyCode::Misc));
    assert_eq!(error.message(), "not enough blocks to calculate median fee");

    mempool.expect_no_requests().await;
    state.expect_no_requests().await;
    read_state.expect_no_requests().await;

    assert!(rpc_tx_queue.now_or_never().is_none());
}

#[tokio::test(flavor = "multi_thread")]
async fn z_getstandardfees_not_enough_blocks_start_underflow() {
    let _init_guard = zebra_test::init();

    let mut mempool: MockService<_, _, _, BoxError> = MockService::build().for_unit_tests();
    let mut state: MockService<_, _, _, BoxError> = MockService::build().for_unit_tests();
    let mut read_state: MockService<_, _, _, BoxError> = MockService::build().for_unit_tests();

    let (chain_tip, chain_tip_sender) = MockChainTip::new();
    chain_tip_sender.send_best_tip_height(Height(52));

    let (_tx, rx) = tokio::sync::watch::channel(None);
    let (rpc, rpc_tx_queue) = RpcImpl::new(
        Mainnet,
        Default::default(),
        Default::default(),
        "0.0.1",
        "RPC test",
        Buffer::new(mempool.clone(), 1),
        Buffer::new(state.clone(), 1),
        Buffer::new(read_state.clone(), 1),
        MockService::build().for_unit_tests(),
        MockSyncStatus::default(),
        chain_tip,
        MockAddressBookPeers::default(),
        rx,
        None,
    );

    let error = rpc
        .z_getstandardfees()
        .await
        .expect_err("expected not enough blocks error");

    assert_eq!(error.code(), i32::from(LegacyCode::Misc));
    assert_eq!(error.message(), "not enough blocks to calculate median fee");

    mempool.expect_no_requests().await;
    state.expect_no_requests().await;
    read_state.expect_no_requests().await;

    assert!(rpc_tx_queue.now_or_never().is_none());
}

#[tokio::test(flavor = "multi_thread")]
async fn z_getstandardfees_no_chain_tip() {
    let _init_guard = zebra_test::init();

    let mut mempool: MockService<_, _, _, BoxError> = MockService::build().for_unit_tests();
    let mut state: MockService<_, _, _, BoxError> = MockService::build().for_unit_tests();
    let mut read_state: MockService<_, _, _, BoxError> = MockService::build().for_unit_tests();

    let (chain_tip, _chain_tip_sender) = MockChainTip::new();

    let (_tx, rx) = tokio::sync::watch::channel(None);
    let (rpc, rpc_tx_queue) = RpcImpl::new(
        Mainnet,
        Default::default(),
        Default::default(),
        "0.0.1",
        "RPC test",
        Buffer::new(mempool.clone(), 1),
        Buffer::new(state.clone(), 1),
        Buffer::new(read_state.clone(), 1),
        MockService::build().for_unit_tests(),
        MockSyncStatus::default(),
        chain_tip,
        MockAddressBookPeers::default(),
        rx,
        None,
    );

    let error = rpc
        .z_getstandardfees()
        .await
        .expect_err("expected no chain tip error");

    assert_eq!(error.code(), i32::from(LegacyCode::Misc));
    assert_eq!(error.message(), "No blocks in state");

    mempool.expect_no_requests().await;
    state.expect_no_requests().await;
    read_state.expect_no_requests().await;

    assert!(rpc_tx_queue.now_or_never().is_none());
}

#[tokio::test(flavor = "multi_thread")]
async fn z_getstandardfees_block_not_found() {
    let _init_guard = zebra_test::init();

    let mut mempool: MockService<_, _, _, BoxError> = MockService::build().for_unit_tests();
    let mut state: MockService<_, _, _, BoxError> = MockService::build().for_unit_tests();
    let mut read_state: MockService<_, _, _, BoxError> = MockService::build().for_unit_tests();

    let (chain_tip, chain_tip_sender) = MockChainTip::new();
    chain_tip_sender.send_best_tip_height(Height(55));

    let (_tx, rx) = tokio::sync::watch::channel(None);
    let (rpc, rpc_tx_queue) = RpcImpl::new(
        Mainnet,
        Default::default(),
        Default::default(),
        "0.0.1",
        "RPC test",
        Buffer::new(mempool.clone(), 1),
        Buffer::new(state.clone(), 1),
        Buffer::new(read_state.clone(), 1),
        MockService::build().for_unit_tests(),
        MockSyncStatus::default(),
        chain_tip,
        MockAddressBookPeers::default(),
        rx,
        None,
    );

    let rpc_future = tokio::spawn(async move { rpc.z_getstandardfees().await });

    let request = ReadRequest::Block(HashOrHeight::Height(Height(1)));
    read_state
        .expect_request(request)
        .await
        .respond(ReadResponse::Block(None));

    let error = rpc_future
        .await
        .expect("rpc task should not panic")
        .expect_err("expected block not found error");

    assert_eq!(error.code(), i32::from(LegacyCode::Misc));
    assert_eq!(
        error.message(),
        "block not found while calculating median fee"
    );

    mempool.expect_no_requests().await;
    state.expect_no_requests().await;
    read_state.expect_no_requests().await;

    assert!(rpc_tx_queue.now_or_never().is_none());
}

fn make_block(height: u32, base_value: i64, fee_zats: i64, header: &Arc<Header>) -> Arc<Block> {
    let coinbase_value = base_value + fee_zats;
    let coinbase_tx = make_coinbase_tx(height, coinbase_value);
    let spend_tx = make_spend_tx(coinbase_tx.as_ref(), base_value);

    Arc::new(Block {
        header: header.clone(),
        transactions: vec![coinbase_tx, spend_tx],
    })
}

fn make_coinbase_tx(height: u32, value_zats: i64) -> Arc<Transaction> {
    let input = transparent::Input::new_coinbase(Height(height), vec![], None);
    let output = transparent::Output::new_coinbase(
        Amount::<NonNegative>::new(value_zats),
        transparent::Script::new(&[]),
    );

    Arc::new(Transaction::V1 {
        inputs: vec![input],
        outputs: vec![output],
        lock_time: LockTime::unlocked(),
    })
}

fn make_spend_tx(prev_tx: &Transaction, output_value_zats: i64) -> Arc<Transaction> {
    let outpoint = transparent::OutPoint::from_usize(prev_tx.hash(), 0);
    let input = transparent::Input::PrevOut {
        outpoint,
        unlock_script: transparent::Script::new(&[]),
        sequence: 0,
    };
    let output = transparent::Output {
        value: Amount::<NonNegative>::new(output_value_zats),
        lock_script: transparent::Script::new(&[]),
    };

    Arc::new(Transaction::V1 {
        inputs: vec![input],
        outputs: vec![output],
        lock_time: LockTime::unlocked(),
    })
}

#[tokio::test(flavor = "multi_thread")]
async fn calculate_transaction_fee_uses_tx_cache() {
    let _init_guard = zebra_test::init();

    let mut read_state: MockService<_, _, _, BoxError> = MockService::build().for_unit_tests();
    let mut tx_cache: HashMap<_, _> = HashMap::new();
    let block_outputs: HashMap<_, _> = HashMap::new();

    let prev_tx = make_coinbase_tx(1, 10);
    let spend_tx = make_spend_tx(prev_tx.as_ref(), 8);

    tx_cache.insert(prev_tx.hash(), (prev_tx.clone(), Height(1)));

    let fee = calculate_transaction_fee(&mut read_state, &mut tx_cache, &spend_tx, &block_outputs)
        .await
        .expect("fee should be computed")
        .expect("non-coinbase fee should exist");

    assert_eq!(fee.zatoshis(), 2);
    read_state.expect_no_requests().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn calculate_transaction_fee_fetches_prev_tx_from_read_state() {
    let _init_guard = zebra_test::init();

    let mut read_state: MockService<_, _, _, BoxError> = MockService::build().for_unit_tests();
    let mut read_state_handle = read_state.clone();
    let mut tx_cache: HashMap<_, _> = HashMap::new();
    let block_outputs: HashMap<_, _> = HashMap::new();

    let prev_tx = make_coinbase_tx(1, 10);
    let prev_hash = prev_tx.hash();
    let spend_tx = make_spend_tx(prev_tx.as_ref(), 8);

    let fee_fut =
        calculate_transaction_fee(&mut read_state, &mut tx_cache, &spend_tx, &block_outputs);
    let respond_fut = async move {
        let responder = read_state_handle
            .expect_request(ReadRequest::Transaction(prev_hash))
            .await;
        let mined_tx = zebra_state::MinedTx::new(prev_tx, Height(1), 1, chrono::Utc::now());
        responder.respond(ReadResponse::Transaction(Some(mined_tx)));
    };

    let (fee_result, _) = tokio::join!(fee_fut, respond_fut);
    let fee = fee_result
        .expect("fee should be computed")
        .expect("non-coinbase fee should exist");

    assert_eq!(fee.zatoshis(), 2);
    assert!(tx_cache.contains_key(&prev_hash));
    read_state.expect_no_requests().await;
}
