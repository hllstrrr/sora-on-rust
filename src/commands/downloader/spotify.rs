use crate::cmd;
use serde::{Deserialize, Serialize};
use wacore::{download::MediaType, proto_helpers::build_quote_context_with_info};
use waproto::whatsapp::{self as wa};
#[derive(Serialize, Debug)]
struct Song {
    url: String,
}
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct SpotifyData {
    title: String,
    original_video_url: String,
    cover_url: String,
    author_name: String,
}

cmd!(
    Spotify,
    name: "spotify",
    aliases: ["song", "play"],
    category: "downloader",
    execute: |ctx| {
        let mut search_client = ctx.state.spotify_search_client.write().await;
        let tracks = search_client.tracks(ctx.body, 1).await?;
        let mut track = String::new();
        if let Some(result) = tracks.first() {
            track = result.uri.clone();
        }
        let song_url = format!("https://open.spotify.com/track/{}", track.split(':').nth(2).unwrap_or(""));
        let payload = Song { url: song_url };
        println!("{:?}", payload);
        let response = ctx.state.http_client.post("https://gamepvz.com/api/download/get-url")
        .json(&payload)
        .send()
        .await?;
        let result: SpotifyData = response.json().await?;
        println!("{:?}", result);

        println!("Downloading audio...");
        let audio_data = ctx.state.http_client.get(format!("https://gamepvz.com/{}", result.original_video_url)).send().await?.bytes().await?.to_vec();
        println!("Uploading audio...");
        let upload = ctx.client.upload(audio_data, MediaType::Audio).await?;
        let expiration = ctx.state.get_expiration(&ctx.info.source.chat.to_string());
        let ad_reply = wa::context_info::ExternalAdReplyInfo {
            title: Some(result.title),
            body: Some(result.author_name),
            media_type: Some(1),
            thumbnail_url: Some(result.cover_url),
            render_larger_thumbnail: Some(true),
            ..Default::default()
        };
        let chat_jid = ctx.info.source.chat.to_string();
        let mut context_info = build_quote_context_with_info(
            &ctx.info.id,
            &ctx.info.source.sender,
            &ctx.info.source.chat,
            ctx.msg,
        );
        context_info.remote_jid = Some(chat_jid);
        context_info.external_ad_reply = Some(ad_reply);
        if expiration > 0 {
            context_info.expiration = Some(expiration);
        }

        let audio_msg = wa::Message {
            audio_message: Some(Box::new(wa::message::AudioMessage {
                url: Some(upload.url),
                direct_path: Some(upload.direct_path),
                media_key: Some(upload.media_key),
                file_sha256: Some(upload.file_sha256),
                file_enc_sha256: Some(upload.file_enc_sha256),
                file_length: Some(upload.file_length),
                mimetype: Some("audio/mpeg".to_string()),
                context_info: Some(Box::new(context_info)),
                ..Default::default()
            })),
            ..Default::default()
        };
        ctx.client.send_message(ctx.info.source.chat.clone(), audio_msg).await?;
        println!("Done!");
    }
);