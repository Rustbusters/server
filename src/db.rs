use common_utils::{ClientToServerMessage, HostMessage, MessageBody};
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use wg_2024::network::NodeId;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DbMessage {
    id: String, // use the UUID as the key
    src_id: NodeId,
    dest_id: NodeId,
    message: String,
}

impl DbMessage {
    pub fn new(src_id: NodeId, dest_id: NodeId, message: String) -> Self {
        let id = Uuid::new_v4().to_string(); // Generate a new UUID
        DbMessage {
            id,
            src_id,
            dest_id,
            message,
        }
    }
}

pub struct DbManager {
    name: String,
    conn: Connection,
}

impl DbManager {
    pub fn new(db_name: String) -> Result<DbManager> {
        println!("[SERVER] Creating database: {}", db_name);
        let conn = Connection::open(&db_name)?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                src_id INTEGER NOT NULL,
                dest_id INTEGER NOT NULL,
                message TEXT NOT NULL
            )",
            [],
        )?;
        Ok(DbManager {
            name: db_name,
            conn,
        })
    }

    /// Inserts a message into the database
    pub fn insert(&self, message: DbMessage) -> Result<()> {
        self.conn.execute(
            "INSERT INTO messages (id, src_id, dest_id, message) VALUES (?1, ?2, ?3, ?4)",
            params![message.id, message.src_id, message.dest_id, message.message],
        )?;
        Ok(())
    }

    /// Retrieves a message by its ID
    pub fn get(&self, id: Uuid) -> Result<Option<DbMessage>> {
        let mut stmt = self
            .conn
            .prepare("SELECT src_id, dest_id, message FROM messages WHERE id = ?1")?;
        let mut rows = stmt.query(params![id.to_string()])?;
        if let Some(row) = rows.next()? {
            let src_id: NodeId = row.get(0)?;
            let dest_id: NodeId = row.get(1)?;
            let message: String = row.get(2)?;
            Ok(Some(DbMessage {
                id: id.to_string(),
                src_id,
                dest_id,
                message,
            }))
        } else {
            Ok(None)
        }
    }

    /// Retrieves all messages from the database
    pub fn get_all(&self) -> Result<Vec<DbMessage>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, src_id, dest_id, message FROM messages")?;
        let rows = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            let src_id: NodeId = row.get(1)?;
            let dest_id: NodeId = row.get(2)?;
            let message: String = row.get(3)?;
            Ok(DbMessage {
                id,
                src_id,
                dest_id,
                message,
            })
        })?;

        let mut messages = Vec::new();
        for message in rows {
            messages.push(message?);
        }
        Ok(messages)
    }

    /// Removes a message by its ID
    pub fn remove(&self, id: Uuid) -> Result<()> {
        self.conn.execute(
            "DELETE FROM messages WHERE id = ?1",
            params![id.to_string()],
        )?;
        Ok(())
    }
}
