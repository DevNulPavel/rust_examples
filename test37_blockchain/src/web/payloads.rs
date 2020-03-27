use crate::core::BlockIndex;
use uuid::Uuid;
use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Copy, Clone, Eq, PartialEq, Debug)]
pub struct Id {
    id: Uuid,
}

impl Id {
    pub fn new() -> Id {
        Id { id: Uuid::new_v4() }
    }
}

#[derive(Deserialize, Serialize, Copy, Clone, Eq, PartialEq, Debug)]
pub struct NewTransactionResult {
    pub block_index: BlockIndex,
}
