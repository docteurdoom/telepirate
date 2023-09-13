#![allow(unused)]
#[macro_use] extern crate log;
pub const CRATE_NAME: &str = module_path!();
mod logger;
mod pirate;
mod bot;
mod misc;
mod database;

#[tokio::main]
async fn main() {
    misc::boot();
    bot::run().await;
}