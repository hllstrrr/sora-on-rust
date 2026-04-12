use crate::cmd;
use serde::Deserialize;
use regex::Regex;
use std::sync::LazyLock;

static SPOTIFY_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"open\.spotify\.com/track/([a-zA-Z0-9]+)").unwrap()
});

#[derive(Debug, Deserialize)]
struct ApiResponse {
    data: Data,
}

#[derive(Debug, Deserialize)]
struct Data {
    media: Media,
    title: String,
    artist: Vec<Artist>,
    cover: Vec<Cover>,
}

#[derive(Debug, Deserialize)]
struct Media {
    url: String,
}
#[derive(Debug, Deserialize)]
struct Artist {
    name: String,
}
#[derive(Debug, Deserialize)]
struct Cover {
    url: String,
}
cmd!(
    Spotify,
    name: "spotify",
    aliases: ["song", "play"],
    category: "downloader",
    execute: |ctx| {
        ctx.react("🕒").await?;
        let track_id = if let Some(caps) = SPOTIFY_REGEX.captures(ctx.body) {
            caps.get(1).map(|m| m.as_str().to_string())
        } else {
            let mut search_client = ctx.state.spotify_search_client.write().await;
            let tracks = search_client.tracks(ctx.body, 1).await?;
            
            tracks.first().map(|result| {
                result.uri.split(':').nth(2).unwrap_or("").to_string()
            })
        };
        println!("{:?}", track_id);
        let id = match track_id {
            Some(id) if !id.is_empty() => id,
            _ => {
                ctx.reply("Not found.").await?;
                return Ok(());
            }
        };
        let song_url = format!("https://open.spotify.com/track/{}", id);
        let response = ctx.state.http_client.get(format!("https://chocomilk.amira.us.kg/v1/download/spotify?url={}", song_url)).send().await?;
        let resp: ApiResponse = response.json().await?;
        let artist_name = resp.data.artist
            .iter()
            .map(|a| a.name.as_str())
            .collect::<Vec<_>>()
            .join(", ");

        send_audio!(
            context: ctx,
            audio_data: resp.data.media.url,
            dst: ctx.info.source.chat,
            reply: true,
            config_context: |context_info: &mut waproto::whatsapp::ContextInfo| {
                context_info.external_ad_reply = Some(waproto::whatsapp::context_info::ExternalAdReplyInfo {
                    title: Some(resp.data.title),
                    body: Some(artist_name),
                    media_type: Some(1),
                    thumbnail_url: Some(resp.data.cover[0].url.clone()),
                    render_larger_thumbnail: Some(true),
                    ..Default::default()
                });
            }
        ).await?;
        ctx.react("✅").await?;
        println!("Done!");
    }
);
