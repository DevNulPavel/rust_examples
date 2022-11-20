use std::sync::atomic::{AtomicU64, Ordering};

use crate::{DiskLocation, ObjectId};

#[derive(Default)]
pub struct LocationTable {
    pt: pagetable::PageTable<AtomicU64>,
    max_object_id: AtomicU64,
}

impl LocationTable {
    pub fn load(&self, object_id: ObjectId) -> Option<DiskLocation> {
        let raw = self.pt.get(object_id).load(Ordering::Acquire);
        DiskLocation::from_raw(raw)
    }

    pub fn store(&self, object_id: ObjectId, location: DiskLocation) {
        self.max_object_id.fetch_max(object_id, Ordering::Release);

        self.pt
            .get(object_id)
            .store(location.to_raw(), Ordering::Release);
    }

    pub fn cas(
        &self,
        object_id: ObjectId,
        old_location: DiskLocation,
        new_location: DiskLocation,
    ) -> Result<(), DiskLocation> {
        self.max_object_id.fetch_max(object_id, Ordering::Release);

        self.pt
            .get(object_id)
            .compare_exchange(
                old_location.to_raw(),
                new_location.to_raw(),
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .map(|_| ())
            .map_err(|r| DiskLocation::from_raw(r).unwrap())
    }

    pub fn fetch_max(
        &self,
        object_id: ObjectId,
        new_location: DiskLocation,
    ) -> Result<Option<DiskLocation>, Option<DiskLocation>> {
        self.max_object_id.fetch_max(object_id, Ordering::Release);

        let max_result = self
            .pt
            .get(object_id)
            .fetch_max(new_location.to_raw(), Ordering::AcqRel);

        if max_result < new_location.to_raw() {
            Ok(DiskLocation::from_raw(max_result))
        } else {
            assert_ne!(max_result, new_location.to_raw());
            Err(DiskLocation::from_raw(max_result))
        }
    }

    pub fn max_object_id(&self) -> u64 {
        self.max_object_id.load(Ordering::Acquire)
    }

    #[cfg(feature = "runtime_validation")]
    pub fn iter<'a>(&'a self) -> impl 'a + Iterator<Item = (ObjectId, DiskLocation)> {
        (0..=self.max_object_id.load(Ordering::Acquire)).filter_map(|object_id| {
            if let Some(loc) = self.load(object_id) {
                Some((object_id, loc))
            } else {
                None
            }
        })
    }
}
