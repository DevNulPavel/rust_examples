use std::{
    ops::{
        Deref,
        DerefMut,
    },
    hash::{
        Hash,
        Hasher,
    },
    ops::{
        Bound,
        RangeBounds,
    },
};

use crate::{
    pool,
    Shared,
    Unique,
    WeakShared,
};

type BytesInner = Shared<Vec<u8>>;
type BytesMutInner = Unique<Vec<u8>>;
type BytesWeakInner = WeakShared<Vec<u8>>;

#[derive(PartialEq, Hash, Debug)]
pub struct BytesMut {
    unique: BytesMutInner,
}

impl BytesMut {
    pub fn new_detached(value: Vec<u8>) -> Self {
        Self { unique: BytesMutInner::new_detached(value), }
    }

    pub fn freeze(mut self) -> Bytes {
        self.unique.shrink_to_fit();
        let inner = self.unique.freeze();
        let offset_to = inner.len();
        Bytes { inner, offset_from: 0, offset_to, }
    }

    pub fn freeze_range<R>(self, range: R) -> Bytes where R: RangeBounds<usize> {
        self.freeze().subrange(range)
    }
}

impl AsRef<BytesMutInner> for BytesMut {
    #[inline]
    fn as_ref(&self) -> &BytesMutInner {
        &self.unique
    }
}

impl AsRef<Vec<u8>> for BytesMut {
    #[inline]
    fn as_ref(&self) -> &Vec<u8> {
        self.unique.as_ref()
    }
}

impl Deref for BytesMut {
    type Target = BytesMutInner;

    #[inline]
    fn deref(&self) -> &BytesMutInner {
        self.as_ref()
    }
}

impl AsMut<BytesMutInner> for BytesMut {
    #[inline]
    fn as_mut(&mut self) -> &mut BytesMutInner {
        &mut self.unique
    }
}

impl AsMut<Vec<u8>> for BytesMut {
    #[inline]
    fn as_mut(&mut self) -> &mut Vec<u8> {
        self.unique.as_mut()
    }
}

impl DerefMut for BytesMut {
    #[inline]
    fn deref_mut(&mut self) -> &mut BytesMutInner {
        self.as_mut()
    }
}

#[derive(Clone, Debug)]
pub struct Bytes {
    inner: BytesInner,
    offset_from: usize,
    offset_to: usize,
}

#[derive(Clone, Debug)]
pub struct BytesWeak {
    inner: BytesWeakInner,
    offset_from: usize,
    offset_to: usize,
}

impl AsRef<[u8]> for Bytes {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        &self.inner[self.offset_from .. self.offset_to]
    }
}

impl Deref for Bytes {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &[u8] {
        self.as_ref()
    }
}

impl PartialEq for Bytes {
    fn eq(&self, other: &Bytes) -> bool {
        self.as_ref() == other.as_ref()
    }
}

impl Eq for Bytes { }

impl Hash for Bytes {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_ref().hash(state);
    }
}

impl Bytes {
    pub fn downgrade(&self) -> BytesWeak {
        BytesWeak {
            inner: self.inner.downgrade(),
            offset_from: self.offset_from,
            offset_to: self.offset_to,
        }
    }

    pub fn subrange<R>(&self, range: R) -> Bytes where R: RangeBounds<usize> {
        self.clone().into_subrange(range)
    }

    pub fn into_subrange<R>(mut self, range: R) -> Bytes where R: RangeBounds<usize> {
        self.focus_subrange(range);
        self
    }

    pub fn focus_subrange<R>(&mut self, range: R) where R: RangeBounds<usize> {
        let self_offset_from = self.offset_from;
        let self_offset_to = self.offset_to;
        match range.start_bound() {
            Bound::Unbounded =>
                (),
            Bound::Included(&offset) if offset + self_offset_from <= self_offset_to =>
                self.offset_from = offset + self_offset_from,
            Bound::Included(offset) =>
                panic!(
                    "Bytes::subrange start offset = {} not in range [{}, {}]",
                    offset,
                    0,
                    self_offset_to - self_offset_from,
                ),
            Bound::Excluded(..) =>
                unreachable!(),
        }
        match range.end_bound() {
            Bound::Unbounded =>
                (),
            Bound::Included(&offset) if offset + self_offset_from >= self.offset_from && offset + self_offset_from < self_offset_to =>
                self.offset_to = offset + self_offset_from + 1,
            Bound::Included(offset) =>
                panic!(
                    "Bytes::subrange included end offset = {} not in range [{}, {})",
                    offset,
                    self.offset_from - self_offset_from,
                    self_offset_to - self_offset_from,
                ),
            Bound::Excluded(&offset) if offset + self_offset_from >= self.offset_from && offset + self_offset_from <= self_offset_to =>
                self.offset_to = offset + self_offset_from,
            Bound::Excluded(offset) =>
                panic!(
                    "Bytes::subrange excluded end offset = {} not in range [{}, {}]",
                    offset,
                    self.offset_from - self_offset_from,
                    self_offset_to - self_offset_from,
                ),
        }
    }

    pub fn clone_subslice<'a>(&'a self, slice: &'a [u8]) -> Bytes {
        // safe because both the starting and other pointer are either in bounds or one
        // byte past the end of the same allocated object (checked by two asserts)
        unsafe {
            let ptr_range = self.inner.as_ptr_range();
            let slice_ptr = slice.as_ptr();
            assert!(ptr_range.contains(&slice_ptr) || (slice.is_empty() && slice_ptr == ptr_range.end));
            assert!(ptr_range.end >= slice_ptr.add(slice.len()));
            let offset_from = slice_ptr.offset_from(self.inner.as_ptr()) as usize;
            let offset_to = offset_from + slice.len();
            assert!(offset_from >= self.offset_from);
            assert!(offset_to <= self.offset_to);
            Bytes {
                inner: self.inner.clone(),
                offset_from,
                offset_to,
            }
        }
    }
}

impl BytesWeak {
    pub fn upgrade(&self) -> Option<Bytes> {
        self.inner.upgrade()
            .map(|arc| Bytes {
                inner: arc,
                offset_from: self.offset_from,
                offset_to: self.offset_to,
            })
    }
}


#[derive(Clone, Debug)]
pub struct BytesPool {
    kind: BytesPoolKind,
}

#[derive(Clone, Debug)]
enum BytesPoolKind {
    Attached {
        pool: pool::Pool<Vec<u8>>,
    },
    Detached,
}

impl Default for BytesPool {
    fn default() -> Self {
        Self::new()
    }
}

impl BytesPool {
    pub fn new() -> BytesPool {
        BytesPool {
            kind: BytesPoolKind::Attached {
                pool: pool::Pool::new(),
            },
        }
    }

    pub fn new_detached() -> BytesPool {
        BytesPool {
            kind: BytesPoolKind::Detached,
        }
    }

    pub fn lend(&self) -> BytesMut {
        match &self.kind {
            BytesPoolKind::Attached { pool, } => {
                let mut bytes = pool.lend(Vec::new);
                bytes.clear();
                BytesMut { unique: bytes, }
            },
            BytesPoolKind::Detached =>
                BytesMut::new_detached(Vec::new()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BytesMut,
    };

    #[test]
    fn freeze_00() {
        let bytes = BytesMut::new_detached(vec![0, 1, 2, 3, 4])
            .freeze();
        assert_eq!(&*bytes, &[0, 1, 2, 3, 4]);
    }

    #[test]
    fn freeze_range_00() {
        let bytes = BytesMut::new_detached(vec![0, 1, 2, 3, 4])
            .freeze_range(.. 3);
        assert_eq!(&*bytes, &[0, 1, 2]);
    }

    #[test]
    fn freeze_range_01() {
        let bytes = BytesMut::new_detached(vec![0, 1, 2, 3, 4])
            .freeze_range(..= 3);
        assert_eq!(&*bytes, &[0, 1, 2, 3]);
    }

    #[test]
    fn freeze_range_02() {
        let bytes = BytesMut::new_detached(vec![0, 1, 2, 3, 4])
            .freeze_range(2 ..);
        assert_eq!(&*bytes, &[2, 3, 4]);
    }

    #[test]
    fn freeze_range_03() {
        let bytes = BytesMut::new_detached(vec![0, 1, 2, 3, 4])
            .freeze_range(2 .. 4);
        assert_eq!(&*bytes, &[2, 3]);
    }

    #[test]
    #[should_panic]
    fn freeze_range_04() {
        let _bytes = BytesMut::new_detached(vec![0, 1, 2, 3, 4])
            .freeze_range(2 ..= 5);
    }

    #[test]
    #[should_panic]
    fn freeze_range_05() {
        let _bytes = BytesMut::new_detached(vec![0, 1, 2, 3, 4])
            .freeze_range(.. 6);
    }

    #[test]
    fn freeze_subrange_00() {
        let bytes = BytesMut::new_detached(vec![0, 1, 2, 3, 4])
            .freeze_range(.. 3)
            .subrange(1 ..);
        assert_eq!(&*bytes, &[1, 2]);
    }

    #[test]
    fn freeze_subrange_01() {
        let bytes = BytesMut::new_detached(vec![0, 1, 2, 3, 4])
            .freeze_range(..= 3)
            .subrange(2 ..= 2);
        assert_eq!(&*bytes, &[2]);
    }

    #[test]
    fn freeze_subrange_02() {
        let bytes = BytesMut::new_detached(vec![0, 1, 2, 3, 4])
            .freeze_range(2 ..)
            .subrange(3 ..);
        assert_eq!(&*bytes, &[]);
    }

    #[test]
    fn freeze_subrange_03() {
        let bytes = BytesMut::new_detached(vec![0, 1, 2, 3, 4])
            .freeze_range(2 .. 4)
            .subrange(.. 1);
        assert_eq!(&*bytes, &[2]);
    }

    #[test]
    #[should_panic]
    fn freeze_subrange_04() {
        let _bytes = BytesMut::new_detached(vec![0, 1, 2, 3, 4])
            .freeze_range(2 ..= 3)
            .subrange(3 ..);
    }

    #[test]
    #[should_panic]
    fn freeze_subrange_05() {
        let _bytes = BytesMut::new_detached(vec![0, 1, 2, 3, 4])
            .freeze_range(1 .. 4)
            .subrange(1 ..= 3);
    }

    #[test]
    fn clone_subslice_00() {
        let bytes = BytesMut::new_detached(vec![0, 1, 2, 3, 4])
            .freeze();
        let subslice = &bytes[2 .. 4];
        let bytes_cloned = bytes.clone_subslice(subslice);
        assert_eq!(&*bytes_cloned, &[2, 3]);
    }

    #[test]
    fn clone_subslice_01() {
        let bytes = BytesMut::new_detached(vec![0, 1, 2, 3, 4])
            .freeze();
        let subslice = &bytes[1 .. 4];
        let bytes_cloned_a = bytes.clone_subslice(subslice);
        assert_eq!(&*bytes_cloned_a, &[1, 2, 3]);
        let bytes_cloned_b = bytes_cloned_a.clone_subslice(&subslice[1 ..]);
        assert_eq!(&*bytes_cloned_b, &[2, 3]);
        let bytes_cloned_c = bytes_cloned_b.clone_subslice(&subslice[1 .. 2]);
        assert_eq!(&*bytes_cloned_c, &[2]);
    }

    #[test]
    #[should_panic]
    fn clone_subslice_02() {
        let bytes = BytesMut::new_detached(vec![0, 1, 2, 3, 4])
            .freeze();
        let subslice = &bytes[1 .. 4];
        let bytes_cloned_a = bytes.clone_subslice(subslice);
        assert_eq!(&*bytes_cloned_a, &[1, 2, 3]);
        let bytes_cloned_b = bytes_cloned_a.clone_subslice(&subslice[1 ..]);
        assert_eq!(&*bytes_cloned_b, &[2, 3]);
        let _bytes_cloned_c = bytes_cloned_b.clone_subslice(subslice);
    }

    #[test]
    fn clone_subslice_03() {
        let bytes = BytesMut::new_detached(vec![])
            .freeze();
        let subslice = &bytes[0 .. 0];
        let bytes_cloned = bytes.clone_subslice(subslice);
        assert_eq!(&*bytes_cloned, &[]);
    }

    #[test]
    fn clone_subslice_04() {
        let bytes = BytesMut::new_detached(vec![0, 1, 2])
            .freeze();
        let subslice = &bytes[3 .. 3];
        let bytes_cloned = bytes.clone_subslice(subslice);
        assert_eq!(&*bytes_cloned, &[]);
    }
}
