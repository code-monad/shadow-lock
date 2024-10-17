use ckb_std::error::SysError;

/// Error
#[repr(i8)]
pub enum ShadowLockError {
    IndexOutOfBound = -1,
    ItemMissing = -2,
    LengthNotEnough = -3,
    Encoding = -4,
    Unknown = -100,

    OwnershipVerificationFailure = -110,

    ForbidTradeVerificationFailure = -111,
    SelfDestructionVerificationFailure = -112,
}

impl From<SysError> for ShadowLockError {
    fn from(err: SysError) -> Self {
        use SysError::*;
        match err {
            IndexOutOfBound => Self::IndexOutOfBound,
            ItemMissing => Self::ItemMissing,
            LengthNotEnough(_) => Self::LengthNotEnough,
            Encoding => Self::Encoding,
            _ => Self::Unknown,
        }
    }
}
