use async_trait::async_trait;
use whatsapp_rust::client::Client;
use wacore::types::message::MessageInfo;
use waproto::whatsapp::Message;
use linkme::distributed_slice;
use crate::state::AppState;
use std::sync::Arc;

#[distributed_slice]
pub static COMMANDS: [&(dyn Command + Sync)] = [..];

#[async_trait]
pub trait Command: Send + Sync {
    fn name(&self) -> &str;
    fn aliases(&self) -> &[&str];
    fn category(&self) -> &str;
    async fn execute(&self, client: &Client, msg: &Message, info: &MessageInfo, state: &Arc<AppState>) -> anyhow::Result<()>;
}

#[macro_export]
macro_rules! cmd {
    ($struct_name:ident, name: $name:expr, aliases: [$($alias:expr),*], category: $cat:expr, execute: |$client:ident, $msg:ident, $info:ident, $state: ident| $body:block) => {
        pub struct $struct_name;

        #[async_trait::async_trait]
        impl crate::commands::cmd::Command for $struct_name {
            fn name(&self) -> &str { $name }
            fn aliases(&self) -> &[&str] { &[$($alias),*] }
            fn category(&self) -> &str { $cat }
            async fn execute(&self, $client: &whatsapp_rust::client::Client, $msg: &waproto::whatsapp::Message, $info: &wacore::types::message::MessageInfo, $state: &std::sync::Arc<crate::state::AppState> ) -> anyhow::Result<()> {
                $body;
                Ok(())
            }
        }

        #[linkme::distributed_slice(crate::commands::cmd::COMMANDS)]
        static COMMAND: &(dyn crate::commands::cmd::Command + Sync) = &$struct_name;
    };
}