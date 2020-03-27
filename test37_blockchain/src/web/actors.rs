use super::payloads::*;
use crate::core::{Block, BlockIndex, Blockchain, Chain, Node, Transaction};
use actix::prelude::*;
use futures::Future;
use std::collections::HashSet;
use serde_derive::Serialize;

pub struct BlockchainServerActor {
    // ID текущего актора
    id: Id,

    bloxi: Blockchain,
}

impl BlockchainServerActor {
    pub fn new() -> BlockchainServerActor {
        let id = Id::new();
        let bloxi = Blockchain::new();
        BlockchainServerActor { id, bloxi }
    }
}

impl Actor for BlockchainServerActor {
    type Context = Context<Self>;
}

////////////////////////////////////////////////////////////////////////////////////////////////

// Сообщение получения ID
pub struct GetId;

// Описываем методы по получению ID
simple_req_resp_impl!(GetId, Id);

// Описываем обработчик сообщения актором
impl Handler<GetId> for BlockchainServerActor {
    type Result = Id;

    fn handle(&mut self, _: GetId, _: &mut Self::Context) -> Self::Result {
        self.id.clone()
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////

// Сообщение типа NewTransaction
pub struct NewTransaction(pub Transaction);

// Описываем методы актора
simple_req_resp_impl!(NewTransaction, BlockIndex);

// Описываем обработчик сообщения актором
impl Handler<NewTransaction> for BlockchainServerActor {
    type Result = BlockIndex;

    fn handle(&mut self, NewTransaction(transaction): NewTransaction, _: &mut Self::Context) -> Self::Result {
        let block_idx = self.bloxi.add_transaction(transaction);
        block_idx
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////

// Сообщение майнинга
pub struct Mine;

// Описываем методы актора
simple_req_resp_impl!(Mine, Block);

impl Handler<Mine> for BlockchainServerActor {
    type Result = Block;

    fn handle(&mut self, _: Mine, _: &mut Self::Context) -> Self::Result {
        self.bloxi.mine();
        self.bloxi.last_block().clone()
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////

// Сообщение получение цепочки
pub struct GetChain;

// Описываем методы актора
simple_req_resp_impl!(GetChain, Chain);

impl Handler<GetChain> for BlockchainServerActor {
    type Result = Chain;

    fn handle(&mut self, _: GetChain, _: &mut Self::Context) -> Self::Result {
        self.bloxi.chain()
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////

// Сообщение добавления нода
pub struct AddNode(pub Node);

// Реузльтат - набор нодов
#[derive(Serialize)]
pub struct CurrentNodes {
    nodes: HashSet<Node>,
}

// Описываем методы актора
simple_req_resp_impl!(AddNode, CurrentNodes);

impl Handler<AddNode> for BlockchainServerActor {
    type Result = CurrentNodes;

    fn handle(&mut self, AddNode(node): AddNode, _: &mut Self::Context) -> Self::Result {
        let nodes = self.bloxi.register_node(node).clone();
        CurrentNodes { nodes }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////

pub struct Reconcile;

impl Message for Reconcile {
    type Result = Result<Chain, ()>;
}

impl Handler<Reconcile> for BlockchainServerActor {
    type Result = Box<dyn Future<Item = Chain, Error = ()>>;

    fn handle(&mut self, _: Reconcile, context: &mut Self::Context) -> Self::Result {
        let self_addr = context.address().clone();
        let f = self
            .bloxi
            .reconcile()
            .and_then(move |reconciled| self_addr.send(UpdateBloxi(reconciled)).map_err(|_| ()));
        Box::new(f)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////

pub struct UpdateBloxi(Blockchain);

impl Message for UpdateBloxi {
    type Result = Chain;
}

impl Handler<UpdateBloxi> for BlockchainServerActor {
    type Result = Chain;

    fn handle(&mut self, UpdateBloxi(update): UpdateBloxi, _: &mut Self::Context) -> Self::Result {
        self.bloxi = update;
        self.bloxi.chain()
    }
}
