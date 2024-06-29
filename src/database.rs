use crate::CRATE_NAME;
use std::error::Error;
use surrealdb::{engine::local::Db, engine::local::File, Surreal};
use teloxide::types::{ChatId, MessageId};

pub async fn initialize() -> Surreal<Db> {
    info!("Initializing database ...");
    let db_result = Surreal::new::<File>("./surrealdb").await;
    match db_result {
        Ok(db) => {
            db.use_ns(CRATE_NAME).use_db(CRATE_NAME).await.unwrap();
            return db;
        }
        Err(e) => {
            error!("{}", e);
            std::process::exit(1);
        }
    }
}

pub async fn intodb(
    chatid: ChatId,
    msgid: MessageId,
    db: &Surreal<Db>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let chat: String = chatid.0.to_string();
    trace!(
        "Recording message ID {} from Chat {} into DB ...",
        msgid.0,
        &chat
    );
    let _: Vec<MessageId> = db.create(chat).content(msgid).await?;
    Ok(())
}

pub async fn get_trash_message_ids(
    chatid: ChatId,
    db: &Surreal<Db>,
) -> Result<Vec<MessageId>, Box<dyn Error + Send + Sync>> {
    let message_ids: Vec<MessageId> = db.select(chatid.0.to_string()).await?;
    Ok(message_ids)
}

pub async fn delete_trash_from_chat(
    chatid: ChatId,
    db: &Surreal<Db>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    trace!("Cleaning up the database for chat ID {} ...", chatid.0);
    let _: Vec<MessageId> = db.delete(chatid.to_string()).await?;
    Ok(())
}
