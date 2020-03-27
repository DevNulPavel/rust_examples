use super::block::*;
use super::blockchain::*;


#[derive(PartialEq, Eq, Debug, Deserialize, Serialize)]
pub struct Chain {
    pub blocks: Vec<Block>,
}


impl Chain {
    pub fn is_valid(&self) -> bool {
        let blocks = &self.blocks;
        if blocks.len() > 0 {
            blocks
                .iter()
                .skip(1)
                .zip(blocks.iter())
                .all(|(current_block, last_block)| {
                    let hash_check = current_block.previous_hash == last_block.hash();
                    let proof_check = Blockchain::valid_proof(last_block, &current_block.proof);
                    hash_check && proof_check
                })
        } else {
            false
        }
    }
}