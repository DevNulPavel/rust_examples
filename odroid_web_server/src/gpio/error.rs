
#[derive(Debug)]
pub enum GPIOError {
    SudoRequired,
    WhichExecuteFailed,
    WhichParseError,
    GPIOAppNotFound,
    GPIORunFailed,
    GPIOWaitFailed,
    GPIOSpawnFailed,
}
