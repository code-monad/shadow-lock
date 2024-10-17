use ckb_std::{
    ckb_constants::Source,
    high_level::{
        find_cell_by_data_hash, load_cell_data_hash, load_cell_lock_hash, load_cell_type_hash,
        QueryIter,
    },
};

use crate::errors::ShadowLockError;

pub struct FeatureFlags {
    pub delegate_script_type: bool,
    pub forbid_trade: bool,
    pub self_destruction: bool,
    pub restrict_delegate_data: bool,
}

#[derive(Eq, PartialEq)]
pub enum LoadHashTarget {
    Type,
    Lock,
    #[allow(dead_code)]
    Data, // Maybe?
}

impl FeatureFlags {
    pub fn unpack(flag_bits: u8) -> FeatureFlags {
        FeatureFlags {
            delegate_script_type: (flag_bits & 0b00000001) != 0,
            forbid_trade: (flag_bits & 0b00000010) != 0,
            self_destruction: (flag_bits & 0b00000100) != 0,
            restrict_delegate_data: (flag_bits & 0b00001000) != 0,
        }
    }

    pub fn get_delegate_target(&self) -> LoadHashTarget {
        if self.delegate_script_type {
            LoadHashTarget::Type
        } else {
            LoadHashTarget::Lock
        }
    }
}

pub struct UnpackedShadowlockArgs {
    pub flags: FeatureFlags,
    pub ref_hash: [u8; 32],
    pub data_hash: Option<[u8; 32]>,
}

pub fn unpack_script_args(args: &[u8]) -> Result<UnpackedShadowlockArgs, ShadowLockError> {
    if args.len() < 33 {
        return Err(ShadowLockError::LengthNotEnough);
    }
    let flags = FeatureFlags::unpack(args[0]);
    let ref_hash: [u8; 32] = args[1..33].try_into().unwrap();
    let data_hash: Option<[u8; 32]> = if flags.restrict_delegate_data {
        if args.len() < 65 {
            return Err(ShadowLockError::LengthNotEnough);
        }
        Some(args[33..65].try_into().unwrap())
    } else {
        None
    };

    Ok(UnpackedShadowlockArgs {
        flags,
        ref_hash,
        data_hash,
    })
}

pub fn check_input_output_contain_same_cell(
    input_index: usize,
    source: Source,
    check_data: bool,
    check_lock: bool,
) -> Result<Option<usize>, ShadowLockError> {
    let input_type_hash = load_cell_type_hash(input_index, source)?;

    let data_position = if check_data {
        let data_hash = load_cell_data_hash(input_index, source)?;
        find_cell_by_data_hash(&data_hash, Source::Output)?
    } else {
        None
    };

    let lock_position = if check_lock {
        let lock_hash = load_cell_lock_hash(input_index, source)?;
        QueryIter::new(load_cell_lock_hash, Source::Output).position(|x| x == lock_hash)
    } else {
        None
    };

    let type_position =
        QueryIter::new(load_cell_type_hash, Source::Output).position(|x| x == input_type_hash);

    // Now check if all positions (type, lock, data) are Some and are equal
    if let (Some(tp), Some(dp), Some(lp)) = (type_position, data_position, lock_position) {
        if tp == dp && dp == lp {
            return Ok(Some(tp)); // Return the position if all are equal
        }
    }

    // Return None if any of the checks failed or if positions are not equal
    Ok(None)
}

pub fn delegate_data_owner_check(
    delegate_data_hash: Option<[u8; 32]>,
    index: usize,
    source: Source,
) -> Result<bool, ShadowLockError> {
    Ok(delegate_data_hash.is_some_and(|delegate_data_hash| {
        load_cell_data_hash(index, source)
            .is_ok_and(|cell_data_hash| cell_data_hash == delegate_data_hash)
    }))
}
