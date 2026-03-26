use async_trait::async_trait;
use whatsapp_rust::client::Client;
use wacore::types::message::MessageInfo;
use waproto::whatsapp::Message;
use linkme::distributed_slice;
use crate::state::AppState;
use std::sync::Arc;

pub struct Context {
    pub client: Arc<Client>,
    pub msg: Arc<Message>,
    pub info: Arc<MessageInfo>,
    pub state: Arc<AppState>,
}

impl Context {
    pub async fn reply(&self, text: &str) -> anyhow::Result<String> {
        let msg_id = crate::send_msg!(
            self.client,
            self.msg,
            self.info,
            self.state,
            dst: self.info.source.chat,
            text: text,
            reply: true
        )
        .await?;
        Ok(msg_id)
    }
}
#[distributed_slice]
pub static COMMANDS: [&(dyn Command + Sync)] = [..];

#[async_trait]
pub trait Command: Send + Sync {
    fn name(&self) -> &str;
    fn aliases(&self) -> &[&str];
    fn category(&self) -> &str;
    async fn execute(&self, ctx: Context) -> anyhow::Result<()>;
}

#[macro_export]
macro_rules! cmd {
    ($struct_name:ident, name: $name:expr, aliases: [$($alias:expr),*], category: $cat:expr, execute: |$ctx: ident| $body:block) => {
        pub struct $struct_name;

        #[async_trait::async_trait]
        impl crate::commands::cmd::Command for $struct_name {
            fn name(&self) -> &str { $name }
            fn aliases(&self) -> &[&str] { &[$($alias),*] }
            fn category(&self) -> &str { $cat }
            async fn execute(&self, $ctx: crate::commands::cmd::Context ) -> anyhow::Result<()> {
                $body;
                Ok(())
            }
        }

        #[linkme::distributed_slice(crate::commands::cmd::COMMANDS)]
        static COMMAND: &(dyn crate::commands::cmd::Command + Sync) = &$struct_name;
    };
}