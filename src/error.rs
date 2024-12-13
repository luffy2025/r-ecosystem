use std::fmt::Display;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(#[from] std::num::ParseIntError),

    #[error("Serde error: {0}")]
    Serialize(#[from] serde_json::Error),

    #[error("Big error: {0}")]
    BigError(Box<BigError>),

    #[error("Custom error: {0}")]
    Custom(String),
}

#[derive(Debug)]
pub struct BigError {
    a: String,
    b: Vec<String>,
    c: [u8; 64],
    d: u64,
}

impl Display for BigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "BigError {{ a: {}, b: {:?}, c: {:?}, d: {} }}",
            self.a, self.b, self.c, self.d
        )
    }
}
