use teloxide::{prelude::*, update_listeners::webhooks, utils::command::BotCommands};
use ngrok::prelude::*;
use teloxide::Bot;
use std::error::Error;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "These commands are supported:")]
enum Command {
    #[command(description = "Display help.")]
    Help,
    //#[command(description = "Download an mp3 file from the provided link.")]
    //Mp3(String),
}

pub async fn init() -> Result<Bot, Box<dyn Error>> {
    info!("Building ngrok tunnel ...");
    let listener = ngrok::Session::builder()
       .authtoken_from_env()
       .connect()
       .await?
       .http_endpoint()
       .listen()
       .await?;

    info!("Initializing the bot ...");
    let bot = Bot::from_env();

    info!("Setting up the webhook ...");
    let addr = ([127, 0, 0, 1], 8443).into();
    let url = listener.url().parse().unwrap();
    let listener = webhooks::axum(bot.clone(), webhooks::Options::new(addr, url))
        .await?;
    Ok(bot)
}

async fn answer(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    match cmd {
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string()).await?;
        }
        //Command::Mp3(link) => {}
    };

    Ok(())
}

pub async fn run(bot: Bot) {
    match init().await {
        Ok(bot) => {
            Command::repl(bot, answer).await;
        }
        Err(reason) => {
            dbg!(reason);
        }
    }
}