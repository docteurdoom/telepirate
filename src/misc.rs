use std::fs::remove_dir_all;
use std::path::PathBuf;
use std::thread;
use std::time;
use validators::prelude::*;
use validators::url::Url;

#[derive(Validator)]
#[validator(http_url(local(Allow)))]
pub struct HttpURL {
    url: Url,
    is_https: bool,
}

pub fn cleanup(relative_destination_path: PathBuf) {
    debug!("Deleting the working directory ...");
    remove_dir_all(relative_destination_path).unwrap();
}

pub fn boot() {
    use crate::logger;
    logger::init();
    checkdep("yt-dlp");
    checkdep("ffmpeg");
}

fn checkdep(dep: &str) {
    trace!("Checking dependency {} ...", dep);
    let result_output = std::process::Command::new(dep).arg("--help").output();
    if let Err(e) = result_output {
        if let std::io::ErrorKind::NotFound = e.kind() {
            error!("{} is not found. Please install {} first.", dep, dep);
            std::process::exit(1);
        }
    }
}

pub fn sleep(secs: u32) {
    let time = time::Duration::from_secs(secs.into());
    thread::sleep(time);
}

pub fn url_is_valid(url: &str) -> bool {
    return HttpURL::parse_string(url).is_ok();
}
