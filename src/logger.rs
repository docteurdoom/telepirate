use fern::colors::{Color, ColoredLevelConfig};
use std::time::SystemTime;
use fern;

pub fn init() {
    let colors = ColoredLevelConfig::new()
    .info(Color::Green)
    .debug(Color::Magenta)
    .trace(Color::Blue);
    
    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[ {} {} ] {}",
                colors.color(record.level()),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Trace)
        .level_for("rustls", log::LevelFilter::Error)
        .level_for("tracing", log::LevelFilter::Error)
        .level_for("ngrok", log::LevelFilter::Error)
        .level_for("muxado", log::LevelFilter::Error)
        .level_for("tokio", log::LevelFilter::Error)
        .level_for("tokio_util", log::LevelFilter::Error)
        .level_for("mio", log::LevelFilter::Error)
        .level_for("hyper", log::LevelFilter::Error)
        .level_for("reqwest", log::LevelFilter::Error)
        .level_for("want", log::LevelFilter::Error)
        .chain(std::io::stdout())
        .chain(fern::log_file("debug.log").unwrap())
        .apply().unwrap();
}