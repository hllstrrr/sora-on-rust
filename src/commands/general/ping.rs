use crate::{cmd, commands::cmd::Context};
use std::time::Instant;
use tokio::net::TcpStream;
use waproto::whatsapp as wa;

cmd!(
    Ping,
    name: "ping",
    aliases: ["p"],
    category: "general",
    execute: |ctx| {
        ping(ctx).await?;
    }
);

// rustfmt works here
async fn ping(ctx: Context<'_>) -> anyhow::Result<()> {
    let server_wangsaf = "g.whatsapp.net:443";
    let start = Instant::now();
    match TcpStream::connect(server_wangsaf).await {
        Ok(_) => {
            let latency = start.elapsed();
            let ping = ctx.reply(&format!("```Pong!\n----------------------\nNetwork Latency: {}ms\nResponse   Time: Measuring...```", latency.as_millis())).await?;
            let rtt = start.elapsed();
            let final_text = wa::Message {
                    conversation: Some(format!("```Pong!\n----------------------\nNetwork Latency: {}ms\nResponse   Time: {}ms```", latency.as_millis(), rtt.as_millis()).to_string()),
                    ..Default::default()
                };

            ctx.client
                .edit_message(ctx.info.source.chat.clone(), ping, final_text)
                .await?;
        }
        Err(e) => {
            println!("Error connecting to {}: {}", server_wangsaf, e);
        }
    }
    Ok(())
}
