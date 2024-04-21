use std::fs::remove_dir_all;
use std::path::PathBuf;
use std::thread;
use std::time;

pub fn cleanup(paths: Vec<PathBuf>) {
    debug!("Cleaning up the working directory ...");
    paths.into_iter().for_each(|mut location| {
        trace!("Deleting {} ...", location.display());
        location.pop();
        let _ = remove_dir_all(location);
    });
}

pub fn boot() {
    use crate::logger;
    logger::init();

    checkdep("yt-dlp");
    checkdep("ffmpeg");
}

fn checkdep(dep: &str) {
    debug!("Checking dependency {} ...", dep);
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
