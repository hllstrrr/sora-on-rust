use std::fs;
use std::path::Path;
use std::sync::Arc;
use whatsapp_rust::bot::Bot;
use whatsapp_rust::store::SqliteStore;
use whatsapp_rust_tokio_transport::TokioWebSocketTransportFactory;
use whatsapp_rust_ureq_http_client::UreqHttpClient;
use wacore::pair_code::{PairCodeOptions, PlatformId};
use crate::handler::event_handler;
use crate::state::AppState;
use crate::config::AppConfig;
pub async fn create_bot(config: Arc<AppConfig>, state: Arc<AppState>) -> anyhow::Result<Bot> {

    let db_path = Path::new(&config.db_path);
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent)?;
        log::info!("Ensured directory {:?} exists", parent);
    }
    let backend = Arc::new(SqliteStore::new(&config.db_path).await?);
    let bot = Bot::builder()
        .skip_history_sync()
        .with_backend(backend)
        .with_transport_factory(TokioWebSocketTransportFactory::new())
        .with_http_client(UreqHttpClient::new())
        .with_pair_code(PairCodeOptions {
            phone_number: config.phone_number.clone(), 
            show_push_notification: true,
            custom_code: Some(config.custom_code.clone()),
            platform_id: PlatformId::Chrome,
            platform_display: "Chrome (Linux)".to_string(),
        })
        .on_event(move |event, client| {
            let st = Arc::clone(&state);
            let cfg = Arc::clone(&config);
            async move {
                event_handler(event, client, cfg, st).await;
            }        
        })
        .build()
        .await?;

    Ok(bot)
}