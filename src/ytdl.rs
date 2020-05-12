use serde::Deserialize;
use serenity::{
    prelude::SerenityError,
    voice::{ytdl, AudioSource},
};
use std::{
    boxed::Box,
    io::{self, ErrorKind},
    process::Command,
};

#[cfg(test)]
mod test {
    #[test]
    fn ytdl_meta() {
        let meta = super::metadata("https://www.youtube.com/watch?v=CGw0juHIoh4");
        assert!(meta.is_ok());
        let meta = meta.unwrap();

        assert_eq!(meta.description, "Track 2 of Exit Plan: https://fearofdark.bandcamp.com/album/exit-plan\n\nFeatures a guitar solo from Danimal Cannon.");
        assert_eq!(meta.duration, 191);
        assert_eq!(
            meta.thumbnail,
            "https://i.ytimg.com/vi/CGw0juHIoh4/maxresdefault.jpg"
        );
        assert_eq!(meta.title, meta.fulltitle);
        assert_eq!(meta.upload_date, "20200503");
        assert_eq!(meta.uploader_url, "http://www.youtube.com/user/fodsteve1");
        assert_eq!(meta.uploader, "Fearofdark");
        assert_eq!(
            meta.webpage_url,
            "https://www.youtube.com/watch?v=CGw0juHIoh4"
        );
    }
}

/// Map a ytdl spawn error to a serenity error
fn map_ytdl_spawn_err(err: io::Error) -> SerenityError {
    match err.kind() {
        ErrorKind::NotFound => SerenityError::Other("youtube-dl is not installed"),
        _ => err.into(),
    }
}

/// Wrapper around serenity::voice::ytdl
pub fn stream_url(url: &str) -> serenity::Result<Box<dyn AudioSource>> {
    ytdl(url).map_err(|err| match &err {
        SerenityError::Io(e) => match e.kind() {
            ErrorKind::NotFound => SerenityError::Other("youtube-dl or ffmpeg is not installed"),
            _ => err,
        },
        _ => err,
    })
}

#[derive(Debug, Deserialize)]
/// Metadata about a ytdl source
pub struct YtdlMetadata {
    /// The date of the upload in YYYYMMDD
    upload_date: String,
    /// The title of the video
    title: String,
    /// The full title of the video
    fulltitle: String,
    /// The current view count
    view_count: u32,
    /// The video description
    description: String,
    /// The name of the video uploader
    uploader: String,
    /// The thumbnail of the video
    thumbnail: String,
    /// The url to the video
    webpage_url: String,
    /// The url to the uploader's page
    uploader_url: String,
    /// The duration of the video
    duration: u32,
}

/// Get the metatdata for a url from ytdl
pub fn metadata(url: &str) -> serenity::Result<YtdlMetadata> {
    let youtube_dl = Command::new("youtube-dl")
        .args(&["-j", "--no-playlist", "--ignore-config", url])
        .output()
        .map_err(map_ytdl_spawn_err)?;

    let json_result = String::from_utf8(youtube_dl.stdout)
        .map_err(|_| SerenityError::Other("Failed to read output of youtube-dl"))?;

    return Ok(serde_json::from_str(&json_result)?);
}

/// Search youtube for the search term
pub fn search(term: &str, max_results: u8) -> serenity::Result<Box<[YtdlMetadata]>> {
    let youtube_dl = Command::new("youtube-dl")
        .args(&["-j",  "--no-playlist", "--ignore-config", &format!("ytsearch{}:{}", max_results, term)])
        .output()
        .map_err(map_ytdl_spawn_err)?;

    let json_result = String::from_utf8(youtube_dl.stdout)
        .map_err(|_| SerenityError::Other("Failed to read output of youtube-dl"))?;

    return Ok(serde_json::from_str(&format!("[{}]", json_result))?);
}
