#[macro_use]
extern crate libtor_derive;
extern crate tor_sys;

pub trait Expand: std::fmt::Debug {
    fn expand(&self) -> String;
}

#[derive(Debug, Expand)]
pub enum TorFlag {
    #[expand_to("-f {}")]
    ConfigFile(String),
    #[expand_to("--passphrase-fd {}")]
    PassphraseFD(u32),
}

#[derive(Debug, Expand)]
pub enum TorSubcommand {
    #[expand_to("--hash-password {password}")]
    HashPassword{password: String},
    #[expand_to("--verify-config")]
    VerifyConfig,
    #[expand_to("--list-fingerprint")]
    ListFingerprint,
    #[expand_to("--version")]
    Version,
    #[expand_to("--keygen")]
    Keygen{
        #[expand_to(ignore)]
        password: Option<String>
    },
    #[expand_to("--keygen --newpass")]
    KeygenNewpass{
        #[expand_to(ignore)]
        old_password: Option<String>,
        #[expand_to(ignore)]
        new_password: Option<String>
    },
}

pub struct Tor {
    subcommand: Option<TorSubcommand>,
    flags: Vec<TorFlag>,
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn it_works() {
        let config_file = TorFlag::ConfigFile("file".to_string());
        println!("{}", config_file.expand());

        let a = TorSubcommand::KeygenNewpass{old_password: None, new_password: None};
        println!("{}", a.expand());
    }
}
