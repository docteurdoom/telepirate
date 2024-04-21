#[macro_use]
extern crate log;
pub const CRATE_NAME: &str = module_path!();
mod bot;
mod database;
mod logger;
mod misc;
mod pirate;

#[tokio::main]
async fn main() {
    misc::boot();
    bot::run().await;
}
