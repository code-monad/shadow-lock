use crate::Loader;
use ckb_testtool::builtin::ALWAYS_SUCCESS;
use ckb_testtool::ckb_types::{bytes::Bytes, core::TransactionBuilder, packed::*, prelude::*};
use ckb_testtool::context::Context;

// Include your tests here
// See https://github.com/xxuejie/ckb-native-build-sample/blob/main/tests/src/tests.rs for more examples

fn build_lock_args(mode: u8, script_hash: [u8; 32]) -> Bytes {
    Bytes::copy_from_slice(
        &[mode]
            .iter()
            .chain(script_hash.iter())
            .cloned()
            .collect::<Vec<u8>>(),
    )
}

// generated unit test for contract shadow-lock
#[test]
fn test_shadow_lock() {
    // deploy contract
    let mut context = Context::default();
    let shadow_lock_bin: Bytes = Loader::default().load_binary("shadow-lock");
    let always_success: Bytes = ALWAYS_SUCCESS.clone();

    let always_success_out_point = context.deploy_cell(always_success);

    let test_original_lock_args = Bytes::copy_from_slice(&[1u8; 32]);

    let test_original_lock_script = context
        .build_script(&always_success_out_point, test_original_lock_args)
        .expect("failed to build script");

    let original_lock_script_hash = test_original_lock_script.calc_script_hash().unpack();

    println!(
        "test original lock script hash is {:?}",
        original_lock_script_hash.0
    );

    let out_point = context.deploy_cell(shadow_lock_bin);

    // disable all features
    let mode = 0b00000000;

    // composed shadow lock args
    let lock_args = build_lock_args(mode, original_lock_script_hash.0);

    let lock_script = context.build_script(&out_point, lock_args).expect("script");

    // prepare cells
    // first cell, normal lock cell as key unlocker
    let input_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(1000u64.pack())
            .lock(test_original_lock_script.clone())
            .build(),
        Bytes::new(),
    );
    let input = CellInput::new_builder()
        .previous_output(input_out_point)
        .build();

    // second cell, shadow lock cell
    let input_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(500u64.pack())
            .lock(lock_script.clone())
            .build(),
        Bytes::new(),
    );

    let input2 = CellInput::new_builder()
        .previous_output(input_out_point)
        .build();

    let outputs = vec![
        CellOutput::new_builder()
            .capacity(1000u64.pack())
            .lock(test_original_lock_script.clone())
            .build(),
        CellOutput::new_builder()
            .capacity(500u64.pack())
            .lock(test_original_lock_script.clone())
            .build(),
    ];

    let outputs_data = vec![Bytes::new(); 2];

    // build transaction
    // success one
    let tx = TransactionBuilder::default()
        .inputs([input, input2])
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .build();
    let tx = context.complete_tx(tx);

    // run
    let cycles = context
        .verify_tx(&tx, 10_000_000)
        .expect("pass verification");
    println!("consume cycles: {}", cycles);
}

#[test]
fn test_shadow_lock_failure() {
    // deploy contract
    let mut context = Context::default();
    let shadow_lock_bin: Bytes = Loader::default().load_binary("shadow-lock");
    let always_success: Bytes = ALWAYS_SUCCESS.clone();

    let always_success_out_point = context.deploy_cell(always_success);

    let test_original_lock_args = Bytes::copy_from_slice(&[1u8; 32]);

    let test_original_lock_script = context
        .build_script(&always_success_out_point, test_original_lock_args)
        .expect("failed to build script");

    let original_lock_script_hash = test_original_lock_script.calc_script_hash().unpack();

    println!(
        "test original lock script hash is {:?}",
        original_lock_script_hash.0
    );

    let out_point = context.deploy_cell(shadow_lock_bin);

    let mode = 0b00000110;

    // composed shadow lock args
    let lock_args = build_lock_args(mode, original_lock_script_hash.0);

    let lock_script = context.build_script(&out_point, lock_args).expect("script");

    // prepare cells
    // first cell, normal lock cell as key unlocker
    let input_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(1000u64.pack())
            .lock(test_original_lock_script.clone())
            .build(),
        Bytes::new(),
    );
    let _input = CellInput::new_builder()
        .previous_output(input_out_point)
        .build();

    // second cell, shadow lock cell
    let input_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(500u64.pack())
            .lock(lock_script.clone())
            .build(),
        Bytes::new(),
    );

    let input2 = CellInput::new_builder()
        .previous_output(input_out_point)
        .build();

    let outputs = vec![
        CellOutput::new_builder()
            .capacity(1000u64.pack())
            .lock(test_original_lock_script.clone())
            .build(),
        CellOutput::new_builder()
            .capacity(500u64.pack())
            .lock(test_original_lock_script.clone())
            .build(),
    ];

    let outputs_data = vec![Bytes::new(); 2];

    // build transaction
    // failure one
    let tx = TransactionBuilder::default()
        .inputs([input2])
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .build();
    let tx = context.complete_tx(tx);

    // run
    let cycles = context
        .verify_tx(&tx, 10_000_000)
        .expect_err("pass verification");
    println!("consume cycles: {}", cycles);
}

#[test]
fn test_shadow_lock_self_destruction_verify() {
    // deploy contract
    let mut context = Context::default();
    let shadow_lock_bin: Bytes = Loader::default().load_binary("shadow-lock");
    let always_success: Bytes = ALWAYS_SUCCESS.clone();

    let always_success_out_point = context.deploy_cell(always_success);

    let test_original_lock_args = Bytes::copy_from_slice(&[1u8; 32]);

    let test_original_lock_script = context
        .build_script(&always_success_out_point, test_original_lock_args)
        .expect("failed to build script");

    let original_lock_script_hash = test_original_lock_script.calc_script_hash().unpack();

    println!(
        "test original lock script hash is {:?}",
        original_lock_script_hash.0
    );

    let out_point = context.deploy_cell(shadow_lock_bin);

    // delegate script hash = lock.hash
    // forbid trade = true
    // self destruction = true
    let mode = 0b00000110;

    // composed shadow lock args
    let lock_args = build_lock_args(mode, original_lock_script_hash.0);

    let lock_script = context
        .build_script(&out_point, lock_args.clone())
        .expect("lock script build");

    let type_script = context
        .build_script(&out_point, lock_args)
        .expect("type script build");

    let type_script = ScriptOpt::new_builder()
        .set(Some(type_script.clone()))
        .build();

    // prepare cells
    // first cell, normal lock cell as key unlocker
    let input_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(1000u64.pack())
            .lock(test_original_lock_script.clone())
            .build(),
        Bytes::new(),
    );
    let input = CellInput::new_builder()
        .previous_output(input_out_point)
        .build();

    // second cell, shadow lock cell
    let input_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(500u64.pack())
            .type_(type_script.clone())
            .lock(lock_script.clone())
            .build(),
        Bytes::new(),
    );

    let input2 = CellInput::new_builder()
        .previous_output(input_out_point)
        .build();

    let outputs = vec![
        CellOutput::new_builder()
            .capacity(1000u64.pack())
            .type_(type_script.clone())
            .lock(test_original_lock_script.clone())
            .build(),
        CellOutput::new_builder()
            .capacity(500u64.pack())
            .lock(lock_script.clone())
            .build(),
    ];

    let outputs_data = vec![Bytes::new(); 2];

    // build transaction
    // success one
    let tx = TransactionBuilder::default()
        .inputs([input, input2])
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .build();
    let tx = context.complete_tx(tx);

    // run
    let cycles = context
        .verify_tx(&tx, 10_000_000)
        .expect_err("pass verification");
    println!("consume cycles: {}", cycles);
}

#[test]
fn test_shadow_lock_self_destruction_pass_verify() {
    // deploy contract
    let mut context = Context::default();
    let shadow_lock_bin: Bytes = Loader::default().load_binary("shadow-lock");
    let always_success: Bytes = ALWAYS_SUCCESS.clone();

    let always_success_out_point = context.deploy_cell(always_success);

    let test_original_lock_args = Bytes::copy_from_slice(&[1u8; 32]);

    let test_original_lock_script = context
        .build_script(&always_success_out_point, test_original_lock_args)
        .expect("failed to build script");

    let original_lock_script_hash = test_original_lock_script.calc_script_hash().unpack();

    println!(
        "test original lock script hash is {:?}",
        original_lock_script_hash.0
    );

    let out_point = context.deploy_cell(shadow_lock_bin);

    // delegate script hash = lock.hash
    // forbid trade = true
    // self destruction = true
    let mode = 0b00000110;

    // composed shadow lock args
    let lock_args = build_lock_args(mode, original_lock_script_hash.0);

    let lock_script = context
        .build_script(&out_point, lock_args.clone())
        .expect("lock script build");

    let type_script = context
        .build_script(&out_point, lock_args)
        .expect("type script build");

    let type_script = ScriptOpt::new_builder()
        .set(Some(type_script.clone()))
        .build();

    // prepare cells
    // first cell, normal lock cell as key unlocker
    let input_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(1000u64.pack())
            .lock(test_original_lock_script.clone())
            .build(),
        Bytes::new(),
    );
    let input = CellInput::new_builder()
        .previous_output(input_out_point)
        .build();

    // second cell, shadow lock cell
    let input_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(500u64.pack())
            .type_(type_script.clone())
            .lock(lock_script.clone())
            .build(),
        Bytes::new(),
    );

    let input2 = CellInput::new_builder()
        .previous_output(input_out_point)
        .build();

    let outputs = vec![
        CellOutput::new_builder()
            .capacity(1000u64.pack())
            .lock(test_original_lock_script.clone())
            .build(),
        CellOutput::new_builder()
            .capacity(500u64.pack())
            .lock(lock_script.clone())
            .build(),
    ];

    let outputs_data = vec![Bytes::new(); 2];

    // build transaction
    // success one
    let tx = TransactionBuilder::default()
        .inputs([input, input2])
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .build();
    let tx = context.complete_tx(tx);

    // run
    let cycles = context
        .verify_tx(&tx, 10_000_000)
        .expect("pass verification");
    println!("consume cycles: {}", cycles);
}

#[test]
fn test_shadow_lock_forbid_trade_verify() {
    // deploy contract
    let mut context = Context::default();
    let shadow_lock_bin: Bytes = Loader::default().load_binary("shadow-lock");
    let always_success: Bytes = ALWAYS_SUCCESS.clone();

    let always_success_out_point = context.deploy_cell(always_success);

    let test_original_lock_args = Bytes::copy_from_slice(&[1u8; 32]);

    let test_original_lock_script = context
        .build_script(&always_success_out_point, test_original_lock_args)
        .expect("failed to build script");

    let test_trade_lock_args = Bytes::copy_from_slice(&[2u8; 32]);
    let test_trade_lock_script = context
        .build_script(&always_success_out_point, test_trade_lock_args)
        .expect("failed to build script");

    let original_lock_script_hash = test_original_lock_script.calc_script_hash().unpack();

    println!(
        "test original lock script hash is {:?}",
        original_lock_script_hash.0
    );

    let out_point = context.deploy_cell(shadow_lock_bin);

    // delegate script hash = lock.hash
    // forbid trade = true
    // self destruction = false
    let mode = 0b00000010;

    // composed shadow lock args
    let lock_args = build_lock_args(mode, original_lock_script_hash.0);

    let lock_script = context
        .build_script(&out_point, lock_args.clone())
        .expect("lock script build");

    let type_script = context
        .build_script(&out_point, lock_args)
        .expect("type script build");

    let type_script = ScriptOpt::new_builder()
        .set(Some(type_script.clone()))
        .build();

    // prepare cells
    // first cell, normal lock cell as key unlocker
    let input_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(1000u64.pack())
            .lock(test_original_lock_script.clone())
            .build(),
        Bytes::new(),
    );
    let input = CellInput::new_builder()
        .previous_output(input_out_point)
        .build();

    // second cell, shadow lock cell
    let input_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(500u64.pack())
            .type_(type_script.clone())
            .lock(lock_script.clone())
            .build(),
        Bytes::new(),
    );

    let input2 = CellInput::new_builder()
        .previous_output(input_out_point)
        .build();

    let outputs = vec![
        CellOutput::new_builder()
            .capacity(1000u64.pack())
            .lock(test_original_lock_script.clone())
            .build(),
        CellOutput::new_builder()
            .capacity(500u64.pack())
            .type_(type_script.clone())
            .lock(test_trade_lock_script.clone())
            .build(),
    ];

    let outputs_data = vec![Bytes::new(); 2];

    // build transaction
    // success one
    let tx = TransactionBuilder::default()
        .inputs([input, input2])
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .build();
    let tx = context.complete_tx(tx);

    // run
    let cycles = context
        .verify_tx(&tx, 10_000_000)
        .expect_err("pass verification");
    println!("consume cycles: {}", cycles);
}

#[test]
fn test_shadow_lock_forbid_trade_pass_verify() {
    // deploy contract
    let mut context = Context::default();
    let shadow_lock_bin: Bytes = Loader::default().load_binary("shadow-lock");
    let always_success: Bytes = ALWAYS_SUCCESS.clone();

    let always_success_out_point = context.deploy_cell(always_success);

    let test_original_lock_args = Bytes::copy_from_slice(&[1u8; 32]);

    let test_original_lock_script = context
        .build_script(&always_success_out_point, test_original_lock_args)
        .expect("failed to build script");

    let original_lock_script_hash = test_original_lock_script.calc_script_hash().unpack();

    println!(
        "test original lock script hash is {:?}",
        original_lock_script_hash.0
    );

    let out_point = context.deploy_cell(shadow_lock_bin);

    // delegate script hash = lock.hash
    // forbid trade = true
    // self destruction = false
    let mode = 0b00000010;

    // composed shadow lock args
    let lock_args = build_lock_args(mode, original_lock_script_hash.0);

    let lock_script = context
        .build_script(&out_point, lock_args.clone())
        .expect("lock script build");

    let type_script = context
        .build_script(&out_point, lock_args)
        .expect("type script build");

    let type_script = ScriptOpt::new_builder()
        .set(Some(type_script.clone()))
        .build();

    // prepare cells
    // first cell, normal lock cell as key unlocker
    let input_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(1000u64.pack())
            .lock(test_original_lock_script.clone())
            .build(),
        Bytes::new(),
    );
    let input = CellInput::new_builder()
        .previous_output(input_out_point)
        .build();

    // second cell, shadow lock cell
    let input_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(500u64.pack())
            .type_(type_script.clone())
            .lock(lock_script.clone())
            .build(),
        Bytes::new(),
    );

    let input2 = CellInput::new_builder()
        .previous_output(input_out_point)
        .build();

    let outputs = vec![
        CellOutput::new_builder()
            .capacity(1000u64.pack())
            .lock(test_original_lock_script.clone())
            .build(),
        CellOutput::new_builder()
            .capacity(500u64.pack())
            .type_(type_script.clone())
            .lock(test_original_lock_script.clone())
            .build(),
    ];

    let outputs_data = vec![Bytes::new(); 2];

    // build transaction
    // success one
    let tx = TransactionBuilder::default()
        .inputs([input, input2])
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .build();
    let tx = context.complete_tx(tx);

    // run
    let cycles = context
        .verify_tx(&tx, 10_000_000)
        .expect("pass verification");
    println!("consume cycles: {}", cycles);
}
