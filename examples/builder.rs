use anyhow::Result;
use chrono::{DateTime, Utc};
use derive_builder::Builder;

#[allow(unused)]
#[derive(Debug, Builder)]
struct User {
    #[builder(setter(into), default)]
    name: String,

    #[builder(setter(into, strip_option), default)]
    email: Option<String>,

    #[builder(default = "22")]
    age: u32,

    #[builder(setter(custom))]
    dob: DateTime<Utc>,

    #[builder(default = "vec![]", setter(each = "into"))]
    skills: Vec<String>,
}

impl User {
    fn build() -> UserBuilder {
        UserBuilder::default()
    }
}

impl UserBuilder {
    pub fn dob(&mut self, value: &str) -> &mut Self {
        self.dob = DateTime::parse_from_rfc3339(value)
            .map(|dt| dt.with_timezone(&Utc))
            .ok();
        self
    }
}

fn main() -> Result<()> {
    // let user0 = User::build().build()?;
    // println!("default: {:?}", user0);

    let user1 = User::build()
        .name("Alice")
        .age(30)
        .email("abc@xyz.com")
        .dob("2021-01-01T00:00:00Z")
        .skills(vec!["Rust".to_string()])
        .build()?;
    println!("custom: {:?}", user1);

    Ok(())
}
