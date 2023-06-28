use ytd_rs::{YoutubeDL, Arg};
use std::path::PathBuf;
use std::error::Error;

pub async fn mp3(link: &str) -> Result<(), Box<dyn Error>> {
    info!("Downloading {} ...", link);
    let mp3args = vec![
        Arg::new_with_arg("--concurrent-fragments", "100000"),
        Arg::new("--restrict-filenames"),
        Arg::new("--windows-filenames"),
        Arg::new("--no-write-info-json"),
        Arg::new("--no-embed-metadata"),
        Arg::new("--extract-audio"),
        Arg::new_with_arg("--audio-format", "mp3"),
        Arg::new_with_arg("--audio-quality", "0"),
    ];
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
    let ytd = YoutubeDL::new(&path, mp3args, link)?;

    let download = ytd.download()?;
    // info!("File saved to {}", destination);
    Ok(())
}

async fn mp4() {
    let mp4args = vec![
        Arg::new_with_arg("--concurrent-fragments", "100000"),
        Arg::new("--restrict-filenames"),
        Arg::new("--windows-filenames"),
        Arg::new("--no-write-info-json"),
        Arg::new("--no-embed-metadata"),
        Arg::new_with_arg("--format", "bv*[ext=mp4]+ba[ext=m4a]/b[ext=mp4]"),
    ];
}