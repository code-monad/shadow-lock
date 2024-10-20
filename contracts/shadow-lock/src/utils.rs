use crate::errors::ShadowLockError;
use alloc::{vec, vec::Vec};
use ckb_std::{
    ckb_constants::Source,
    debug,
    high_level::{load_cell_data_hash, load_cell_lock_hash, load_cell_type_hash, QueryIter},
};

#[derive(Debug)]
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

#[derive(Debug)]
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
) -> Result<Vec<usize>, ShadowLockError> {
    debug!("input_index: {input_index}, source: {:?}", source);
    let input_type_hash = load_cell_type_hash(input_index, source)?;

    let data_position = if check_data {
        let data_hash = load_cell_data_hash(input_index, source)?;
        QueryIter::new(load_cell_data_hash, Source::Output)
            .enumerate()
            .filter(|(_, x)| x == &data_hash)
            .map(|(position, _)| position)
            .collect::<Vec<usize>>()
    } else {
        vec![]
    };

    let lock_position = if check_lock {
        let lock_hash = load_cell_lock_hash(input_index, source)?;
        QueryIter::new(load_cell_lock_hash, Source::Output)
            .enumerate()
            .filter(|(_, x)| x == &lock_hash)
            .map(|(position, _)| position)
            .collect::<Vec<usize>>()
    } else {
        vec![]
    };

    let found_same_cell = QueryIter::new(load_cell_type_hash, Source::Output)
        .enumerate()
        .filter(|(_, x)| x == &input_type_hash)
        .filter(|(tp, _)| {
            let data_matches = !check_data || data_position.contains(&tp);
            let lock_matches = !check_lock || lock_position.contains(&tp);
            debug!("index: {tp}, data_matches: {data_matches}, lock_matches: {lock_matches}");
            data_matches && lock_matches
        })
        .map(|(same_index, _)| same_index)
        .collect::<Vec<usize>>();

    // Now check if all positions (type, lock, data) are Some and are equal

    // Return None if any of the checks failed or if positions are not equal
    Ok(found_same_cell)
}

pub fn delegate_data_owner_check(
    delegate_data_hash: Option<[u8; 32]>,
    index: usize,
    source: Source,
) -> Result<bool, ShadowLockError> {
    if delegate_data_hash.is_some() {
        let cell_data_hash = load_cell_data_hash(index, source)?;
        Ok(cell_data_hash == delegate_data_hash.unwrap())
    } else {
        Ok(true)
    }
}
