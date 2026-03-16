use async_trait::async_trait;
use whatsapp_rust::client::Client;
use wacore::types::message::MessageInfo;
use linkme::distributed_slice;

#[distributed_slice]
pub static COMMANDS: [&(dyn BotCommand + Sync)] = [..];

#[async_trait]
pub trait BotCommand: Send + Sync {
    fn name(&self) -> &str;
    fn aliases(&self) -> &[&str];
    fn category(&self) -> &str;
    async fn execute(&self, client: &Client, info: &MessageInfo) -> anyhow::Result<()>;
}

#[macro_export]
macro_rules! cmd {
    ($struct_name:ident, name: $name:expr, aliases: [$($alias:expr),*], category: $cat:expr, execute: |$client:ident, $msg:ident, $info:ident| $body:block) => {
        pub struct $struct_name;

        #[async_trait::async_trait]
        impl crate::commands::cmd::BotCommand for $struct_name {
            fn name(&self) -> &str { $name }
            fn aliases(&self) -> &[&str] { &[$($alias),*] }
            fn category(&self) -> &str { $cat }
            async fn execute(&self, $client: &whatsapp_rust::client::Client, $info: &wacore::types::message::MessageInfo) -> anyhow::Result<()> {
                $body;
                Ok(())
            }
        }

        #[linkme::distributed_slice(crate::commands::cmd::COMMANDS)]
        static COMMAND: &(dyn crate::commands::cmd::BotCommand + Sync) = &$struct_name;
    };
}