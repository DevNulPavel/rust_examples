mod global;
mod io_handle;
mod this;

pub(crate) use self::{
    global::try_set_global_reactor,
    io_handle::{IoHandle, IoHandleOwned, IoHandleRef},
    this::Reactor,
};
