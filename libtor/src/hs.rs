#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Hidden service version
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum HiddenServiceVersion {
    #[deprecated(note = "Please migrate to V3 hidden services")]
    V2 = 2,
    V3 = 3,
}

impl std::fmt::Display for HiddenServiceVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", *self as u8)
    }
}

/// Hidden service authorization type for authorized clients
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum HiddenServiceAuthType {
    Basic,
    Stealth,
}
