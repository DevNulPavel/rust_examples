use anyhow as failure; // FIXME: remove once `ProtobufConvert` derive is improved (ECR-4316)
use exonum::crypto::PublicKey;
use exonum_derive::{BinaryValue, ObjectHash};
use exonum_proto::ProtobufConvert;

use super::proto;

///////////////////////////////////////////////////////////////////////////////////////////////

/// Тип транзакции - создание нового кошелька
#[derive(Clone, Debug, Serialize, Deserialize, ProtobufConvert, BinaryValue, ObjectHash)]
#[protobuf_convert(source = "proto::CreateWalletTx")]
pub struct CreateWalletTx {
    /// Имя владельца кошелька
    pub name: String,
}

impl CreateWalletTx {
    /// Создание кошелька с именем владельца
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////

/// Тип транзакции для перекидывания денег между кошельками
#[derive(Clone, Debug, Serialize, Deserialize, ProtobufConvert, BinaryValue, ObjectHash)]
#[protobuf_convert(source = "proto::TransferTx")]
pub struct TransferTx {
    /// Публичный ключ получателя
    pub to: PublicKey,
    /// Сколько денег кидаем
    pub amount: u64,
    /// Вспомогательный номер для надежности транзакции
    /// [idempotence]: https://en.wikipedia.org/wiki/Idempotence
    pub seed: u64,
}