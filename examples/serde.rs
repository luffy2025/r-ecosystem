use anyhow::Result;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use chacha20poly1305::aead::{Aead, OsRng};
use chacha20poly1305::{AeadCore, ChaCha20Poly1305, KeyInit};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use std::fmt::Display;
use std::str::FromStr;

#[serde_as]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct User {
    name: String,
    email: Option<String>,
    age: u32,
    #[serde(rename = "dateOfBirth")]
    dob: chrono::DateTime<chrono::Utc>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    learned_skills: Vec<String>,
    #[serde(serialize_with = "b64_encode", deserialize_with = "b64_decode")]
    data: Vec<u8>,
    #[serde(skip)]
    private_content: String,
    #[serde_as(as = "Vec<DisplayFromStr>")]
    url: Vec<http::Uri>,
    #[serde_as(as = "DisplayFromStr")]
    sensitive: SensitiveData,
}

#[derive(Debug, PartialEq)]
struct SensitiveData(String);

fn main() -> Result<()> {
    let user = User {
        name: "Alice".to_string(),
        email: Some("abc@xyz.com".to_string()),
        age: 22,
        dob: chrono::Utc::now(),
        // learned_skills: vec!["Rust".to_string(), "Python".to_string()],
        learned_skills: vec![],
        data: "hello world!".as_bytes().to_vec(),
        private_content: "pillow talk".to_string(),
        url: vec![
            "https://www.rust-lang.org".parse()?,
            "https://www.python.org".parse()?,
        ],
        sensitive: SensitiveData::new("secret"),
    };
    println!("user: {:?}", user);

    let json = serde_json::to_string(&user)?;
    println!("json: {}", json);

    let mut user1 = serde_json::from_str::<User>(&json)?;
    user1.private_content = user.private_content.clone();
    println!("user1: {:?}", user1);

    assert_eq!(user, user1);

    Ok(())
}

fn b64_encode<S>(data: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let encoded = URL_SAFE_NO_PAD.encode(data);
    serializer.serialize_str(&encoded)
}

fn b64_decode<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let encoded = String::deserialize(deserializer)?;
    let decoded = URL_SAFE_NO_PAD
        .decode(encoded.as_bytes())
        .map_err(serde::de::Error::custom)?;
    Ok(decoded)
}

impl Display for SensitiveData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = encrypt(self.0.as_bytes()).expect("encryption failed");
        write!(f, "{}", str)
    }
}

impl FromStr for SensitiveData {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let str = decrypt(s)?;
        Ok(Self(str))
    }
}

impl SensitiveData {
    fn new(s: &str) -> Self {
        Self(s.to_string())
    }
}

const KEY: &[u8] = b"0123456789ABCDEF0123456789ABCDEF";

fn encrypt(data: &[u8]) -> Result<String> {
    let cipher = ChaCha20Poly1305::new(KEY.into());
    let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, data.as_ref())
        .map_err(anyhow::Error::msg)?;

    let nonce_crypt_text: Vec<_> = nonce.iter().copied().chain(ciphertext).collect();
    let encoded = URL_SAFE_NO_PAD.encode(&nonce_crypt_text);

    Ok(encoded)
}

fn decrypt(str: &str) -> Result<String> {
    let decoded = URL_SAFE_NO_PAD.decode(str.as_bytes())?;
    let cipher = ChaCha20Poly1305::new(KEY.into());
    let nonce = decoded[..12].into();
    let decrypted = cipher
        .decrypt(nonce, &decoded[12..])
        .map_err(anyhow::Error::msg)?;
    let text = String::from_utf8(decrypted)?.to_string();

    Ok(text)
}
