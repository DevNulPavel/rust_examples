use exonum::runtime::{ExecutionContext, ExecutionError};
use exonum_derive::{exonum_interface, interface_method, ServiceDispatcher, ServiceFactory};
use exonum_rust_runtime::{api::ServiceApiBuilder, DefaultInstance, Service};
use crate::{
    api::CryptocurrencyApi,
    errors::Error,
    schema::{CurrencySchema, Wallet},
    transactions::{CreateWalletTx, TransferTx},
};

/// Начальный баланс нового кошелька
const INIT_BALANCE: u64 = 100;

//////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Интерфейс крипто-валюты
#[exonum_interface]
pub trait CryptocurrencyInterface<Ctx> {
    /// Выходное значение метода данного интерфеса
    type Output;

    /// Создаем кошелек с данным именем
    #[interface_method(id = 0)]
    fn create_wallet(&self, ctx: Ctx, arg: CreateWalletTx) -> Self::Output;

    /// Передаем значения валюты из одного кошелька в другой
    #[interface_method(id = 1)]
    fn transfer(&self, ctx: Ctx, arg: TransferTx) -> Self::Output;
}

//////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Реализация сервиса
#[derive(Debug, ServiceFactory, ServiceDispatcher)]
#[service_dispatcher(implements("CryptocurrencyInterface"))]
#[service_factory(proto_sources = "crate::proto")]
pub struct CryptocurrencyService;

impl CryptocurrencyInterface<ExecutionContext<'_>> for CryptocurrencyService {
    // Выходное значение
    type Output = Result<(), ExecutionError>;

    /// Создание кошелька, обработка сообщения
    fn create_wallet(&self, context: ExecutionContext<'_>, arg: CreateWalletTx) -> Self::Output {
        // Кто вызвал?
        let author = context
            .caller()
            .author()
            .expect("Wrong `TxCreateWallet` initiator");

        // Создаем схему
        let mut schema = CurrencySchema::new(context.service_data());

        // Проверяем, что нет такого кошелька
        if schema.wallets.get(&author).is_none() {
            // Создаем новый кошелек
            let wallet = Wallet::new(&author, &arg.name, INIT_BALANCE);
            
            println!("Created wallet: {:?}", wallet);

            // Помещаем новый кошелек в кошельки
            schema.wallets.put(&author, wallet);

            Ok(())
        } else {
            Err(Error::WalletAlreadyExists.into())
        }
    }

    /// Метод отправки денег из одного кошелька в другой
    fn transfer(&self, context: ExecutionContext<'_>, arg: TransferTx) -> Self::Output {
        // Кто отправляет
        let author = context
            .caller()
            .author()
            .expect("Wrong 'TxTransfer' initiator");
        if author == arg.to {
            return Err(Error::SenderSameAsReceiver.into());
        }

        // Получаем схему кошельков
        let mut schema = CurrencySchema::new(context.service_data());

        // Кошелек отправителя и получателя получаем из всех кошельков
        let sender = schema.wallets
            .get(&author)
            .ok_or(Error::SenderNotFound)?;
        let receiver = schema.wallets
            .get(&arg.to)
            .ok_or(Error::ReceiverNotFound)?;

        // Сколько передаем
        let amount = arg.amount;

        // Если есть деньги у отравителя
        if sender.balance >= amount {
            // Отнимаем у отправителя, добавляем получателю
            let sender = sender.decrease(amount);
            let receiver = receiver.increase(amount);

            println!("Transfer between wallets: {:?} => {:?}", sender, receiver);

            // Сохраняем кошельки?
            schema.wallets.put(&author, sender);
            schema.wallets.put(&arg.to, receiver);

            Ok(())
        } else {
            Err(Error::InsufficientCurrencyAmount.into())
        }
    }
}

impl Service for CryptocurrencyService {
    fn wire_api(&self, builder: &mut ServiceApiBuilder) {
        CryptocurrencyApi::wire(builder);
    }
}

// Specify default instantiation parameters for the service.
impl DefaultInstance for CryptocurrencyService {
    const INSTANCE_ID: u32 = 101;
    const INSTANCE_NAME: &'static str = "cryptocurrency";
}