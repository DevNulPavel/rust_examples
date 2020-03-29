use exonum::crypto::PublicKey;
use exonum_rust_runtime::api::{self, ServiceApiBuilder, ServiceApiState};
use crate::schema::{CurrencySchema, Wallet};


/// Структура, описывающая запрос параметров запроса get_wallet
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct WalletQuery {
    /// Публичный ключ кошелька
    pub pub_key: PublicKey,
}

/// Описание публичного API
#[derive(Debug, Clone, Copy)]
pub struct CryptocurrencyApi;

impl CryptocurrencyApi {
    /// Обработчик запроса кошелька
    pub async fn get_wallet(state: ServiceApiState, query: WalletQuery) -> api::Result<Wallet> {
        let schema = CurrencySchema::new(state.service_data());
        schema
            .wallets
            .get(&query.pub_key)
            .ok_or_else(|| {
                api::Error::not_found()
                    .title("Wallet not found")
            })
    }

    /// Обработчик запроса выгрузки всех кошельков из хранилища
    pub async fn get_wallets(state: ServiceApiState, _query: ()) -> api::Result<Vec<Wallet>> {
        let schema = CurrencySchema::new(state.service_data());
        Ok(schema.wallets
                .values()
                .collect())
    }

    /// ServiceApiBuilder облегчает конвертацию между запросами чтения и REST запросами
    pub fn wire(builder: &mut ServiceApiBuilder) {
        // Назначаем обработчики для кокретных путей REST
        builder
            .public_scope()
            .endpoint("v1/wallet", Self::get_wallet)
            .endpoint("v1/wallets", Self::get_wallets);
    }
}