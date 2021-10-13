#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

macro_rules! display_like_debug {
    ($type:ty) => {
        impl std::fmt::Display for $type {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{:?}", self)
            }
        }
    };
}

pub trait Joiner: std::fmt::Debug + std::clone::Clone {
    fn joiner(&self) -> String;
    fn new() -> Self;
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CommaJoiner {}
impl Joiner for CommaJoiner {
    fn joiner(&self) -> String {
        ",".to_string()
    }

    fn new() -> Self {
        CommaJoiner {}
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SpaceJoiner {}
impl Joiner for SpaceJoiner {
    fn joiner(&self) -> String {
        " ".to_string()
    }

    fn new() -> Self {
        SpaceJoiner {}
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DisplayVec<T: std::fmt::Debug + std::fmt::Display, J: Joiner> {
    vec: Vec<T>,
    joiner: J,
}

impl<T: std::fmt::Debug + std::fmt::Display, J: Joiner> std::fmt::Display for DisplayVec<T, J> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let joined: String = self
            .vec
            .iter()
            .map(|v| format!("{}", v))
            .collect::<Vec<String>>()
            .join(&self.joiner.joiner());
        write!(f, "{}", joined)
    }
}

impl<T: std::fmt::Debug + std::fmt::Display, J: Joiner> From<Vec<T>> for DisplayVec<T, J> {
    fn from(vec: Vec<T>) -> DisplayVec<T, J> {
        DisplayVec {
            vec,
            joiner: J::new(),
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DisplayOption<T: std::fmt::Debug + std::fmt::Display> {
    option: Option<T>,
}

impl<T: std::fmt::Debug + std::fmt::Display> std::fmt::Display for DisplayOption<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.option {
            Some(val) => write!(f, "{}", val),
            None => Ok(()),
        }
    }
}

impl<T: std::fmt::Debug + std::fmt::Display> From<Option<T>> for DisplayOption<T> {
    fn from(option: Option<T>) -> DisplayOption<T> {
        DisplayOption { option }
    }
}
