use super::proof;
use super::block::Block;


#[derive(PartialEq, Eq, Debug, Deserialize, Serialize)]
pub struct Chain {
    pub blocks: Vec<Block>,
}


impl Chain {
    /// Валидная ли данная цепочка
    pub fn is_valid(&self) -> bool {
        let blocks = &self.blocks;
        if blocks.len() > 0 {
            blocks
                .iter()
                .skip(1) // Пропускаем первый Genesis блок
                .zip(blocks.iter()) // Сцепляем с итератором блоков, для проверки прошлого блока
                .all(|(current_block, prev_block)| {
                    // Хэш прошлого блока совпадает
                    let hash_check = current_block.previous_hash == prev_block.hash();
                    // Доказательство тоже верное
                    let proof_check = proof::is_valid_proof(prev_block, &current_block.proof);

                    hash_check && proof_check
                })
        } else {
            false
        }
    }
}