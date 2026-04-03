use async_trait::async_trait;
use std::sync::LazyLock;
use whatsapp_rust::client::Client;
use wacore::types::message::MessageInfo;
use waproto::whatsapp::Message;
use linkme::distributed_slice;
use crate::state::AppState;
use std::{collections::HashMap, sync::Arc};

pub struct Context<'a> {
    pub client: Arc<Client>,
    pub msg: &'a Message,
    pub info: &'a MessageInfo,
    pub state: Arc<AppState>,
}

impl<'a> Context<'a> {
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
    async fn execute(&self, ctx: Context<'_>) -> anyhow::Result<()>;
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
            async fn execute(&self, $ctx: crate::commands::cmd::Context<'_> ) -> anyhow::Result<()> {
                $body;
                Ok(())
            }
        }

        #[linkme::distributed_slice(crate::commands::cmd::COMMANDS)]
        static COMMAND: &(dyn crate::commands::cmd::Command + Sync) = &$struct_name;
    };
}


pub static COMMAND_MAP: LazyLock<HashMap<String, &'static (dyn Command + Sync)>> = LazyLock::new(|| {
    let mut map = HashMap::with_capacity(COMMANDS.len() * 2);
    for &cmd in COMMANDS {
        map.insert(cmd.name().to_lowercase(), cmd);
        for alias in cmd.aliases() {
            map.insert(alias.to_lowercase(), cmd);
        }
    }
    map
});