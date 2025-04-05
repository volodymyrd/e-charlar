use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::time::SystemTime;
use uuid::Uuid;

/// User public address.
pub type Address = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub uuid: Uuid,
    pub address: Address,
    pub created: SystemTime,
}

impl PartialEq for User {
    fn eq(&self, other: &Self) -> bool {
        self.uuid == other.uuid
    }
}

impl Eq for User {}

impl Hash for User {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.uuid.hash(state);
    }
}

impl User {
    pub fn new(address: Address) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            address,
            created: SystemTime::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    pub uuid: Uuid,
    pub name: String,
    pub created: SystemTime,
    pub owners: HashSet<Uuid>,
    pub members: HashSet<Uuid>,
}

impl Room {
    pub fn new(name: &str, by: &User) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            name: name.to_string(),
            created: SystemTime::now(),
            owners: HashSet::from([by.uuid]),
            members: HashSet::from([by.uuid]),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    Text,
    File,
    Audio,
    Video,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Content {
    Text(String),
    File(String),
    Audio(String),
    Video(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub uuid: Uuid,
    pub message_type: MessageType,
    pub created: SystemTime,
    pub owner: Uuid,
    pub content: Content,
}

impl Message {
    pub fn new_text(text: &str, sender: &User) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            message_type: MessageType::Text,
            created: SystemTime::now(),
            owner: sender.uuid,
            content: Content::Text(text.to_string()),
        }
    }
}
