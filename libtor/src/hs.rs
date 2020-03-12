/// Hidden service version
#[derive(Debug, Clone, Copy)]
pub enum HiddenServiceVersion {
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
pub enum HiddenServiceAuthType {
    Basic,
    Stealth,
}
