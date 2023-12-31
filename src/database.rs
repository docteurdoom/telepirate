use sled::Db;
use std::error::Error;
use std::str::from_utf8 as bts;
use teloxide::types::{ChatId, MessageId};

pub fn init(chat: ChatId) -> Result<Db, Box<dyn Error + Send + Sync>> {
    let id = chat.0;
    let path = format!("database/{}", id);
    debug!("Initializing database for chat ID {} ...", id);
    let db = sled::open(path)?;
    Ok(db)
}

fn serialize(x: Vec<i32>) -> String {
    let s = serde_json::to_string(&x).unwrap();
    return s;
}

fn deserialize(x: impl Into<String>) -> Vec<i32> {
    let v = serde_json::from_str(&x.into()).unwrap();
    return v;
}

pub fn intodb(
    chatid: ChatId,
    msgid: MessageId,
    db: &Db,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let chat: String = chatid.0.to_string();
    let mid = msgid.0;
    trace!(
        "Recording message ID {} from Chat {} into DB ...",
        mid,
        &chat
    );
    let values = db.get(&chat)?;
    match values {
        None => {
            let mut message_ids: Vec<i32> = Vec::new();
            message_ids.push(mid);
            let serialized = serialize(message_ids);
            db.insert(chat, &serialized[..])?;
        }
        Some(entries) => {
            let decoded = bts(&entries)?;
            let mut message_ids: Vec<i32> = deserialize(decoded);
            message_ids.push(mid);
            let serialized = serialize(message_ids);
            db.insert(chat, &serialized[..])?;
        }
    }
    Ok(())
}

pub fn get_trash_message_ids(
    chatid: ChatId,
    db: &Db,
) -> Result<Vec<MessageId>, Box<dyn Error + Send + Sync>> {
    let chat: String = chatid.0.to_string();
    let raw_iv_data = &db.get(&chat)?.unwrap();
    let raw_data = bts(raw_iv_data)?;
    let deserialized_raw_data = deserialize(raw_data);
    let message_ids = deserialized_raw_data
        .into_iter()
        .map(|id| MessageId(id))
        .collect();
    Ok(message_ids)
}
