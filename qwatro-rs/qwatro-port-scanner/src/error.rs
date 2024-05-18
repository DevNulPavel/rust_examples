use thiserror::Error;

/// Ошибки сканера портов
#[derive(Debug, Eq, PartialEq, Error)]
pub enum ScannerError {
    #[error("invalid port range: min can't be greater than max")]
    PortRangeMinGreaterThanMax,
    #[error("invalid port range: port value can't be equal to zero")]
    PortEqualsZero,
}
