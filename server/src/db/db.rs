use shared::types::{Message, Room, User};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum ConfigName {
    Path,
    // Host,
    // Port,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ConfigValue {
    Path(PathBuf),
    // Host(String),
    // Port(u16),
}

pub(crate) trait Db {
    fn open(config: &HashMap<ConfigName, ConfigValue>) -> anyhow::Result<Box<dyn DbConnection>>;
}

pub(crate) trait DbConnection {
    //fn close(&self) -> anyhow::Result<()>;
    fn find_user(&self, user_uuid: &Uuid) -> anyhow::Result<User>;
    fn save_user(&self, user: &User) -> anyhow::Result<()>;
    fn find_room(&self, room_uuid: &Uuid) -> anyhow::Result<Room>;
    fn save_room(&self, room: &Room) -> anyhow::Result<()>;
    fn save_message(&self, room: &Room, message: &Message) -> anyhow::Result<()>;
    fn find_messages(
        &self,
        room_uuid: &Uuid,
        limit: usize,
        at: Option<SystemTime>,
    ) -> anyhow::Result<(Vec<Message>, Option<SystemTime>)>;
}
