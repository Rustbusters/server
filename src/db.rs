use common_utils::{ClientToServerMessage, HostMessage, MessageBody};
use wg_2024::network::NodeId;

use serde::{Deserialize, Serialize};
use sled::{Db, IVec};

use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DbMessage {
    id: String, // use the UUID as the key
    message: HostMessage,
}

impl DbMessage {
    pub fn new(message: HostMessage) -> Self {
        let id = Uuid::new_v4().to_string(); // Generate a new UUID
        DbMessage { id, message }
    }
}

pub struct DbManager {
    db: Db,
}

impl DbManager {
    pub fn new(db_name: String) -> Option<DbManager> {
        if let Ok(db) = sled::open(db_name) {
            Some(DbManager { db })
        } else {
            None
        }
    }

    /// Inserts a message into the database
    pub fn insert(&self, message: DbMessage) -> sled::Result<()> {
        let key = message.id.to_string(); // Use the UUID as the key
        let value = serde_json::to_vec(&message).unwrap(); // Serialize the message to JSON
        self.db.insert(key.as_bytes(), value)?; // Insert into the database
        Ok(())
    }

    /// Retrieves a message by its ID
    pub fn get(&self, id: Uuid) -> sled::Result<Option<DbMessage>> {
        let key = id.to_string(); // Use the UUID as the key
        if let Some(value) = self.db.get(key.as_bytes())? {
            let message: DbMessage = serde_json::from_slice(&value).unwrap(); // Deserialize the message from JSON
            Ok(Some(message))
        } else {
            Ok(None) // Return None if the key doesn't exist
        }
    }

    /// Retrieves all messages from the database
    pub fn get_all(&self) -> sled::Result<Vec<DbMessage>> {
        let mut messages = Vec::new();
        for result in self.db.iter() {
            let (_, value) = result?;
            let db_message: DbMessage = serde_json::from_slice(&value).unwrap();

            messages.push(db_message);
        }
        Ok(messages)
    }

    /// Retrieves all messages from a specific sender
    // pub fn get_by_sender(&self, sender_id: NodeId) -> sled::Result<Vec<DbMessage>> {
    //     let mut messages = Vec::new();
    //     for result in self.db.iter() {
    //         let (_, value) = result?;
    //         let db_message: DbMessage = serde_json::from_slice(&value).unwrap();
    //         match db_message.message {
    //             HostMessage::FromClient(client_to_server) => match client_to_server {
    //                 ClientToServerMessage::SendPrivateMessage {
    //                     recipient_id,
    //                     message,
    //                 } => {
    //                     if message.sender_id == sender_id {
    //                         messages.push({id: db_message.id.clone(), message: db_message.message});
    //                     }
    //                 }
    //                 _ => {}
    //             },
    //             _ => {}
    //         }
    //     }
    //     Ok(messages)
    // }

    /// Removes a message by its ID
    pub fn remove(&self, id: Uuid) -> sled::Result<()> {
        let key = id.to_string(); // Use the UUID as the key
        self.db.remove(key.as_bytes())?; // Remove the key from the database
        Ok(())
    }
}
