use iggy::utils::duration::IggyDuration;
use moka::future::Cache;

#[derive(Debug)]
pub struct MessageDeduplicator {
    cache: Cache<u128, bool>,
}

impl MessageDeduplicator {
    /// Creates a new message deduplicator with the given max entries and time to live for each ID.
    pub fn new(max_entries: Option<u64>, ttl: Option<IggyDuration>) -> Self {
        let mut cache = Cache::builder();
        if let Some(max_entries) = max_entries {
            cache = cache.max_capacity(max_entries);
        }
        if let Some(ttl) = ttl {
            cache = cache.time_to_live(ttl.get_duration());
        }

        Self {
            cache: cache.build(),
        }
    }

    /// Checks if the given ID exists.
    pub fn exists(&self, id: &u128) -> bool {
        self.cache.contains_key(id)
    }

    /// Inserts the given ID.
    pub async fn insert(&self, id: u128) {
        self.cache.insert(id, true).await;
    }

    /// Tries to insert the given ID, returns false if it already exists.
    pub async fn try_insert(&self, id: &u128) -> bool {
        if self.exists(id) {
            false
        } else {
            self.insert(*id).await;
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test]
    async fn message_deduplicator_should_insert_only_unique_identifiers() {
        let max_entries = 1000;
        let ttl = "1s".parse::<IggyDuration>().unwrap();
        let deduplicator = MessageDeduplicator::new(Some(max_entries), Some(ttl));
        for i in 0..max_entries {
            let id = i as u128;
            assert!(deduplicator.try_insert(&id).await);
            assert!(deduplicator.exists(&id));
            assert!(!deduplicator.try_insert(&id).await);
        }
    }

    #[tokio::test]
    async fn message_deduplicator_should_evict_identifiers_after_given_time_to_live() {
        let max_entries = 3;
        let ttl = "100ms".parse::<IggyDuration>().unwrap();
        let deduplicator = MessageDeduplicator::new(Some(max_entries), Some(ttl));
        for i in 0..max_entries {
            let id = i as u128;
            assert!(deduplicator.try_insert(&id).await);
            assert!(deduplicator.exists(&id));
            sleep(2 * ttl.get_duration()).await;
            assert!(!deduplicator.exists(&id));
            assert!(deduplicator.try_insert(&id).await);
        }
    }
}
