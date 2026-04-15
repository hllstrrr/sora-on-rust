use std::process::Stdio;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use waproto::whatsapp::Message;
use whatsapp_rust::client::Client;
use whatsapp_rust::Jid;
use waproto::whatsapp as wa;

use crate::state::AppState;

pub trait MessageExt {
    fn get_expiration_timer(&self) -> Option<u32>;
    fn text_content(&self) -> Option<&String>;
}

impl MessageExt for Message {
    fn get_expiration_timer(&self) -> Option<u32> {
        if let Some(proto) = &self.protocol_message
            && let Some(exp) = proto.ephemeral_expiration
        {
            return Some(exp);
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
        } else {
            None
        };
        context.map(|ctx| ctx.expiration.unwrap_or(0))
    }

    fn text_content(&self) -> Option<&String> {
        if let Some(t) = &self.conversation {
            return Some(t);
        }
        if let Some(m) = &self.extended_text_message {
            return m.text.as_ref();
        }
        if let Some(m) = &self.image_message {
            return m.caption.as_ref();
        }
        if let Some(m) = &self.video_message {
            return m.caption.as_ref();
        }
        if let Some(m) = &self.document_message {
            return m.caption.as_ref();
        }
        if let Some(m) = &self.document_with_caption_message {
            return m
                .message
                .as_ref()
                .and_then(|nested| nested.document_message.as_ref())
                .and_then(|doc| doc.caption.as_ref());
        }
        None
    }
}
pub async fn get_media_bytes(state: Arc<AppState>, data: Vec<u8>) -> anyhow::Result<Vec<u8>> {
    if let Ok(url_str) = String::from_utf8(data.clone())
        && url_str.starts_with("http")
    {
        let resp = state.http_client.get(url_str).send().await?;
        return Ok(resp.bytes().await?.to_vec());
    }
    Ok(data)
}

pub async fn generate_video_thumbnail(video_bytes: &[u8]) -> anyhow::Result<Vec<u8>> {
    let mut child = Command::new("ffmpeg")
        .args([
            "-i",
            "pipe:0",
            "-ss",
            "00:00:01",
            "-vframes",
            "1",
            "-f",
            "image2",
            "-vf",
            "scale=160:-1",
            "-q:v",
            "2",
            "pipe:1",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;

    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| anyhow::anyhow!("Unable to open stdin ffmpeg"))?;

    let video_vec = video_bytes.to_vec();
    tokio::spawn(async move {
        let _ = stdin.write_all(&video_vec).await;
    });

    let output = child.wait_with_output().await?;
    if output.status.success() {
        Ok(output.stdout)
    } else {
        Err(anyhow::anyhow!("ffmpeg: unable to process the video"))
    }
}

pub async fn send_warmup(
    client: Arc<Client>,
    chat_jid: Jid,
    msg_id: String,
    participant: Option<String>,
) -> anyhow::Result<()> {
    println!("Sending warmup message");
    let warmup_msg = wa::Message {
        reaction_message: Some(wa::message::ReactionMessage {
            key: Some(wa::MessageKey {
                remote_jid: Some(chat_jid.to_string()),
                from_me: Some(false),
                id: Some(msg_id),
                participant,
            }),
            text: Some("".to_string()),
            sender_timestamp_ms: Some(chrono::Utc::now().timestamp_millis()),
            ..Default::default()
        }),
        ..Default::default()
    };

    client.send_message(chat_jid, warmup_msg).await?;
    Ok(())
}