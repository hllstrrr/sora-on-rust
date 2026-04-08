use crate::cmd;
use serde::Deserialize;
use wacore::{download::MediaType, proto_helpers::build_quote_context_with_info};
use waproto::whatsapp as wa;

#[derive(Debug, Deserialize)]
pub struct ApiResponse {
    pub status: bool,
    pub result: Result,
}

#[derive(Debug, Deserialize)]
pub struct Result {
    pub thumbnail: String,
    pub contents: Vec<Content>,
    pub metadata: Metadata,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Content {
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct Metadata {
    pub username: String,
    pub title: String,
}

cmd!(
    Reels,
    name: "reels",
    aliases: ["ig", "instagram"],
    category: "downloader",
    execute: |ctx| {
        if ctx.body.is_empty() {
            ctx.react("❔").await?;
            return Ok(());
        }
        let api = "https://api.apigratis.cc/downloader/instagram?url=".to_string();
        let target = ctx.body;
        let response = ctx.state.http_client.get(api + target).send().await?;
        let data: ApiResponse = response.json().await?;
        if !data.status {
            ctx.reply("Not found").await?;
            return Ok(())
        }
        let thumb_data = ctx.state.http_client.get(data.result.thumbnail.clone()).send().await?.bytes().await?.to_vec();
        println!("URL VID: {}", data.result.contents[0].url.clone());
        let vid_data = ctx.state.http_client.get(data.result.contents[0].url.clone()).send().await?.bytes().await?.to_vec();
        let upload = ctx.client.upload(vid_data, MediaType::Video).await?;
        let mut context_info = build_quote_context_with_info(
            &ctx.info.id,
            &ctx.info.source.sender,
            &ctx.info.source.chat,
            ctx.msg,
        );
        let chat_jid = ctx.info.source.chat.clone();
        context_info.remote_jid = Some(chat_jid.to_string());
        let expiration = ctx.state.get_expiration(&ctx.info.source.chat.to_string());
        if expiration > 0 {
            context_info.expiration = Some(expiration);
        }
        let text = format!("*Author:* {}\n{}", data.result.metadata.username, data.result.metadata.title);
        let vid_msg = wa::Message {
            video_message: Some(Box::new(wa::message::VideoMessage {
                url: Some(upload.url),
                direct_path: Some(upload.direct_path),
                media_key: Some(upload.media_key),
                file_sha256: Some(upload.file_sha256),
                file_enc_sha256: Some(upload.file_enc_sha256),
                file_length: Some(upload.file_length),
                jpeg_thumbnail: Some(thumb_data),
                mimetype: Some("video/mp4".to_string()),
                caption: Some(text),
                context_info: Some(Box::new(context_info)),
                ..Default::default()
            })),
            ..Default::default()
        };
        ctx.client.send_message(ctx.info.source.chat.clone(), vid_msg).await?;
        println!("Done!");
    }
);
