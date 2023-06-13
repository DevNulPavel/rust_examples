use std::{
    sync::{atomic, Arc},
    thread,
    time::Instant,
};
use crossbeam::utils::Backoff;
use crate::pool::{BuildError, Counters, SpawnError, Stats};

struct Bucket<J> {
    slot: BucketSlot<J>,
    touch_tag: TouchTag,
}

struct BucketSlot<J> {
    jobs_queue: crossbeam::queue::SegQueue<J>,
}

struct TouchTag {
    tag: atomic::AtomicU64,
}

impl Default for TouchTag {
    fn default() -> TouchTag {
        TouchTag {
            tag: atomic::AtomicU64::new(0),
        }
    }
}

#[derive(Debug)]
struct TouchTagDecoded {
    taken_by: usize,
    jobs_count: usize,
}

impl TouchTag {
    const JOBS_COUNT_MASK: u64 = u32::MAX as u64;
    const TAKEN_BY_MASK: u64 = !Self::JOBS_COUNT_MASK;

    fn load(&self) -> u64 {
        self.tag.load(atomic::Ordering::Relaxed)
    }

    fn try_set(&self, prev_tag: u64, new_tag: u64) -> Result<(), u64> {
        self.tag
            .compare_exchange_weak(
                prev_tag,
                new_tag,
                atomic::Ordering::Acquire,
                atomic::Ordering::Relaxed,
            )
            .map(|_| ())
    }

    fn decompose(tag: u64) -> TouchTagDecoded {
        TouchTagDecoded {
            taken_by: ((tag & Self::TAKEN_BY_MASK) >> 32) as usize,
            jobs_count: (tag & Self::JOBS_COUNT_MASK) as usize,
        }
    }

    fn compose(decoded: TouchTagDecoded) -> u64 {
        let mut tag = (decoded.taken_by as u64) << 32;
        tag |= decoded.jobs_count as u64;
        tag
    }
}

impl<J> Default for Bucket<J> {
    fn default() -> Self {
        Self {
            slot: BucketSlot {
                jobs_queue: crossbeam::queue::SegQueue::new(),
            },
            touch_tag: TouchTag::default(),
        }
    }
}

pub struct Inner<J> {
    buckets: Vec<Bucket<J>>,
    spawn_index_counter: atomic::AtomicUsize,
    await_index_counter: atomic::AtomicUsize,
    is_terminated: atomic::AtomicBool,
    counters: Arc<Counters>,
}

impl<J> Inner<J> {
    pub(super) fn new(workers_count: usize, counters: Arc<Counters>) -> Result<Self, BuildError> {
        Ok(Self {
            buckets: (0..workers_count).map(|_| Bucket::default()).collect(),
            spawn_index_counter: atomic::AtomicUsize::new(0),
            await_index_counter: atomic::AtomicUsize::new(0),
            is_terminated: atomic::AtomicBool::new(false),
            counters,
        })
    }

    pub(super) fn force_terminate(&self, threads: &[thread::Thread]) {
        self.is_terminated.store(true, atomic::Ordering::SeqCst);
        for thread in threads {
            thread.unpark();
        }
    }

    pub(super) fn spawn(&self, job: J, threads: &[thread::Thread]) -> Result<(), SpawnError> {
        let bucket_index = self
            .spawn_index_counter
            .fetch_add(1, atomic::Ordering::Relaxed)
            % self.buckets.len();
        let bucket = &self.buckets[bucket_index];

        let mut prev_tag = bucket.touch_tag.load();
        loop {
            if self.is_terminated() {
                return Err(SpawnError::ThreadPoolGone);
            }

            let decoded = TouchTag::decompose(prev_tag);
            let new_tag = TouchTag::compose(TouchTagDecoded {
                taken_by: 0,
                jobs_count: decoded.jobs_count + 1,
            });
            if let Err(changed_tag) = bucket.touch_tag.try_set(prev_tag, new_tag) {
                prev_tag = changed_tag;
                self.counters
                    .spawn_touch_tag_collisions
                    .fetch_add(1, atomic::Ordering::Relaxed);
                continue;
            }
            bucket.slot.jobs_queue.push(job);

            // notify possibly parked worker
            if decoded.taken_by > 0 {
                let worker_index = decoded.taken_by - 1;
                threads[worker_index].unpark();
            }

            break;
        }

        self.counters
            .spawn_total_count
            .fetch_add(1, atomic::Ordering::Relaxed);
        Ok(())
    }

    pub(super) fn acquire_job(&self, worker_index: usize, stats: &mut Stats) -> Option<J> {
        let now = Instant::now();
        let maybe_job = self.actually_acquire_job(worker_index, stats);
        stats.acquire_job_time += now.elapsed();
        stats.acquire_job_count += 1;
        maybe_job
    }

    fn actually_acquire_job(&self, worker_index: usize, stats: &mut Stats) -> Option<J> {
        'pick_bucket: loop {
            let bucket_index = self
                .await_index_counter
                .fetch_add(1, atomic::Ordering::Relaxed)
                % self.buckets.len();
            let bucket = &self.buckets[bucket_index];

            let backoff = Backoff::new();
            let mut prev_tag = bucket.touch_tag.load();
            loop {
                if self.is_terminated() {
                    return None;
                }

                let decoded = TouchTag::decompose(prev_tag);

                if decoded.jobs_count == 0 {
                    // empty bucket encountered, have to wait for a job to appear

                    let now = Instant::now();
                    if !backoff.is_completed() {
                        // spin a little
                        backoff.snooze();
                        stats.acquire_job_backoff_time += now.elapsed();
                        stats.acquire_job_backoff_count += 1;
                        prev_tag = bucket.touch_tag.load();
                        continue;
                    }

                    // park the worker if there is no job for a long time
                    if decoded.taken_by != worker_index + 1 {
                        if decoded.taken_by == 0 {
                            // try to acquire parking lot on this bucket
                            let new_tag = TouchTag::compose(TouchTagDecoded {
                                taken_by: worker_index + 1,
                                jobs_count: 0,
                            });
                            if let Err(changed_tag) = bucket.touch_tag.try_set(prev_tag, new_tag) {
                                prev_tag = changed_tag;
                                backoff.reset();
                                continue;
                            }
                        } else {
                            // there is another thread parked on this bucket, proceed to the next one
                            stats.acquire_job_taken_by_collisions += 1;
                            continue 'pick_bucket;
                        }
                    }

                    thread::park();
                    stats.acquire_job_thread_park_time += now.elapsed();
                    stats.acquire_job_thread_park_count += 1;
                    prev_tag = bucket.touch_tag.load();
                    backoff.reset();
                    continue;
                }

                // non-empty bucket, try to reserve a job
                let new_tag = TouchTag::compose(TouchTagDecoded {
                    taken_by: 0,
                    jobs_count: decoded.jobs_count - 1,
                });
                if let Err(changed_tag) = bucket.touch_tag.try_set(prev_tag, new_tag) {
                    prev_tag = changed_tag;
                    continue;
                }

                break;
            }

            // try to pop a job
            let now = Instant::now();
            backoff.reset();
            loop {
                if let Some(job) = bucket.slot.jobs_queue.pop() {
                    stats.acquire_job_seg_queue_pop_time += now.elapsed();
                    stats.acquire_job_seg_queue_pop_count += 1;
                    return Some(job);
                }
                backoff.snooze();
            }
        }
    }

    pub(super) fn is_terminated(&self) -> bool {
        self.is_terminated.load(atomic::Ordering::Relaxed)
    }
}
