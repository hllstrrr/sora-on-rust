#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;
#[macro_use]
mod macros;
mod commands;
mod handler;
mod client;
mod config;
mod state;
mod utils;

use chrono::Local;
use log::info;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format(|buf, record| {
            use std::io::Write;
            writeln!(buf, "{} [{}] - {}", Local::now().format("%H:%M:%S"), record.level(), record.args())
        })
        .init();
    let config = Arc::new(config::AppConfig::load()?);
    let state = state::AppState::load(config.clone());
    let mut bot = client::create_bot(config, state).await?;
    info!("Starting Bot...");
    
    let client = bot.client().clone();
    let bot_handle = bot.run().await?;
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("SIGINT received, Performing graceful shutdown...");
            client.disconnect().await;
        }
        res = bot_handle => {
            res?;
        }
    }
    Ok(())
}