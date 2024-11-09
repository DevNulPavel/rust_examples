use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Data {
    identifier: String,
    index: usize,
    hash: [u8;32],
}

impl Data {
    pub fn new(identifier: String, index: usize, hash: [u8;32]) -> Self {
        Data {
            identifier,
            index,
            hash
        }
    }
    pub fn get_hash(&self) -> [u8;32] {
        self.hash
    }
    pub fn get_index(&self) -> usize {
        self.index
    }
    pub fn get_identifier(&self) -> String {
        self.identifier.clone()
    }
}