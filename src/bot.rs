use crate::{
    database::{self, delete_trash_from_chat},
    misc::{cleanup, sleep},
    pirate::{self, FileType},
};
use dptree::case;
use ngrok::prelude::*;
use std::error::Error;
use surrealdb::engine::local::Db;
use surrealdb::Surreal;
use teloxide::types::ChatKind;
use teloxide::{
    dispatching::UpdateHandler, prelude::*, update_listeners::webhooks, utils::command::BotCommands,
};
use tokio::task;

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

async fn bot_init() -> Result<Bot, Box<dyn Error>> {
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
    loop {
        match bot_init().await {
            Ok(bot) => {
                info!("Connection has been established.");
                let db = database::initialize().await;
                dispatcher(bot, db).await;
            }
            Err(reason) => {
                error!("{}", reason);
            }
        }
        warn!("Could not establish connection. Trying again after 30 seconds.");
        sleep(30);
    }
}

async fn dispatcher(bot: Bot, db: Surreal<Db>) {
    Dispatcher::builder(bot, handler().await)
        .enable_ctrlc_handler()
        .dependencies(dptree::deps![db])
        .default_handler(|upd| async move {
            warn!("Unhandled update: {:?}", upd);
        })
        .error_handler(LoggingErrorHandler::with_custom_text("Handler error."))
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

async fn start(bot: Bot, msg_from_user: Message, db: Surreal<Db>) -> HandlerResult {
    let chat_id = msg_from_user.chat.id;
    let msg_id = msg_from_user.id;
    let username = getuser(&msg_from_user);
    let command_descriptions = Command::descriptions().to_string();
    send_and_remember_msg(&bot, chat_id, &db, &command_descriptions).await?;
    database::intodb(chat_id, msg_id, &db).await?;
    info!("User @{} has /start'ed the bot.", username);
    Ok(())
}

async fn help(bot: Bot, msg_from_user: Message, db: Surreal<Db>) -> HandlerResult {
    let chat_id = msg_from_user.chat.id;
    let msg_id = msg_from_user.id;
    let username = getuser(&msg_from_user);
    let command_descriptions = Command::descriptions().to_string();
    send_and_remember_msg(&bot, chat_id, &db, &command_descriptions).await?;
    database::intodb(chat_id, msg_id, &db).await?;
    info!("User @{} asked for /help.", username);
    Ok(())
}

async fn mp3(link: String, bot: Bot, msg_from_user: Message, db: Surreal<Db>) -> HandlerResult {
    let filetype = FileType::Mp3;
    process_request(link, filetype, &bot, msg_from_user, &db).await?;
    Ok(())
}

async fn mp4(link: String, bot: Bot, msg_from_user: Message, db: Surreal<Db>) -> HandlerResult {
    let filetype = FileType::Mp4;
    process_request(link, filetype, &bot, msg_from_user, &db).await?;
    Ok(())
}

async fn voice(link: String, bot: Bot, msg_from_user: Message, db: Surreal<Db>) -> HandlerResult {
    let filetype = FileType::Voice;
    process_request(link, filetype, &bot, msg_from_user, &db).await?;
    Ok(())
}

async fn gif(link: String, bot: Bot, msg_from_user: Message, db: Surreal<Db>) -> HandlerResult {
    let filetype = FileType::Gif;
    process_request(link, filetype, &bot, msg_from_user, &db).await?;
    Ok(())
}

async fn clear(bot: Bot, msg_from_user: Message, db: Surreal<Db>) -> HandlerResult {
    let chat_id = msg_from_user.chat.id;
    let msg_id = msg_from_user.id;
    database::intodb(chat_id, msg_id, &db).await?;
    purge_trash_messages(chat_id, &db, &bot).await?;
    info!("User @{} has cleaned up the chat.", getuser(&msg_from_user));
    Ok(())
}

fn getuser(msg_from_user: &Message) -> String {
    let chatkind = &msg_from_user.chat.kind;
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
    chat_id: ChatId,
    db: &Surreal<Db>,
    bot: &Bot,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let ids = database::get_trash_message_ids(chat_id, db).await?;
    for id in ids.into_iter() {
        trace!("Deleting Message ID {} from Chat {} ...", id.0, chat_id.0);
        match bot.delete_message(chat_id, id).await {
            Err(e) => {
                error!("Can't delete a message: {}", e);
            }
            _ => {}
        }
    }
    delete_trash_from_chat(chat_id, db).await?;
    Ok(())
}

async fn process_request(link: String, filetype: FileType, bot: &Bot, msg_from_user: Message, db: &Surreal<Db>) -> HandlerResult {
    let chat_id = msg_from_user.chat.id;
    let msg_id = msg_from_user.id;
    let username = getuser(&msg_from_user);
    database::intodb(chat_id, msg_id, &db).await?;
    if link_is_valid(&link) {
        send_and_remember_msg(&bot, chat_id, db, "Please wait ...").await?;
        info!("User @{} asked for /{}.", &username, filetype.as_str());
        // The clone is moved into pirate module for error reporting.
        let downloads_result = match &filetype {
            FileType::Mp3 => task::spawn_blocking(move || pirate::mp3(link)).await,
            FileType::Mp4 => task::spawn_blocking(move || pirate::mp4(link)).await,
            FileType::Voice => task::spawn_blocking(move || pirate::ogg(link)).await,
            FileType::Gif => task::spawn_blocking(move || pirate::gif(link)).await,
        }?;

        match downloads_result {
            Err(error) => {
                warn!("{}", error);
                send_and_remember_msg(&bot, chat_id, &db, &error.to_string()).await?;
            }
            Ok(files) => {
                if files.botfiles.len() != 0 {
                    for file in files.botfiles.into_iter() {
                        trace!("Sending {} to @{} ...", filetype.as_str(), &username);
                        match &filetype {
                            FileType::Mp3 => {
                                bot.send_audio(msg_from_user.chat.id, file).await?;
                            }
                            FileType::Mp4 => {
                                bot.send_video(msg_from_user.chat.id, file).await?;
                            }
                            FileType::Voice => {
                                bot.send_voice(msg_from_user.chat.id, file).await?;
                            }
                            FileType::Gif => {
                                bot.send_animation(msg_from_user.chat.id, file).await?;
                            }
                        }
                    }
                    info!("Files have been delivered to @{}.", &username);
                    purge_trash_messages(chat_id, db, &bot).await?;
                    cleanup(files.paths);
                } else {
                    let error_text = "This is an error. For some reason I haven't downloaded any files.";
                    warn!("{}", error_text);
                    send_and_remember_msg(&bot, chat_id, db, error_text).await?;
                }
            }
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
        send_and_remember_msg(&bot, chat_id, db, &correct_usage).await?;
        debug!("Reminded user @{} of a correct /{} usage.", username, ftype);
    }
    Ok(())
}

pub async fn send_and_remember_msg(bot: &Bot, chat_id: ChatId, db: &Surreal<Db>, text: &str) -> HandlerResult {
    let message = bot.send_message(chat_id, text).await?;
    database::intodb(chat_id, message.id, &db).await?;
    Ok(())
}
