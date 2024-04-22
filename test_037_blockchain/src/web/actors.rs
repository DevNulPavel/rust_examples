use super::payloads::*;
use crate::core::{Block, BlockIndex, Blockchain, Chain, Node, Transaction};
use actix::prelude::*;
use futures::Future;
use std::collections::HashSet;
use serde_derive::Serialize;

pub struct BlockchainServerActor {
    // ID текущего актора
    id: Id,
    // Непосредственно сам блокчейн
    blockchain: Blockchain,
}

impl BlockchainServerActor {
    pub fn new() -> BlockchainServerActor {
        let id = Id::new();
        let bloxi = Blockchain::new();
        BlockchainServerActor { 
            id, 
            blockchain: bloxi 
        }
    }
}

impl Actor for BlockchainServerActor {
    type Context = Context<Self>;
}

////////////////////////////////////////////////////////////////////////////////////////////////

// Сообщение получения ID
pub struct GetIdMessage;

// Описываем методы по получению ID
simple_req_resp_impl!(GetIdMessage, Id);

// Описываем обработчик сообщения актором
impl Handler<GetIdMessage> for BlockchainServerActor {
    type Result = Id;

    fn handle(&mut self, _: GetIdMessage, _: &mut Self::Context) -> Self::Result {
        self.id.clone()
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////

// Сообщение типа NewTransaction
pub struct NewTransactionMessage(pub Transaction);

// Описываем методы актора
simple_req_resp_impl!(NewTransactionMessage, BlockIndex);

// Описываем обработчик сообщения актором
impl Handler<NewTransactionMessage> for BlockchainServerActor {
    type Result = BlockIndex;

    fn handle(&mut self, NewTransactionMessage(transaction): NewTransactionMessage, _: &mut Self::Context) -> Self::Result {
        let block_idx = self.blockchain.add_transaction(transaction);
        block_idx
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////

// Сообщение майнинга
pub struct MineMessage;

// Описываем методы актора
simple_req_resp_impl!(MineMessage, Block);

impl Handler<MineMessage> for BlockchainServerActor {
    type Result = Block;

    fn handle(&mut self, _: MineMessage, _: &mut Self::Context) -> Self::Result {
        // Майним и возвращаем получившийся блок
        self.blockchain.mine();
        self.blockchain.last_block().clone()
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////

// Сообщение получение цепочки
pub struct GetChainMessage;

// Описываем методы актора
simple_req_resp_impl!(GetChainMessage, Chain);

impl Handler<GetChainMessage> for BlockchainServerActor {
    type Result = Chain;

    fn handle(&mut self, _: GetChainMessage, _: &mut Self::Context) -> Self::Result {
        // Возвращает текущую цепочку у блока
        self.blockchain.chain()
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////

// Сообщение добавления нода
pub struct AddNodeMessage(pub Node);

// Реузльтат - набор нодов
#[derive(Serialize)]
pub struct CurrentNodes {
    nodes: HashSet<Node>,
}

// Описываем методы актора
simple_req_resp_impl!(AddNodeMessage, CurrentNodes);

impl Handler<AddNodeMessage> for BlockchainServerActor {
    type Result = CurrentNodes;

    fn handle(&mut self, AddNodeMessage(node): AddNodeMessage, _: &mut Self::Context) -> Self::Result {
        // Добавляем новый нод в систему и возвращаем новый список
        let nodes = self.blockchain.register_node(node).clone();
        CurrentNodes { 
            nodes 
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ReconcileMessage;

impl Message for ReconcileMessage {
    type Result = Result<Chain, ()>;
}

impl Handler<ReconcileMessage> for BlockchainServerActor {
    type Result = Box<dyn Future<Item = Chain, Error = ()>>;

    fn handle(&mut self, _: ReconcileMessage, context: &mut Self::Context) -> Self::Result {
        let self_addr = context.address().clone();
        let f = self
            .blockchain
            // Определяем, какая из всех цепочек у нодов правдивая (самая длинная)
            .reconcile()
            .and_then(move |reconciled| {
                // После получения настоящего блокчейна из всех нодов
                // Отправляем самому себе сообщение обновления блокчейна на новый
                self_addr
                    .send(UpdateBlockchainMessage(reconciled))
                    .map_err(|_| ())
            });
        Box::new(f)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////

pub struct UpdateBlockchainMessage(Blockchain);

impl Message for UpdateBlockchainMessage {
    type Result = Chain;
}

impl Handler<UpdateBlockchainMessage> for BlockchainServerActor {
    type Result = Chain;

    fn handle(&mut self, UpdateBlockchainMessage(update): UpdateBlockchainMessage, _: &mut Self::Context) -> Self::Result {
        // Обновляем текущую цепочку на новую
        self.blockchain = update;
        self.blockchain.chain()
    }
}
