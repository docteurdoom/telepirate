use sled::{Db, Error};
use std::str::from_utf8 as bts;
use serde::Deserialize;
use teloxide::types::{ChatId, MessageId};

pub fn init() -> Db {
    debug!("Initializing database ...");
    let db = sled::open("database/").unwrap();
    return db;
}

fn serialize(x: Vec<i32>) -> String {
    let s = serde_json::to_string(&x).unwrap();
    return s;
}

fn deserialize(x: impl Into<String>) -> Vec<i32> {
    let v = serde_json::from_str(&x.into()).unwrap();
    return v;
}

pub fn intodb(chatid: ChatId, msgid: MessageId, db: &Db) -> Result<(), Error> {
    let chat: String = chatid.0.to_string();
    let mid = msgid.0;
    trace!("Recording message ID {} from Chat {} into DB.", mid, &chat);
    let values = db.get(&chat)?;
    match values {
        None => {
            let mut message_ids: Vec<i32> = Vec::new();
            message_ids.push(mid);
            let serialized = serialize(message_ids);
            db.insert(chat, &serialized[..]);
        }
        Some(entries) => {
            let decoded = bts(&entries).unwrap();
            let mut message_ids: Vec<i32> = deserialize(decoded);
            message_ids.push(mid);
            let serialized = serialize(message_ids);
            db.insert(chat, &serialized[..]);
        }
    }
    Ok(())
}

pub fn get_trash_message_ids(chatid: ChatId, db: &Db) -> Result<Vec<MessageId>, Error> {
    let chat: String = chatid.0.to_string();
    let mut message_ids = Vec::new();
    let raw_iv_data = &db.get(&chat)?.unwrap();
    let raw_data = bts(raw_iv_data).unwrap();
    let deserialized_raw_data = deserialize(raw_data);
    message_ids = deserialized_raw_data
        .into_iter()
        .map(|id| MessageId(id))
        .collect();
    Ok(message_ids)
}