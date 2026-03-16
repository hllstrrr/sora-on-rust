use std::sync::Arc;
use whatsapp_rust::bot::Bot;
use whatsapp_rust::store::SqliteStore;
use whatsapp_rust_tokio_transport::TokioWebSocketTransportFactory;
use whatsapp_rust_ureq_http_client::UreqHttpClient;
use wacore::pair_code::{PairCodeOptions, PlatformId};
use crate::handler::event_handler;

pub async fn create_bot() -> anyhow::Result<Bot> {
    let backend = Arc::new(SqliteStore::new("whatsapp.db").await?);
    
    let bot = Bot::builder()
        .with_backend(backend)
        .with_transport_factory(TokioWebSocketTransportFactory::new())
        .with_http_client(UreqHttpClient::new())
        .with_pair_code(PairCodeOptions {
            phone_number: "62895404956278".to_string(),
            show_push_notification: true,
            custom_code: Some("HELLSTAR".to_string()),
            platform_id: PlatformId::Chrome,
            platform_display: "Chrome (Linux)".to_string(),
        })
        .on_event(|event, client| async move {
            event_handler(event, client).await;
        })
        .build()
        .await?;

    Ok(bot)
}