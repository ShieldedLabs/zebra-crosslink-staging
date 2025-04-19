//! Fixed test vectors for RPC methods.

use std::sync::Arc;

use futures::FutureExt;
use tower::buffer::Buffer;

use zebra_chain::{
    amount::Amount,
    block::Block,
    chain_tip::{mock::MockChainTip, NoChainTip},
    history_tree::HistoryTree,
    parameters::Network::*,
    serialization::{ZcashDeserializeInto, ZcashSerialize},
    transaction::UnminedTxId,
};
use zebra_network::address_book_peers::MockAddressBookPeers;
use zebra_node_services::BoxError;

use zebra_state::{GetBlockTemplateChainInfo, IntoDisk, LatestChainTip, ReadStateService};
use zebra_test::mock_service::MockService;

use super::super::*;

#[tokio::test(flavor = "multi_thread")]
async fn rpc_getinfo() {
    let _init_guard = zebra_test::init();

    let mut mempool: MockService<_, _, _, BoxError> = MockService::build().for_unit_tests();
    let mut state: MockService<_, _, _, BoxError> = MockService::build().for_unit_tests();

    let (_tx, rx) = tokio::sync::watch::channel(None);
    let (rpc, rpc_tx_queue_task_handle) = RpcImpl::new(
        "0.0.1",
        "/Zebra:RPC test/",
        Mainnet,
        false,
        true,
        Buffer::new(mempool.clone(), 1),
        Buffer::new(state.clone(), 1),
        NoChainTip,
        MockAddressBookPeers::new(vec![]),
        rx,
    );

    let getinfo_future = tokio::spawn(async move { rpc.get_info().await });

    // Make the mock service respond with
    let response_handler = state
        .expect_request(zebra_state::ReadRequest::ChainInfo)
        .await;
    response_handler.respond(zebra_state::ReadResponse::ChainInfo(
        GetBlockTemplateChainInfo {
            tip_hash: Mainnet.genesis_hash(),
            tip_height: Height::MIN,
            chain_history_root: HistoryTree::default().hash(),
            expected_difficulty: Default::default(),
            cur_time: zebra_chain::serialization::DateTime32::now(),
            min_time: zebra_chain::serialization::DateTime32::now(),
            max_time: zebra_chain::serialization::DateTime32::now(),
        },
    ));

    let get_info = getinfo_future
        .await
        .expect("getinfo future should not panic")
        .expect("getinfo future should not return an error");

    // make sure there is a `build` field in the response,
    // and that is equal to the provided string, with an added 'v' version prefix.
    assert_eq!(get_info.build, "v0.0.1");

    // make sure there is a `subversion` field,
    // and that is equal to the Zebra user agent.
    assert_eq!(get_info.subversion, format!("/Zebra:RPC test/"));

    mempool.expect_no_requests().await;
    state.expect_no_requests().await;

    // The queue task should continue without errors or panics
    let rpc_tx_queue_task_result = rpc_tx_queue_task_handle.now_or_never();
    assert!(rpc_tx_queue_task_result.is_none());
}

// Helper function that returns the nonce, final sapling root and
// block commitments of a given Block.
async fn get_block_data(
    read_state: &ReadStateService,
    block: Arc<Block>,
    height: usize,
) -> ([u8; 32], [u8; 32], [u8; 32]) {
    let zebra_state::ReadResponse::SaplingTree(sapling_tree) = read_state
        .clone()
        .oneshot(zebra_state::ReadRequest::SaplingTree(HashOrHeight::Height(
            (height as u32).try_into().unwrap(),
        )))
        .await
        .expect("should have sapling tree for block hash")
    else {
        panic!("unexpected response to SaplingTree request")
    };

    let mut expected_nonce = *block.header.nonce;
    expected_nonce.reverse();
    let sapling_tree = sapling_tree.expect("should always have sapling root");
    let expected_final_sapling_root: [u8; 32] = if sapling_tree.position().is_some() {
        let mut root: [u8; 32] = sapling_tree.root().into();
        root.reverse();
        root
    } else {
        [0; 32]
    };

    let expected_block_commitments = match block
        .commitment(&Mainnet)
        .expect("Unexpected failure while parsing the blockcommitments field in get_block_data")
    {
        Commitment::PreSaplingReserved(bytes) => bytes,
        Commitment::FinalSaplingRoot(_) => expected_final_sapling_root,
        Commitment::ChainHistoryActivationReserved => [0; 32],
        Commitment::ChainHistoryRoot(root) => root.bytes_in_display_order(),
        Commitment::ChainHistoryBlockTxAuthCommitment(hash) => hash.bytes_in_display_order(),
    };
    (
        expected_nonce,
        expected_final_sapling_root,
        expected_block_commitments,
    )
}

#[tokio::test(flavor = "multi_thread")]
async fn rpc_getblock() {
    let _init_guard = zebra_test::init();

    // Create a continuous chain of mainnet blocks from genesis
    let blocks: Vec<Arc<Block>> = zebra_test::vectors::CONTINUOUS_MAINNET_BLOCKS
        .iter()
        .map(|(_height, block_bytes)| block_bytes.zcash_deserialize_into().unwrap())
        .collect();

    let mut mempool: MockService<_, _, _, BoxError> = MockService::build().for_unit_tests();
    // Create a populated state service
    let (_state, read_state, latest_chain_tip, _chain_tip_change) =
        zebra_state::populated_state(blocks.clone(), &Mainnet).await;

    // Init RPC
    let (_tx, rx) = tokio::sync::watch::channel(None);
    let (rpc, rpc_tx_queue_task_handle) = RpcImpl::new(
        "0.0.1",
        "RPC test",
        Mainnet,
        false,
        true,
        Buffer::new(mempool.clone(), 1),
        read_state.clone(),
        latest_chain_tip,
        MockAddressBookPeers::new(vec![]),
        rx,
    );

    // Make height calls with verbosity=0 and check response
    for (i, block) in blocks.iter().enumerate() {
        let expected_result = GetBlock::Raw(block.clone().into());

        let get_block = rpc
            .get_block(i.to_string(), Some(0u8))
            .await
            .expect("We should have a GetBlock struct");

        assert_eq!(get_block, expected_result);

        let get_block = rpc
            .get_block(block.hash().to_string(), Some(0u8))
            .await
            .expect("We should have a GetBlock struct");

        assert_eq!(get_block, expected_result);
    }

    // Make hash calls with verbosity=0 and check response
    for (i, block) in blocks.iter().enumerate() {
        let expected_result = GetBlock::Raw(block.clone().into());

        let get_block = rpc
            .get_block(blocks[i].hash().to_string(), Some(0u8))
            .await
            .expect("We should have a GetBlock struct");

        assert_eq!(get_block, expected_result);

        let get_block = rpc
            .get_block(block.hash().to_string(), Some(0u8))
            .await
            .expect("We should have a GetBlock struct");

        assert_eq!(get_block, expected_result);
    }

    // Test negative heights: -1 should return block 10, -2 block 9, etc.
    for neg_height in (-10..=-1).rev() {
        // Convert negative height to corresponding index
        let index = (neg_height + (blocks.len() as i32)) as usize;

        let expected_result = GetBlock::Raw(blocks[index].clone().into());

        let get_block = rpc
            .get_block(neg_height.to_string(), Some(0u8))
            .await
            .expect("We should have a GetBlock struct");

        assert_eq!(get_block, expected_result);
    }

    // Create empty note commitment tree information.
    let sapling = SaplingTrees { size: 0 };
    let orchard = OrchardTrees { size: 0 };
    let trees = GetBlockTrees { sapling, orchard };

    // Make height calls with verbosity=1 and check response
    for (i, block) in blocks.iter().enumerate() {
        let get_block = rpc
            .get_block(i.to_string(), Some(1u8))
            .await
            .expect("We should have a GetBlock struct");

        let (expected_nonce, expected_final_sapling_root, expected_block_commitments) =
            get_block_data(&read_state, block.clone(), i).await;

        assert_eq!(
            get_block,
            GetBlock::Object {
                hash: GetBlockHash(block.hash()),
                confirmations: (blocks.len() - i).try_into().expect("valid i64"),
                height: Some(Height(i.try_into().expect("valid u32"))),
                time: Some(block.header.time.timestamp()),
                tx: block
                    .transactions
                    .iter()
                    .map(|tx| GetBlockTransaction::Hash(tx.hash()))
                    .collect(),
                trees,
                size: None,
                version: Some(block.header.version),
                merkle_root: Some(block.header.merkle_root),
                block_commitments: Some(expected_block_commitments),
                final_sapling_root: Some(expected_final_sapling_root),
                final_orchard_root: None,
                nonce: Some(expected_nonce),
                bits: Some(block.header.difficulty_threshold),
                difficulty: Some(
                    block
                        .header
                        .difficulty_threshold
                        .relative_to_network(&Mainnet)
                ),
                previous_block_hash: Some(GetBlockHash(block.header.previous_block_hash)),
                next_block_hash: blocks.get(i + 1).map(|b| GetBlockHash(b.hash())),
                solution: Some(block.header.solution),
            }
        );
    }

    // Make hash calls with verbosity=1 and check response
    for (i, block) in blocks.iter().enumerate() {
        let get_block = rpc
            .get_block(blocks[i].hash().to_string(), Some(1u8))
            .await
            .expect("We should have a GetBlock struct");

        let (expected_nonce, expected_final_sapling_root, expected_block_commitments) =
            get_block_data(&read_state, block.clone(), i).await;

        assert_eq!(
            get_block,
            GetBlock::Object {
                hash: GetBlockHash(block.hash()),
                confirmations: (blocks.len() - i).try_into().expect("valid i64"),
                height: Some(Height(i.try_into().expect("valid u32"))),
                time: Some(block.header.time.timestamp()),
                tx: block
                    .transactions
                    .iter()
                    .map(|tx| GetBlockTransaction::Hash(tx.hash()))
                    .collect(),
                trees,
                size: None,
                version: Some(block.header.version),
                merkle_root: Some(block.header.merkle_root),
                block_commitments: Some(expected_block_commitments),
                final_sapling_root: Some(expected_final_sapling_root),
                final_orchard_root: None,
                nonce: Some(expected_nonce),
                bits: Some(block.header.difficulty_threshold),
                difficulty: Some(
                    block
                        .header
                        .difficulty_threshold
                        .relative_to_network(&Mainnet)
                ),
                previous_block_hash: Some(GetBlockHash(block.header.previous_block_hash)),
                next_block_hash: blocks.get(i + 1).map(|b| GetBlockHash(b.hash())),
                solution: Some(block.header.solution),
            }
        );
    }

    // Make height calls with verbosity=2 and check response
    for (i, block) in blocks.iter().enumerate() {
        let get_block = rpc
            .get_block(i.to_string(), Some(2u8))
            .await
            .expect("We should have a GetBlock struct");

        let (expected_nonce, expected_final_sapling_root, expected_block_commitments) =
            get_block_data(&read_state, block.clone(), i).await;

        // partially compare the expected and actual GetBlock structs
        if let GetBlock::Object {
            hash,
            confirmations,
            height,
            time,
            tx,
            trees,
            size,
            version,
            merkle_root,
            block_commitments,
            final_sapling_root,
            final_orchard_root,
            nonce,
            bits,
            difficulty,
            previous_block_hash,
            next_block_hash,
            solution,
        } = &get_block
        {
            assert_eq!(hash, &GetBlockHash(block.hash()));
            assert_eq!(confirmations, &((blocks.len() - i) as i64));
            assert_eq!(height, &Some(Height(i.try_into().expect("valid u32"))));
            assert_eq!(time, &Some(block.header.time.timestamp()));
            assert_eq!(trees, trees);
            assert_eq!(
                size,
                &Some(block.zcash_serialize_to_vec().unwrap().len() as i64)
            );
            assert_eq!(version, &Some(block.header.version));
            assert_eq!(merkle_root, &Some(block.header.merkle_root));
            assert_eq!(block_commitments, &Some(expected_block_commitments));
            assert_eq!(final_sapling_root, &Some(expected_final_sapling_root));
            assert_eq!(final_orchard_root, &None);
            assert_eq!(nonce, &Some(expected_nonce));
            assert_eq!(bits, &Some(block.header.difficulty_threshold));
            assert_eq!(
                difficulty,
                &Some(
                    block
                        .header
                        .difficulty_threshold
                        .relative_to_network(&Mainnet)
                )
            );
            assert_eq!(
                previous_block_hash,
                &Some(GetBlockHash(block.header.previous_block_hash))
            );
            assert_eq!(
                next_block_hash,
                &blocks.get(i + 1).map(|b| GetBlockHash(b.hash()))
            );
            assert_eq!(solution, &Some(block.header.solution));

            for (actual_tx, expected_tx) in tx.iter().zip(block.transactions.iter()) {
                if let GetBlockTransaction::Object(TransactionObject {
                    hex,
                    height,
                    confirmations,
                    ..
                }) = actual_tx
                {
                    assert_eq!(hex, &(*expected_tx).clone().into());
                    assert_eq!(height, &Some(i.try_into().expect("valid u32")));
                    assert_eq!(
                        confirmations,
                        &Some((blocks.len() - i).try_into().expect("valid i64"))
                    );
                }
            }
        } else {
            panic!("Expected GetBlock::Object");
        }
    }

    // Make hash calls with verbosity=2 and check response
    for (i, block) in blocks.iter().enumerate() {
        let get_block = rpc
            .get_block(blocks[i].hash().to_string(), Some(2u8))
            .await
            .expect("We should have a GetBlock struct");

        let (expected_nonce, expected_final_sapling_root, expected_block_commitments) =
            get_block_data(&read_state, block.clone(), i).await;

        // partially compare the expected and actual GetBlock structs
        if let GetBlock::Object {
            hash,
            confirmations,
            height,
            time,
            tx,
            trees,
            size,
            version,
            merkle_root,
            block_commitments,
            final_sapling_root,
            final_orchard_root,
            nonce,
            bits,
            difficulty,
            previous_block_hash,
            next_block_hash,
            solution,
        } = &get_block
        {
            assert_eq!(hash, &GetBlockHash(block.hash()));
            assert_eq!(confirmations, &((blocks.len() - i) as i64));
            assert_eq!(height, &Some(Height(i.try_into().expect("valid u32"))));
            assert_eq!(time, &Some(block.header.time.timestamp()));
            assert_eq!(trees, trees);
            assert_eq!(
                size,
                &Some(block.zcash_serialize_to_vec().unwrap().len() as i64)
            );
            assert_eq!(version, &Some(block.header.version));
            assert_eq!(merkle_root, &Some(block.header.merkle_root));
            assert_eq!(block_commitments, &Some(expected_block_commitments));
            assert_eq!(final_sapling_root, &Some(expected_final_sapling_root));
            assert_eq!(final_orchard_root, &None);
            assert_eq!(nonce, &Some(expected_nonce));
            assert_eq!(bits, &Some(block.header.difficulty_threshold));
            assert_eq!(
                difficulty,
                &Some(
                    block
                        .header
                        .difficulty_threshold
                        .relative_to_network(&Mainnet)
                )
            );
            assert_eq!(
                previous_block_hash,
                &Some(GetBlockHash(block.header.previous_block_hash))
            );
            assert_eq!(
                next_block_hash,
                &blocks.get(i + 1).map(|b| GetBlockHash(b.hash()))
            );
            assert_eq!(solution, &Some(block.header.solution));

            for (actual_tx, expected_tx) in tx.iter().zip(block.transactions.iter()) {
                if let GetBlockTransaction::Object(TransactionObject {
                    hex,
                    height,
                    confirmations,
                    ..
                }) = actual_tx
                {
                    assert_eq!(hex, &(*expected_tx).clone().into());
                    assert_eq!(height, &Some(i.try_into().expect("valid u32")));
                    assert_eq!(
                        confirmations,
                        &Some((blocks.len() - i).try_into().expect("valid i64"))
                    );
                }
            }
        } else {
            panic!("Expected GetBlock::Object");
        }
    }

    // Make height calls with no verbosity (defaults to 1) and check response
    for (i, block) in blocks.iter().enumerate() {
        let get_block = rpc
            .get_block(i.to_string(), None)
            .await
            .expect("We should have a GetBlock struct");

        let (expected_nonce, expected_final_sapling_root, expected_block_commitments) =
            get_block_data(&read_state, block.clone(), i).await;

        assert_eq!(
            get_block,
            GetBlock::Object {
                hash: GetBlockHash(block.hash()),
                confirmations: (blocks.len() - i).try_into().expect("valid i64"),
                height: Some(Height(i.try_into().expect("valid u32"))),
                time: Some(block.header.time.timestamp()),
                tx: block
                    .transactions
                    .iter()
                    .map(|tx| GetBlockTransaction::Hash(tx.hash()))
                    .collect(),
                trees,
                size: None,
                version: Some(block.header.version),
                merkle_root: Some(block.header.merkle_root),
                block_commitments: Some(expected_block_commitments),
                final_sapling_root: Some(expected_final_sapling_root),
                final_orchard_root: None,
                nonce: Some(expected_nonce),
                bits: Some(block.header.difficulty_threshold),
                difficulty: Some(
                    block
                        .header
                        .difficulty_threshold
                        .relative_to_network(&Mainnet)
                ),
                previous_block_hash: Some(GetBlockHash(block.header.previous_block_hash)),
                next_block_hash: blocks.get(i + 1).map(|b| GetBlockHash(b.hash())),
                solution: Some(block.header.solution),
            }
        );
    }

    // Make hash calls with no verbosity (defaults to 1) and check response
    for (i, block) in blocks.iter().enumerate() {
        let get_block = rpc
            .get_block(blocks[i].hash().to_string(), None)
            .await
            .expect("We should have a GetBlock struct");

        let (expected_nonce, expected_final_sapling_root, expected_block_commitments) =
            get_block_data(&read_state, block.clone(), i).await;

        assert_eq!(
            get_block,
            GetBlock::Object {
                hash: GetBlockHash(block.hash()),
                confirmations: (blocks.len() - i).try_into().expect("valid i64"),
                height: Some(Height(i.try_into().expect("valid u32"))),
                time: Some(block.header.time.timestamp()),
                tx: block
                    .transactions
                    .iter()
                    .map(|tx| GetBlockTransaction::Hash(tx.hash()))
                    .collect(),
                trees,
                size: None,
                version: Some(block.header.version),
                merkle_root: Some(block.header.merkle_root),
                block_commitments: Some(expected_block_commitments),
                final_sapling_root: Some(expected_final_sapling_root),
                final_orchard_root: None,
                nonce: Some(expected_nonce),
                bits: Some(block.header.difficulty_threshold),
                difficulty: Some(
                    block
                        .header
                        .difficulty_threshold
                        .relative_to_network(&Mainnet)
                ),
                previous_block_hash: Some(GetBlockHash(block.header.previous_block_hash)),
                next_block_hash: blocks.get(i + 1).map(|b| GetBlockHash(b.hash())),
                solution: Some(block.header.solution),
            }
        );
    }

    mempool.expect_no_requests().await;

    // The queue task should continue without errors or panics
    let rpc_tx_queue_task_result = rpc_tx_queue_task_handle.now_or_never();
    assert!(rpc_tx_queue_task_result.is_none());
}

#[tokio::test(flavor = "multi_thread")]
async fn rpc_getblock_parse_error() {
    let _init_guard = zebra_test::init();

    let mut mempool: MockService<_, _, _, BoxError> = MockService::build().for_unit_tests();
    let mut state: MockService<_, _, _, BoxError> = MockService::build().for_unit_tests();

    // Init RPC
    let (_tx, rx) = tokio::sync::watch::channel(None);
    let (rpc, rpc_tx_queue_task_handle) = RpcImpl::new(
        "0.0.1",
        "RPC test",
        Mainnet,
        false,
        true,
        Buffer::new(mempool.clone(), 1),
        Buffer::new(state.clone(), 1),
        NoChainTip,
        MockAddressBookPeers::new(vec![]),
        rx,
    );

    // Make sure we get an error if Zebra can't parse the block height.
    assert!(rpc
        .get_block("not parsable as height".to_string(), Some(0u8))
        .await
        .is_err());

    assert!(rpc
        .get_block("not parsable as height".to_string(), Some(1u8))
        .await
        .is_err());

    assert!(rpc
        .get_block("not parsable as height".to_string(), None)
        .await
        .is_err());

    mempool.expect_no_requests().await;
    state.expect_no_requests().await;

    // The queue task should continue without errors or panics
    let rpc_tx_queue_task_result = rpc_tx_queue_task_handle.now_or_never();
    assert!(rpc_tx_queue_task_result.is_none());
}

#[tokio::test(flavor = "multi_thread")]
async fn rpc_getblock_missing_error() {
    let _init_guard = zebra_test::init();

    let mut mempool: MockService<_, _, _, BoxError> = MockService::build().for_unit_tests();
    let mut state: MockService<_, _, _, BoxError> = MockService::build().for_unit_tests();

    // Init RPC
    let (_tx, rx) = tokio::sync::watch::channel(None);
    let (rpc, rpc_tx_queue_task_handle) = RpcImpl::new(
        "0.0.1",
        "RPC test",
        Mainnet,
        false,
        true,
        Buffer::new(mempool.clone(), 1),
        Buffer::new(state.clone(), 1),
        NoChainTip,
        MockAddressBookPeers::new(vec![]),
        rx,
    );

    // Make sure Zebra returns the correct error code `-8` for missing blocks
    // https://github.com/zcash/lightwalletd/blob/v0.4.16/common/common.go#L287-L290
    let block_future = tokio::spawn(async move { rpc.get_block("0".to_string(), Some(0u8)).await });

    // Make the mock service respond with no block
    let response_handler = state
        .expect_request(zebra_state::ReadRequest::Block(Height(0).into()))
        .await;
    response_handler.respond(zebra_state::ReadResponse::Block(None));

    let block_response = block_future.await.expect("block future should not panic");
    let block_response =
        block_response.expect_err("unexpected success from missing block state response");
    assert_eq!(block_response.code(), ErrorCode::ServerError(-8).code());

    // Now check the error string the way `lightwalletd` checks it
    assert_eq!(
        serde_json::to_string(&block_response)
            .expect("unexpected error serializing JSON error")
            .split(':')
            .nth(1)
            .expect("unexpectedly low number of error fields")
            .split(',')
            .next(),
        Some("-8")
    );

    mempool.expect_no_requests().await;
    state.expect_no_requests().await;

    // The queue task should continue without errors or panics
    let rpc_tx_queue_task_result = rpc_tx_queue_task_handle.now_or_never();
    assert!(rpc_tx_queue_task_result.is_none());
}

#[tokio::test(flavor = "multi_thread")]
async fn rpc_getblockheader() {
    let _init_guard = zebra_test::init();

    // Create a continuous chain of mainnet blocks from genesis
    let blocks: Vec<Arc<Block>> = zebra_test::vectors::CONTINUOUS_MAINNET_BLOCKS
        .iter()
        .map(|(_height, block_bytes)| block_bytes.zcash_deserialize_into().unwrap())
        .collect();

    let mut mempool: MockService<_, _, _, BoxError> = MockService::build().for_unit_tests();
    // Create a populated state service
    let (_state, read_state, latest_chain_tip, _chain_tip_change) =
        zebra_state::populated_state(blocks.clone(), &Mainnet).await;

    // Init RPC
    let (_tx, rx) = tokio::sync::watch::channel(None);
    let (rpc, rpc_tx_queue_task_handle) = RpcImpl::new(
        "0.0.1",
        "RPC test",
        Mainnet,
        false,
        true,
        Buffer::new(mempool.clone(), 1),
        read_state.clone(),
        latest_chain_tip,
        MockAddressBookPeers::new(vec![]),
        rx,
    );

    // Make height calls with verbose=false and check response
    for (i, block) in blocks.iter().enumerate() {
        let expected_result = GetBlockHeader::Raw(HexData(
            block
                .header
                .clone()
                .zcash_serialize_to_vec()
                .expect("test block header should serialize"),
        ));

        let hash = block.hash();
        let height = Height(i as u32);

        for hash_or_height in [HashOrHeight::from(height), hash.into()] {
            let get_block_header = rpc
                .get_block_header(hash_or_height.to_string(), Some(false))
                .await
                .expect("we should have a GetBlockHeader struct");
            assert_eq!(get_block_header, expected_result);
        }

        let zebra_state::ReadResponse::SaplingTree(sapling_tree) = read_state
            .clone()
            .oneshot(zebra_state::ReadRequest::SaplingTree(height.into()))
            .await
            .expect("should have sapling tree for block hash")
        else {
            panic!("unexpected response to SaplingTree request")
        };

        let mut expected_nonce = *block.header.nonce;
        expected_nonce.reverse();
        let sapling_tree = sapling_tree.expect("should always have sapling root");
        let expected_final_sapling_root: [u8; 32] = if sapling_tree.position().is_some() {
            let mut root: [u8; 32] = sapling_tree.root().into();
            root.reverse();
            root
        } else {
            [0; 32]
        };

        let expected_result = GetBlockHeader::Object(Box::new(GetBlockHeaderObject {
            hash: GetBlockHash(hash),
            confirmations: 11 - i as i64,
            height,
            version: 4,
            merkle_root: block.header.merkle_root,
            block_commitments: block.header.commitment_bytes.0,
            final_sapling_root: expected_final_sapling_root,
            sapling_tree_size: sapling_tree.count(),
            time: block.header.time.timestamp(),
            nonce: expected_nonce,
            solution: block.header.solution,
            bits: block.header.difficulty_threshold,
            difficulty: block
                .header
                .difficulty_threshold
                .relative_to_network(&Mainnet),
            previous_block_hash: GetBlockHash(block.header.previous_block_hash),
            next_block_hash: blocks.get(i + 1).map(|b| GetBlockHash(b.hash())),
        }));

        for hash_or_height in [HashOrHeight::from(Height(i as u32)), block.hash().into()] {
            let get_block_header = rpc
                .get_block_header(hash_or_height.to_string(), Some(true))
                .await
                .expect("we should have a GetBlockHeader struct");
            assert_eq!(get_block_header, expected_result);
        }
    }

    // Test negative heights: -1 should return a header for block 10, -2 block header 9, etc.
    for neg_height in (-10..=-1).rev() {
        // Convert negative height to corresponding index
        let index = (neg_height + (blocks.len() as i32)) as usize;

        let expected_result = GetBlockHeader::Raw(HexData(blocks[index].header.clone().as_bytes()));

        let get_block = rpc
            .get_block_header(neg_height.to_string(), Some(false))
            .await
            .expect("We should have a GetBlock struct");

        assert_eq!(get_block, expected_result);
    }

    mempool.expect_no_requests().await;

    // The queue task should continue without errors or panics
    let rpc_tx_queue_task_result = rpc_tx_queue_task_handle.now_or_never();
    assert!(rpc_tx_queue_task_result.is_none());
}

#[tokio::test(flavor = "multi_thread")]
async fn rpc_getbestblockhash() {
    let _init_guard = zebra_test::init();

    // Create a continuous chain of mainnet blocks from genesis
    let blocks: Vec<Arc<Block>> = zebra_test::vectors::CONTINUOUS_MAINNET_BLOCKS
        .iter()
        .map(|(_height, block_bytes)| block_bytes.zcash_deserialize_into().unwrap())
        .collect();

    // Get the hash of the block at the tip using hardcoded block tip bytes.
    // We want to test the RPC response is equal to this hash
    let tip_block = blocks.last().unwrap();
    let tip_block_hash = tip_block.hash();

    // Get a mempool handle
    let mut mempool: MockService<_, _, _, BoxError> = MockService::build().for_unit_tests();
    // Create a populated state service, the tip will be in `NUMBER_OF_BLOCKS`.
    let (_state, read_state, latest_chain_tip, _chain_tip_change) =
        zebra_state::populated_state(blocks.clone(), &Mainnet).await;

    // Init RPC
    let (_tx, rx) = tokio::sync::watch::channel(None);
    let (rpc, rpc_tx_queue_task_handle) = RpcImpl::new(
        "0.0.1",
        "RPC test",
        Mainnet,
        false,
        true,
        Buffer::new(mempool.clone(), 1),
        read_state,
        latest_chain_tip,
        MockAddressBookPeers::new(vec![]),
        rx,
    );

    // Get the tip hash using RPC method `get_best_block_hash`
    let get_best_block_hash = rpc
        .get_best_block_hash()
        .expect("We should have a GetBlockHash struct");
    let response_hash = get_best_block_hash.0;

    // Check if response is equal to block 10 hash.
    assert_eq!(response_hash, tip_block_hash);

    mempool.expect_no_requests().await;

    // The queue task should continue without errors or panics
    let rpc_tx_queue_task_result = rpc_tx_queue_task_handle.now_or_never();
    assert!(rpc_tx_queue_task_result.is_none());
}

#[tokio::test(flavor = "multi_thread")]
async fn rpc_getrawtransaction() {
    let _init_guard = zebra_test::init();

    // Create a continuous chain of mainnet blocks from genesis
    let blocks: Vec<Arc<Block>> = zebra_test::vectors::CONTINUOUS_MAINNET_BLOCKS
        .iter()
        .map(|(_height, block_bytes)| block_bytes.zcash_deserialize_into().unwrap())
        .collect();

    let mut mempool: MockService<_, _, _, BoxError> = MockService::build().for_unit_tests();
    // Create a populated state service
    let (_state, read_state, _latest_chain_tip, _chain_tip_change) =
        zebra_state::populated_state(blocks.clone(), &Mainnet).await;

    let (latest_chain_tip, latest_chain_tip_sender) = MockChainTip::new();
    latest_chain_tip_sender.send_best_tip_height(Height(10));

    // Init RPC
    let (_tx, rx) = tokio::sync::watch::channel(None);
    let (rpc, rpc_tx_queue_task_handle) = RpcImpl::new(
        "0.0.1",
        "RPC test",
        Mainnet,
        false,
        true,
        Buffer::new(mempool.clone(), 1),
        read_state.clone(),
        latest_chain_tip,
        MockAddressBookPeers::new(vec![]),
        rx,
    );

    // Test case where transaction is in mempool.
    // Skip genesis because its tx is not indexed.
    for block in blocks.iter().skip(1) {
        for tx in block.transactions.iter() {
            let mempool_req = mempool
                .expect_request_that(|request| {
                    if let mempool::Request::TransactionsByMinedId(ids) = request {
                        ids.len() == 1 && ids.contains(&tx.hash())
                    } else {
                        false
                    }
                })
                .map(|responder| {
                    responder.respond(mempool::Response::Transactions(vec![UnminedTx {
                        id: UnminedTxId::Legacy(tx.hash()),
                        transaction: tx.clone(),
                        size: 0,
                        conventional_fee: Amount::zero(),
                    }]));
                });

            let rpc_req = rpc.get_raw_transaction(tx.hash().encode_hex(), Some(0u8));

            let (rsp, _) = futures::join!(rpc_req, mempool_req);
            let get_tx = rsp.expect("we should have a `GetRawTransaction` struct");

            if let GetRawTransaction::Raw(raw_tx) = get_tx {
                assert_eq!(raw_tx.as_ref(), tx.zcash_serialize_to_vec().unwrap());
            } else {
                unreachable!("Should return a Raw enum")
            }
        }
    }

    let make_mempool_req = |tx_hash: transaction::Hash| {
        let mut mempool = mempool.clone();

        async move {
            mempool
                .expect_request_that(|request| {
                    if let mempool::Request::TransactionsByMinedId(ids) = request {
                        ids.len() == 1 && ids.contains(&tx_hash)
                    } else {
                        false
                    }
                })
                .await
                .respond(mempool::Response::Transactions(vec![]));
        }
    };

    let run_state_test_case = |block_idx: usize, block: Arc<Block>, tx: Arc<Transaction>| {
        let read_state = read_state.clone();
        let txid = tx.hash();
        let hex_txid = txid.encode_hex::<String>();

        let get_tx_verbose_0_req = rpc.get_raw_transaction(hex_txid.clone(), Some(0u8));
        let get_tx_verbose_1_req = rpc.get_raw_transaction(hex_txid, Some(1u8));

        async move {
            let (response, _) = futures::join!(get_tx_verbose_0_req, make_mempool_req(txid));
            let get_tx = response.expect("We should have a GetRawTransaction struct");
            if let GetRawTransaction::Raw(raw_tx) = get_tx {
                assert_eq!(raw_tx.as_ref(), tx.zcash_serialize_to_vec().unwrap());
            } else {
                unreachable!("Should return a Raw enum")
            }

            let (response, _) = futures::join!(get_tx_verbose_1_req, make_mempool_req(txid));

            let GetRawTransaction::Object(TransactionObject {
                hex,
                height,
                confirmations,
                ..
            }) = response.expect("We should have a GetRawTransaction struct")
            else {
                unreachable!("Should return a Raw enum")
            };

            let height = height.expect("state requests should have height");
            let confirmations = confirmations.expect("state requests should have confirmations");

            assert_eq!(hex.as_ref(), tx.zcash_serialize_to_vec().unwrap());
            assert_eq!(height, block_idx as u32);

            let depth_response = read_state
                .oneshot(zebra_state::ReadRequest::Depth(block.hash()))
                .await
                .expect("state request should succeed");

            let zebra_state::ReadResponse::Depth(depth) = depth_response else {
                panic!("unexpected response to Depth request");
            };

            let expected_confirmations = 1 + depth.expect("depth should be Some");

            (confirmations, expected_confirmations)
        }
    };

    // Test case where transaction is _not_ in mempool.
    // Skip genesis because its tx is not indexed.
    for (block_idx, block) in blocks.iter().enumerate().skip(1) {
        for tx in block.transactions.iter() {
            let (confirmations, expected_confirmations) =
                run_state_test_case(block_idx, block.clone(), tx.clone()).await;
            assert_eq!(confirmations, expected_confirmations);
        }
    }

    // Test case where transaction is _not_ in mempool with a fake chain tip height of 0
    // Skip genesis because its tx is not indexed.
    latest_chain_tip_sender.send_best_tip_height(Height(0));
    for (block_idx, block) in blocks.iter().enumerate().skip(1) {
        for tx in block.transactions.iter() {
            let (confirmations, expected_confirmations) =
                run_state_test_case(block_idx, block.clone(), tx.clone()).await;

            let is_confirmations_within_bounds = confirmations <= expected_confirmations;

            assert!(
                is_confirmations_within_bounds,
                "confirmations should be at or below depth + 1"
            );
        }
    }

    // Test case where transaction is _not_ in mempool with a fake chain tip height of 0
    // Skip genesis because its tx is not indexed.
    latest_chain_tip_sender.send_best_tip_height(Height(20));
    for (block_idx, block) in blocks.iter().enumerate().skip(1) {
        for tx in block.transactions.iter() {
            let (confirmations, expected_confirmations) =
                run_state_test_case(block_idx, block.clone(), tx.clone()).await;

            let is_confirmations_within_bounds = confirmations <= expected_confirmations;

            assert!(
                is_confirmations_within_bounds,
                "confirmations should be at or below depth + 1"
            );
        }
    }

    // The queue task should continue without errors or panics
    let rpc_tx_queue_task_result = rpc_tx_queue_task_handle.now_or_never();
    assert!(rpc_tx_queue_task_result.is_none());
}

#[tokio::test(flavor = "multi_thread")]
async fn rpc_getaddresstxids_invalid_arguments() {
    let _init_guard = zebra_test::init();

    let mut mempool: MockService<_, _, _, BoxError> = MockService::build().for_unit_tests();

    // Create a continuous chain of mainnet blocks from genesis
    let blocks: Vec<Arc<Block>> = zebra_test::vectors::CONTINUOUS_MAINNET_BLOCKS
        .iter()
        .map(|(_height, block_bytes)| block_bytes.zcash_deserialize_into().unwrap())
        .collect();

    // Create a populated state service
    let (_state, read_state, latest_chain_tip, _chain_tip_change) =
        zebra_state::populated_state(blocks.clone(), &Mainnet).await;

    let (_tx, rx) = tokio::sync::watch::channel(None);
    let (rpc, rpc_tx_queue_task_handle) = RpcImpl::new(
        "0.0.1",
        "RPC test",
        Mainnet,
        false,
        true,
        Buffer::new(mempool.clone(), 1),
        Buffer::new(read_state.clone(), 1),
        latest_chain_tip,
        MockAddressBookPeers::new(vec![]),
        rx,
    );

    // call the method with an invalid address string
    let rpc_rsp = rpc
        .get_address_tx_ids(GetAddressTxIdsRequest {
            addresses: vec!["t1invalidaddress".to_owned()],
            start: Some(1),
            end: Some(2),
        })
        .await
        .unwrap_err();

    assert_eq!(rpc_rsp.code(), ErrorCode::ServerError(-5).code());

    mempool.expect_no_requests().await;

    // create a valid address
    let address = "t3Vz22vK5z2LcKEdg16Yv4FFneEL1zg9ojd".to_string();
    let addresses = vec![address.clone()];

    // call the method with start greater than end
    let start: Option<u32> = Some(2);
    let end: Option<u32> = Some(1);
    let error = rpc
        .get_address_tx_ids(GetAddressTxIdsRequest {
            addresses: addresses.clone(),
            start,
            end,
        })
        .await
        .unwrap_err();
    assert_eq!(
        error.message(),
        "start Height(2) must be less than or equal to end Height(1)".to_string()
    );

    mempool.expect_no_requests().await;

    // The queue task should continue without errors or panics
    let rpc_tx_queue_task_result = rpc_tx_queue_task_handle.now_or_never();
    assert!(rpc_tx_queue_task_result.is_none());
}

#[tokio::test(flavor = "multi_thread")]
async fn rpc_getaddresstxids_response() {
    let _init_guard = zebra_test::init();

    for network in Network::iter() {
        let blocks: Vec<Arc<Block>> = network
            .blockchain_map()
            .iter()
            .map(|(_height, block_bytes)| block_bytes.zcash_deserialize_into().unwrap())
            .collect();
        // The first few blocks after genesis send funds to the same founders reward address,
        // in one output per coinbase transaction.
        //
        // Get the coinbase transaction of the first block
        // (the genesis block coinbase transaction is ignored by the consensus rules).
        let first_block_first_transaction = &blocks[1].transactions[0];

        // Get the address.
        let address = first_block_first_transaction.outputs()[1]
            .address(&network)
            .unwrap();

        // Create a populated state service
        let (_state, read_state, latest_chain_tip, _chain_tip_change) =
            zebra_state::populated_state(blocks.to_owned(), &network).await;

        if network == Mainnet {
            // Exhaustively test possible block ranges for mainnet.
            for start in 1..=10 {
                for end in start..=10 {
                    rpc_getaddresstxids_response_with(
                        &network,
                        Some(start),
                        Some(end),
                        &address,
                        &read_state,
                        &latest_chain_tip,
                        (end - start + 1) as usize,
                    )
                    .await;
                }
            }
        } else {
            // Just test the full range for testnet.
            rpc_getaddresstxids_response_with(
                &network,
                Some(1),
                Some(10),
                &address,
                &read_state,
                &latest_chain_tip,
                // response should be limited to the chain size.
                10,
            )
            .await;
        }

        // No range arguments should be equivalent to the full range.
        rpc_getaddresstxids_response_with(
            &network,
            None,
            None,
            &address,
            &read_state,
            &latest_chain_tip,
            10,
        )
        .await;

        // Range of 0s should be equivalent to the full range.
        rpc_getaddresstxids_response_with(
            &network,
            Some(0),
            Some(0),
            &address,
            &read_state,
            &latest_chain_tip,
            10,
        )
        .await;

        // Start and outside of the range should use the chain tip.
        rpc_getaddresstxids_response_with(
            &network,
            Some(11),
            Some(11),
            &address,
            &read_state,
            &latest_chain_tip,
            1,
        )
        .await;

        // End outside the range should use the chain tip.
        rpc_getaddresstxids_response_with(
            &network,
            None,
            Some(11),
            &address,
            &read_state,
            &latest_chain_tip,
            10,
        )
        .await;

        // Start outside the range should use the chain tip.
        rpc_getaddresstxids_response_with(
            &network,
            Some(11),
            None,
            &address,
            &read_state,
            &latest_chain_tip,
            1,
        )
        .await;
    }
}

async fn rpc_getaddresstxids_response_with(
    network: &Network,
    start: Option<u32>,
    end: Option<u32>,
    address: &transparent::Address,
    read_state: &ReadStateService,
    latest_chain_tip: &LatestChainTip,
    expected_response_len: usize,
) {
    let mut mempool: MockService<_, _, _, BoxError> = MockService::build().for_unit_tests();

    let (_tx, rx) = tokio::sync::watch::channel(None);
    let (rpc, rpc_tx_queue_task_handle) = RpcImpl::new(
        "0.0.1",
        "RPC test",
        network.clone(),
        false,
        true,
        Buffer::new(mempool.clone(), 1),
        Buffer::new(read_state.clone(), 1),
        latest_chain_tip.clone(),
        MockAddressBookPeers::new(vec![]),
        rx,
    );

    // call the method with valid arguments
    let addresses = vec![address.to_string()];
    let response = rpc
        .get_address_tx_ids(GetAddressTxIdsRequest {
            addresses,
            start,
            end,
        })
        .await
        .expect("arguments are valid so no error can happen here");

    // One founders reward output per coinbase transactions, no other transactions.
    assert_eq!(response.len(), expected_response_len);

    mempool.expect_no_requests().await;

    // Shut down the queue task, to close the state's file descriptors.
    // (If we don't, opening ~100 simultaneous states causes process file descriptor limit errors.)
    //
    // TODO: abort all the join handles in all the tests, except one?
    rpc_tx_queue_task_handle.abort();

    // The queue task should not have panicked or exited by itself.
    // It can still be running, or it can have exited due to the abort.
    let rpc_tx_queue_task_result = rpc_tx_queue_task_handle.now_or_never();
    assert!(
        rpc_tx_queue_task_result.is_none()
            || rpc_tx_queue_task_result
                .unwrap()
                .unwrap_err()
                .is_cancelled()
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn rpc_getaddressutxos_invalid_arguments() {
    let _init_guard = zebra_test::init();

    let mut mempool: MockService<_, _, _, BoxError> = MockService::build().for_unit_tests();
    let mut state: MockService<_, _, _, BoxError> = MockService::build().for_unit_tests();

    let (_tx, rx) = tokio::sync::watch::channel(None);
    let rpc = RpcImpl::new(
        "0.0.1",
        "RPC test",
        Mainnet,
        false,
        true,
        Buffer::new(mempool.clone(), 1),
        Buffer::new(state.clone(), 1),
        NoChainTip,
        MockAddressBookPeers::new(vec![]),
        rx,
    );

    // call the method with an invalid address string
    let error = rpc
        .0
        .get_address_utxos(AddressStrings::new(vec!["t1invalidaddress".to_owned()]))
        .await
        .unwrap_err();

    assert_eq!(error.code(), ErrorCode::ServerError(-5).code());

    mempool.expect_no_requests().await;
    state.expect_no_requests().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn rpc_getaddressutxos_response() {
    let _init_guard = zebra_test::init();

    let blocks: Vec<Arc<Block>> = zebra_test::vectors::CONTINUOUS_MAINNET_BLOCKS
        .iter()
        .map(|(_height, block_bytes)| block_bytes.zcash_deserialize_into().unwrap())
        .collect();

    // get the first transaction of the first block
    let first_block_first_transaction = &blocks[1].transactions[0];
    // get the address, this is always `t3Vz22vK5z2LcKEdg16Yv4FFneEL1zg9ojd`
    let address = &first_block_first_transaction.outputs()[1]
        .address(&Mainnet)
        .unwrap();

    let mut mempool: MockService<_, _, _, BoxError> = MockService::build().for_unit_tests();
    // Create a populated state service
    let (_state, read_state, latest_chain_tip, _chain_tip_change) =
        zebra_state::populated_state(blocks.clone(), &Mainnet).await;

    let (_tx, rx) = tokio::sync::watch::channel(None);
    let rpc = RpcImpl::new(
        "0.0.1",
        "RPC test",
        Mainnet,
        false,
        true,
        Buffer::new(mempool.clone(), 1),
        Buffer::new(read_state.clone(), 1),
        latest_chain_tip,
        MockAddressBookPeers::new(vec![]),
        rx,
    );

    // call the method with a valid address
    let addresses = vec![address.to_string()];
    let response = rpc
        .0
        .get_address_utxos(AddressStrings::new(addresses))
        .await
        .expect("address is valid so no error can happen here");

    // there are 10 outputs for provided address
    assert_eq!(response.len(), 10);

    mempool.expect_no_requests().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn rpc_getblockcount() {
    use zebra_chain::chain_sync_status::MockSyncStatus;
    use zebra_network::address_book_peers::MockAddressBookPeers;

    let _init_guard = zebra_test::init();

    // Create a continuous chain of mainnet blocks from genesis
    let blocks: Vec<Arc<Block>> = zebra_test::vectors::CONTINUOUS_MAINNET_BLOCKS
        .iter()
        .map(|(_height, block_bytes)| block_bytes.zcash_deserialize_into().unwrap())
        .collect();

    // Get the height of the block at the tip using hardcoded block tip bytes.
    // We want to test the RPC response is equal to this hash
    let tip_block = blocks.last().unwrap();
    let tip_block_height = tip_block.coinbase_height().unwrap();

    // Get a mempool handle
    let mut mempool: MockService<_, _, _, BoxError> = MockService::build().for_unit_tests();
    // Create a populated state service, the tip will be in `NUMBER_OF_BLOCKS`.
    let (state, read_state, latest_chain_tip, _chain_tip_change) =
        zebra_state::populated_state(blocks.clone(), &Mainnet).await;

    let (
        block_verifier_router,
        _transaction_verifier,
        _parameter_download_task_handle,
        _max_checkpoint_height,
    ) = zebra_consensus::router::init_test(
        zebra_consensus::Config::default(),
        &Mainnet,
        state.clone(),
    )
    .await;

    // Init RPC
    let get_block_template_rpc = GetBlockTemplateRpcImpl::new(
        &Mainnet,
        Default::default(),
        Buffer::new(mempool.clone(), 1),
        read_state,
        latest_chain_tip.clone(),
        block_verifier_router,
        MockSyncStatus::default(),
        MockAddressBookPeers::default(),
        None,
    );

    // Get the tip height using RPC method `get_block_count`
    let get_block_count = get_block_template_rpc
        .get_block_count()
        .expect("We should have a number");

    // Check if response is equal to block 10 hash.
    assert_eq!(get_block_count, tip_block_height.0);

    mempool.expect_no_requests().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn rpc_getblockcount_empty_state() {
    use zebra_chain::chain_sync_status::MockSyncStatus;
    use zebra_network::address_book_peers::MockAddressBookPeers;

    let _init_guard = zebra_test::init();

    // Get a mempool handle
    let mut mempool: MockService<_, _, _, BoxError> = MockService::build().for_unit_tests();
    // Create an empty state
    let (state, read_state, latest_chain_tip, _chain_tip_change) =
        zebra_state::init_test_services(&Mainnet);

    let (
        block_verifier_router,
        _transaction_verifier,
        _parameter_download_task_handle,
        _max_checkpoint_height,
    ) = zebra_consensus::router::init_test(
        zebra_consensus::Config::default(),
        &Mainnet,
        state.clone(),
    )
    .await;

    // Init RPC
    let get_block_template_rpc = get_block_template_rpcs::GetBlockTemplateRpcImpl::new(
        &Mainnet,
        Default::default(),
        Buffer::new(mempool.clone(), 1),
        read_state,
        latest_chain_tip.clone(),
        block_verifier_router,
        MockSyncStatus::default(),
        MockAddressBookPeers::default(),
        None,
    );

    // Get the tip height using RPC method `get_block_count
    let get_block_count = get_block_template_rpc.get_block_count();

    // state an empty so we should get an error
    assert!(get_block_count.is_err());

    // Check the error we got is the correct one
    assert_eq!(
        get_block_count.err().unwrap().message(),
        "No blocks in state"
    );

    mempool.expect_no_requests().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn rpc_getpeerinfo() {
    use zebra_chain::chain_sync_status::MockSyncStatus;
    use zebra_network::{address_book_peers::MockAddressBookPeers, types::PeerServices};

    let _init_guard = zebra_test::init();
    let network = Mainnet;

    // Get a mempool handle
    let mut mempool: MockService<_, _, _, BoxError> = MockService::build().for_unit_tests();
    // Create an empty state
    let (state, read_state, latest_chain_tip, _chain_tip_change) =
        zebra_state::init_test_services(&Mainnet);

    let (
        block_verifier_router,
        _transaction_verifier,
        _parameter_download_task_handle,
        _max_checkpoint_height,
    ) = zebra_consensus::router::init_test(
        zebra_consensus::Config::default(),
        &network,
        state.clone(),
    )
    .await;

    // Add a connected outbound peer
    let outbound_mock_peer_address = zebra_network::types::MetaAddr::new_connected(
        std::net::SocketAddr::new(
            std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)),
            network.default_port(),
        )
        .into(),
        &PeerServices::NODE_NETWORK,
        false,
    )
    .into_new_meta_addr(
        std::time::Instant::now(),
        zebra_chain::serialization::DateTime32::now(),
    );

    // Add a connected inbound peer
    let inbound_mock_peer_address = zebra_network::types::MetaAddr::new_connected(
        std::net::SocketAddr::new(
            std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)),
            44444,
        )
        .into(),
        &PeerServices::NODE_NETWORK,
        true,
    )
    .into_new_meta_addr(
        std::time::Instant::now(),
        zebra_chain::serialization::DateTime32::now(),
    );

    // Add a peer that is not connected and will not be displayed in the RPC output
    let not_connected_mock_peer_adderess = zebra_network::types::MetaAddr::new_initial_peer(
        std::net::SocketAddr::new(
            std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)),
            55555,
        )
        .into(),
    )
    .into_new_meta_addr(
        std::time::Instant::now(),
        zebra_chain::serialization::DateTime32::now(),
    );

    let mock_address_book = MockAddressBookPeers::new(vec![
        outbound_mock_peer_address,
        inbound_mock_peer_address,
        not_connected_mock_peer_adderess,
    ]);

    // Init RPC
    let get_block_template_rpc = get_block_template_rpcs::GetBlockTemplateRpcImpl::new(
        &network,
        Default::default(),
        Buffer::new(mempool.clone(), 1),
        read_state,
        latest_chain_tip.clone(),
        block_verifier_router,
        MockSyncStatus::default(),
        mock_address_book,
        None,
    );

    // Call `get_peer_info`
    let get_peer_info = get_block_template_rpc
        .get_peer_info()
        .await
        .expect("We should have an array of addresses");

    // Response of length should be 2. We have 2 connected peers and 1 unconnected peer in the address book.
    assert_eq!(get_peer_info.len(), 2);

    let mut res_iter = get_peer_info.into_iter();
    // Check for the outbound peer
    assert_eq!(
        res_iter
            .next()
            .expect("there should be a mock peer address"),
        outbound_mock_peer_address.into()
    );

    // Check for the inbound peer
    assert_eq!(
        res_iter
            .next()
            .expect("there should be a mock peer address"),
        inbound_mock_peer_address.into()
    );

    mempool.expect_no_requests().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn rpc_getblockhash() {
    use zebra_chain::chain_sync_status::MockSyncStatus;
    use zebra_network::address_book_peers::MockAddressBookPeers;

    let _init_guard = zebra_test::init();

    // Create a continuous chain of mainnet blocks from genesis
    let blocks: Vec<Arc<Block>> = zebra_test::vectors::CONTINUOUS_MAINNET_BLOCKS
        .iter()
        .map(|(_height, block_bytes)| block_bytes.zcash_deserialize_into().unwrap())
        .collect();

    let mut mempool: MockService<_, _, _, BoxError> = MockService::build().for_unit_tests();
    // Create a populated state service
    let (state, read_state, latest_chain_tip, _chain_tip_change) =
        zebra_state::populated_state(blocks.clone(), &Mainnet).await;

    let (
        block_verifier_router,
        _transaction_verifier,
        _parameter_download_task_handle,
        _max_checkpoint_height,
    ) = zebra_consensus::router::init_test(
        zebra_consensus::Config::default(),
        &Mainnet,
        state.clone(),
    )
    .await;

    // Init RPC
    let get_block_template_rpc = get_block_template_rpcs::GetBlockTemplateRpcImpl::new(
        &Mainnet,
        Default::default(),
        Buffer::new(mempool.clone(), 1),
        read_state,
        latest_chain_tip.clone(),
        tower::ServiceBuilder::new().service(block_verifier_router),
        MockSyncStatus::default(),
        MockAddressBookPeers::default(),
        None,
    );

    // Query the hashes using positive indexes
    for (i, block) in blocks.iter().enumerate() {
        let get_block_hash = get_block_template_rpc
            .get_block_hash(i.try_into().expect("usize always fits in i32"))
            .await
            .expect("We should have a GetBlockHash struct");

        assert_eq!(get_block_hash, GetBlockHash(block.clone().hash()));
    }

    // Query the hashes using negative indexes
    for i in (-10..=-1).rev() {
        let get_block_hash = get_block_template_rpc
            .get_block_hash(i)
            .await
            .expect("We should have a GetBlockHash struct");

        assert_eq!(
            get_block_hash,
            GetBlockHash(blocks[(10 + (i + 1)) as usize].hash())
        );
    }

    mempool.expect_no_requests().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn rpc_getmininginfo() {
    use zebra_chain::chain_sync_status::MockSyncStatus;
    use zebra_network::address_book_peers::MockAddressBookPeers;

    let _init_guard = zebra_test::init();

    // Create a continuous chain of mainnet blocks from genesis
    let blocks: Vec<Arc<Block>> = zebra_test::vectors::CONTINUOUS_MAINNET_BLOCKS
        .iter()
        .map(|(_height, block_bytes)| block_bytes.zcash_deserialize_into().unwrap())
        .collect();

    // Create a populated state service
    let (_state, read_state, latest_chain_tip, _chain_tip_change) =
        zebra_state::populated_state(blocks.clone(), &Mainnet).await;

    // Init RPC
    let get_block_template_rpc = get_block_template_rpcs::GetBlockTemplateRpcImpl::new(
        &Mainnet,
        Default::default(),
        MockService::build().for_unit_tests(),
        read_state,
        latest_chain_tip.clone(),
        MockService::build().for_unit_tests(),
        MockSyncStatus::default(),
        MockAddressBookPeers::default(),
        None,
    );

    get_block_template_rpc
        .get_mining_info()
        .await
        .expect("get_mining_info call should succeed");
}

#[tokio::test(flavor = "multi_thread")]
async fn rpc_getnetworksolps() {
    use zebra_chain::chain_sync_status::MockSyncStatus;
    use zebra_network::address_book_peers::MockAddressBookPeers;

    let _init_guard = zebra_test::init();

    // Create a continuous chain of mainnet blocks from genesis
    let blocks: Vec<Arc<Block>> = zebra_test::vectors::CONTINUOUS_MAINNET_BLOCKS
        .iter()
        .map(|(_height, block_bytes)| block_bytes.zcash_deserialize_into().unwrap())
        .collect();

    // Create a populated state service
    let (_state, read_state, latest_chain_tip, _chain_tip_change) =
        zebra_state::populated_state(blocks.clone(), &Mainnet).await;

    // Init RPC
    let get_block_template_rpc = get_block_template_rpcs::GetBlockTemplateRpcImpl::new(
        &Mainnet,
        Default::default(),
        MockService::build().for_unit_tests(),
        read_state,
        latest_chain_tip.clone(),
        MockService::build().for_unit_tests(),
        MockSyncStatus::default(),
        MockAddressBookPeers::default(),
        None,
    );

    let get_network_sol_ps_inputs = [
        // num_blocks, height, return value
        (None, None, Ok(2)),
        (Some(-4), None, Ok(2)),
        (Some(-3), Some(0), Ok(0)),
        (Some(-2), Some(-4), Ok(2)),
        (Some(-1), Some(10), Ok(2)),
        (Some(-1), Some(i32::MAX), Ok(2)),
        (Some(0), None, Ok(2)),
        (Some(0), Some(0), Ok(0)),
        (Some(0), Some(-3), Ok(2)),
        (Some(0), Some(10), Ok(2)),
        (Some(0), Some(i32::MAX), Ok(2)),
        (Some(1), None, Ok(4096)),
        (Some(1), Some(0), Ok(0)),
        (Some(1), Some(-2), Ok(4096)),
        (Some(1), Some(10), Ok(4096)),
        (Some(1), Some(i32::MAX), Ok(4096)),
        (Some(i32::MAX), None, Ok(2)),
        (Some(i32::MAX), Some(0), Ok(0)),
        (Some(i32::MAX), Some(-1), Ok(2)),
        (Some(i32::MAX), Some(10), Ok(2)),
        (Some(i32::MAX), Some(i32::MAX), Ok(2)),
    ];

    for (num_blocks_input, height_input, return_value) in get_network_sol_ps_inputs {
        let get_network_sol_ps_result = get_block_template_rpc
            .get_network_sol_ps(num_blocks_input, height_input)
            .await;
        assert_eq!(
            get_network_sol_ps_result, return_value,
            "get_network_sol_ps({num_blocks_input:?}, {height_input:?}) result\n\
             should be {return_value:?},\n\
             got: {get_network_sol_ps_result:?}"
        );
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn rpc_getblocktemplate() {
    // test getblocktemplate with a miner P2SH address
    rpc_getblocktemplate_mining_address(true).await;
    // test getblocktemplate with a miner P2PKH address
    rpc_getblocktemplate_mining_address(false).await;
}

async fn rpc_getblocktemplate_mining_address(use_p2pkh: bool) {
    use zebra_chain::{
        amount::NonNegative,
        block::{Hash, MAX_BLOCK_BYTES, ZCASH_BLOCK_VERSION},
        chain_sync_status::MockSyncStatus,
        serialization::DateTime32,
        transaction::{zip317, VerifiedUnminedTx},
        work::difficulty::{CompactDifficulty, ExpandedDifficulty, U256},
    };
    use zebra_consensus::MAX_BLOCK_SIGOPS;
    use zebra_network::address_book_peers::MockAddressBookPeers;
    use zebra_state::{GetBlockTemplateChainInfo, ReadRequest, ReadResponse};

    use crate::methods::{
        get_block_template_rpcs::{
            constants::{
                GET_BLOCK_TEMPLATE_CAPABILITIES_FIELD, GET_BLOCK_TEMPLATE_MUTABLE_FIELD,
                GET_BLOCK_TEMPLATE_NONCE_RANGE_FIELD,
            },
            get_block_template::{self, GetBlockTemplateRequestMode},
            types::long_poll::LONG_POLL_ID_LENGTH,
        },
        hex_data::HexData,
        tests::utils::fake_history_tree,
    };

    let _init_guard = zebra_test::init();

    let mut mempool: MockService<_, _, _, BoxError> = MockService::build().for_unit_tests();

    let read_state = MockService::build().for_unit_tests();
    let block_verifier_router = MockService::build().for_unit_tests();

    let mut mock_sync_status = MockSyncStatus::default();
    mock_sync_status.set_is_close_to_tip(true);

    let network = Network::Mainnet;
    let miner_address = match use_p2pkh {
        false => Some(transparent::Address::from_script_hash(
            network.kind(),
            [0x7e; 20],
        )),
        true => Some(transparent::Address::from_pub_key_hash(
            network.kind(),
            [0x7e; 20],
        )),
    };

    #[allow(clippy::unnecessary_struct_initialization)]
    let mining_config = crate::config::mining::Config {
        miner_address: miner_address.clone(),
        extra_coinbase_data: None,
        debug_like_zcashd: true,
        internal_miner: true,
    };

    // nu5 block height
    let fake_tip_height = NetworkUpgrade::Nu5.activation_height(&Mainnet).unwrap();
    // nu5 block hash
    let fake_tip_hash =
        Hash::from_hex("0000000000d723156d9b65ffcf4984da7a19675ed7e2f06d9e5d5188af087bf8").unwrap();
    //  nu5 block time + 1
    let fake_min_time = DateTime32::from(1654008606);
    // nu5 block time + 12
    let fake_cur_time = DateTime32::from(1654008617);
    // nu5 block time + 123
    let fake_max_time = DateTime32::from(1654008728);
    let fake_difficulty = CompactDifficulty::from(ExpandedDifficulty::from(U256::one()));

    let (mock_chain_tip, mock_chain_tip_sender) = MockChainTip::new();
    mock_chain_tip_sender.send_best_tip_height(fake_tip_height);
    mock_chain_tip_sender.send_best_tip_hash(fake_tip_hash);
    mock_chain_tip_sender.send_estimated_distance_to_network_chain_tip(Some(0));

    // Init RPC
    let get_block_template_rpc = GetBlockTemplateRpcImpl::new(
        &Mainnet,
        mining_config,
        Buffer::new(mempool.clone(), 1),
        read_state.clone(),
        mock_chain_tip,
        block_verifier_router,
        mock_sync_status.clone(),
        MockAddressBookPeers::default(),
        None,
    );

    // Fake the ChainInfo response
    let make_mock_read_state_request_handler = || {
        let mut read_state = read_state.clone();

        async move {
            read_state
                .expect_request_that(|req| matches!(req, ReadRequest::ChainInfo))
                .await
                .respond(ReadResponse::ChainInfo(GetBlockTemplateChainInfo {
                    expected_difficulty: fake_difficulty,
                    tip_height: fake_tip_height,
                    tip_hash: fake_tip_hash,
                    cur_time: fake_cur_time,
                    min_time: fake_min_time,
                    max_time: fake_max_time,
                    chain_history_root: fake_history_tree(&Mainnet).hash(),
                }));
        }
    };

    let make_mock_mempool_request_handler = |transactions, last_seen_tip_hash| {
        let mut mempool = mempool.clone();
        async move {
            mempool
                .expect_request(mempool::Request::FullTransactions)
                .await
                .respond(mempool::Response::FullTransactions {
                    transactions,
                    transaction_dependencies: Default::default(),
                    last_seen_tip_hash,
                });
        }
    };

    let get_block_template_fut = get_block_template_rpc.get_block_template(None);
    let (get_block_template, ..) = tokio::join!(
        get_block_template_fut,
        make_mock_mempool_request_handler(vec![], fake_tip_hash),
        make_mock_read_state_request_handler(),
    );

    let get_block_template::Response::TemplateMode(get_block_template) =
        get_block_template.expect("unexpected error in getblocktemplate RPC call")
    else {
        panic!("this getblocktemplate call without parameters should return the `TemplateMode` variant of the response")
    };

    let coinbase_transaction =
        Transaction::zcash_deserialize(get_block_template.coinbase_txn.data.as_ref())
            .expect("coinbase transaction data should be deserializable");

    assert_eq!(
        coinbase_transaction
            .outputs()
            .first()
            .unwrap()
            .address(&network),
        miner_address
    );

    assert_eq!(
        get_block_template.capabilities,
        GET_BLOCK_TEMPLATE_CAPABILITIES_FIELD.to_vec()
    );
    assert_eq!(get_block_template.version, ZCASH_BLOCK_VERSION);
    assert!(get_block_template.transactions.is_empty());
    assert_eq!(
        get_block_template.target,
        ExpandedDifficulty::from_hex(
            "0000000000000000000000000000000000000000000000000000000000000001"
        )
        .expect("test vector is valid")
    );
    assert_eq!(get_block_template.min_time, fake_min_time);
    assert_eq!(
        get_block_template.mutable,
        GET_BLOCK_TEMPLATE_MUTABLE_FIELD.to_vec()
    );
    assert_eq!(
        get_block_template.nonce_range,
        GET_BLOCK_TEMPLATE_NONCE_RANGE_FIELD
    );
    assert_eq!(get_block_template.sigop_limit, MAX_BLOCK_SIGOPS);
    assert_eq!(get_block_template.size_limit, MAX_BLOCK_BYTES);
    assert_eq!(get_block_template.cur_time, fake_cur_time);
    assert_eq!(
        get_block_template.bits,
        CompactDifficulty::from_hex("01010000").expect("test vector is valid")
    );
    assert_eq!(get_block_template.height, 1687105); // nu5 height
    assert_eq!(get_block_template.max_time, fake_max_time);

    // Coinbase transaction checks.
    assert!(get_block_template.coinbase_txn.required);
    assert!(!get_block_template.coinbase_txn.data.as_ref().is_empty());
    assert_eq!(get_block_template.coinbase_txn.depends.len(), 0);
    if use_p2pkh {
        // there is one sig operation if miner address is p2pkh.
        assert_eq!(get_block_template.coinbase_txn.sigops, 1);
    } else {
        // everything in the coinbase is p2sh.
        assert_eq!(get_block_template.coinbase_txn.sigops, 0);
    }
    // Coinbase transaction checks for empty blocks.
    assert_eq!(
        get_block_template.coinbase_txn.fee,
        Amount::<NonNegative>::zero()
    );

    mock_chain_tip_sender.send_estimated_distance_to_network_chain_tip(Some(200));
    let get_block_template_sync_error = get_block_template_rpc
        .get_block_template(None)
        .await
        .expect_err("needs an error when estimated distance to network chain tip is far");

    assert_eq!(
        get_block_template_sync_error.code(),
        ErrorCode::ServerError(-10).code()
    );

    mock_sync_status.set_is_close_to_tip(false);

    mock_chain_tip_sender.send_estimated_distance_to_network_chain_tip(Some(0));
    let get_block_template_sync_error = get_block_template_rpc
        .get_block_template(None)
        .await
        .expect_err("needs an error when syncer is not close to tip");

    assert_eq!(
        get_block_template_sync_error.code(),
        ErrorCode::ServerError(-10).code()
    );

    mock_chain_tip_sender.send_estimated_distance_to_network_chain_tip(Some(200));
    let get_block_template_sync_error = get_block_template_rpc
        .get_block_template(None)
        .await
        .expect_err("needs an error when syncer is not close to tip or estimated distance to network chain tip is far");

    assert_eq!(
        get_block_template_sync_error.code(),
        ErrorCode::ServerError(-10).code()
    );

    let get_block_template_sync_error = get_block_template_rpc
        .get_block_template(Some(get_block_template::JsonParameters {
            mode: GetBlockTemplateRequestMode::Proposal,
            ..Default::default()
        }))
        .await
        .expect_err("needs an error when called in proposal mode without data");

    assert_eq!(
        get_block_template_sync_error.code(),
        ErrorCode::InvalidParams.code()
    );

    let get_block_template_sync_error = get_block_template_rpc
        .get_block_template(Some(get_block_template::JsonParameters {
            data: Some(HexData("".into())),
            ..Default::default()
        }))
        .await
        .expect_err("needs an error when passing in block data in template mode");

    assert_eq!(
        get_block_template_sync_error.code(),
        ErrorCode::InvalidParams.code()
    );

    // The long poll id is valid, so it returns a state error instead
    let get_block_template_sync_error = get_block_template_rpc
        .get_block_template(Some(get_block_template::JsonParameters {
            // This must parse as a LongPollId.
            // It must be the correct length and have hex/decimal digits.
            long_poll_id: Some(
                "0".repeat(LONG_POLL_ID_LENGTH)
                    .parse()
                    .expect("unexpected invalid LongPollId"),
            ),
            ..Default::default()
        }))
        .await
        .expect_err("needs an error when the state is empty");

    assert_eq!(
        get_block_template_sync_error.code(),
        ErrorCode::ServerError(-10).code()
    );

    // Try getting mempool transactions with a different tip hash

    let tx = Arc::new(Transaction::V1 {
        inputs: vec![],
        outputs: vec![],
        lock_time: transaction::LockTime::unlocked(),
    });

    let unmined_tx = UnminedTx {
        transaction: tx.clone(),
        id: tx.unmined_id(),
        size: tx.zcash_serialized_size(),
        conventional_fee: 0.try_into().unwrap(),
    };

    let conventional_actions = zip317::conventional_actions(&unmined_tx.transaction);

    let verified_unmined_tx = VerifiedUnminedTx {
        transaction: unmined_tx,
        miner_fee: 0.try_into().unwrap(),
        legacy_sigop_count: 0,
        conventional_actions,
        unpaid_actions: 0,
        fee_weight_ratio: 1.0,
        time: None,
        height: None,
    };

    let next_fake_tip_hash =
        Hash::from_hex("0000000000b6a5024aa412120b684a509ba8fd57e01de07bc2a84e4d3719a9f1").unwrap();

    mock_sync_status.set_is_close_to_tip(true);

    mock_chain_tip_sender.send_estimated_distance_to_network_chain_tip(Some(0));

    let (get_block_template, ..) = tokio::join!(
        get_block_template_rpc.get_block_template(None),
        make_mock_mempool_request_handler(vec![verified_unmined_tx], next_fake_tip_hash),
        make_mock_read_state_request_handler(),
    );

    let get_block_template::Response::TemplateMode(get_block_template) =
        get_block_template.expect("unexpected error in getblocktemplate RPC call")
    else {
        panic!("this getblocktemplate call without parameters should return the `TemplateMode` variant of the response")
    };

    // mempool transactions should be omitted if the tip hash in the GetChainInfo response from the state
    // does not match the `last_seen_tip_hash` in the FullTransactions response from the mempool.
    assert!(get_block_template.transactions.is_empty());

    mempool.expect_no_requests().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn rpc_submitblock_errors() {
    use zebra_chain::chain_sync_status::MockSyncStatus;
    use zebra_network::address_book_peers::MockAddressBookPeers;

    use crate::methods::{get_block_template_rpcs::types::submit_block, hex_data::HexData};

    let _init_guard = zebra_test::init();

    // Create a continuous chain of mainnet blocks from genesis
    let blocks: Vec<Arc<Block>> = zebra_test::vectors::CONTINUOUS_MAINNET_BLOCKS
        .iter()
        .map(|(_height, block_bytes)| block_bytes.zcash_deserialize_into().unwrap())
        .collect();

    let mut mempool: MockService<_, _, _, BoxError> = MockService::build().for_unit_tests();
    // Create a populated state service
    let (state, read_state, latest_chain_tip, _chain_tip_change) =
        zebra_state::populated_state(blocks, &Mainnet).await;

    // Init RPCs
    let (
        block_verifier_router,
        _transaction_verifier,
        _parameter_download_task_handle,
        _max_checkpoint_height,
    ) = zebra_consensus::router::init_test(
        zebra_consensus::Config::default(),
        &Mainnet,
        state.clone(),
    )
    .await;

    // Init RPC
    let get_block_template_rpc = GetBlockTemplateRpcImpl::new(
        &Mainnet,
        Default::default(),
        Buffer::new(mempool.clone(), 1),
        read_state,
        latest_chain_tip.clone(),
        block_verifier_router,
        MockSyncStatus::default(),
        MockAddressBookPeers::default(),
        None,
    );

    // Try to submit pre-populated blocks and assert that it responds with duplicate.
    for (_height, &block_bytes) in zebra_test::vectors::CONTINUOUS_MAINNET_BLOCKS.iter() {
        let submit_block_response = get_block_template_rpc
            .submit_block(HexData(block_bytes.into()), None)
            .await;

        assert_eq!(
            submit_block_response,
            Ok(submit_block::ErrorResponse::Duplicate.into())
        );
    }

    let submit_block_response = get_block_template_rpc
        .submit_block(
            HexData(zebra_test::vectors::BAD_BLOCK_MAINNET_202_BYTES.to_vec()),
            None,
        )
        .await;

    assert_eq!(
        submit_block_response,
        Ok(submit_block::ErrorResponse::Rejected.into())
    );

    mempool.expect_no_requests().await;

    // See zebrad::tests::acceptance::submit_block for success case.
}

#[tokio::test(flavor = "multi_thread")]
async fn rpc_validateaddress() {
    use get_block_template_rpcs::types::validate_address;
    use zebra_chain::{chain_sync_status::MockSyncStatus, chain_tip::mock::MockChainTip};
    use zebra_network::address_book_peers::MockAddressBookPeers;

    let _init_guard = zebra_test::init();

    let (mock_chain_tip, _mock_chain_tip_sender) = MockChainTip::new();

    // Init RPC
    let get_block_template_rpc = get_block_template_rpcs::GetBlockTemplateRpcImpl::new(
        &Mainnet,
        Default::default(),
        MockService::build().for_unit_tests(),
        MockService::build().for_unit_tests(),
        mock_chain_tip,
        MockService::build().for_unit_tests(),
        MockSyncStatus::default(),
        MockAddressBookPeers::default(),
        None,
    );

    let validate_address = get_block_template_rpc
        .validate_address("t3fqvkzrrNaMcamkQMwAyHRjfDdM2xQvDTR".to_string())
        .await
        .expect("we should have a validate_address::Response");

    assert!(
        validate_address.is_valid,
        "Mainnet founder address should be valid on Mainnet"
    );

    let validate_address = get_block_template_rpc
        .validate_address("t2UNzUUx8mWBCRYPRezvA363EYXyEpHokyi".to_string())
        .await
        .expect("We should have a validate_address::Response");

    assert_eq!(
        validate_address,
        validate_address::Response::invalid(),
        "Testnet founder address should be invalid on Mainnet"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn rpc_z_validateaddress() {
    use get_block_template_rpcs::types::z_validate_address;
    use zebra_chain::{chain_sync_status::MockSyncStatus, chain_tip::mock::MockChainTip};
    use zebra_network::address_book_peers::MockAddressBookPeers;

    let _init_guard = zebra_test::init();

    let (mock_chain_tip, _mock_chain_tip_sender) = MockChainTip::new();

    // Init RPC
    let get_block_template_rpc = get_block_template_rpcs::GetBlockTemplateRpcImpl::new(
        &Mainnet,
        Default::default(),
        MockService::build().for_unit_tests(),
        MockService::build().for_unit_tests(),
        mock_chain_tip,
        MockService::build().for_unit_tests(),
        MockSyncStatus::default(),
        MockAddressBookPeers::default(),
        None,
    );

    let z_validate_address = get_block_template_rpc
        .z_validate_address("t3fqvkzrrNaMcamkQMwAyHRjfDdM2xQvDTR".to_string())
        .await
        .expect("we should have a z_validate_address::Response");

    assert!(
        z_validate_address.is_valid,
        "Mainnet founder address should be valid on Mainnet"
    );

    let z_validate_address = get_block_template_rpc
        .z_validate_address("t2UNzUUx8mWBCRYPRezvA363EYXyEpHokyi".to_string())
        .await
        .expect("We should have a z_validate_address::Response");

    assert_eq!(
        z_validate_address,
        z_validate_address::Response::invalid(),
        "Testnet founder address should be invalid on Mainnet"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn rpc_getdifficulty() {
    use zebra_chain::{
        block::Hash,
        chain_sync_status::MockSyncStatus,
        chain_tip::mock::MockChainTip,
        serialization::DateTime32,
        work::difficulty::{CompactDifficulty, ExpandedDifficulty, ParameterDifficulty as _, U256},
    };

    use zebra_network::address_book_peers::MockAddressBookPeers;

    use zebra_state::{GetBlockTemplateChainInfo, ReadRequest, ReadResponse};

    use crate::{config::mining::Config, methods::tests::utils::fake_history_tree};

    let _init_guard = zebra_test::init();

    let mempool: MockService<_, _, _, BoxError> = MockService::build().for_unit_tests();

    let read_state = MockService::build().for_unit_tests();
    let block_verifier_router = MockService::build().for_unit_tests();

    let mut mock_sync_status = MockSyncStatus::default();
    mock_sync_status.set_is_close_to_tip(true);

    #[allow(clippy::unnecessary_struct_initialization)]
    let mining_config = Config {
        miner_address: None,
        extra_coinbase_data: None,
        debug_like_zcashd: true,
        internal_miner: true,
    };

    // nu5 block height
    let fake_tip_height = NetworkUpgrade::Nu5.activation_height(&Mainnet).unwrap();
    // nu5 block hash
    let fake_tip_hash =
        Hash::from_hex("0000000000d723156d9b65ffcf4984da7a19675ed7e2f06d9e5d5188af087bf8").unwrap();
    //  nu5 block time + 1
    let fake_min_time = DateTime32::from(1654008606);
    // nu5 block time + 12
    let fake_cur_time = DateTime32::from(1654008617);
    // nu5 block time + 123
    let fake_max_time = DateTime32::from(1654008728);

    let (mock_chain_tip, mock_chain_tip_sender) = MockChainTip::new();
    mock_chain_tip_sender.send_best_tip_height(fake_tip_height);
    mock_chain_tip_sender.send_best_tip_hash(fake_tip_hash);
    mock_chain_tip_sender.send_estimated_distance_to_network_chain_tip(Some(0));

    // Init RPC
    let get_block_template_rpc = GetBlockTemplateRpcImpl::new(
        &Mainnet,
        mining_config,
        Buffer::new(mempool.clone(), 1),
        read_state.clone(),
        mock_chain_tip,
        block_verifier_router,
        mock_sync_status.clone(),
        MockAddressBookPeers::default(),
        None,
    );

    // Fake the ChainInfo response: smallest numeric difficulty
    // (this is invalid on mainnet and testnet under the consensus rules)
    let fake_difficulty = CompactDifficulty::from(ExpandedDifficulty::from(U256::MAX));
    let mut read_state1 = read_state.clone();
    let mock_read_state_request_handler = async move {
        read_state1
            .expect_request_that(|req| matches!(req, ReadRequest::ChainInfo))
            .await
            .respond(ReadResponse::ChainInfo(GetBlockTemplateChainInfo {
                expected_difficulty: fake_difficulty,
                tip_height: fake_tip_height,
                tip_hash: fake_tip_hash,
                cur_time: fake_cur_time,
                min_time: fake_min_time,
                max_time: fake_max_time,
                chain_history_root: fake_history_tree(&Mainnet).hash(),
            }));
    };

    let get_difficulty_fut = get_block_template_rpc.get_difficulty();
    let (get_difficulty, ..) = tokio::join!(get_difficulty_fut, mock_read_state_request_handler,);

    // Our implementation is slightly different to `zcashd`, so we require 6 significant figures
    // of accuracy in our unit tests. (Most clients will hide more than 2-3.)
    assert_eq!(format!("{:.9}", get_difficulty.unwrap()), "0.000122072");

    // Fake the ChainInfo response: difficulty limit - smallest valid difficulty
    let pow_limit = Mainnet.target_difficulty_limit();
    let fake_difficulty = pow_limit.into();
    let mut read_state2 = read_state.clone();
    let mock_read_state_request_handler = async move {
        read_state2
            .expect_request_that(|req| matches!(req, ReadRequest::ChainInfo))
            .await
            .respond(ReadResponse::ChainInfo(GetBlockTemplateChainInfo {
                expected_difficulty: fake_difficulty,
                tip_height: fake_tip_height,
                tip_hash: fake_tip_hash,
                cur_time: fake_cur_time,
                min_time: fake_min_time,
                max_time: fake_max_time,
                chain_history_root: fake_history_tree(&Mainnet).hash(),
            }));
    };

    let get_difficulty_fut = get_block_template_rpc.get_difficulty();
    let (get_difficulty, ..) = tokio::join!(get_difficulty_fut, mock_read_state_request_handler,);

    assert_eq!(format!("{:.5}", get_difficulty.unwrap()), "1.00000");

    // Fake the ChainInfo response: fractional difficulty
    let fake_difficulty = pow_limit * 2 / 3;
    let mut read_state3 = read_state.clone();
    let mock_read_state_request_handler = async move {
        read_state3
            .expect_request_that(|req| matches!(req, ReadRequest::ChainInfo))
            .await
            .respond(ReadResponse::ChainInfo(GetBlockTemplateChainInfo {
                expected_difficulty: fake_difficulty.into(),
                tip_height: fake_tip_height,
                tip_hash: fake_tip_hash,
                cur_time: fake_cur_time,
                min_time: fake_min_time,
                max_time: fake_max_time,
                chain_history_root: fake_history_tree(&Mainnet).hash(),
            }));
    };

    let get_difficulty_fut = get_block_template_rpc.get_difficulty();
    let (get_difficulty, ..) = tokio::join!(get_difficulty_fut, mock_read_state_request_handler,);

    assert_eq!(format!("{:.5}", get_difficulty.unwrap()), "1.50000");

    // Fake the ChainInfo response: large integer difficulty
    let fake_difficulty = pow_limit / 4096;
    let mut read_state4 = read_state.clone();
    let mock_read_state_request_handler = async move {
        read_state4
            .expect_request_that(|req| matches!(req, ReadRequest::ChainInfo))
            .await
            .respond(ReadResponse::ChainInfo(GetBlockTemplateChainInfo {
                expected_difficulty: fake_difficulty.into(),
                tip_height: fake_tip_height,
                tip_hash: fake_tip_hash,
                cur_time: fake_cur_time,
                min_time: fake_min_time,
                max_time: fake_max_time,
                chain_history_root: fake_history_tree(&Mainnet).hash(),
            }));
    };

    let get_difficulty_fut = get_block_template_rpc.get_difficulty();
    let (get_difficulty, ..) = tokio::join!(get_difficulty_fut, mock_read_state_request_handler,);

    assert_eq!(format!("{:.2}", get_difficulty.unwrap()), "4096.00");
}

#[tokio::test(flavor = "multi_thread")]
async fn rpc_z_listunifiedreceivers() {
    let _init_guard = zebra_test::init();

    use zebra_chain::{chain_sync_status::MockSyncStatus, chain_tip::mock::MockChainTip};
    use zebra_network::address_book_peers::MockAddressBookPeers;

    let _init_guard = zebra_test::init();

    let (mock_chain_tip, _mock_chain_tip_sender) = MockChainTip::new();

    // Init RPC
    let get_block_template_rpc = get_block_template_rpcs::GetBlockTemplateRpcImpl::new(
        &Mainnet,
        Default::default(),
        MockService::build().for_unit_tests(),
        MockService::build().for_unit_tests(),
        mock_chain_tip,
        MockService::build().for_unit_tests(),
        MockSyncStatus::default(),
        MockAddressBookPeers::default(),
        None,
    );

    // invalid address
    assert!(get_block_template_rpc
        .z_list_unified_receivers("invalid string for an address".to_string())
        .await
        .is_err());

    // address taken from https://github.com/zcash-hackworks/zcash-test-vectors/blob/master/test-vectors/zcash/unified_address.json#L4
    let response = get_block_template_rpc.z_list_unified_receivers("u1l8xunezsvhq8fgzfl7404m450nwnd76zshscn6nfys7vyz2ywyh4cc5daaq0c7q2su5lqfh23sp7fkf3kt27ve5948mzpfdvckzaect2jtte308mkwlycj2u0eac077wu70vqcetkxf".to_string()).await.unwrap();
    assert_eq!(response.orchard(), None);
    assert_eq!(
        response.sapling(),
        Some(String::from(
            "zs1mrhc9y7jdh5r9ece8u5khgvj9kg0zgkxzdduyv0whkg7lkcrkx5xqem3e48avjq9wn2rukydkwn"
        ))
    );
    assert_eq!(
        response.p2pkh(),
        Some(String::from("t1V9mnyk5Z5cTNMCkLbaDwSskgJZucTLdgW"))
    );
    assert_eq!(response.p2sh(), None);

    // address taken from https://github.com/zcash-hackworks/zcash-test-vectors/blob/master/test-vectors/zcash/unified_address.json#L39
    let response = get_block_template_rpc.z_list_unified_receivers("u12acx92vw49jek4lwwnjtzm0cssn2wxfneu7ryj4amd8kvnhahdrq0htsnrwhqvl92yg92yut5jvgygk0rqfs4lgthtycsewc4t57jyjn9p2g6ffxek9rdg48xe5kr37hxxh86zxh2ef0u2lu22n25xaf3a45as6mtxxlqe37r75mndzu9z2fe4h77m35c5mrzf4uqru3fjs39ednvw9ay8nf9r8g9jx8rgj50mj098exdyq803hmqsek3dwlnz4g5whc88mkvvjnfmjldjs9hm8rx89ctn5wxcc2e05rcz7m955zc7trfm07gr7ankf96jxwwfcqppmdefj8gc6508gep8ndrml34rdpk9tpvwzgdcv7lk2d70uh5jqacrpk6zsety33qcc554r3cls4ajktg03d9fye6exk8gnve562yadzsfmfh9d7v6ctl5ufm9ewpr6se25c47huk4fh2hakkwerkdd2yy3093snsgree5lt6smejfvse8v".to_string()).await.unwrap();
    assert_eq!(response.orchard(), Some(String::from("u10c5q7qkhu6f0ktaz7jqu4sfsujg0gpsglzudmy982mku7t0uma52jmsaz8h24a3wa7p0jwtsjqt8shpg25cvyexzlsw3jtdz4v6w70lv")));
    assert_eq!(response.sapling(), None);
    assert_eq!(
        response.p2pkh(),
        Some(String::from("t1dMjwmwM2a6NtavQ6SiPP8i9ofx4cgfYYP"))
    );
    assert_eq!(response.p2sh(), None);
}
