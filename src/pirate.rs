use crate::misc::cleanup;
use glob::glob;
use humantime::format_rfc3339_seconds as timestamp;
use regex::Regex;
use std::error::Error;
use std::path::PathBuf;
use std::time::SystemTime;
use ytd_rs::{Arg, YoutubeDL};
use uuid::Uuid;

type DownloadsResult = Result<Downloads, Box<dyn Error + Send + Sync>>;

#[derive(Default, Debug, Clone)]
pub struct Downloads {
    pub filetype: FileType,
    pub paths: Vec<PathBuf>,
    pub folder: PathBuf,
}

#[derive(Default, Debug, Clone)]
pub enum FileType {
    #[default]
    Mp3,
    Mp4,
    Voice,
    Gif,
}

impl FileType {
    pub fn as_str<'a>(&self) -> &'a str {
        return match self {
            FileType::Mp3 => "mp3",
            FileType::Mp4 => "mp4",
            FileType::Voice => "opus",
            FileType::Gif => "gif",
        };
    }
}

pub fn mp3(url: String) -> DownloadsResult {
    let args = vec![
        Arg::new_with_arg("--concurrent-fragments", "100000"),
        Arg::new_with_arg("--skip-playlist-after-errors", "5000"),
        Arg::new_with_arg("--output", "%(title)s.mp3"),
        Arg::new("--windows-filenames"),
        Arg::new("--no-write-info-json"),
        Arg::new("--no-embed-metadata"),
        Arg::new("--extract-audio"),
        Arg::new_with_arg("--audio-format", "mp3"),
        Arg::new_with_arg("--audio-quality", "0"),
    ];
    let filetype = FileType::Mp3;
    let downloaded = dl(url, args, filetype)?;
    Ok(downloaded)
}

pub fn mp4(url: String) -> DownloadsResult {
    let args = vec![
        Arg::new_with_arg("--concurrent-fragments", "100000"),
        Arg::new_with_arg("--skip-playlist-after-errors", "5000"),
        Arg::new_with_arg("--max-filesize", "50M"),
        Arg::new_with_arg("--output", "%(title)s.mp4"),
        Arg::new("--windows-filenames"),
        Arg::new("--no-write-info-json"),
        Arg::new("--no-embed-metadata"),
        Arg::new_with_arg("--format", "bv*[ext=mp4]+ba[ext=m4a]/b[ext=mp4]"),
    ];
    let filetype = FileType::Mp4;
    let downloaded = dl(url, args, filetype)?;
    Ok(downloaded)
}

pub fn ogg(url: String) -> DownloadsResult {
    let args = vec![
        Arg::new_with_arg("--concurrent-fragments", "100000"),
        Arg::new_with_arg("--skip-playlist-after-errors", "5000"),
        Arg::new("--windows-filenames"),
        Arg::new("--no-write-info-json"),
        Arg::new("--no-embed-metadata"),
        Arg::new("--extract-audio"),
        Arg::new_with_arg("--audio-format", "opus"),
        Arg::new_with_arg("--audio-quality", "64K"),
    ];
    let filetype = FileType::Voice;
    let downloaded = dl(url, args, filetype)?;
    Ok(downloaded)
}

pub fn gif(url: String) -> DownloadsResult {
    let args = vec![
        Arg::new_with_arg("--concurrent-fragments", "100000"),
        Arg::new_with_arg("--skip-playlist-after-errors", "5000"),
        Arg::new_with_arg("--max-filesize", "50M"),
        Arg::new_with_arg("--output", "%(title)s.mp4"),
        Arg::new("--windows-filenames"),
        Arg::new("--no-write-info-json"),
        Arg::new("--no-embed-metadata"),
        Arg::new_with_arg("--format-sort", "ext:mp4,codec:h264"),
        Arg::new_with_arg("--format", "bv"),
    ];
    let filetype = FileType::Gif;
    let downloaded = dl(url, args, filetype)?;
    Ok(downloaded)
}

fn dl(url: String, args: Vec<Arg>, filetype: FileType) -> DownloadsResult {
    trace!("Downloading {}(s) from {} ...", filetype.as_str(), url);
    // UUID is used because thats my choice.
    let destination_basename = Uuid::new_v4();
    let relative_destination_path = &format!("./downloads/{}", destination_basename)[..];
    let path = PathBuf::from(relative_destination_path);
    let ytd = YoutubeDL::new(&path, args, &url)?;
    let _ = ytd.download()?;
    let mut paths: Vec<PathBuf> = Vec::new();
    let regex = Regex::new(r"(.*)(\.opus)").unwrap();
    let fileformat = filetype.as_str();
    let filepaths = match filetype {
        FileType::Gif => glob(&format!("{}/*mp4", relative_destination_path))?,
        _ => glob(&format!("{}/*{}", relative_destination_path, fileformat))?,
    };
    for entry in filepaths {
        match entry {
            Ok(mut file_path) => {
                // Telegram allows bots sending only files under 50 MB.
                let filesize = file_path.metadata()?.len();
                if filesize < 50_000_000 {
                    let filename = file_path.to_str().unwrap();
                    // Rename .opus into .ogg because Telegram requires so to display wave pattern.
                    if let Some(captures) = regex.captures(filename) {
                        let oldname = captures.get(0).unwrap().as_str();
                        let timestamp = timestamp(SystemTime::now())
                            .to_string()
                            .replace(":", "-")
                            .replace("T", "_")
                            .replace("Z", "");
                        let newname = format!("{}/audio_{}.ogg", relative_destination_path, timestamp);
                        std::fs::rename(oldname, &newname)?;
                        file_path = PathBuf::from(newname);
                    }
                    paths.push(file_path);
                }
            }
            _ => {}
        }
    }
    let file_amount = paths.len();
    info!("{} {}(s) to send.", file_amount, filetype.as_str());
    if file_amount == 0 {
        cleanup(relative_destination_path.into());
    }
    let downloads = Downloads {
        filetype,
        paths,
        folder: relative_destination_path.into(),
    };
    Ok(downloads)
}
