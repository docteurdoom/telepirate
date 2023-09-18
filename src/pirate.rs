use ytd_rs::{YoutubeDL, Arg};
use std::path::PathBuf;
use glob::glob;
use teloxide::types::InputFile;
use crate::misc::cleanup;
use std::error::Error;
use regex::Regex;
use humantime::format_rfc3339_seconds as timestamp;
use std::time::SystemTime;

#[derive(Default, Debug, Clone)]
pub struct Subject {
    pub filetype: FileType,
    pub botfiles: Vec<InputFile>,
    pub paths: Vec<PathBuf>,
}

#[derive(Default, Debug, Clone)]
pub enum FileType {
    #[default]
    Mp3,
    Mp4,
    Voice,
    Gif,
}

pub type SubjectResult = Result<Subject, Box<dyn Error + Send + Sync>>;

impl FileType {
    pub fn as_str<'a>(&self) -> &'a str {
        return match self {
            FileType::Mp3 => { "mp3" }
            FileType::Mp4 => { "mp4" }
            FileType::Voice => { "opus" }
            FileType::Gif => { "gif" }
        }
    }
}

pub fn mp3(link: String) -> Subject {
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
    let downloaded = dl(link, args, filetype);
    return downloaded;
}

pub fn mp4(link: String) -> Subject {
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
    let downloaded = dl(link, args, filetype);
    return downloaded;
}

pub fn ogg(link: String) -> Subject {
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
    let downloaded = dl(link, args, filetype);
    return downloaded;
}

pub fn gif(link: String) -> Subject {
    let args = vec![
        Arg::new_with_arg("--concurrent-fragments", "100000"),
        Arg::new_with_arg("--skip-playlist-after-errors", "5000"),
        Arg::new_with_arg("--max-filesize", "50M"),
        Arg::new_with_arg("--output", "%(title)s.mp4"),
        Arg::new("--windows-filenames"),
        Arg::new("--no-write-info-json"),
        Arg::new("--no-embed-metadata"),
        Arg::new_with_arg("--format", "bv*[ext=mp4]/b[ext=mp4]"),
    ];
    let filetype = FileType::Gif;
    let downloaded = dl(link, args, filetype);
    return downloaded;
}

fn dl(link: String, args: Vec<Arg>, filetype: FileType) -> Subject {
    trace!("Downloading {}(s) from {} ...", filetype.as_str(), link);
    let basename: &str = link
        .split("/")
        .collect::<Vec<&str>>()
        .last()
        .unwrap()
        .split("?v=")
        .collect::<Vec<&str>>()
        .last()
        .unwrap();
    let destination = &format!("./downloads/{}", basename)[..];
    let path = PathBuf::from(destination);
    let ytd = YoutubeDL::new(&path, args, &link).unwrap();
    let download_result = ytd.download();
    if let Err(ref e) = download_result {
        warn!("Yt-dlp error: {}", e);
        cleanup(vec![PathBuf::from(destination)]);
    }
    
    let mut paths: Vec<PathBuf> = Vec::new();
    let regex = Regex::new(r"(.*)(\.opus)").unwrap();
    let fileformat = filetype.as_str();
    let filepaths = match filetype {
        FileType::Gif => { glob(&format!("{}/*mp4", destination)).unwrap() }
        _ => { glob(&format!("{}/*{}", destination, fileformat)).unwrap() }
    };
    for entry in filepaths {
        match entry {
            Ok(mut file_path) => {
                // Telegram allows bots sending only files under 50 MB.
                let filesize = file_path.metadata().unwrap().len();
                if filesize < 50_000_000 {
                    let filename = file_path.to_str().unwrap();
                    // Rename .opus into .ogg because Telegram requires so to display wave
                    if let Some(captures) = regex.captures(filename) {
                        let oldname = captures.get(0).unwrap().as_str();
                        let timestamp = timestamp(SystemTime::now())
                            .to_string()
                            .replace(":", "-")
                            .replace("T", "_")
                            .replace("Z", "");
                        let newname = format!("audio_{}.ogg", timestamp);
                        std::fs::rename(oldname, &newname);
                        file_path = PathBuf::from(newname);
                    }
                    paths.push(file_path);
                }
            }
            _ => {}
        }
    }
    let file_amount = paths.len();
    trace!("{} {}(s) to send", file_amount, filetype.as_str());
    if file_amount == 0 {
        cleanup(vec![PathBuf::from(destination)]);
    }
    let tg_files = paths
        .iter()
        .map(|file| InputFile::file(&file))
        .collect();
    let subject = Subject {
        filetype,
        botfiles: tg_files,
        paths,
    };
    return subject;
}
