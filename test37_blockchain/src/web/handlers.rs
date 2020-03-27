use std::boxed::Box;
use actix::prelude::Addr;
use actix::MailboxError;
use actix_web::error::{ErrorBadRequest, ErrorInternalServerError};
use actix_web::{dev::Handler, Error, FromRequest, HttpRequest, Json};
use futures::Future;
use log::error;
//use serde_derive::{Deserialize, Serialize};

use super::actors::*;
use super::payloads::*;
use crate::core::{Block, Chain, Node, Transaction};


////////////////////////////////////////////////////////////////////////////////////////////////////

// Обработчик с актором внутри
pub struct GetIdHandler(pub Addr<BlockchainServerActor>);

// Реализация обработки конвертации в json для запросов
json_responder_impl!(Id);

impl<S> Handler<S> for GetIdHandler {
    type Result = Box<dyn Future<Item = Id, Error = Error>>;

    /// Обрабатываем
    fn handle(&self, _: &HttpRequest<S>) -> Box<dyn Future<Item = Id, Error = Error>> {
        // Отправляем актору сообщение получения его id
        // Возвращаем Future
        let f = self.0
            .send(GetId)
            .map_err(|e| { 
                ErrorInternalServerError(e)
            });
        Box::new(f)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct NewTransactionHandler(pub Addr<BlockchainServerActor>);

// Реализация обработки конвертации в json для запросов
json_responder_impl!(NewTransactionResult);

impl<S: 'static> Handler<S> for NewTransactionHandler {
    type Result = Box<dyn Future<Item = NewTransactionResult, Error = Error>>;

    /// Handle request
    fn handle(
        &self,
        req: &HttpRequest<S>,
    ) -> Box<dyn Future<Item = NewTransactionResult, Error = Error>> {
        let f_transaction = Json::<Transaction>::extract(req).map_err(|e| ErrorBadRequest(e));
        let owned_actor = self.0.clone(); // so we can send it into the flatMapped future.
        let f = f_transaction.and_then(move |transaction| {
            owned_actor
                .send(NewTransaction(transaction.0.clone()))
                .map_err(|e| ErrorInternalServerError(e))
                .map(|block_index| NewTransactionResult { block_index })
        });
        Box::new(f)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct MineHandler(pub Addr<BlockchainServerActor>);

json_responder_impl!(Block);

impl<S: 'static> Handler<S> for MineHandler {
    type Result = Box<dyn Future<Item = Block, Error = Error>>;

    /// Handle request
    fn handle(&self, _: &HttpRequest<S>) -> Box<dyn Future<Item = Block, Error = Error>> {
        let f = self.0.send(Mine).map_err(|e| ErrorInternalServerError(e));
        Box::new(f)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct GetChainHandler(pub Addr<BlockchainServerActor>);

json_responder_impl!(Chain);

impl<S: 'static> Handler<S> for GetChainHandler {
    type Result = Box<dyn Future<Item = Chain, Error = Error>>;

    /// Handle request
    fn handle(&self, _: &HttpRequest<S>) -> Box<dyn Future<Item = Chain, Error = Error>> {
        let f = self
            .0
            .send(GetChain)
            .map_err(|e| ErrorInternalServerError(e));
        Box::new(f)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct AddNodeHandler(pub Addr<BlockchainServerActor>);

json_responder_impl!(CurrentNodes);

impl<S: 'static> Handler<S> for AddNodeHandler {
    type Result = Box<dyn Future<Item = CurrentNodes, Error = Error>>;

    /// Handle request
    fn handle(&self, req: &HttpRequest<S>) -> Box<dyn Future<Item = CurrentNodes, Error = Error>> {
        let f_node = Json::<Node>::extract(req).map_err(|e| ErrorBadRequest(e));
        let owned_actor = self.0.clone(); // so we can send it into the flatMapped future.
        let f = f_node.and_then(move |node| {
            owned_actor
                .send(AddNode(node.0.clone()))
                .map_err(|e| ErrorInternalServerError(e))
        });
        Box::new(f)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ReconcileHandler(pub Addr<BlockchainServerActor>);

impl<S: 'static> Handler<S> for ReconcileHandler {
    type Result = Box<dyn Future<Item = Chain, Error = Error>>;

    /// Handle request
    fn handle(&self, _: &HttpRequest<S>) -> Box<dyn Future<Item = Chain, Error = Error>> {
        let cloned_addr = self.0.clone();
        let f = self
            .0
            .send(Reconcile)
            .and_then(move |r| {
                let b: Box<dyn Future<Item = Chain, Error = MailboxError>> = match r {
                    Ok(c) => Box::new(futures::future::ok(c)),
                    Err(_) => {
                        error!("Failed to reconcile, just returning current chain");
                        Box::new(cloned_addr.send(GetChain))
                    }
                };
                b
            })
            .map_err(|e| ErrorInternalServerError(e));
        Box::new(f)
    }
}
