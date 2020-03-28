
use crypto::sha2::Sha256;
use crypto::digest::Digest;
use serde_json::to_string;
use log::warn;

use super::user_id::UserId;
use super::transaction::Transaction;
use super::proof::BlockProof;
use super::index::BlockIndex;
use super::hash::BlockHash;


#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Block {
    // Индекс блока в цепочке
    pub(super) index: BlockIndex,
    // Время создания данного блока
    pub(super) timestamp: i64,
    // Кто в конечном счете смог смайнить блок
    pub(super) mined_by: UserId,
    // Список транзакций, которые запечатаны в блоке
    pub(super) transactions: Vec<Transaction>,
    // Число, доказательство работы:
    // хэш теущего блока + прошлый хэш + это число - на выходе должно дать хэш с 0000 вначале
    pub(super) proof: BlockProof,
    // Хэш прошлого блока
    pub(super) previous_hash: BlockHash,
}

impl Block {
    // Хэш от текущего блока
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