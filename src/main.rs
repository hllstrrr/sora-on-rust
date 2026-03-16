mod commands;
mod handler;
mod client;

use chrono::Local;
use log::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format(|buf, record| {
            use std::io::Write;
            writeln!(buf, "{} [{}] - {}", Local::now().format("%H:%M:%S"), record.level(), record.args())
        })
        .init();

    let mut bot = client::create_bot().await?;
    info!("Starting Bot...");
    bot.run().await?.await?;
    Ok(())
}