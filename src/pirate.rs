use ytd_rs::{YoutubeDL, Arg};
use std::path::PathBuf;
use glob::glob;
use teloxide::types::InputFile;
use crate::misc::cleanup;

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
}

pub type SubjectResult = Result<Subject, Box<dyn Error + Send + Sync>>;

impl FileType {
    fn determine(args: &Vec<Arg>) -> Self {
        return if args.len() == 10 {
            FileType::Mp3
        } else if args.len() == 8 {
            FileType::Mp4
        } else {
            error!("Unknown FileType!");
            std::process::exit(10);
        }
    }
    pub fn as_str<'a>(&self) -> &'a str {
        return match self {
            FileType::Mp3 => {
                "mp3"
            }
            FileType::Mp4 => {
                "mp4"
            }
        }
    }
}

pub fn mp3(link: String) -> SubjectResult {
    let mp3args = vec![
        Arg::new_with_arg("--concurrent-fragments", "100000"),
        Arg::new_with_arg("--max-filesize", "50M"),
        Arg::new_with_arg("--skip-playlist-after-errors", "5000"),
        Arg::new_with_arg("--output", "%(title)s"),
        Arg::new("--windows-filenames"),
        Arg::new("--no-write-info-json"),
        Arg::new("--no-embed-metadata"),
        Arg::new("--extract-audio"),
        Arg::new_with_arg("--audio-format", "mp3"),
        Arg::new_with_arg("--audio-quality", "0"),
    ];
    let downloaded = dl(link, mp3args);
    return downloaded;
}

pub fn mp4(link: String) -> SubjectResult {
    let mp4args = vec![
        Arg::new_with_arg("--concurrent-fragments", "100000"),
        Arg::new_with_arg("--max-filesize", "50M"),
        Arg::new_with_arg("--skip-playlist-after-errors", "5000"),
        Arg::new_with_arg("--output", "%(title)s"),
        Arg::new("--windows-filenames"),
        Arg::new("--no-write-info-json"),
        Arg::new("--no-embed-metadata"),
        Arg::new_with_arg("--format", "bv*[ext=mp4]+ba[ext=m4a]/b[ext=mp4]"),
    ];
    let downloaded = dl(link, mp4args);
    return downloaded;
}

use std::error::Error;
fn dl(link: String, args: Vec<Arg>) -> SubjectResult {
    let filetype = FileType::determine(&args);
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

    let download_result = ytd.download()?;
    let mut paths: Vec<PathBuf> = Vec::new();
    for entry in glob(&format!("{}/*{}", destination, filetype.as_str())[..])? {
        match entry {
            Ok(file_path) => {
                paths.push(file_path);
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
    Ok(subject)
}
