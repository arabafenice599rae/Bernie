//! Codici d'errore di §7. Il valore numerico è il codice custom del programma.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum BernieError {
    ZeroAmount = 1,
    ExceedsSupply = 2,
    Dust = 3,
    ZeroPayout = 4,
    Slippage = 5,
    NoHolders = 6,
    // 7 riservato: era `NotEmpty`, non più usato dalla v1.6.
    PenaltyOutOfRange = 8,
    PriceOutOfRange = 9,
    Overflow = 10,
    InvariantViolated = 11,
    NothingToClaim = 12,
    TransferToSelf = 13,
    SupplyMismatch = 14,
    MissingDelegation = 15,
    MetadataTooLong = 16,
    /// Il token account del redeem non appartiene al firmatario (v1.6).
    NotOwner = 17,
}

impl BernieError {
    pub const fn code(self) -> u32 {
        self as u32
    }

    /// Errore dal codice custom (7 è riservato).
    pub const fn from_code(code: u32) -> Option<Self> {
        Some(match code {
            1 => Self::ZeroAmount,
            2 => Self::ExceedsSupply,
            3 => Self::Dust,
            4 => Self::ZeroPayout,
            5 => Self::Slippage,
            6 => Self::NoHolders,
            8 => Self::PenaltyOutOfRange,
            9 => Self::PriceOutOfRange,
            10 => Self::Overflow,
            11 => Self::InvariantViolated,
            12 => Self::NothingToClaim,
            13 => Self::TransferToSelf,
            14 => Self::SupplyMismatch,
            15 => Self::MissingDelegation,
            16 => Self::MetadataTooLong,
            17 => Self::NotOwner,
            _ => return None,
        })
    }

    /// Nome usato nel modello di riferimento e nei vettori JSON.
    pub const fn name(self) -> &'static str {
        match self {
            Self::ZeroAmount => "ZeroAmount",
            Self::ExceedsSupply => "ExceedsSupply",
            Self::Dust => "Dust",
            Self::ZeroPayout => "ZeroPayout",
            Self::Slippage => "Slippage",
            Self::NoHolders => "NoHolders",
            Self::PenaltyOutOfRange => "PenaltyOutOfRange",
            Self::PriceOutOfRange => "PriceOutOfRange",
            Self::Overflow => "Overflow",
            Self::InvariantViolated => "InvariantViolated",
            Self::NothingToClaim => "NothingToClaim",
            Self::TransferToSelf => "TransferToSelf",
            Self::SupplyMismatch => "SupplyMismatch",
            Self::MissingDelegation => "MissingDelegation",
            Self::MetadataTooLong => "MetadataTooLong",
            Self::NotOwner => "NotOwner",
        }
    }
}
