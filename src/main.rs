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
    let mut bot = client::create_bot(config.clone(), state.clone()).await?;
    info!("Starting Bot...");
    
    let client = bot.client().clone();
    let bot_handle = bot.run().await?;
    if config.warmup == "high" {
        let state_worker = state;
        let client_worker = bot.client().clone();
        
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(300)).await;
                info!("Running periodic high warmup for {} chats...", state_worker.last_messages.len());
                for entry in state_worker.last_messages.iter() {
                    let chat_jid_str = entry.key();
                    let (msg_id, participant) = entry.value();
                    let msg_id = msg_id.clone();
                    let participant = participant.clone();
                
                    if let Ok(chat_jid) = chat_jid_str.parse::<whatsapp_rust::Jid>() {
                        let client_clone = client_worker.clone();
                        tokio::spawn(async move {
                            let _ = crate::utils::send_warmup(
                                client_clone,
                                chat_jid,
                                msg_id,
                                participant
                            ).await;
                        });
                    }
                }
            }
        });
    }

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