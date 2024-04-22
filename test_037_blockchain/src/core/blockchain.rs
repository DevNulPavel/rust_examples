use std::collections::HashSet;
use std::mem;
use chrono::Utc;
use futures::{future, Future};
use log::{info, warn};
use actix_web::{client, HttpMessage};

use super::*;
use super::block::*;
use super::chain::*;
use super::hash::BlockHash;
use super::index::BlockIndex;

const MAX_TRANSACTIONS_BEFORE_PROCESS: usize = 100;


#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Blockchain {
    // Id текущего юзера, который хранит цепочку?
    user_id: UserId,
    // Цепочка всех блоков
    chain: Vec<Block>,
    // Транзакции, которые мы накапливаем, пока не создадим новый блок
    unprocessed_transactions: Vec<Transaction>,
    // Ноды, подключенные к системе блокчейна
    nodes: HashSet<Node>,
}

impl Blockchain {
    pub fn new() -> Blockchain {
        // Создаем пустую цепочку
        let mut result = Blockchain {
            user_id: UserId::generate(),
            chain: Vec::with_capacity(1000),
            unprocessed_transactions: Vec::with_capacity(MAX_TRANSACTIONS_BEFORE_PROCESS),
            nodes: HashSet::with_capacity(1000),
        };
        
        // Добавляем базовый первый блок
        result.new_block(BlockProof(0), BlockHash(String::from("0000000000")));

        // Результат
        result
    }

    /// Добавляем новый блок в цепочку c доказательством работы
    pub fn new_block(&mut self, proof: BlockProof, previous_block_hash: BlockHash) -> () {
        // Заменяем в блокчейне все накопленные транзакции на пустые
        let transactions = mem::replace(&mut self.unprocessed_transactions, 
                                                           Vec::with_capacity(MAX_TRANSACTIONS_BEFORE_PROCESS));
        
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
        self.unprocessed_transactions.push(transaction);

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

        // const SYSTEM_USER_ID: UserId = UserId(0);
        // Создаем транзакцию на 100 денег от текущего юзера к новому
        // let _ = self.add_transaction(Transaction {
        //     from: SYSTEM_USER_ID,
        //     to: self.user_id,
        //     amount: 100,
        // });
        
        // Хэш последнего блока
        let previous_hash = self.last_block().hash();

        // Создаем новый блок с данными транзакции
        self.new_block(proof, previous_hash);
    }

    /// Создание доказательства работы
    fn proof_of_work(&self) -> BlockProof {
        // Берем нулевое доказательство
        let mut current_proof = BlockProof(0);

        // Подбираем валидное значение хэша, пока не подберется доказательство
        while !proof::is_valid_proof(self.last_block(), &current_proof) {
            current_proof.0 += 1;
        }

        current_proof
    }

    pub fn chain(&self) -> Chain {
        Chain {
            blocks: self.chain.clone(),
        }
    }

    pub fn last_block(&self) -> &Block {
        self.chain.last().expect("Block chain with no blocks!")
    }

    // Добавляем новый нод к системе
    pub fn register_node(&mut self, node: Node) -> &HashSet<Node> {
        self.nodes.insert(node);
        &self.nodes
    }

    /// Специальный вызов, который валидирует цепочки всех нодов, выбирая самую длинную, так как
    /// именно она считается саой правильной, так как поддельные цепочки будут меньше, так как не смогут
    /// угнаться за настоящей цепочкой
    pub fn reconcile(&self) -> impl Future<Item = Blockchain, Error = ()> {
        // Клонируем цепочку
        let self_clone = self.clone();
        
        // Получаем фьючи, в которых будут цепочки от других нодов
        let node_chain_futures: Vec<_> = self
            .nodes
            .iter()
            .map(|node| {
                // Берем адрес нода
                let mut url = node.address.clone();
                // Путь - цепочка
                url.set_path("/chain");
                
                info!("Getting chain from node [{:?}] using url [{}]", node, url);

                // Создаем запрос
                let fut_or_err = client::get(url)
                    .finish()
                    .map_err(|e| {
                        warn!("Failed to build a request object for node [{:?}]: [{}]", node, e);
                        ()
                    })
                    .map(|req| {
                        // Выполняем запрос
                        let fut = req
                            .send()
                            .map_err(|e| {
                                warn!("Request failed: [{}]", e);
                                ()
                            })
                            .and_then(|resp| {
                                // Обрабатываем ответ
                                resp.body()
                                    .map_err(|e| {
                                        warn!("Failed to read body from response: [{}]", e);
                                        ()
                                    })
                                    .map(|bytes| {
                                        // Парсим Json в Chain
                                        let chain_result = serde_json::from_slice(&bytes)
                                            .map_err(|e| {
                                                warn!("Failed to demarshal response to a chain: [{}]", e);
                                                ()
                                            })
                                            .and_then(|received_chain: Chain| {
                                                if received_chain.is_valid() {
                                                    Ok(received_chain)
                                                } else {
                                                    Err(())
                                                }
                                            });
                                        chain_result
                                    })
                            });

                        // Оборачиваем в Box нашу Future
                        let boxed: Box<dyn Future<Item = Result<Chain, ()>, Error = ()>> = Box::new(fut);
                        boxed
                    });
                fut_or_err
                    .unwrap_or_else(|_| Box::new(future::ok(Err(()))))
            })
            .collect();

        // Ждем результаты от всех нодов
        let future_node_chains = future::join_all(node_chain_futures);

        let valid_blockchain_fut = future_node_chains
            .map(move |chains| {
                // Итерируемся по полученным цепочкам для поиска самой длинной
                chains
                    .into_iter()
                    // Упаковываем в одну на основании копии текущего блокчейна
                    .fold(self_clone, |acc, next_fetch_attempt| {
                        // Выбираем из всех блокчейном от других нодов самую длинную цепочку
                        // Она должна быть правильной, так как если кто-то подделал,
                        // то поддельная цепочка будет меньше всего по длине
                        let acc_err_clone = acc.clone();
                        next_fetch_attempt
                            .map(|next| {
                                // Если длина полученной цепочки длиннее текущей - значит берем эту длинную цепочку
                                // Если нет - оставляем старую
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
            .map_err(|_| ());

        valid_blockchain_fut
    }
}