use alloc::{borrow::ToOwned as _, vec::Vec};
use ckb_std::{
    ckb_constants::Source,
    ckb_types::prelude::Unpack,
    debug,
    high_level::{
        load_cell_lock_hash, load_cell_type_hash, load_script, load_script_hash, QueryIter,
    },
};

use crate::{
    errors::ShadowLockError,
    utils::{
        check_input_output_contain_same_cell, delegate_data_owner_check, unpack_script_args,
        LoadHashTarget,
    },
};

pub fn main() -> Result<(), ShadowLockError> {
    let script_hash = load_script_hash()?;

    let shadow_in_input = QueryIter::new(load_cell_lock_hash, Source::GroupInput)
        .enumerate()
        .filter(|(_, lock_hash)| lock_hash[..] == script_hash[..])
        .map(|(index, _)| index)
        .collect::<Vec<usize>>();

    let script = load_script()?;
    let args: Vec<u8> = script.args().unpack();
    let unpacked_args = unpack_script_args(&args)?;

    debug!("unpacked args: {:?}", unpacked_args);

    // if forbid trade, then this
    if unpacked_args.flags.forbid_trade {
        debug!("now do forbid trade verify");
        for input_index in shadow_in_input.to_owned() {
            let output_pos =
                check_input_output_contain_same_cell(input_index, Source::GroupInput, true, false)?;

            // valid target lock hash is only: current lock hash, and delegate lock hash
            for index in output_pos {
                let output_lock_hash = load_cell_lock_hash(index, Source::Output)?;
                let delegate_target = unpacked_args.flags.get_delegate_target();
                if delegate_target == LoadHashTarget::Lock
                    && (output_lock_hash != script_hash
                        && output_lock_hash != unpacked_args.ref_hash)
                {
                    return Err(ShadowLockError::ForbidTradeVerificationFailure);
                }
            }
        }
    }

    // if self destruction is set, then this cell must be destroyed after unlock
    if unpacked_args.flags.self_destruction {
        debug!("now do self destruction check");
        for input_index in shadow_in_input {
            if !check_input_output_contain_same_cell(input_index, Source::GroupInput, true, false)?
                .is_empty()
            {
                return Err(ShadowLockError::SelfDestructionVerificationFailure);
            }
        }
    }

    // now let's do ownership verification
    let ownership_verification = match unpacked_args.flags.get_delegate_target() {
        LoadHashTarget::Type => {
            // type hash check
            QueryIter::new(load_cell_type_hash, Source::Input)
                .enumerate()
                .any(|(index, type_hash)| {
                    type_hash.unwrap_or_default() == unpacked_args.ref_hash
		    // data hash check if needed
                        && delegate_data_owner_check(unpacked_args.data_hash, index, Source::Input)
                            .is_ok_and(|check_result| check_result)
                })
        }
        LoadHashTarget::Lock => {
            // lock hash check
            QueryIter::new(load_cell_lock_hash, Source::Input)
                .enumerate()
                .any(|(index, lock_hash)| {
                    lock_hash == unpacked_args.ref_hash &&
		    // data hash check if needed
			delegate_data_owner_check(unpacked_args.data_hash, index, Source::Input).map_or(false, |x| x )
                })
        }
        _ => unreachable!(),
    };

    if !ownership_verification {
        return Err(ShadowLockError::OwnershipVerificationFailure);
    }

    Ok(())
}
