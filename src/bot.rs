use teloxide::{prelude::*, update_listeners::webhooks, utils::command::BotCommands};
use ngrok::prelude::*;
use teloxide::Bot;
use crate::{pirate, misc, database};
use std::error::Error;
use teloxide::types::ChatKind;
use sled::Db;
use crate::misc::cleanup;
use crate::pirate::{FileType, Subject};

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "These commands are supported:")]
enum Command {
    #[command(description = "start the bot.")]
    Start,
    #[command(description = "display this help.")]
    Help,
    #[command(description = "download audio.")]
    Mp3(String),
    #[command(description = "download video.")]
    Mp4(String),
    #[command(description = "delete trash messages.")]
    C,
}

pub async fn init() -> Result<Bot, Box<dyn Error>> {
    ctrlc::set_handler(move || {
        misc::r();
        info!("Stopping ...");
        std::process::exit(0);
    });

    debug!("Building ngrok tunnel ...");
    let listener = ngrok::Session::builder()
       .authtoken_from_env()
       .connect()
       .await?
       .http_endpoint()
       .listen()
       .await?;

    debug!("Initializing the bot ...");
    let bot = Bot::from_env();

    debug!("Setting up the webhook ...");
    let addr = ([127, 0, 0, 1], 8443).into();
    let url = listener.url().parse().unwrap();
    let listener = webhooks::axum(bot.clone(), webhooks::Options::new(addr, url))
        .await?;
    Ok(bot)
}

fn getuser(msg: &Message) -> String {
    let chatkind = &msg.chat.kind;
    let mut username: String = String::new();
    match chatkind {
        ChatKind::Private(chat) => {
            match &chat.username {
                None => {
                    username = "noname".to_string()
                }
                Some(name) => {
                    username = name.clone();
                }
            };
        }
        _ => {}
    }
    return username;
}

fn link_is_valid(link: &str) -> bool {
    link.len() != 0
}

pub async fn purge_trash_messages(chatid: ChatId, db: &Db, bot: &Bot) -> ResponseResult<()> {
    trace!("Deleting garbage messages ...");
    let ids = database::get_trash_message_ids(chatid, db).unwrap();
    for id in ids.into_iter() {
        bot.delete_message(chatid, id).await?;
    }
    db.remove(chatid.to_string());
    Ok(())
}

async fn process_request(link: String, filetype: FileType, bot: Bot, msg: Message, db: &Db) -> ResponseResult<()> {
    if link_is_valid(&link) {
        let message = bot.send_message(msg.chat.id, "Please wait ...").await?;
        debug!("User @{} asked for /{}", getuser(&message), filetype.as_str());
        let mut files = Subject::default();
        let mut correct_usage = String::new();
        match &filetype {
            FileType::Mp3 => { files = pirate::mp3(&link[..]).await; }
            FileType::Mp4 => { files = pirate::mp4(&link[..]).await; }
            _ => {}
        }

        if files.botfiles.len() != 0 {
            for file in files.botfiles.into_iter() {
                info!("Sending the {} ...", filetype.as_str());
                bot.send_audio(msg.chat.id, file).await?;
            }
            database::intodb(msg.chat.id, msg.id, db);
            database::intodb(msg.chat.id, message.id, db);
            purge_trash_messages(msg.chat.id, db, &bot).await?;
            trace!("Cleaning up files");
            cleanup(files.paths);
        } else {
            let error_msg = bot.send_message(msg.chat.id, "Error. Can't download.").await?;
            database::intodb(msg.chat.id, msg.id, db);
            database::intodb(msg.chat.id, message.id, db);
            database::intodb(msg.chat.id, error_msg.id, db);
        }
    } else {
        let ftype = filetype.as_str();
        let correct_usage = format!("Correct usage:\n\n/{} https://valid_{}_url", ftype, ftype);
        let message = bot.send_message(msg.chat.id, &correct_usage).await?;
        database::intodb(msg.chat.id, msg.id, db);
        database::intodb(msg.chat.id, message.id, db);
        debug!("Reminding user @{} of a correct /{} usage", getuser(&message), ftype);
    }
    Ok(())
}

async fn answer(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    let db = &database::init();
    match cmd {
        Command::Start => {
            let message: Message = bot.send_message(msg.chat.id, Command::descriptions().to_string()).await?;
            database::intodb(msg.chat.id, msg.id, db);
            database::intodb(msg.chat.id, message.id, db);
            debug!("User @{} has /start'ed the bot", getuser(&message));
        }
        Command::Help => {
            let message = bot.send_message(msg.chat.id, Command::descriptions().to_string()).await?;
            database::intodb(msg.chat.id, msg.id, db);
            database::intodb(msg.chat.id, message.id, db);
            debug!("User @{} asked for /help", getuser(&message));
        }
        Command::Mp3(link) => {
            let filetype = FileType::Mp3;
            process_request(link, filetype, bot, msg, db).await;
        }
        Command::Mp4(link) => {
            let filetype = FileType::Mp4;
            process_request(link, filetype, bot, msg, db).await;
        }
        Command::C => {
            database::intodb(msg.chat.id, msg.id, db);
            purge_trash_messages(msg.chat.id, db, &bot).await?;
        }
    };

    Ok(())
}

pub async fn run() {
    match init().await {
        Ok(bot) => {
            info!("Connection has been established.");
            Command::repl(bot, answer).await;
        }
        Err(reason) => {
            dbg!(reason);
        }
    }
}