use anyhow as failure; // FIXME: remove once `ProtobufConvert` derive is improved (ECR-4316)
use exonum::{
    crypto::PublicKey,
    merkledb::{
        access::{Access, FromAccess},
        MapIndex,
    },
};
use exonum_derive::{BinaryValue, FromAccess, ObjectHash};
use exonum_proto::ProtobufConvert;
use super::proto;

////////////////////////////////////////////////////////////////////////////////////////////////

// Описывает данные в кошельках под их именами
// Описание сериализации:
// https://exonum.com/doc/version/latest/architecture/serialization

////////////////////////////////////////////////////////////////////////////////////////////////

/// Структура кошелька, используемая для сохранения данных сервиса
#[derive(Clone, Debug, Serialize, Deserialize, ProtobufConvert, BinaryValue, ObjectHash)]
#[protobuf_convert(source = "proto::Wallet")]
pub struct Wallet {
    /// Публичный ключ владельца кошелька
    pub pub_key: PublicKey,
    /// Имя владельца кошелька
    pub name: String,
    /// Текущий баланс
    pub balance: u64,
}

/// Дополнительные методы для управления балансов, основанные на неизменяемости и создании нового кошелька
impl Wallet {
    /// Новый кошелек
    pub fn new(&pub_key: &PublicKey, name: &str, balance: u64) -> Self {
        Self {
            pub_key,
            name: name.to_owned(),
            balance,
        }
    }

    /// Возвращает копию кошелька с балансом, увеличенным на специфическое значение
    pub fn increase(self, amount: u64) -> Self {
        let balance = self.balance + amount;
        Self::new(&self.pub_key, &self.name, balance)
    }

    /// Возвращает копию кошелька с балансом, уменьшенным на специфическое значение
    pub fn decrease(self, amount: u64) -> Self {
        debug_assert!(self.balance >= amount);
        let balance = self.balance - amount;
        Self::new(&self.pub_key, &self.name, balance)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////

/// Схема хранилища кошельков
#[derive(Debug, FromAccess)]
pub struct CurrencySchema<T: Access> {
    /// Соответствие публичных ключей пользователей и кошельков
    pub wallets: MapIndex<T::Base, PublicKey, Wallet>
}

impl<T: Access> CurrencySchema<T> {
    /// Создание новой схемы
    pub fn new(access: T) -> Self {
        Self::from_root(access).unwrap()
    }
}