use sha3::{Sha3_512, Sha3_256, Digest};
use serde::{Serialize, Deserialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WORMBlock {
    pub index: u64,
    pub timestamp: u64,
    pub payload_hash: String,
    pub previous_hash: String,
    pub block_hash: String,
}

pub struct WORMLedger {
    pub chain: Vec<WORMBlock>,
}

impl WORMLedger {
    pub fn new() -> Self {
        let genesis_block = WORMBlock {
            index: 0,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            payload_hash: "0".repeat(128),
            previous_hash: "GENESIS_ROOT_PHI_SOVEREIGN".to_string(),
            block_hash: "850b2ef003f53db90f704b16e54869fc6ba6265eae13aafde9046fe0e4d99dab".to_string(),
        };
        Self { chain: vec![genesis_block] }
    }

    pub fn append_block(&mut self, payload: &[u8]) -> &WORMBlock {
        let prev_block = self.chain.last().unwrap();
        let index = prev_block.index + 1;
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        let mut p_hasher = Sha3_512::new();
        p_hasher.update(payload);
        let payload_hash = format!("{:x}", p_hasher.finalize());

        let block_raw = format!("{}:{}:{}:{}", index, timestamp, payload_hash, prev_block.block_hash);
        let mut b_hasher = Sha3_256::new();
        b_hasher.update(block_raw.as_bytes());
        let block_hash = format!("{:x}", b_hasher.finalize());

        self.chain.push(WORMBlock {
            index, timestamp, payload_hash,
            previous_hash: prev_block.block_hash.clone(),
            block_hash,
        });
        self.chain.last().unwrap()
    }

    pub fn verify_integrity(&self) -> bool {
        for i in 1..self.chain.len() {
            let current = &self.chain[i];
            let previous = &self.chain[i - 1];

            if current.previous_hash != previous.block_hash {
                return false;
            }

            let block_raw = format!("{}:{}:{}:{}", current.index, current.timestamp, current.payload_hash, current.previous_hash);
            let mut b_hasher = Sha3_256::new();
            b_hasher.update(block_raw.as_bytes());
            if current.block_hash != format!("{:x}", b_hasher.finalize()) {
                return false;
            }
        }
        true
    }
}
