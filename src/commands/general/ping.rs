use crate::cmd;
use waproto::whatsapp as wa;

cmd!(
    Ping,
    name: "ping",
    aliases: ["p"],
    category: "general",
    execute: |client, _msg, info| {
        let pong = wa::Message {
            conversation: Some("Pong!".to_string()),
            ..Default::default()
        };
        client.send_message(info.source.chat.clone(), pong).await?;
    }
);