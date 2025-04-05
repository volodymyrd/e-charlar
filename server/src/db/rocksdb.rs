use crate::db::db::{ConfigName, ConfigValue, DbConnection};
use crate::db::Db;
use anyhow::anyhow;
use bincode::{deserialize, serialize};
use rocksdb::{ColumnFamily, ColumnFamilyDescriptor, IteratorMode, Options, DB};
use shared::types::{Message, Room, User};
use std::collections::HashMap;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

enum Column {
    Users,
    Rooms,
    Messages,
}

impl Column {
    fn col_name(col: Column) -> &'static str {
        match col {
            Column::Users => "users",
            Column::Rooms => "rooms",
            Column::Messages => "messages",
        }
    }

    fn iter() -> impl Iterator<Item = Column> {
        [Column::Users, Column::Rooms, Column::Messages].into_iter()
    }
}

struct RocksDb {
    db: DB,
}

impl RocksDb {
    fn new(path: &Path) -> anyhow::Result<Self> {
        let mut db_opts = Options::default();
        db_opts.create_if_missing(true);
        db_opts.create_missing_column_families(true);

        Ok(Self {
            db: DB::open_cf_descriptors(
                &db_opts,
                path,
                Column::iter()
                    .map(|c| ColumnFamilyDescriptor::new(Column::col_name(c), Options::default()))
                    .collect::<Vec<_>>(),
            )?,
        })
    }

    fn column(&self, column: Column) -> &ColumnFamily {
        match column {
            Column::Users => self.db.cf_handle(Column::col_name(Column::Users)).unwrap(),
            Column::Rooms => self.db.cf_handle(Column::col_name(Column::Rooms)).unwrap(),
            Column::Messages => self
                .db
                .cf_handle(Column::col_name(Column::Messages))
                .unwrap(),
        }
    }
}

impl Db for RocksDb {
    fn open(config: &HashMap<ConfigName, ConfigValue>) -> anyhow::Result<Box<dyn DbConnection>> {
        if let Some(ConfigValue::Path(path)) = config.get(&ConfigName::Path) {
            match RocksDb::new(path.as_path()) {
                Ok(db) => Ok(Box::new(db)),
                Err(e) => Err(anyhow!("Failed to open RocksDB: {}", e)),
            }
        } else {
            Err(anyhow!("Path to DB not setup"))
        }
    }
}

impl DbConnection for RocksDb {
    fn find_user(&self, user_uuid: &Uuid) -> anyhow::Result<User> {
        Ok(deserialize(
            &self
                .db
                .get_cf(self.column(Column::Users), user_uuid)?
                .unwrap(),
        )?)
    }

    fn save_user(&self, user: &User) -> anyhow::Result<()> {
        self.db
            .put_cf(&self.column(Column::Users), user.uuid, serialize(user)?)?;
        Ok(())
    }

    fn find_room(&self, room_uuid: &Uuid) -> anyhow::Result<Room> {
        Ok(deserialize(
            &self
                .db
                .get_cf(self.column(Column::Rooms), room_uuid)?
                .unwrap(),
        )?)
    }

    fn save_room(&self, room: &Room) -> anyhow::Result<()> {
        self.db
            .put_cf(&self.column(Column::Rooms), room.uuid, serialize(room)?)?;
        Ok(())
    }

    fn save_message(&self, room: &Room, message: &Message) -> anyhow::Result<()> {
        let reverse_ts = u128::MAX - message.created.duration_since(UNIX_EPOCH)?.as_millis();
        self.db.put_cf(
            &self.column(Column::Messages),
            format!("{}_{reverse_ts}_{}", room.uuid, message.uuid),
            serialize(message)?,
        )?;
        Ok(())
    }

    fn find_messages(
        &self,
        room_uuid: &Uuid,
        limit: usize,
        at: Option<SystemTime>,
    ) -> anyhow::Result<(Vec<Message>, Option<SystemTime>)> {
        let iter = if let Some(t) = at {
            let reverse_ts = u128::MAX - t.duration_since(UNIX_EPOCH)?.as_millis();
            let start_key = format!("{}_{reverse_ts}", room_uuid);
            self.db.iterator_cf(
                self.column(Column::Messages),
                IteratorMode::From(start_key.as_bytes(), rocksdb::Direction::Forward),
            )
        } else {
            self.db
                .iterator_cf(self.column(Column::Messages), IteratorMode::Start)
        };

        let mut count = 0;
        let mut messages = Vec::with_capacity(limit);
        let mut next = None;
        for item in iter {
            let (_, v) = item?;
            let message: Message = deserialize(&v)?;

            count += 1;
            if count > limit {
                next = Some(message.created);
                break;
            }
            messages.push(message);
        }
        Ok((messages, next))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::types::Content;
    use std::thread;
    use std::time::Duration;
    use tempfile::TempDir;

    #[test]
    fn test_store_and_retrieve_user() {
        let user1 = User::new("user1".to_string());
        let user_uuid = user1.uuid;
        let created = user1.created;

        let db = open_db();
        db.save_user(&user1).expect("User should be saved");

        let u1 = db.find_user(&user_uuid).unwrap();

        assert_eq!(u1.uuid, user_uuid);
        assert_eq!(u1.created, created);
    }

    #[test]
    fn test_store_and_retrieve_room() {
        let user1 = User::new("user1".to_string());
        let room_name = "Room1";
        let room1 = Room::new(room_name, &user1);
        let room_uuid = room1.uuid;
        let created = room1.created;

        let db = open_db();
        db.save_room(&room1).expect("Room should be saved");

        let r1 = db.find_room(&room_uuid).unwrap();

        assert_eq!(r1.uuid, room_uuid);
        assert_eq!(r1.created, created);
        assert_eq!(r1.name, room_name);
        let member_uuid = r1.members.iter().nth(0).unwrap();
        assert_eq!(*member_uuid, user1.uuid);
        let owner_uuid = r1.owners.iter().nth(0).unwrap();
        assert_eq!(*owner_uuid, user1.uuid);
    }

    #[test]
    fn test_store_and_retrieve_message() {
        let db = open_db();

        let user1 = User::new("user1".to_string());
        let user2 = User::new("user2".to_string());
        let chat1 = &Room::new("chat1", &user1);
        // Generate 20 messages - 10 from user1 and 10 from user 2.
        for i in 0..10 {
            db.save_message(
                &chat1,
                &Message::new_text(&format!("user1: Message {i}"), &user1),
            )
            .expect("Message should be saved");

            thread::sleep(Duration::from_millis(5));

            db.save_message(
                &chat1,
                &Message::new_text(&format!("user2: Message {i}"), &user2),
            )
            .expect("Message should be saved");

            thread::sleep(Duration::from_millis(5));
        }

        let mut next_token = None;
        let mut message_id = 9;
        let mut count_messages = 0;
        let page_size = 12;
        loop {
            let messages = db
                .find_messages(&chat1.uuid, page_size, next_token)
                .expect(&format!("Return {} messages", page_size));
            next_token = messages.1;
            let messages = messages.0;
            if count_messages == 0 {
                assert_eq!(messages.len(), page_size);
            } else {
                assert_eq!(messages.len(), 20 - page_size);
            }

            let mut user_id = 1;
            for m in &messages {
                count_messages += 1;
                if user_id == 1 {
                    user_id = 2;
                } else {
                    user_id = 1;
                }
                let text = match &m.content {
                    Content::Text(text) => text,
                    _ => "undefined",
                };
                assert_eq!(text, format!("user{}: Message {}", user_id, message_id));
                if user_id == 1 {
                    message_id -= 1;
                }
            }
            if next_token.is_none() {
                break;
            }
        }
        assert_eq!(count_messages, 20);
    }

    fn open_db() -> Box<dyn DbConnection> {
        let temp_dir = TempDir::new().unwrap();
        let config = HashMap::from([(ConfigName::Path, ConfigValue::Path(temp_dir.into_path()))]);
        RocksDb::open(&config).expect("Db should be opened")
    }
}
