use anyhow::{Context, Result};
use ecosystem::MyError;
use std::{fs, io};

fn main() -> Result<(), anyhow::Error> {
    println!("size of MyError is {}", size_of::<MyError>());
    println!("size of io Error is {}", size_of::<io::Error>());
    println!(
        "size of ParseIntError is {}",
        size_of::<std::num::ParseIntError>()
    );
    println!("size of BigError is {}", size_of::<ecosystem::BigError>());
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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
