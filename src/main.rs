#[cfg(target_os = "windows")]
compile_error!("Sorry but this program and it's author don't want their code to be compiled in garbage OS like Windogs. Please delete your OS and install linux instead. Tq.\n- hllstr");

#[unsafe(no_mangle)]
pub static malloc_conf: [u8; 73] = *b"background_thread:true,dirty_decay_ms:1000,muzzy_decay_ms:1000,narenas:1\0";

#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

#[macro_use]
mod macros;
mod client;
mod commands;
mod config;
mod handler;
mod state;
mod utils;

use chrono::Local;
use log::info;
use std::sync::Arc;

use crate::config::WarmupMode;


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if cfg!(windows) {
        panic!("Please delete your garbage OS and install Linux instead to run this program.");
    }
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format(|buf, record| {
            use std::io::Write;
            writeln!(
                buf,
                "{} [{}] - {}",
                Local::now().format("%H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .init();
    let config = Arc::new(config::AppConfig::load()?);
    let state = state::AppState::load(config.clone());
    let mut bot = client::create_bot(config.clone(), state.clone()).await?;
    info!("Starting Bot...");

    let client = bot.client().clone();
    let bot_handle = bot.run().await?;

    let state_worker = state.clone();
    let client_worker = bot.client().clone();

    tokio::spawn(async move {
        loop {
            let current_warmup = state_worker.get_warmup();
            let current_interval = state_worker.get_warmup_interval();

            if current_warmup == WarmupMode::High {
                let targets: Vec<_> = state_worker
                    .last_messages
                    .iter()
                    .map(|entry| {
                        (
                            entry.key().clone(),
                            entry.value().0.clone(),
                            entry.value().1.clone(),
                        )
                    })
                    .collect();

                if !targets.is_empty() {
                    info!(
                        "Running periodic high warmup for {} chats (Interval: {}s)...",
                        targets.len(),
                        current_interval
                    );

                    for (chat_jid, msg_id, participant) in targets {
                        let client_clone = client_worker.clone();
                        tokio::spawn(async move {
                            let _ = crate::utils::send_warmup(
                                client_clone,
                                chat_jid,
                                msg_id,
                                participant,
                            )
                            .await;
                        });
                    }
                }

                tokio::time::sleep(std::time::Duration::from_secs(current_interval)).await;
            } else {
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
        }
    });

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
