use waproto::whatsapp::Message;

pub trait MessageExt {
    fn get_expiration_timer(&self) -> Option<u32>;
    fn text_content(&self) -> Option<String>;
}

impl MessageExt for Message {
    fn get_expiration_timer(&self) -> Option<u32> {
        if let Some(proto) = &self.protocol_message {
            if let Some(exp) = proto.ephemeral_expiration {
                return Some(exp);
            }
        }

        if self.conversation.is_some() {
            return Some(0);
        }

        let context = if let Some(m) = &self.extended_text_message {
            m.context_info.as_deref()
        } else if let Some(m) = &self.image_message {
            m.context_info.as_deref()
        } else if let Some(m) = &self.video_message {
            m.context_info.as_deref()
        } else if let Some(m) = &self.document_message {
            m.context_info.as_deref()
        } else if let Some(m) = &self.sticker_message {
            m.context_info.as_deref()
        } else if let Some(m) = &self.audio_message {
            m.context_info.as_deref()
        }
         else {
            None
        };
        context.map(|ctx| ctx.expiration.unwrap_or(0))
    }

    fn text_content(&self) -> Option<String> {
        if let Some(t) = &self.conversation { return Some(t.clone()); }
        if let Some(m) = &self.extended_text_message { return m.text.clone(); }
        if let Some(m) = &self.image_message { return m.caption.clone(); }
        if let Some(m) = &self.video_message { return m.caption.clone(); }
        if let Some(m) = &self.document_message { return m.caption.clone(); }        
        if let Some(m) = &self.document_with_caption_message {
            return m.message.as_ref()
                .and_then(|nested| nested.document_message.as_ref())
                .and_then(|doc| doc.caption.clone());
        }
       None
    }
}