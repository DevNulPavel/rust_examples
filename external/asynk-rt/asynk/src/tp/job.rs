/// Job for worker
pub(crate) type Job = Box<dyn FnOnce() + Send>;
