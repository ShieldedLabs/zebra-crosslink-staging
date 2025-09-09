//! Test-format-based tests

#![cfg(test)]

use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use zebra_chain::block::{Block, Hash as BlockHash, Header as BlockHeader};
use zebra_chain::parameters::testnet::ConfiguredActivationHeights;
use zebra_chain::parameters::Network;
use zebra_chain::serialization::*;
use zebra_crosslink::chain::*;
use zebra_crosslink::test_format::*;
use zebra_crosslink::*;
use zebra_state::crosslink::*;
use zebrad::application::CROSSLINK_TEST_CONFIG_OVERRIDE;
use zebrad::config::ZebradConfig;
use zebrad::prelude::Application;

macro_rules! function_name {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        name.strip_suffix("::f")
            .unwrap()
            .split("::")
            .last()
            .unwrap()
    }};
}

/// Set the global state TEST_NAME
pub fn set_test_name(name: &'static str) {
    *zebra_crosslink::TEST_NAME.lock().unwrap() = name;
}

/// Crosslink Test entrypoint
pub fn test_start() {
    // init globals
    {
        *CROSSLINK_TEST_CONFIG_OVERRIDE.lock().unwrap() = {
            let mut base = ZebradConfig::default();
            base.network.network = Network::new_regtest(
                zebra_chain::parameters::testnet::RegtestParameters::default(),
            );
            base.state.ephemeral = true;

            Some(std::sync::Arc::new(base))
        };
        *zebra_crosslink::TEST_MODE.lock().unwrap() = true;
        *zebra_crosslink::TEST_SHUTDOWN_FN.lock().unwrap() = || {
            // APPLICATION.shutdown(abscissa_core::Shutdown::Graceful);
            std::process::exit(*zebra_crosslink::TEST_FAILED.lock().unwrap());
        }
    }

    use zebrad::application::{ZebradApp, APPLICATION};

    // boot
    let os_args: Vec<_> = std::env::args_os().collect();
    // panic!("OS args: {:?}", os_args);
    let args: Vec<std::ffi::OsString> = vec![
        os_args[0].clone(),
        zebrad::commands::EntryPoint::default_cmd_as_str().into(),
    ];
    // println!("args: {:?}", args);

    #[cfg(feature = "viz_gui")]
    {
        // TODO: gate behind feature-flag
        // TODO: only open the visualization window for the `start` command.
        // i.e.: can we move it to that code without major refactor to make compiler happy?
        let tokio_root_thread_handle = std::thread::spawn(move || {
            ZebradApp::run(&APPLICATION, args);
        });

        zebra_crosslink::viz::main(Some(tokio_root_thread_handle));
    }

    #[cfg(not(feature = "viz_gui"))]
    ZebradApp::run(&APPLICATION, args);
}

/// Run a Crosslink Test from a dynamic byte array.
pub fn test_bytes(bytes: Vec<u8>) {
    *zebra_crosslink::TEST_INSTR_BYTES.lock().unwrap() = bytes;
    test_start();
}

/// Run a Crosslink Test from a file path.
pub fn test_path(path: PathBuf) {
    *zebra_crosslink::TEST_INSTR_PATH.lock().unwrap() = Some(path);
    test_start();
}

#[ignore]
#[test]
fn read_from_file() {
    test_path("../crosslink-test-data/blocks.zeccltf".into());
}

const REGTEST_BLOCK_BYTES: &[&[u8]] = &[
    //                                                                     i   h
    include_bytes!("../../crosslink-test-data/test_pow_block_0.bin"), //   0,  1: 02a610...
    include_bytes!("../../crosslink-test-data/test_pow_block_1.bin"), //   1,  2: 02d711...
    include_bytes!("../../crosslink-test-data/test_pow_block_2.bin"), //   2,  fork 3: 098207...
    include_bytes!("../../crosslink-test-data/test_pow_block_3.bin"), //   3,  3: 0032241...
    include_bytes!("../../crosslink-test-data/test_pow_block_4.bin"), //   4,  4: 0247f1a...
    include_bytes!("../../crosslink-test-data/test_pow_block_6.bin"), //   5,  5: 0ab3a5d8...
    include_bytes!("../../crosslink-test-data/test_pow_block_7.bin"), //   6,  6
    include_bytes!("../../crosslink-test-data/test_pow_block_9.bin"), //   7,  7
    include_bytes!("../../crosslink-test-data/test_pow_block_10.bin"), //  8,  8
    include_bytes!("../../crosslink-test-data/test_pow_block_12.bin"), //  9,  9
    include_bytes!("../../crosslink-test-data/test_pow_block_13.bin"), // 10, 10
    include_bytes!("../../crosslink-test-data/test_pow_block_14.bin"), // 11, 11
    include_bytes!("../../crosslink-test-data/test_pow_block_15.bin"), // 12, 12
    include_bytes!("../../crosslink-test-data/test_pow_block_17.bin"), // 13, 13
    include_bytes!("../../crosslink-test-data/test_pow_block_18.bin"), // 14, 14
    include_bytes!("../../crosslink-test-data/test_pow_block_19.bin"), // 15, 15
    include_bytes!("../../crosslink-test-data/test_pow_block_21.bin"), // 16, 16
    include_bytes!("../../crosslink-test-data/test_pow_block_22.bin"), // 17, 17
    include_bytes!("../../crosslink-test-data/test_pow_block_23.bin"), // 18, 18
    include_bytes!("../../crosslink-test-data/test_pow_block_25.bin"), // 19, 19
    include_bytes!("../../crosslink-test-data/test_pow_block_26.bin"), // 20, 20
    include_bytes!("../../crosslink-test-data/test_pow_block_28.bin"), // 21, 21
    include_bytes!("../../crosslink-test-data/test_pow_block_29.bin"), // 22, 22
];
const REGTEST_BLOCK_BYTES_N: usize = REGTEST_BLOCK_BYTES.len();

fn regtest_block_hashes() -> [BlockHash; REGTEST_BLOCK_BYTES_N] {
    let mut hashes = [BlockHash([0; 32]); REGTEST_BLOCK_BYTES_N];
    for i in 0..REGTEST_BLOCK_BYTES_N {
        // ALT: BlockHeader::
        hashes[i] = Block::zcash_deserialize(REGTEST_BLOCK_BYTES[i])
            .unwrap()
            .hash();
    }
    hashes
}

const REGTEST_POS_BLOCK_BYTES: &[&[u8]] = &[
    include_bytes!("../../crosslink-test-data/test_pos_block_5.bin"),
    include_bytes!("../../crosslink-test-data/test_pos_block_8.bin"),
    include_bytes!("../../crosslink-test-data/test_pos_block_11.bin"),
    include_bytes!("../../crosslink-test-data/test_pos_block_16.bin"),
    include_bytes!("../../crosslink-test-data/test_pos_block_20.bin"),
    include_bytes!("../../crosslink-test-data/test_pos_block_24.bin"),
    include_bytes!("../../crosslink-test-data/test_pos_block_27.bin"),
];

const REGTEST_POW_IDX_FINALIZED_BY_POS_BLOCK: &[usize] = &[1, 4, 6, 10, 13, 16, 18];

#[test]
fn crosslink_expect_pos_height_on_boot() {
    set_test_name(function_name!());
    let mut tf = TF::new(&PROTOTYPE_PARAMETERS);

    tf.push_instr_expect_pos_chain_length(0, 0);

    test_bytes(tf.write_to_bytes());
}

#[test]
fn crosslink_expect_pow_height_on_boot() {
    set_test_name(function_name!());
    let mut tf = TF::new(&PROTOTYPE_PARAMETERS);

    tf.push_instr_expect_pow_chain_length(1, 0);

    test_bytes(tf.write_to_bytes());
}

#[test]
fn crosslink_expect_first_pow_to_not_be_a_no_op() {
    set_test_name(function_name!());
    let mut tf = TF::new(&PROTOTYPE_PARAMETERS);

    tf.push_instr_load_pow_bytes(REGTEST_BLOCK_BYTES[0], 0);
    tf.push_instr_expect_pow_chain_length(2, 0);

    test_bytes(tf.write_to_bytes());
}

#[test]
fn crosslink_push_example_pow_chain_only() {
    set_test_name(function_name!());
    let mut tf = TF::new(&PROTOTYPE_PARAMETERS);

    for i in 0..REGTEST_BLOCK_BYTES.len() {
        tf.push_instr_load_pow_bytes(REGTEST_BLOCK_BYTES[i], 0);
        tf.push_instr_expect_pow_chain_length(2 + i - (i >= 3) as usize, 0);
    }
    tf.push_instr_expect_pow_chain_length(1 - 1 + REGTEST_BLOCK_BYTES.len(), 0);

    test_bytes(tf.write_to_bytes());
}

#[test]
fn crosslink_push_example_pow_chain_each_block_twice() {
    set_test_name(function_name!());
    let mut tf = TF::new(&PROTOTYPE_PARAMETERS);

    for i in 0..REGTEST_BLOCK_BYTES.len() {
        tf.push_instr_load_pow_bytes(REGTEST_BLOCK_BYTES[i], 0);
        tf.push_instr_load_pow_bytes(REGTEST_BLOCK_BYTES[i], SHOULD_FAIL);
        tf.push_instr_expect_pow_chain_length(2 + i - (i >= 3) as usize, 0);
    }
    tf.push_instr_expect_pow_chain_length(1 - 1 + REGTEST_BLOCK_BYTES.len(), 0);

    test_bytes(tf.write_to_bytes());
}

#[test]
fn crosslink_push_example_pow_chain_again_should_not_change_the_pow_chain_length() {
    set_test_name(function_name!());
    let mut tf = TF::new(&PROTOTYPE_PARAMETERS);

    for i in 0..REGTEST_BLOCK_BYTES.len() {
        tf.push_instr_load_pow_bytes(REGTEST_BLOCK_BYTES[i], 0);
        tf.push_instr_expect_pow_chain_length(2 + i - (i >= 3) as usize, 0);
    }
    tf.push_instr_expect_pow_chain_length(1 - 1 + REGTEST_BLOCK_BYTES.len(), 0);

    for i in 0..REGTEST_BLOCK_BYTES.len() {
        tf.push_instr_load_pow_bytes(REGTEST_BLOCK_BYTES[i], SHOULD_FAIL);
        tf.push_instr_expect_pow_chain_length(1 - 1 + REGTEST_BLOCK_BYTES.len(), 0);
    }

    test_bytes(tf.write_to_bytes());
}

#[test]
fn crosslink_expect_pos_not_pushed_if_pow_blocks_not_present() {
    set_test_name(function_name!());
    let mut tf = TF::new(&PROTOTYPE_PARAMETERS);

    tf.push_instr_load_pos_bytes(REGTEST_POS_BLOCK_BYTES[0], SHOULD_FAIL);
    tf.push_instr_expect_pos_chain_length(0, 0);

    test_bytes(tf.write_to_bytes());
}

#[test]
fn crosslink_expect_pos_height_after_push() {
    set_test_name(function_name!());
    let mut tf = TF::new(&PROTOTYPE_PARAMETERS);

    for i in 0..REGTEST_BLOCK_BYTES.len() {
        tf.push_instr_load_pow_bytes(REGTEST_BLOCK_BYTES[i], 0);
    }
    for i in 0..REGTEST_POS_BLOCK_BYTES.len() {
        tf.push_instr_load_pos_bytes(REGTEST_POS_BLOCK_BYTES[i], 0);
        tf.push_instr_expect_pos_chain_length(1 + i, 0);
    }

    // let write_ok = tf.write_to_file(Path::new("crosslink_expect_pos_height_after_push.zeccltf"));
    // assert!(write_ok);
    test_bytes(tf.write_to_bytes());
}

#[test]
fn crosslink_expect_pos_out_of_order() {
    set_test_name(function_name!());
    let mut tf = TF::new(&PROTOTYPE_PARAMETERS);

    for i in 0..5 {
        tf.push_instr_load_pow_bytes(REGTEST_BLOCK_BYTES[i], 0);
    }

    tf.push_instr_load_pos_bytes(REGTEST_POS_BLOCK_BYTES[0], 0);
    tf.push_instr_load_pos_bytes(REGTEST_POS_BLOCK_BYTES[2], SHOULD_FAIL);
    tf.push_instr_load_pos_bytes(REGTEST_POS_BLOCK_BYTES[1], 0);
    tf.push_instr_expect_pos_chain_length(2, 0);

    test_bytes(tf.write_to_bytes());
}

#[test]
fn crosslink_expect_pos_push_same_block_twice_only_accepted_once() {
    set_test_name(function_name!());
    let mut tf = TF::new(&PROTOTYPE_PARAMETERS);

    for i in 0..REGTEST_BLOCK_BYTES.len() {
        tf.push_instr_load_pow_bytes(REGTEST_BLOCK_BYTES[i], 0);
    }
    tf.push_instr_load_pos_bytes(REGTEST_POS_BLOCK_BYTES[0], 0);
    tf.push_instr_load_pos_bytes(REGTEST_POS_BLOCK_BYTES[0], SHOULD_FAIL);
    tf.push_instr_expect_pos_chain_length(1, 0);

    test_bytes(tf.write_to_bytes());
}

#[test]
fn crosslink_reject_pos_with_signature_on_different_data() {
    set_test_name(function_name!());
    let mut tf = TF::new(&PROTOTYPE_PARAMETERS);

    for i in 0..REGTEST_BLOCK_BYTES.len() {
        tf.push_instr_load_pow_bytes(REGTEST_BLOCK_BYTES[i], 0);
    }

    // modify block data so that the signatures are incorrect
    // NOTE: modifying the last as there is no `previous_block_hash` that this invalidates
    let mut bft_block_and_fat_ptr =
        BftBlockAndFatPointerToIt::zcash_deserialize(REGTEST_POS_BLOCK_BYTES[0]).unwrap();
    // let mut last_pow_hdr =
    bft_block_and_fat_ptr.block.headers.last_mut().unwrap().time += Duration::from_secs(1);
    let new_bytes = bft_block_and_fat_ptr.zcash_serialize_to_vec().unwrap();

    assert!(
        &new_bytes != REGTEST_POS_BLOCK_BYTES[0],
        "test invalidated if the serialization has not been changed"
    );

    tf.push_instr_load_pos_bytes(&new_bytes, SHOULD_FAIL);
    tf.push_instr_expect_pos_chain_length(0, 0);

    test_bytes(tf.write_to_bytes());
}

#[test]
fn crosslink_test_basic_finality() {
    set_test_name(function_name!());
    let mut tf = TF::new(&PROTOTYPE_PARAMETERS);

    let hashes = regtest_block_hashes();

    for i2 in 0..REGTEST_BLOCK_BYTES.len() {
        tf.push_instr_expect_pow_block_finality(&hashes[i2], None, 0);
    }

    // let hashes = Vec::with_capacity(REGTEST_BLOCK_BYTES.len());
    for i in 0..REGTEST_BLOCK_BYTES.len() {
        // hashes.push()
        tf.push_instr_load_pow_bytes(REGTEST_BLOCK_BYTES[i], 0);

        for i2 in 0..REGTEST_BLOCK_BYTES.len() {
            let finality = if i2 > i {
                None
            } else if i2 == 3 && i == 3 {
                Some(TFLBlockFinality::NotYetFinalized)
            } else if i2 == 2 && i > 3 {
                Some(TFLBlockFinality::NotYetFinalized)
            } else {
                Some(TFLBlockFinality::NotYetFinalized)
            };
            tf.push_instr_expect_pow_block_finality(&hashes[i2], finality, 0);
        }
    }

    for i in 0..REGTEST_POS_BLOCK_BYTES.len() {
        tf.push_instr_load_pos_bytes(REGTEST_POS_BLOCK_BYTES[i], 0);

        for i2 in 0..REGTEST_BLOCK_BYTES.len() {
            let finality = if i2 == 2 {
                // unpicked sidechain
                if i < 1 {
                    Some(TFLBlockFinality::NotYetFinalized)
                } else {
                    // Some(TFLBlockFinality::CantBeFinalized)
                    None
                }
            } else if i2 <= REGTEST_POW_IDX_FINALIZED_BY_POS_BLOCK[i] {
                Some(TFLBlockFinality::Finalized)
            } else {
                Some(TFLBlockFinality::NotYetFinalized)
            };
            tf.push_instr_expect_pow_block_finality(&hashes[i2], finality, 0);
        }
    }

    test_bytes(tf.write_to_bytes());
}

#[ignore]
#[test]
fn reject_pos_block_with_lt_sigma_headers() {
    set_test_name(function_name!());
    let mut tf = TF::new(&PROTOTYPE_PARAMETERS);

    for i in 0..4 {
        tf.push_instr_load_pow_bytes(REGTEST_BLOCK_BYTES[i], 0);
    }

    let mut bft_block_and_fat_ptr =
        BftBlockAndFatPointerToIt::zcash_deserialize(REGTEST_POS_BLOCK_BYTES[0]).unwrap();
    bft_block_and_fat_ptr
        .block
        .headers
        .truncate(bft_block_and_fat_ptr.block.headers.len() - 1);
    let new_bytes = bft_block_and_fat_ptr.zcash_serialize_to_vec().unwrap();
    assert!(
        &new_bytes != REGTEST_POS_BLOCK_BYTES[0],
        "test invalidated if the serialization has not been changed"
    );

    tf.push_instr_load_pos_bytes(&new_bytes, 0);
    tf.push_instr_expect_pos_chain_length(0, 0);
}

#[test]
fn crosslink_test_pow_to_pos_link() {
    set_test_name(function_name!());
    let mut tf = TF::new(&PROTOTYPE_PARAMETERS);

    tf.push_instr_load_pow_bytes(REGTEST_BLOCK_BYTES[0], 0);
    tf.push_instr_load_pow_bytes(REGTEST_BLOCK_BYTES[1], 0);
    tf.push_instr_load_pow_bytes(REGTEST_BLOCK_BYTES[2], 0);
    tf.push_instr_load_pow_bytes(REGTEST_BLOCK_BYTES[3], 0);
    tf.push_instr_load_pow_bytes(REGTEST_BLOCK_BYTES[4], 0);

    // TODO: maybe have push_instr return data?
    tf.push_instr_load_pos_bytes(REGTEST_POS_BLOCK_BYTES[0], 0);
    let pos_0 = BftBlockAndFatPointerToIt::zcash_deserialize(REGTEST_POS_BLOCK_BYTES[0]).unwrap();
    let pos_0_fat_ptr = zebra_chain::block::FatPointerToBftBlock {
        vote_for_block_without_finalizer_public_key: pos_0
            .fat_ptr
            .vote_for_block_without_finalizer_public_key,
        signatures: pos_0
            .fat_ptr
            .signatures
            .iter()
            .map(|sig| zebra_chain::block::FatPointerSignature {
                public_key: sig.public_key,
                vote_signature: sig.vote_signature,
            })
            .collect(),
    };

    let mut pow_5 = Block::zcash_deserialize(REGTEST_BLOCK_BYTES[5]).unwrap();
    pow_5.header = Arc::new(BlockHeader {
        version: 5,
        fat_pointer_to_bft_block: pos_0_fat_ptr.clone(),
        ..*pow_5.header
    });
    // let pow_5_hash = pow_5.hash();
    tf.push_instr_load_pow(&pow_5, 0);

    // let mut pow_6 = Block::zcash_deserialize(REGTEST_BLOCK_BYTES[6]).unwrap();
    // pow_6.header = Arc::new(BlockHeader {
    //     version: 5,
    //     fat_pointer_to_bft_block: pos_0_fat_ptr.clone(),
    //     previous_block_hash: pow_5_hash,
    //     ..*pow_6.header
    // });
    // // let pow5_hash = pow_5.hash();
    // tf.push_instr_load_pow(&pow_6, 0);

    // tf.write_to_file(&Path::new(&format!("{}.zeccltf", function_name!())));
    test_bytes(tf.write_to_bytes());
}

#[test]
fn crosslink_reject_pow_chain_fork_that_is_competing_against_a_shorter_finalized_pow_chain() {
    set_test_name(function_name!());
    test_path(PathBuf::from(
        "../crosslink-test-data/wrong_branch_test1_short_pos_long.zeccltf",
    ));
}

#[test]
fn crosslink_pow_switch_to_finalized_chain_fork_even_though_longer_chain_exists() {
    set_test_name(function_name!());
    test_path(PathBuf::from(
        "../crosslink-test-data/wrong_branch_test2_long_short_pos.zeccltf",
    ));
}

// TODO:
// - reject signatures from outside the roster
// - reject pos block with < 2/3rds roster stake
// - reject pos block with signatures from the previous, but not current roster
// > require correctly-signed incorrect data:
//   - reject pos block with > sigma headers
//   - reject pos block with < sigma headers
//   - reject pos block where headers don't form subchain (hdrs[i].hash() != hdrs[i+1].previous_block_hash)
//   - repeat all signature tests but for the *next* pos block's fat pointer back
// - reject pos block that does have the correct fat pointer *hash* to prev block
// ...
