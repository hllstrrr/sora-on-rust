use dashmap::DashMap;
use reqwest::header::{ACCEPT, ACCEPT_LANGUAGE, HeaderMap, HeaderValue, USER_AGENT};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use std::time::Instant;

use crate::config::AppConfig;

pub enum ConfigKey {
    Mode,
    Prefixes,
}

pub enum ConfigValue {
    Text(String),
    List(Vec<String>),
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct ChatSettings {
    pub expiration: u32,
}

pub struct AppState {
    pub http_client: reqwest::Client,
    pub spotify_search_client: tokio::sync::RwLock<spotapi::SpotifySearch>,
    pub settings: DashMap<String, ChatSettings>,
    pub db: sled::Db,
    pub start_time: Instant,
    pub config: Arc<AppConfig>,
    pub mode: RwLock<String>,
    pub prefixes: RwLock<Vec<String>>,
}

impl AppState {
    pub fn load(config: Arc<AppConfig>) -> Arc<Self> {
        let start_time = Instant::now();
        let db = sled::Config::new()
            .path("database/chat")
            .cache_capacity(10 * 1024 * 1024)
            .mode(sled::Mode::HighThroughput)
            .open()
            .expect("Errpr opening sled database");
        let settings = DashMap::new();
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"));
        headers.insert(ACCEPT, HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,video/mp4,*/*;q=0.8"));
        headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("en-US,en;q=0.5"));
        let http_client = reqwest::Client::builder()
            .default_headers(headers)
            .tcp_keepalive(std::time::Duration::from_secs(60))
            .pool_idle_timeout(std::time::Duration::from_secs(90))
            .pool_max_idle_per_host(10)
            .build()
            .unwrap();
        // let http_client = reqwest::Client::new();
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
            spotify_search_client: tokio::sync::RwLock::new(spotapi::SpotifySearch::new()),
            settings,
            db,
            start_time,
            mode: RwLock::new(config.mode.clone()),
            prefixes: RwLock::new(config.prefixes.clone()),
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

    pub fn get_mode(&self) -> String {
        self.mode.read().unwrap().clone()
    }

    pub fn get_prefixes(&self) -> Vec<String> {
        self.prefixes.read().unwrap().clone()
    }

    pub fn set_config(&self, key: ConfigKey, value: ConfigValue) -> Result<(), &'static str> {
        match (key, value) {
            (ConfigKey::Mode, ConfigValue::Text(val)) => {
                let mut mode = self.mode.write().unwrap();
                *mode = val;
                Ok(())
            }
            (ConfigKey::Prefixes, ConfigValue::List(val)) => {
                let mut prefixes = self.prefixes.write().unwrap();
                *prefixes = val;
                Ok(())
            }
            _ => Err("invalid datatype for this field"),
        }
    }
}
