use gasket::error::AsWorkError;
use log::{error, warn};
use pallas::ledger::traverse::MultiEraBlock;
use pallas::network::miniprotocols::Point;
use sled::{Batch, Db, IVec, Tree};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct RollbackData {
    db: Option<Db>,
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
    pub fn open_db(config: Config) -> Self {
        let db = sled::open(config.db_path).or_retry().unwrap();

        RollbackData {
            db: Some(db),
            queue: Vec::default(),
        }

    }

    fn get_db_ref(&self) -> &Db {
        self.db.as_ref().unwrap()
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

    pub fn rollback_pop(&mut self) -> Option<Vec<u8>> {
        self.queue.pop()
    }

    pub fn len(&mut self) -> usize {
        self.queue.len()
    }

    fn get_rollback_range(&self, from: &Point) -> Vec<Vec<u8>> {
        let mut last_valid_block: Option<Vec<u8>> = None;
        let mut current_block: Vec<u8> = vec![];
        let mut blocks_to_roll_back: Vec<Vec<u8>> = vec![];

        let db = self.get_db_ref();

        let slot = from.slot_or_default().to_string();

        last_valid_block = db.get_lt(slot.as_bytes()).unwrap().map(|(_, value)| value.to_vec());

        current_block = match db.get(slot.as_bytes()).unwrap() {
            None => vec![],
            Some(value) => value.to_vec()
        };

        blocks_to_roll_back.push(current_block.to_vec());
        blocks_to_roll_back
    }

    pub fn insert_block(&self, point: &Point, block: &Vec<u8>) -> usize {
        let key = point.slot_or_default();
        let db = self.get_db_ref();
        db.insert(key.to_string().as_bytes(), IVec::from(block.clone()));

        let current_len = db.size_on_disk().unwrap();

        // Trim excess blocks
        if current_len > 2000000 {
            let first = match db.first() {
                Ok(first) => first,
                Err(_) => None
            };

            if let Some((first, _)) = first {
                db.remove(first).expect("todo: panic message");
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
