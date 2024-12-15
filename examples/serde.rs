use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct User {
    name: String,
    email: Option<String>,
    age: u32,
    dob: chrono::DateTime<chrono::Utc>,
    skills: Vec<String>,
}

fn main() -> Result<()> {
    let user = User {
        name: "Alice".to_string(),
        email: Some("abc@xyz.com".to_string()),
        age: 22,
        dob: chrono::Utc::now(),
        skills: vec!["Rust".to_string(), "Python".to_string()],
    };

    let json = serde_json::to_string(&user)?;
    println!("{}", json);

    let user1 = serde_json::from_str::<User>(&json)?;
    assert_eq!(user, user1);

    Ok(())
}
