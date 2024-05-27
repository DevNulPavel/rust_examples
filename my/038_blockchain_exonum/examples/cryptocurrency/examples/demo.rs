use exonum_cli::{NodeBuilder, Spec};

use exonum_cryptocurrency::contracts::CryptocurrencyService;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Инициируем логирование
    exonum::helpers::init_logger()?;

    // Запускам валютный сервис
    NodeBuilder::development_node()?
        // Starts cryptocurrency instance with given id and name
        // immediately after genesis block creation.
        .with(Spec::new(CryptocurrencyService{}).with_default_instance())
        .run()
        .await
}
