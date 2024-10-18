use super::this::Executor;
use std::sync::OnceLock;

////////////////////////////////////////////////////////////////////////////////

/// Синглтон исполнителя глобального
static EXECUTOR: OnceLock<Executor> = OnceLock::new();

////////////////////////////////////////////////////////////////////////////////

/// Делаем текущего исполнителя глобальным синглтоном после инициализации.
///
/// Если у нас уже и так был установлен, то вернет ошибку.
pub(crate) fn try_set_global_executor(e: Executor) -> Result<(), Executor> {
    EXECUTOR.set(e)
}

/// Получаем глобальный синглтон-исполнитель кода.
pub(crate) fn try_get_global_executor_opt() -> Option<&'static Executor> {
    EXECUTOR.get()
}

/// Получаем глобальный синглтон-исполнитель кода.
///
/// Паникуем если он не был установлен ранее.
pub(crate) fn get_global_executor() -> &'static Executor {
    try_get_global_executor_opt().expect("Executor is not set")
}
