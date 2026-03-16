use log::{error, info};
use std::sync::Arc;
use wacore::types::events::Event;
use wacore::proto_helpers::MessageExt;
use whatsapp_rust::client::Client;
use crate::commands::cmd::COMMANDS;

pub async fn event_handler(event: Event, client: Arc<Client>) {
    match event {
        Event::PairingCode { code, .. } => {
            info!("Pair with this code: {}", code);
        }
        Event::Connected(_) => {
            info!("✅ Bot connected successfully!");
        }
        Event::Message(msg, info) => {
            if let Some(text) = msg.text_content() {
                let prefix = "{";
                if !text.starts_with(prefix) { return; }

                let body = &text[prefix.len()..];
                let args: Vec<&str> = body.split_whitespace().collect();
                if args.is_empty() { return; }

                let cmd_name = args[0].to_lowercase();

   
                for cmd in COMMANDS {
                    if cmd.name() == cmd_name || cmd.aliases().contains(&cmd_name.as_str()) {
                        if let Err(e) = cmd.execute(&client, &info).await {
                            error!("Error executing {}: {}", cmd_name, e);
                        }
                        return;
                    }
                }
            }
        }
        _ => {}
    }
}