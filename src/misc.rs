use terminal_size::{terminal_size};
use crossterm::{ExecutableCommand, cursor};
use std::io::{Write, stdout};
use std::fs::remove_dir_all;
use std::path::PathBuf;

pub fn r() {
    let size = terminal_size().unwrap().0.0;
    let mut spaces = String::new();
    for _ in 0..size { spaces += " "; }
    print!("{}\r", spaces);
    stdout().execute(cursor::MoveUp(1));
    update();
}

pub fn update() {
    stdout().flush().unwrap();
}

pub fn cleanup(paths: Vec<PathBuf>) {
    trace!("Cleaning up the working directory ...");
    paths.into_iter().for_each(
        |mut location| {
            location.pop();
            remove_dir_all(location);
        }
    );
}

pub fn boot() {
    use crate::logger;
    logger::init();

    ctrlc::set_handler(move || {
        r();
        info!("Stopping ...");
        std::process::exit(0);
    });

    debug!("Checking dependencies ...");
    match std::process::Command::new("yt-dlp").arg("--version").output() {
        Err(e) => {
            if let std::io::ErrorKind::NotFound = e.kind() {
                error!("yt-dlp is not found. Please install yt-dlp first.");
                std::process::exit(1);
            }
        }
        _ => {}
    }
}