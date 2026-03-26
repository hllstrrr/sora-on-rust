use log::{error, info};
use std::sync::Arc;
use crate::state::AppState;
use wacore::{client::context::SendContextResolver, types::events::Event};
use whatsapp_rust::client::Client;
use crate::commands::cmd::COMMANDS;
use crate::config::AppConfig;
use crate::utils::MessageExt;
use tokio::sync::RwLock;
use std::sync::LazyLock;
static SUPERUSER_LID: LazyLock<RwLock<Option<String>>> = LazyLock::new(|| RwLock::new(None));
pub async fn event_handler(event: Event, client: Arc<Client>, config: Arc<AppConfig>, state: Arc<AppState>) {
    match event {
        Event::Connected(_) => {
            info!("✅ Bot connected successfully!");
            if let Some(su_pn) = &config.superuser {
                if let Some(jid) = client.get_lid_for_phone(su_pn).await {
                    let lid_user = jid.to_string();
                    
                    let mut lock = SUPERUSER_LID.write().await;
                    *lock = Some(lid_user.clone());
                }
            }
            
        }
        Event::Message(msg, info) => {
            // println!("{:#?}", msg);
            let start = std::time::Instant::now();
            if let Some(exp) = msg.get_expiration_timer() {
                state.set_expiration(&info.source.chat.to_string(), exp);
            }
            if let Some(text) = msg.text_content() {
                let matched_prefix = config.prefixes.iter().find(|p| text.starts_with(*p));
             let prefix = match matched_prefix {
                Some(p) => p,
                None => return,
            };
            if config.mode == "self" {
                let sender = &info.source.sender.user;
    
                let me = info.source.is_from_me;
                let su = if info.source.sender.is_lid() {
                    if let Ok(lock) = SUPERUSER_LID.try_read() {
                        lock.as_deref() == Some(sender.as_str())
                    } else {
                        false
                    }
                } else {
                    config.superuser.as_ref() == Some(&sender)
                };
                let privileged = me || su;
                if !privileged {
                    return;
                }
            }
            let msg_arc = Arc::from(msg);
            let info_arc = Arc::new(info);
                let body = &text[prefix.len()..];
                let args: Vec<&str> = body.split_whitespace().collect();
                if args.is_empty() { return; }
                let cmd_name = args[0].to_lowercase();
                for cmd in COMMANDS {
                    if cmd.name() == cmd_name || cmd.aliases().contains(&cmd_name.as_str()) {
                        let ctx = crate::commands::cmd::Context {
                            client: Arc::clone(&client),
                            msg: Arc::clone(&msg_arc),  
                            info: Arc::clone(&info_arc),
                            state: Arc::clone(&state),
                        };
                        println!("Internal: {:?}", start.elapsed());
                        if let Err(e) = cmd.execute(ctx).await {
                            error!("Error executing {}: {}", cmd_name, e);
                        }                        
                    }
                }
            }
            
            let duration = start.elapsed();
            println!("Executed in {:?}", duration);
        }
        _ => {}
    }
}