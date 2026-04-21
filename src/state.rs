use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use std::time::Instant;

use crate::config::{AppConfig, BotMode, WarmupMode};

pub enum ConfigKey {
    Mode,
    Prefixes,
    Warmup,
    WarmupInterval,
}

pub enum ConfigValue {
    Text(String),
    List(Vec<String>),
    Number(u64),
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct ChatSettings {
    pub expiration: u32,
}

pub struct AppState {
    pub http_client: reqwest::Client,
    pub settings: DashMap<String, ChatSettings>,
    pub last_messages: DashMap<whatsapp_rust::Jid, (String, Option<String>)>,
    pub db: sled::Db,
    pub start_time: Instant,
    pub config: Arc<AppConfig>,
    pub mode: RwLock<BotMode>,
    pub prefixes: RwLock<Arc<Vec<String>>>,
    pub warmup: RwLock<WarmupMode>,
    pub warmup_interval: RwLock<u64>,
}

impl AppState {
    pub fn load(config: Arc<AppConfig>) -> Arc<Self> {
        let start_time = Instant::now();
        let db = sled::Config::new()
            .path("database/chat")
            .cache_capacity(2 * 1024 * 1024)
            .open()
            .expect("Error opening sled database");
        let settings = DashMap::new();
        let last_messages = DashMap::new();
        let http_client = reqwest::Client::new();
        
        // hydration from db to cache
        for (key, value) in db.iter().flatten() {
            let jid = String::from_utf8_lossy(&key).to_string();

            if value.len() == 4 {
                let bytes: [u8; 4] = value.as_ref().try_into().unwrap();
                let expiration = u32::from_be_bytes(bytes);
                settings.insert(jid, ChatSettings { expiration });
            }
        }

        Arc::new(Self {
            http_client,
            settings,
            last_messages,
            db,
            start_time,
            prefixes: RwLock::new(Arc::new(config.prefixes.clone())),
            mode: RwLock::new(config.mode),
            warmup: RwLock::new(config.warmup),
            warmup_interval: RwLock::new(config.warmup_interval),
            config,
        })
    }

    pub fn set_expiration(self: Arc<Self>, jid: String, expiration: u32) {
        if let Some(current) = self.settings.get(&jid)
            && current.expiration == expiration
        {
            return;
        }
        let jid_db = jid.clone();
        self.settings.insert(jid, ChatSettings { expiration });
        let state_clone = Arc::clone(&self);
        tokio::task::spawn_blocking(move || {
            let val_bytes = expiration.to_be_bytes();
            if let Err(e) = state_clone.db.insert(jid_db, &val_bytes) {
                log::error!("Error inserting data into sled database: {}", e);
            }
        });
    }
    pub fn get_expiration(&self, jid: &str) -> u32 {
        self.settings.get(jid).map(|s| s.expiration).unwrap_or(0)
    }

    pub fn get_mode(&self) -> BotMode {
        *self.mode.read().unwrap()
    }

    pub fn get_prefixes(&self) -> Arc<Vec<String>> {
        self.prefixes.read().unwrap().clone()
    }

    pub fn get_warmup(&self) -> WarmupMode {
        *self.warmup.read().unwrap()
    }

    pub fn get_warmup_interval(&self) -> u64 {
        *self.warmup_interval.read().unwrap()
    }

    pub fn set_config(&self, key: ConfigKey, value: ConfigValue) -> Result<(), &'static str> {
        match (key, value) {
            (ConfigKey::Mode, ConfigValue::Text(val)) => {
                let mut mode = self.mode.write().unwrap();
                *mode = if val.to_lowercase() == "self" {
                    BotMode::SelfMode
                } else {
                    BotMode::Public
                };
                Ok(())
            }
            (ConfigKey::Prefixes, ConfigValue::List(val)) => {
                let mut prefixes = self.prefixes.write().unwrap();
                *prefixes = val.into();
                Ok(())
            }
            (ConfigKey::Warmup, ConfigValue::Text(val)) => {
                let mut warmup = self.warmup.write().unwrap();
                *warmup = WarmupMode::from(val.as_str());
                Ok(())
            }
            (ConfigKey::WarmupInterval, ConfigValue::Number(val)) => {
                let mut interval = self.warmup_interval.write().unwrap();
                *interval = val;
                Ok(())
            }
            _ => Err("invalid datatype for this field"),
        }
    }
}
