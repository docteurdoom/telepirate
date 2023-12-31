use crate::{database, misc::cleanup, pirate, pirate::FileType};
use dptree::case;
use ngrok::prelude::*;
use sled::Db;
use std::error::Error;
use teloxide::types::ChatKind;
use teloxide::{
    dispatching::UpdateHandler, prelude::*, update_listeners::webhooks, utils::command::BotCommands,
};

type HandlerResult = Result<(), Box<dyn Error + Send + Sync>>;

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
enum Command {
    #[command(description = "start the bot.")]
    Start,
    #[command(description = "display this help.")]
    Help,
    #[command(description = "get audio.")]
    Mp3(String),
    #[command(description = "get video.")]
    Mp4(String),
    #[command(description = "get audio as a voice message.")]
    Voice(String),
    #[command(description = "get video as a animated GIF.")]
    Gif(String),
    #[command(description = "delete trash messages.")]
    C,
}

async fn init() -> Result<Bot, Box<dyn Error>> {
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
    let url = listener.url().parse()?;
    webhooks::axum(bot.clone(), webhooks::Options::new(addr, url)).await?;
    Ok(bot)
}

pub async fn run() {
    match init().await {
        Ok(bot) => {
            info!("Connection has been established");
            dispatcher(bot).await;
        }
        Err(reason) => {
            dbg!(reason);
        }
    }
}

async fn dispatcher(bot: Bot) {
    Dispatcher::builder(bot, handler().await)
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

async fn handler() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    let command_handler = teloxide::filter_command::<Command, _>()
        .branch(case![Command::Start].endpoint(start))
        .branch(case![Command::Help].endpoint(help))
        .branch(case![Command::Mp3(link)].endpoint(mp3))
        .branch(case![Command::Mp4(link)].endpoint(mp4))
        .branch(case![Command::Voice(link)].endpoint(voice))
        .branch(case![Command::Gif(link)].endpoint(gif))
        .branch(case![Command::C].endpoint(clear));

    let message_handler = Update::filter_message().branch(command_handler);

    return message_handler;
}

async fn start(bot: Bot, msg: Message) -> HandlerResult {
    let chat_id = msg.chat.id;
    let db = database::init(chat_id)?;
    let message: Message = bot
        .send_message(chat_id, Command::descriptions().to_string())
        .await?;
    database::intodb(msg.chat.id, msg.id, &db)?;
    database::intodb(msg.chat.id, message.id, &db)?;
    info!("User @{} has /start'ed the bot", getuser(&message));
    Ok(())
}

async fn help(bot: Bot, msg: Message) -> HandlerResult {
    let chat_id = msg.chat.id;
    let db = database::init(chat_id)?;
    let message = bot
        .send_message(chat_id, Command::descriptions().to_string())
        .await?;
    database::intodb(chat_id, msg.id, &db)?;
    database::intodb(chat_id, message.id, &db)?;
    info!("User @{} asked for /help", getuser(&message));
    drop(db);
    Ok(())
}

async fn mp3(link: String, bot: Bot, msg: Message) -> HandlerResult {
    let chat_id = msg.chat.id;
    let db = database::init(chat_id)?;
    let filetype = FileType::Mp3;
    process_request(link, filetype, bot, msg, &db).await?;
    drop(db);
    Ok(())
}

async fn mp4(link: String, bot: Bot, msg: Message) -> HandlerResult {
    let chat_id = msg.chat.id;
    let db = database::init(chat_id)?;
    let filetype = FileType::Mp4;
    process_request(link, filetype, bot, msg, &db).await?;
    drop(db);
    Ok(())
}

async fn voice(link: String, bot: Bot, msg: Message) -> HandlerResult {
    let chat_id = msg.chat.id;
    let db = database::init(chat_id)?;
    let filetype = FileType::Voice;
    process_request(link, filetype, bot, msg, &db).await?;
    drop(db);
    Ok(())
}

async fn gif(link: String, bot: Bot, msg: Message) -> HandlerResult {
    let chat_id = msg.chat.id;
    let db = database::init(chat_id)?;
    let filetype = FileType::Gif;
    process_request(link, filetype, bot, msg, &db).await?;
    drop(db);
    Ok(())
}

async fn clear(bot: Bot, msg: Message) -> HandlerResult {
    let chat_id = msg.chat.id;
    let db = database::init(chat_id)?;
    database::intodb(chat_id, msg.id, &db)?;
    purge_trash_messages(chat_id, &db, &bot).await?;
    info!("User @{} has /c'leaned up the chat", getuser(&msg));
    drop(db);
    Ok(())
}

fn getuser(msg: &Message) -> String {
    let chatkind = &msg.chat.kind;
    let mut username: String = String::new();
    match chatkind {
        ChatKind::Private(chat) => {
            match &chat.username {
                None => username = "noname".to_string(),
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

async fn purge_trash_messages(
    chatid: ChatId,
    db: &Db,
    bot: &Bot,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let ids = database::get_trash_message_ids(chatid, db)?;
    for id in ids.into_iter() {
        trace!("Deleting Message ID {} from Chat {} ...", id.0, chatid.0);
        match bot.delete_message(chatid, id).await {
            Err(e) => {
                error!("Can't delete a message: {}", e);
            }
            _ => {}
        }
    }
    db.remove(chatid.to_string())?;
    Ok(())
}

async fn process_request(
    link: String,
    filetype: FileType,
    bot: Bot,
    msg: Message,
    db: &Db,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    use tokio::task;

    if link_is_valid(&link) {
        let message = bot.send_message(msg.chat.id, "Please wait ...").await?;
        let username = getuser(&message);
        info!("User @{} asked for /{}", &username, filetype.as_str());
        let files = match &filetype {
            FileType::Mp3 => task::spawn_blocking(move || pirate::mp3(link)).await?,
            FileType::Mp4 => task::spawn_blocking(move || pirate::mp4(link)).await?,
            FileType::Voice => task::spawn_blocking(move || pirate::ogg(link)).await?,
            FileType::Gif => task::spawn_blocking(move || pirate::gif(link)).await?,
        }?;

        if files.botfiles.len() != 0 {
            for file in files.botfiles.into_iter() {
                trace!("Sending the {} to @{} ...", filetype.as_str(), &username);
                match &filetype {
                    FileType::Mp3 => {
                        bot.send_audio(msg.chat.id, file).await?;
                    }
                    FileType::Mp4 => {
                        bot.send_video(msg.chat.id, file).await?;
                    }
                    FileType::Voice => {
                        bot.send_voice(msg.chat.id, file).await?;
                    }
                    FileType::Gif => {
                        bot.send_animation(msg.chat.id, file).await?;
                    }
                }
            }
            info!("Files have been delivered to @{}", &username);
            database::intodb(msg.chat.id, msg.id, db)?;
            database::intodb(msg.chat.id, message.id, db)?;
            purge_trash_messages(msg.chat.id, db, &bot).await?;
            cleanup(files.paths);
        } else {
            let error_msg = bot.send_message(msg.chat.id, "Error. The file is too large or the link contains a private resource. Not able to download.").await?;
            database::intodb(msg.chat.id, msg.id, db)?;
            database::intodb(msg.chat.id, message.id, db)?;
            database::intodb(msg.chat.id, error_msg.id, db)?;
        }
    } else {
        let ftype = filetype.as_str();
        let correct_usage = match &filetype {
            FileType::Voice => {
                format!("Correct usage:\n\n/voice https://valid_audio_url")
            }
            FileType::Gif => {
                format!("Correct usage:\n\n/{} https://valid_video_url", ftype)
            }
            _ => {
                format!("Correct usage:\n\n/{} https://valid_{}_url", ftype, ftype)
            }
        };
        let message = bot.send_message(msg.chat.id, &correct_usage).await?;
        database::intodb(msg.chat.id, msg.id, db)?;
        database::intodb(msg.chat.id, message.id, db)?;
        debug!(
            "Reminded user @{} of a correct /{} usage",
            getuser(&message),
            ftype
        );
    }
    Ok(())
}
