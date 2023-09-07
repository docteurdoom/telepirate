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
    trace!("Cleaning up the files ...");
    paths.into_iter().for_each(
        |location| {
            remove_dir_all(location);
        }
    );
}