use crossterm::{cursor, ExecutableCommand};
use std::fs::remove_dir_all;
use std::io::{stdout, Write};
use std::path::PathBuf;
use terminal_size::terminal_size;

pub fn r() {
    let size = terminal_size().unwrap().0 .0;
    let mut spaces = String::new();
    for _ in 0..size {
        spaces += " ";
    }
    print!("{}\r", spaces);
    stdout().execute(cursor::MoveUp(1));
    update();
}

pub fn update() {
    stdout().flush().unwrap();
}

pub fn cleanup(paths: Vec<PathBuf>) {
    trace!("Cleaning up the working directory ...");
    paths.into_iter().for_each(|mut location| {
        location.pop();
        remove_dir_all(location);
    });
}

pub fn boot() {
    use crate::logger;
    logger::init();

    ctrlc::set_handler(move || {
        r();
        info!("Stopping ...");
        std::process::exit(0);
    });
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
