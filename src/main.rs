#![allow(unused)]
#[macro_use] extern crate log;
mod logger;
mod pirate;
mod bot;

#[tokio::main]
async fn main() {
    logger::init();
    info!("Starting up ...");
}