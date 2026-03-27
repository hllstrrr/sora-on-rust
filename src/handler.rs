use log::{error, info};
use wacore::stanza::GroupNotificationAction;
use wacore::types::events::GroupUpdate;
use wacore::types::message::MessageInfo;
use std::sync::Arc;
use crate::state::AppState;
use wacore::{client::context::SendContextResolver, types::events::Event};
use whatsapp_rust::client::Client;
use crate::config::AppConfig;
use crate::utils::MessageExt;
use tokio::sync::RwLock;
use std::sync::LazyLock;

static SUPERUSER_LID: LazyLock<RwLock<Option<String>>> = LazyLock::new(|| RwLock::new(None));

pub async fn event_handler(event: Event, client: Arc<Client>, config: Arc<AppConfig>, state: Arc<AppState>) {
    match event {
        Event::Connected(_) => handle_connected(config, client).await,
        Event::Message(msg, info) => handle_message(msg, client, config, info, state).await,
        Event::GroupUpdate(update) => handle_group_exp(update, state).await,
        _ => {}
    }
}


async fn handle_connected(config: Arc<AppConfig>, client: Arc<Client>) {
    info!("✅ Bot connected successfully!");
    if let Some(su_pn) = &config.superuser {
        if let Some(jid) = client.get_lid_for_phone(su_pn).await {
            let lid_user = jid.to_string();
            
            let mut lock = SUPERUSER_LID.write().await;
            *lock = Some(lid_user.clone());
        }
    }
}

async fn handle_message(msg: Box<waproto::whatsapp::Message>, client: Arc<Client>, config: Arc<AppConfig>, info: MessageInfo, state: Arc<AppState> ) {
            // println!("{:#?}", msg);
            let start = std::time::Instant::now();
            let msg_arc = Arc::new(*msg);
            let info_arc = Arc::new(info);
            if let Some(exp) = msg_arc.get_expiration_timer() {
                state.clone().set_expiration(info_arc.source.chat.to_string(), exp).await;
                println!("Expiration received: {}", exp);
            }
            
            if let Some(text) = msg_arc.text_content() {
                let matched_prefix = config.prefixes.iter().find(|p| text.starts_with(*p));
                let prefix = match matched_prefix {
                    Some(p) => p,
                    None => return,
                };
                if config.mode == "self" {
                    if !is_privileged(info_arc.source.sender.user.as_str(), &info_arc, &config).await {
                        return;
                    }
                }

                let body = text.strip_prefix(prefix).unwrap_or(text);
                let args: Vec<&str> = body.split_whitespace().collect();
                if args.is_empty() { return; }
                let cmd_name = args[0];
                if let Some(cmd) = crate::commands::cmd::COMMAND_MAP.get(&cmd_name.to_ascii_lowercase()) {
                    let ctx = crate::commands::cmd::Context {
                        client: Arc::clone(&client),
                        msg: Arc::clone(&msg_arc),
                        info: Arc::clone(&info_arc),
                        state: Arc::clone(&state),
                    };
                    if let Err(e) = cmd.execute(ctx).await {
                        error!("Error executing command: {}", e);
                    }
                }
            }
            
            let duration = start.elapsed();
            println!("Executed in {:?}", duration);
}

async fn handle_group_exp(update: GroupUpdate, state: Arc<AppState>) {
    match &update.action {
        GroupNotificationAction::Ephemeral{expiration, trigger: _} => {
            state.set_expiration(update.group_jid.to_string(), *expiration).await;
        }
        _ => {}
    }
}


async fn is_privileged(sender: &str, info_arc: &Arc<MessageInfo>, config: &Arc<AppConfig>) -> bool {
    let me = info_arc.source.is_from_me;
    let su = if info_arc.source.sender.is_lid() {
    if let Ok(lock) = SUPERUSER_LID.try_read() {
        lock.as_deref() == Some(sender)
    } else {
        false
    }
    } else {
        config.superuser == Some(sender.to_string())
    };

    let privileged = me || su;
    if !privileged {
        false;
    }
    true
}
