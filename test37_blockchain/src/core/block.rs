
use crypto::sha2::Sha256;
use crypto::digest::Digest;
use serde_json::to_string;
use log::warn;

pub use super::{ BlockIndex, UserId, Transaction, BlockProof, BlockHash };


#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Block {
    pub(super) index: BlockIndex,
    pub(super) timestamp: i64,
    pub(super) mined_by: UserId,
    pub(super) transactions: Vec<Transaction>,
    pub(super) proof: BlockProof,
    pub(super) previous_hash: BlockHash,
}

impl Block {
    pub fn hash(&self) -> BlockHash {
        // Создаем хэшер
        let mut hasher = Sha256::new();

        // Переводим в строку весь блок
        match to_string(self) {
            Ok(json_str) => {
                // Вводим текст в хэш
                hasher.input_str(json_str.as_str());
                // Выдаем результат
                BlockHash(hasher.result_str())
            }
            Err(_) => {
                // Не вышло сконвертировать в строку
                warn!("Failed to marshall into a JSON string, using simple debug t");
                // Получаем строку как дебажный вывод
                let as_str = format!("{:?}", self);
                // Хэшируем
                hasher.input_str(&as_str);
                // Выдаем результат
                BlockHash(hasher.result_str())
            }
        }
    }
}