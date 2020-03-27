use std::collections::HashSet;
use std::mem;
use chrono::Utc;
use crypto::sha2::Sha256;
use rand::Rng;
use url::Url;
use crypto::digest::Digest;
use futures::{future, Future};
use serde_json::to_string;
use log::{info, warn};
use actix_web::{client, HttpMessage};

static SYSTEM_USER_ID: UserId = UserId(0);

/*
 * - Mistake in proof of work https://medium.com/@schubert.konstantin/isnt-there-a-msitake-with-your-proof-of-work-30cf9467f0a5
 * - Need to hash the last block when adding a new block
 */

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Blockchain {
    // Id текущего юзера
    user_id: UserId,
    // Цепочка всех прошлых блоков
    chain: Vec<Block>,
    // Прошлые транзакции, отброшенные транзакции?
    outstanding_transactions: Vec<Transaction>,
    nodes: HashSet<Node>,
}

#[derive(PartialEq, Eq, Debug, Deserialize, Serialize)]
pub struct Chain {
    pub blocks: Vec<Block>,
}

impl Blockchain {
    pub fn new() -> Blockchain {
        // Создаем пустую цепочку
        let mut result = Blockchain {
            user_id: UserId::generate(),
            chain: Vec::with_capacity(1000),
            outstanding_transactions: Vec::with_capacity(100),
            nodes: HashSet::with_capacity(1000),
        };
        // Добавляем базовый первый блок
        result.new_block(BlockProof(0), BlockHash(String::from("0000000000")));
        // Результат
        result
    }

    /// Добавляем новый блок в цепочку
    pub fn new_block(&mut self, proof: BlockProof, previous_block_hash: BlockHash) -> () {
        // Заменяем в блокчейне старые транзакции пустыми, получаем старые
        let transactions =
            mem::replace(&mut self.outstanding_transactions, Vec::with_capacity(100));
        
        // Создаем новый блок
        let block = Block {
            // Индекс нового блока
            index: BlockIndex(self.chain.len() as u128),
            // Время создания
            timestamp: Utc::now().timestamp_millis(),
            // Кто смайнил
            mined_by: self.user_id,
            // Прошлые транзакции
            transactions,
            // Доказательство работы
            proof,
            // Хэш прошлого блока
            previous_hash: previous_block_hash,
        };

        // Сохранем новый блок в цепочку
        self.chain.push(block)
    }

    /// Добавляем транзакцию
    pub fn add_transaction(&mut self, transaction: Transaction) -> BlockIndex {
        // Добавляем транзакцию в очередь необработанных
        self.outstanding_transactions.push(transaction);

        // Индекс блока транзакции - следующий
        if let Some(last_block) = self.chain.last() {
            BlockIndex(last_block.index.0 + 1)
        } else {
            BlockIndex(0)
        }
    }

    /// Майнинг
    pub fn mine(&mut self) -> () {
        // Доказательство работы
        let proof = self.proof_of_work();

        // Создаем транзакцию на 100 денег от текущего юзера к новому
        let _ = self.add_transaction(Transaction {
            from: SYSTEM_USER_ID,
            to: self.user_id,
            amount: 100,
        });
        
        // Хэш последнего блока
        let previous_hash = self.last_block().hash();

        // Создаем новый блок с данными транзакции
        self.new_block(proof, previous_hash);
    }

    /// Создание доказательства работы
    fn proof_of_work(&self) -> BlockProof {
        let mut current_proof = BlockProof(0);
        // Подбираем валидное значение хэша, пока не подберется
        while !Blockchain::valid_proof(self.last_block(), &current_proof) {
            current_proof.0 += 1;
        }
        current_proof
    }

    pub fn chain(&self) -> Chain {
        Chain {
            blocks: self.chain.clone(),
        }
    }

    /// Проверяет, начинается ли суммарный хэш с 0000
    fn valid_proof(last_block: &Block, proof: &BlockProof) -> bool {
        let mashed = format!("{}{}{}", last_block.proof.0, last_block.hash().0, proof.0);
        
        let mut hasher = Sha256::new();
        hasher.input_str(&mashed);

        let hashed = hasher.result_str();
        hashed.starts_with("0000")
    }

    pub fn last_block(&self) -> &Block {
        self.chain.last().expect("Block chain with no blocks!")
    }

    pub fn register_node(&mut self, node: Node) -> &HashSet<Node> {
        self.nodes.insert(node);
        &self.nodes
    }

    pub fn reconcile(&self) -> impl Future<Item = Blockchain, Error = ()> {
        let self_clone = self.clone();
        let node_chain_futures: Vec<_> = self
            .nodes
            .iter()
            .map(|node| {
                let mut url = node.address.clone();
                url.set_path("/chain");
                info!("Getting chain from node [{:?}] using url [{}]", node, url);
                let mut builder = client::get(url);
                let f_or_err = builder
                    .finish()
                    .map_err(|e| {
                        warn!(
                            "Failed to build a request object for node [{:?}]: [{}]",
                            node, e
                        );
                        ()
                    })
                    .map(|req| {
                        let f = req
                            .send()
                            .map_err(|e| {
                                warn!("Request failed: [{}]", e);
                                ()
                            })
                            .and_then(|resp| {
                                resp.body()
                                    .map_err(|e| {
                                        warn!("Failed to read body from response: [{}]", e);
                                        ()
                                    })
                                    .map(|bytes| {
                                        let t = serde_json::from_slice(&bytes)
                                            .map_err(|e| {
                                                warn!(
                                                    "Failed to demarshal response to a chain: [{}]",
                                                    e
                                                );
                                                ()
                                            })
                                            .and_then(|received_chain: Chain| {
                                                if received_chain.is_valid() {
                                                    Ok(received_chain)
                                                } else {
                                                    Err(())
                                                }
                                            });
                                        t
                                    })
                            });

                        let boxed: Box<dyn Future<Item = Result<Chain, ()>, Error = ()>> = Box::new(f);
                        boxed
                    });
                f_or_err.unwrap_or_else(|_| Box::new(future::ok(Err(()))))
            })
            .collect();
        let future_node_chains = future::join_all(node_chain_futures);
        future_node_chains
            .map(move |chains| {
                chains
                    .into_iter()
                    .fold(self_clone, |acc, next_fetch_attempt| {
                        let acc_err_clone = acc.clone();
                        next_fetch_attempt
                            .map(|next| {
                                if next.blocks.len() > acc.chain.len() {
                                    Blockchain {
                                        chain: next.blocks,
                                        ..acc
                                    }
                                } else {
                                    acc
                                }
                            })
                            .unwrap_or_else(|_| acc_err_clone)
                    })
            })
            .map_err(|_| ())
    }
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Block {
    index: BlockIndex,
    timestamp: i64,
    mined_by: UserId,
    transactions: Vec<Transaction>,
    proof: BlockProof,
    previous_hash: BlockHash,
}

impl Block {
    fn hash(&self) -> BlockHash {
        let mut hasher = Sha256::new();

        match to_string(self) {
            Ok(json_str) => {
                hasher.input_str(json_str.as_str());
                BlockHash(hasher.result_str())
            }
            Err(_) => {
                warn!("Failed to marshall into a JSON string, using simple debug t");
                let as_str = format!("{:?}", self);
                hasher.input_str(&as_str);
                BlockHash(hasher.result_str())
            }
        }
    }
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

#[derive(PartialEq, Eq, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct BlockIndex(pub u128);

#[derive(PartialEq, Eq, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct BlockProof(pub u128);

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct BlockHash(pub String);

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct UserId(pub u128 /* Rsa<Public> */);

impl UserId {
    pub fn generate() -> UserId {
        let mut rng = rand::thread_rng();
        UserId(rng.gen())
    }
}

#[derive(PartialEq, Eq, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub from: UserId,
    pub to: UserId,
    pub amount: i128,
}

#[derive(PartialEq, Eq, Hash, Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    #[serde(with = "url_serde")]
    address: Url,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_chain() {
        let mut bloxi = Blockchain::new();
        assert!(bloxi.chain().is_valid()); // all chains are initially valid
        for i in 1..5 {
            bloxi.add_transaction(Transaction {
                from: UserId(i),
                to: UserId(i + 1),
                amount: (i + 2) as i128,
            });
            bloxi.add_transaction(Transaction {
                from: UserId(i + 3),
                to: UserId(i + 4),
                amount: (i + 5) as i128,
            });
            bloxi.add_transaction(Transaction {
                from: UserId(i + 6),
                to: UserId(i + 7),
                amount: (i + 8) as i128,
            });
            bloxi.mine();
        }
        assert!(bloxi.chain().is_valid());
    }
}
