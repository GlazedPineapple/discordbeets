use serde::Deserialize;
use serenity::{
    prelude::SerenityError,
    voice::{ytdl, AudioSource},
};
use std::{
    boxed::Box,
    io::{self, ErrorKind, Read},
    process::Command,
    process::Stdio,
};
use thiserror::Error;

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

#[derive(Error, Debug)]
pub enum YtdlError {
    #[error("Failed to start youtube-dl. is it installed?")]
    NotInstalled,
    #[error("Failed to start youtube-dl or ffmpeg. are they installed?")]
    NotInstalledFfmpeg,
    #[error("Encountered an error while running youtube-dl: {0}")]
    Runtime(String),
    #[error(transparent)]
    Json(#[from] serde_json::error::Error),
    #[error(transparent)]
    Serenity(#[from] SerenityError),
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Utf8(#[from] std::string::FromUtf8Error),
}
type Result<T> = std::result::Result<T, YtdlError>;

/// Map a ytdl spawn error to a serenity error
fn map_ytdl_spawn_err(err: io::Error) -> YtdlError {
    match err.kind() {
        ErrorKind::NotFound => YtdlError::NotInstalled,
        _ => err.into(),
    }
}

/// Wrapper around serenity::voice::ytdl
pub fn stream_url(url: &str) -> Result<Box<dyn AudioSource>> {
    ytdl(url).map_err(|err| match &err {
        SerenityError::Io(e) => match e.kind() {
            ErrorKind::NotFound => YtdlError::NotInstalledFfmpeg,
            _ => YtdlError::from(err),
        },
        _ => YtdlError::from(err),
    })
}

#[derive(Debug, Deserialize)]
/// Metadata about a ytdl source
pub struct YtdlMetadata {
    /// The date of the upload in YYYYMMDD
    pub upload_date: String,
    /// The title of the video
    pub title: String,
    /// The full title of the video
    pub fulltitle: String,
    /// The current view count
    pub view_count: u32,
    /// The video description
    pub description: String,
    /// The name of the video uploader
    pub uploader: String,
    /// The thumbnail of the video
    pub thumbnail: String,
    /// The url to the video
    pub webpage_url: String,
    /// The url to the uploader's page
    pub uploader_url: String,
    /// The duration of the video
    pub duration: u32,
}

/// Get the metatdata for a url from ytdl
pub fn metadata(url: &str) -> Result<YtdlMetadata> {
    let mut youtube_dl = Command::new("youtube-dl")
        .args(&["-j", "--no-playlist", "--ignore-config", url])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(map_ytdl_spawn_err)?;

    let exit_status = youtube_dl.wait()?;

    if !exit_status.success() {
        let mut stderr = youtube_dl.stderr.ok_or(io::Error::new(
            io::ErrorKind::Other,
            "Failed to get the stderr of youtube_dl",
        ))?;

        let mut string = String::new();
        stderr.read_to_string(&mut string)?;

        return Err(YtdlError::Runtime(string));
    }

    let mut stdout = youtube_dl.stdout.ok_or(io::Error::new(
        io::ErrorKind::Other,
        "Failed to get the stderr of youtube_dl",
    ))?;

    let mut string = String::new();
    stdout.read_to_string(&mut string)?;

    return Ok(serde_json::from_str(&string)?);
}

/// Search youtube for the search term
pub fn search(term: &str, max_results: u8) -> Result<Box<[YtdlMetadata]>> {
    let mut youtube_dl = Command::new("youtube-dl")
        .args(&[
            "-j",
            "--no-playlist",
            "--ignore-config",
            &format!("ytsearch{}:\"{}\"", max_results, term),
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(map_ytdl_spawn_err)?;

    let exit_status = youtube_dl.wait()?;

    if !exit_status.success() {
        let mut stderr = youtube_dl.stderr.ok_or(io::Error::new(
            io::ErrorKind::Other,
            "Failed to get the stderr of youtube_dl",
        ))?;

        let mut string = String::new();
        stderr.read_to_string(&mut string)?;

        return Err(YtdlError::Runtime(string));
    }

    let mut stdout = youtube_dl.stdout.ok_or(io::Error::new(
        io::ErrorKind::Other,
        "Failed to get the stderr of youtube_dl",
    ))?;

    let mut string = String::new();
    stdout.read_to_string(&mut string)?;

    return Ok(serde_json::from_str(&format!("[{}]", string))?);
}
