use std::sync::OnceLock;
use super::this::Reactor;

////////////////////////////////////////////////////////////////////////////////

/// Глобальный синглтон реактора
static REACTOR: OnceLock<Reactor> = OnceLock::new();

pub(crate) fn get_global_reactor() -> &'static Reactor {
    REACTOR.get().expect("reactor is not set")
}

pub(crate) fn set_global_reactor(r: Reactor) -> Result<(), Reactor> {
    REACTOR.set(r)
}
