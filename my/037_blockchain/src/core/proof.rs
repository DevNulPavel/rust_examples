use crypto::sha2::Sha256;
use crypto::digest::Digest;

use super::block::Block;


#[derive(PartialEq, Eq, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct BlockProof(pub u128);


/// Проверяет, начинается ли суммарный хэш с 0000
pub(super) fn is_valid_proof(last_block: &Block, proof: &BlockProof) -> bool {
    // Доказательство последнего блока + хэш последнего блока + доказательство нового
    // Суммарно они в итоге должны начинаться на 0000

    let mashed = format!("{}{}{}", last_block.proof.0, last_block.hash().0, proof.0);
    
    let mut hasher = Sha256::new();
    hasher.input_str(&mashed);

    let hashed = hasher.result_str();
    hashed.starts_with("0000")
}