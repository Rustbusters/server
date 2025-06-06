use chrono::Utc;
use log::{info, warn};
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use wg_2024::network::NodeId;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DbMessage {
    id: String, // UUID as the primary key
    src_id: NodeId,
    dest_id: NodeId,
    message: String,
    timestamp: i64, // Unix timestamp
}

impl DbMessage {
    pub fn new(src_id: NodeId, dest_id: NodeId, message: String) -> Self {
        let id = Uuid::new_v4().to_string(); // Generate a new UUID
        let timestamp = Utc::now().timestamp(); // Get current Unix timestamp

        DbMessage {
            id,
            src_id,
            dest_id,
            message,
            timestamp,
        }
    }
}

pub struct DbManager {
    id: NodeId,
    name: String,
    conn: Connection,
}

impl DbManager {
    pub fn new(server_id: NodeId, db_name: String) -> Result<DbManager> {
        info!("[SERVER-{}] Creating database: {}", server_id, db_name);
        let conn = Connection::open(&db_name)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                src_id INTEGER NOT NULL,
                dest_id INTEGER NOT NULL,
                message TEXT NOT NULL,
                timestamp INTEGER NOT NULL
            )",
            [],
        )?;

        Ok(DbManager {
            id: server_id,
            name: db_name,
            conn,
        })
    }

    /// Inserts a message into the database
    pub fn insert(
        &self,
        src_id: NodeId,
        dest_id: NodeId,
        message: String,
    ) -> Result<DbMessage, rusqlite::Error> {
        let db_message = DbMessage::new(src_id, dest_id, message);
        self.conn.execute(
            "INSERT INTO messages (id, src_id, dest_id, message, timestamp) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![db_message.id, db_message.src_id, db_message.dest_id, db_message.message, db_message.timestamp],
        )?;
        Ok(db_message)
    }

    /// Retrieves a message by its ID
    pub fn get(&self, id: Uuid) -> Result<Option<DbMessage>> {
        let mut stmt = self
            .conn
            .prepare("SELECT src_id, dest_id, message, timestamp FROM messages WHERE id = ?1")?;
        let mut rows = stmt.query(params![id.to_string()])?;

        if let Some(row) = rows.next()? {
            let src_id: NodeId = row.get(0)?;
            let dest_id: NodeId = row.get(1)?;
            let message: String = row.get(2)?;
            let timestamp: i64 = row.get(3)?;

            Ok(Some(DbMessage {
                id: id.to_string(),
                src_id,
                dest_id,
                message,
                timestamp,
            }))
        } else {
            Ok(None)
        }
    }

    /// Retrieves all messages from the database
    pub fn get_all(&self) -> Result<Vec<DbMessage>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, src_id, dest_id, message, timestamp FROM messages")?;
        let rows = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            let src_id: NodeId = row.get(1)?;
            let dest_id: NodeId = row.get(2)?;
            let message: String = row.get(3)?;
            let timestamp: i64 = row.get(4)?;
            Ok(DbMessage {
                id,
                src_id,
                dest_id,
                message,
                timestamp,
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
