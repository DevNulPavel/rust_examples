mod ray;

#[cfg(all(feature = "multi_threaded", feature = "allow_unsafe"))]
mod render_multi_thread_unsafe;

#[cfg(all(feature = "multi_threaded", not(feature = "allow_unsafe")))]
mod render_multi_thread;

#[cfg(not(feature = "multi_threaded"))]
mod render_single_thread;

pub(crate) use self::ray::Ray;

#[cfg(all(feature = "multi_threaded", feature = "allow_unsafe"))]
pub(crate) use self::render_multi_thread_unsafe::render;

#[cfg(all(feature = "multi_threaded", not(feature = "allow_unsafe")))]
pub(crate) use self::render_multi_thread::render;

#[cfg(not(feature = "multi_threaded"))]
pub(crate) use self::render_single_thread::render;
