use anyhow::{Context, Result};
use std::fmt::Display;
use std::{fs, io};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

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

fn main() -> Result<(), anyhow::Error> {
    println!("size of MyError is {}", size_of::<MyError>());
    println!("size of io Error is {}", size_of::<io::Error>());
    println!(
        "size of ParseIntError is {}",
        size_of::<std::num::ParseIntError>()
    );
    println!("size of BigError is {}", size_of::<BigError>());
    println!("size of serde Error is {}", size_of::<serde_json::Error>());
    println!("size of Custom Error is {}", size_of::<String>());

    // io error
    let filename = "non-existent-file.txt";
    let _fd =
        fs::File::open(filename).with_context(|| format!("Can not find file: {}", filename))?;

    // custom error
    fail_with_error()?;

    Ok(())
}

fn fail_with_error() -> Result<(), MyError> {
    Err(MyError::Custom("An error occurred".to_string()))
}
