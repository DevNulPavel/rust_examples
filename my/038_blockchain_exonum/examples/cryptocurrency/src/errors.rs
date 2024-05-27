use exonum_derive::ExecutionFail;

/// Error codes emitted by `TxCreateWallet` and/or `TxTransfer` transactions during execution.
#[derive(Debug, ExecutionFail)]
pub enum Error {
    /// Wallet already exists.
    ///
    /// Can be emitted by `TxCreateWallet`.
    WalletAlreadyExists = 0,

    /// Sender doesn't exist.
    ///
    /// Can be emitted by `TxTransfer`.
    SenderNotFound = 1,

    /// Receiver doesn't exist.
    ///
    /// Can be emitted by `TxTransfer`.
    ReceiverNotFound = 2,

    /// Insufficient currency amount.
    ///
    /// Can be emitted by `TxTransfer`.
    InsufficientCurrencyAmount = 3,
    
    /// Sender same as receiver.
    ///
    /// Can be emitted by `TxTransfer`.
    SenderSameAsReceiver = 4,
}