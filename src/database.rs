use std::error::Error;
use surrealdb::{engine::local::Db, engine::local::File, Surreal};
use teloxide::types::{ChatId, MessageId};
use crate::CRATE_NAME;

pub async fn init(chat_id: ChatId) -> Result<Surreal<Db>, Box<dyn Error + Send + Sync>> {
    let chat_id_string = chat_id.0.to_string();
    info!("Initializing database for chat ID {} ...", &chat_id_string);
    let db = Surreal::new::<File>("./surrealdb").await?;
    db.use_ns(CRATE_NAME).use_db(CRATE_NAME).await?;
    Ok(db)
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
