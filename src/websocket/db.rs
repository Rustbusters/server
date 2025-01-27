
use wg_2024::network::NodeId;

use serde::{Deserialize, Serialize};
use sled::{Db, IVec};

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    id: NodeId,
    messages: Vec<String>,
}

pub struct DbManager {
    db: Db,
}

impl DbManager {
    pub fn new(db_name: String) -> DbManager {
        Self { db: sled::open(db_name).unwrap() }
    }

    pub fn insert(&self, key: String, value: String) {
        self.db.insert(key.as_bytes(), value.as_bytes()).unwrap();
    }

    pub fn get(&self, key: String) -> Option<String> {
        match self.db.get(key.as_bytes()) {
            Ok(Some(value)) => Some(String::from_utf8(value.to_vec()).unwrap()),
            Ok(None) => None,
            Err(e) => {
                eprintln!("Error getting value from DB: {}", e);
                None
            }
        }
    }

    pub fn remove(&self, key: String) {
        self.db.remove(key.as_bytes()).unwrap();
    }

    pub fn get_all(&self) -> Vec<(String, String)> {
        self.db.iter()
            .map(|res| {
                let (key, value) = res.unwrap();
                (String::from_utf8(key.to_vec()).unwrap(), String::from_utf8(value.to_vec()).unwrap())
            })
            .collect()
    }
}