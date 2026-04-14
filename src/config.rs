use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Deserialize, Serialize, Clone)]
pub struct AppConfig {
    pub prefixes: Vec<String>,
    pub session_path: String,
    pub custom_code: String,
    pub mode: String,
    pub warmup: String,
    #[serde(skip)]
    pub phone_number: String,
    #[serde(skip_serializing)]
    pub superuser: Option<String>,
}

impl AppConfig {
    pub fn load() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();
        let phone = std::env::var("PHONE_NUMBER").expect("PHONE_NUMBER must be set in .env");
        let su = std::env::var("SUPERUSER").ok();
        let toml_str = fs::read_to_string("Config.toml")?;
        let mut config: AppConfig = toml::from_str(&toml_str)?;
        config.superuser = su;
        config.phone_number = phone;
        Ok(config)
    }
}