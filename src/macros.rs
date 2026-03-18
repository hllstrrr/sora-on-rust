#[macro_export]
macro_rules! send_msg {
    ($client:expr, $info:expr, $state:expr, dst: $dst:expr, text: $text:expr, reply: $is_reply:expr) => {{
        let mut context = waproto::whatsapp::ContextInfo::default();
        let expiration = $state.get_expiration(&$dst.to_string());
        if expiration > 0 {
            context.expiration = Some(expiration);
        }

        if $is_reply {
            context.stanza_id = Some($info.id.clone());
            context.participant = Some($info.source.sender.to_string());
        }

        let mentions: Vec<String> = $text
            .split_whitespace()
            .filter(|word| word.starts_with('@') && word.len() > 10)
            .map(|m| format!("{}@s.whatsapp.net", &m[1..]))
            .collect();
        
        if !mentions.is_empty() {
            context.mentioned_jid = mentions;
        }

        let message = waproto::whatsapp::Message {
            extended_text_message: Some(Box::new(waproto::whatsapp::message::ExtendedTextMessage {
                text: Some($text.to_string()),
                context_info: Some(Box::new(context)),
                ..Default::default()
            })),
            ..Default::default()
        };

        $client.send_message($dst.clone(), message)
    }};
}