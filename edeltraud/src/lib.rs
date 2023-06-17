#![forbid(unsafe_code)]

mod builder;
mod inner;
mod pool;

#[cfg(test)]
mod tests;

pub use self::{
    builder::Builder,
    pool::{
        job, job_async, AsyncJob, AsyncResult, BuildError, Computation, Counters, Edeltraud,
        Handle, Job, JobUnit, SpawnError, Stats,
    },
};
