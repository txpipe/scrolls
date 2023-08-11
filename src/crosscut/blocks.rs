use gasket::error::AsWorkError;
use pallas::network::miniprotocols::Point;
use serde::{Deserialize, Serialize};
use crate::Error;

#[derive(Clone)]
pub struct RollbackData {
    db: Option<sled::Db>,
    queue: Vec<Vec<u8>>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub struct Config {
    pub db_path: String,
    pub consumed_ring_path: String,
    pub produced_ring_path: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            db_path: "/opt/scrolls/block_buffer".to_string(),
            consumed_ring_path: "/opt/scrolls/consumed_buffer".to_string(),
            produced_ring_path: "/opt/scrolls/produced_buffer".to_string(),
        }
    }
}

impl From<Config> for RollbackData {
    fn from(config: Config) -> Self {
        RollbackData::open_db(config)
    }
}

impl RollbackData {
    fn get_db_ref(&self) -> &sled::Db {
        self.db.as_ref().unwrap()
    }

    fn get_rollback_range(&self, from: &Point) -> Vec<Vec<u8>> {
        let mut current_block: Vec<u8> = vec![];
        let mut blocks_to_roll_back: Vec<Vec<u8>> = vec![];

        let db = self.get_db_ref();

        let slot = from.slot_or_default().to_string();

        current_block = match db.get(slot.as_bytes()).unwrap() {
            None => vec![],
            Some(value) => value.to_vec()
        };

        blocks_to_roll_back.push(current_block.to_vec());

        let mut clear_blocks = sled::Batch::default();

        let mut last_seen_slot = slot.clone().to_string();
        if let Ok(block) = &db.get_gt(last_seen_slot.as_bytes()) {
            while let Some((next_key, next_block)) = block {
                last_seen_slot = String::from_utf8(next_key.to_vec()).unwrap();
                clear_blocks.remove(next_key);
                blocks_to_roll_back.push(next_block.to_vec())
            }

            db.apply_batch(clear_blocks).map_err(crate::Error::storage).expect("todo: map storage error");
        }

        blocks_to_roll_back
    }

    pub fn open_db(config: Config) -> Self {
        let db = sled::open(config.db_path).or_retry().unwrap();
        RollbackData {
            db: Some(db),
            queue: Vec::default(),
        }
    }

    pub fn close(&self) -> sled::Result<usize> {
        self.get_db_ref().flush()
    }

    pub fn enqueue_rollback_batch(&mut self, from: &Point) -> bool {
        let blocks = self.get_rollback_range(from);

        let emt: Vec<u8> = Vec::default();

        match !blocks.is_empty() && !blocks.first().unwrap_or(&emt).is_empty() {
            false => false,
            true => {
                self.queue.clear();
                self.queue.extend(blocks);
                true
            }
        }
    }

    pub fn rollback_pop(&mut self) -> Result<Option<sled::IVec>, Error> {
        match self.queue.pop() {
            None => Ok(None),
            Some(popped) => {
               self.get_db_ref().remove(popped).map_err(Error::storage)
            }
        }
    }

    pub fn len(&mut self) -> usize {
        self.queue.len()
    }

    pub fn insert_block(&self, point: &Point, block: &Vec<u8>) -> usize {
        let key = point.slot_or_default();
        let db = self.get_db_ref();
        db.insert(key.to_string().as_bytes(), sled::IVec::from(block.clone())).expect("todo map storage error");

        let current_len = db.size_on_disk().unwrap();

        // Trim excess blocks
        if current_len > 2000000 {
            let first = match db.first() {
                Ok(first) => first,
                Err(_) => None
            };

            if let Some((first, _)) = first {
                db.remove(first).expect("todo: map storage error");
            }
        }

        db.flush().unwrap()
    }

    pub fn get_block_at_point(&self, point: &Point) -> Option<Vec<u8>> {
        match self.get_db_ref().get(point.slot_or_default().to_string().as_bytes()) {
            Ok(block) => match block {
                None => None,
                Some(block) => Some(block.to_vec()),
            },
            Err(_) => None,
        }
    }
}
