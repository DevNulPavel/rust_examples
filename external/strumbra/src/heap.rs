use std::{
    alloc::Layout,
    cell::Cell,
    marker::PhantomData,
    ptr::NonNull,
    sync::atomic::{self, AtomicUsize},
};

////////////////////////////////////////////////////////////

/// Trait for thin pointer to an array that can be dropped using a user-provided length.
/// Трейт для тощего указателя на массив, который может быть уничтожен
/// с помощью дополнительного указания размера этого самого массива данных
///
/// # Safety
///
/// Типы, которые реализуют данный трейт должны корректно обрабатывать размер.
pub unsafe trait ThinDrop {
    /// Уничтожение буффера через тонкий указатель с использованием переданного размера этого буффера
    ///
    /// # Safety
    ///
    /// - Вызывающий должен обеспечить, что длина равна правильному размер аллоцированных байт.
    /// - Вызывающий должен обеспечить, что объект после вызова деаллокации не будет никак использован.
    /// - Вызываться метод должен один раз за время работы приложения.
    ///   Отсутствие вызова приведет к утечке памяти.
    unsafe fn thin_drop(&self, len: usize);
}

////////////////////////////////////////////////////////////

/// Трейт специальный для клонирования данных с указанием конкретной длины данных.
///
/// # Safety
/// - Типы, которые реализуют данный трейт должны корректно использовать передаваемую длину
pub unsafe trait ThinClone {
    /// Клонируем нижележащий буфер через "тощий" указатель.
    ///
    /// # Safety
    /// - Вызывающий должен обеспечить, что длина должна быть равна количеству аллоцированных байт.
    unsafe fn thin_clone(&self, len: usize) -> Self;
}

////////////////////////////////////////////////////////////

/// Трейт для тонкого указателя на данные, которые могут быть представлены как байты с
/// использованием переданного на вход размера этих самых данных.
///
/// # Safety
/// - Типы, которые реализуют данный трейт должны корректно использовать перреданную длину.
pub unsafe trait ThinAsBytes {
    /// Слайс байт, получаемый с помощью дочернего буфера при помощи указания длины данных.
    ///
    /// # Safety
    /// - Вызывающий должен обеспечить, что длина должна быть равна количеству аллоцированных байт.
    unsafe fn thin_as_bytes(&self, len: usize) -> &[u8];
}

////////////////////////////////////////////////////////////

/// Указатель на аллоцированные в куче данные
/// вида `u8*`.
#[repr(C)]
#[allow(missing_debug_implementations)]
pub struct BoxDynBytes {
    /// Указатель на данные в куче, указатель имеет тип `u8*`.
    ptr: NonNull<u8>,

    /// Дополнительный фантомный тип данных, чтобы указать, что мы владеем
    /// здесь слайсом данных, но без указания конкретного размера -
    /// что нам как раз и нужно.
    ///
    /// Для указания размера использовался бы fat-reference вида `&[u8]`.
    phantom: PhantomData<[u8]>,
}

/// # Safety:
///
/// + `UniqueDynBytes` is the only owner of its data.
unsafe impl Send for BoxDynBytes {}

/// # Safety:
///
/// + `UniqueDynBytes` is immutable.
unsafe impl Sync for BoxDynBytes {}

impl From<&[u8]> for BoxDynBytes {
    fn from(bytes: &[u8]) -> Self {
        let ptr = if bytes.is_empty() {
            NonNull::dangling()
        } else {
            let layout = array_layout::<u8>(bytes.len());
            // Safety:
            // + Our layout is always guaranteed to be of a non-zero sized type due to the if
            // statement that we have.
            let nullable = unsafe { std::alloc::alloc(layout) };
            let Some(ptr) = NonNull::new(nullable) else {
                std::alloc::handle_alloc_error(layout);
            };
            // Safety:
            // + We are copying `bytes.len()` bytes into a buffer of the same size that we just
            // allocated.
            unsafe {
                std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr.as_ptr(), bytes.len());
            }
            ptr
        };
        Self {
            ptr,
            phantom: PhantomData,
        }
    }
}

impl From<Vec<u8>> for BoxDynBytes {
    fn from(bytes: Vec<u8>) -> Self {
        let ptr = if bytes.is_empty() {
            NonNull::dangling()
        } else {
            // Safety:
            // + We create a `NonNull` from the result of `Box::into_raw` which is guaranteed to be
            // non-null and aligned.
            unsafe { NonNull::new_unchecked(Box::into_raw(bytes.into_boxed_slice()).cast()) }
        };
        Self {
            ptr,
            phantom: PhantomData,
        }
    }
}

unsafe impl ThinDrop for BoxDynBytes {
    unsafe fn thin_drop(&self, len: usize) {
        if len > 0 {
            // Safety:
            // + We only allocate using the default global allocator.
            // + We require that the caller passes in a `len` matching the number of allocated bytes.
            unsafe {
                std::alloc::dealloc(self.ptr.as_ptr(), array_layout::<u8>(len));
            }
        }
    }
}

unsafe impl ThinClone for BoxDynBytes {
    unsafe fn thin_clone(&self, len: usize) -> Self {
        let ptr = if len == 0 {
            NonNull::dangling()
        } else {
            let layout = array_layout::<u8>(len);
            // Safety:
            // + Our layout is always guaranteed to be of a non-zero sized type due to the if
            // statement that we have.
            let nullable = unsafe { std::alloc::alloc(layout) };
            let Some(ptr) = NonNull::new(nullable) else {
                std::alloc::handle_alloc_error(layout);
            };
            // Safety:
            // + We require the caller to pass in a valid `len` corresponding to the number of
            // allocated bytes.
            // + We are copying `len` bytes into a buffer of the same size that we just
            // allocated.
            unsafe {
                std::ptr::copy_nonoverlapping(self.ptr.as_ptr(), ptr.as_ptr(), len);
            }
            ptr
        };
        Self {
            ptr,
            phantom: PhantomData,
        }
    }
}

unsafe impl ThinAsBytes for BoxDynBytes {
    #[inline]
    unsafe fn thin_as_bytes(&self, len: usize) -> &[u8] {
        if len == 0 {
            Default::default()
        } else {
            // Safety:
            // + We ensure that the pointer is aligned and the data it points to is properly
            // initialized.
            // + We have access to `&self`, thus the bytes have not been deallocated.
            // + We return a slice having the same lifetime as `&self`.
            std::slice::from_raw_parts(self.ptr.as_ptr(), len)
        }
    }
}

#[repr(C)]
struct ArcDynBytesInner<T: ?Sized> {
    count: AtomicUsize,
    data: T,
}

impl<T> ArcDynBytesInner<[T]> {
    #[inline]
    fn cast(ptr: *mut T, len: usize) -> *mut Self {
        // Type-casting magic to create a fat pointer to a dynamically sized type.
        let fake_slice = std::ptr::slice_from_raw_parts_mut(ptr, len);
        fake_slice as *mut Self
    }
}

#[repr(C)]
#[allow(missing_debug_implementations)]
pub struct ArcDynBytes {
    ptr: NonNull<ArcDynBytesInner<[u8; 0]>>,
    phantom: PhantomData<ArcDynBytesInner<[u8]>>,
}

/// # Safety:
///
/// + `SharedDynBytes` keeps track of the number of references to its data using an atomic counter and
///   allows shared ownership across threads.
unsafe impl Send for ArcDynBytes {}

/// # Safety:
///
/// + `SharedDynBytes` is immutable.
unsafe impl Sync for ArcDynBytes {}

impl From<&[u8]> for ArcDynBytes {
    fn from(bytes: &[u8]) -> Self {
        let ptr = if bytes.is_empty() {
            NonNull::dangling()
        } else {
            let layout = arc_dyn_bytes_inner_layout(bytes.len());
            // Safety:
            // + Our layout is always guaranteed to be of a non-zero sized type due to the if
            // statement that we have.
            let nullable = unsafe { std::alloc::alloc(layout) };
            let nullable_fat_ptr = ArcDynBytesInner::<[u8]>::cast(nullable, bytes.len());
            let Some(fat_ptr) = NonNull::new(nullable_fat_ptr) else {
                std::alloc::handle_alloc_error(layout)
            };
            // Safety:
            // + We just allocated for a new `ArcDynBytesInner<[T]>` with enough space to
            // contain `len` bytes.
            // + We require the caller to pass in a valid `len` corresponding to the number of
            // allocated bytes.
            // + We are copying `len` bytes into a buffer of the same size.
            unsafe {
                std::ptr::write(
                    std::ptr::addr_of_mut!((*fat_ptr.as_ptr()).count),
                    AtomicUsize::new(1),
                );
                std::ptr::copy_nonoverlapping(
                    bytes.as_ptr(),
                    std::ptr::addr_of_mut!((*fat_ptr.as_ptr()).data).cast(),
                    bytes.len(),
                );
            }
            fat_ptr.cast()
        };
        Self {
            ptr,
            phantom: PhantomData,
        }
    }
}

impl From<Vec<u8>> for ArcDynBytes {
    #[inline]
    fn from(bytes: Vec<u8>) -> Self {
        Self::from(&bytes[..])
    }
}

unsafe impl ThinDrop for ArcDynBytes {
    unsafe fn thin_drop(&self, len: usize) {
        if len > 0 {
            // Safety:
            // + We have access to `&self`, thus the pointer has not been deallocated.
            let inner = unsafe { &*self.ptr.as_ptr() };
            if inner.count.fetch_sub(1, atomic::Ordering::Release) == 1 {
                inner.count.load(atomic::Ordering::Acquire);
                // Safety:
                // + We require that the caller passes in a `len` matching the number of allocated bytes.
                unsafe {
                    std::alloc::dealloc(
                        self.ptr.as_ptr().cast::<u8>(),
                        arc_dyn_bytes_inner_layout(len),
                    );
                };
            }
        }
    }
}

unsafe impl ThinClone for ArcDynBytes {
    unsafe fn thin_clone(&self, len: usize) -> Self {
        let ptr = if len == 0 {
            NonNull::dangling()
        } else {
            // Safety:
            // + We never deallocate the pointer if the reference count is at least 1.
            // + We can deference the pointer because we are accessing it through a reference to
            // [`SharedDynBytes`] which means the reference count is at least 1.
            let inner = unsafe { &*self.ptr.as_ptr() };
            inner.count.fetch_add(1, atomic::Ordering::Relaxed);
            self.ptr
        };

        Self {
            ptr,
            phantom: PhantomData,
        }
    }
}

unsafe impl ThinAsBytes for ArcDynBytes {
    #[inline]
    unsafe fn thin_as_bytes(&self, len: usize) -> &[u8] {
        if len == 0 {
            Default::default()
        } else {
            let fat_ptr = ArcDynBytesInner::<[u8]>::cast(self.ptr.as_ptr().cast::<u8>(), len);
            // Safety:
            // + We have access to `&self`, thus the pointer has not been deallocated.
            let ptr = unsafe { (*fat_ptr).data.as_ptr() };
            // Safety:
            // + We have access to `&self`, thus the bytes have not been deallocated.
            // + We require that the caller passes a valid length for the slice.
            unsafe { std::slice::from_raw_parts(ptr, len) }
        }
    }
}

#[repr(C)]
struct RcDynBytesInner<T: ?Sized> {
    count: Cell<usize>,
    data: T,
}

impl<T> RcDynBytesInner<[T]> {
    #[inline]
    fn cast(ptr: *mut T, len: usize) -> *mut Self {
        // Type-casting magic to create a fat pointer to a dynamically sized type.
        let fake_slice = std::ptr::slice_from_raw_parts_mut(ptr, len);
        fake_slice as *mut Self
    }
}

#[repr(C)]
#[allow(missing_debug_implementations)]
pub struct RcDynBytes {
    ptr: NonNull<RcDynBytesInner<[u8; 0]>>,
    phantom: PhantomData<RcDynBytesInner<[u8]>>,
}

impl From<&[u8]> for RcDynBytes {
    fn from(bytes: &[u8]) -> Self {
        let ptr = if bytes.is_empty() {
            NonNull::dangling()
        } else {
            let layout = rc_dyn_bytes_inner_layout(bytes.len());
            // Safety:
            // + Our layout is always guaranteed to be of a non-zero sized type due to the if
            // statement that we have.
            let nullable = unsafe { std::alloc::alloc(layout) };
            let nullable_fat_ptr = RcDynBytesInner::<[u8]>::cast(nullable, bytes.len());
            let Some(fat_ptr) = NonNull::new(nullable_fat_ptr) else {
                std::alloc::handle_alloc_error(layout)
            };
            // Safety:
            // + We just allocated for a new `RcDynBytesInner<[T]>` with enough space to
            // contain `len` bytes.
            // + We require the caller to pass in a valid `len` corresponding to the number of
            // allocated bytes.
            // + We are copying `len` bytes into a buffer of the same size.
            unsafe {
                std::ptr::write(
                    std::ptr::addr_of_mut!((*fat_ptr.as_ptr()).count),
                    Cell::new(1),
                );
                std::ptr::copy_nonoverlapping(
                    bytes.as_ptr(),
                    std::ptr::addr_of_mut!((*fat_ptr.as_ptr()).data).cast(),
                    bytes.len(),
                );
            }
            fat_ptr.cast()
        };
        Self {
            ptr,
            phantom: PhantomData,
        }
    }
}

impl From<Vec<u8>> for RcDynBytes {
    #[inline]
    fn from(bytes: Vec<u8>) -> Self {
        Self::from(&bytes[..])
    }
}

unsafe impl ThinDrop for RcDynBytes {
    unsafe fn thin_drop(&self, len: usize) {
        if len > 0 {
            // Safety:
            // + We have access to `&self`, thus the pointer has not been deallocated.
            let inner = unsafe { &*self.ptr.as_ptr() };
            let ref_count = inner.count.get();
            inner.count.set(ref_count - 1);
            if ref_count == 1 {
                // Safety:
                // + We require that the caller passes in a `len` matching the number of allocated bytes.
                unsafe {
                    std::alloc::dealloc(
                        self.ptr.as_ptr().cast::<u8>(),
                        rc_dyn_bytes_inner_layout(len),
                    );
                };
            }
        }
    }
}

unsafe impl ThinClone for RcDynBytes {
    unsafe fn thin_clone(&self, len: usize) -> Self {
        let ptr = if len == 0 {
            NonNull::dangling()
        } else {
            // Safety:
            // + We never deallocate the pointer if the reference count is at least 1.
            // + We can deference the pointer because we are accessing it through a reference to
            // [`SharedDynBytes`] which means the reference count is at least 1.
            let inner = unsafe { &*self.ptr.as_ptr() };
            let ref_count = inner.count.get();
            inner.count.set(ref_count + 1);
            self.ptr
        };

        Self {
            ptr,
            phantom: PhantomData,
        }
    }
}

unsafe impl ThinAsBytes for RcDynBytes {
    #[inline]
    unsafe fn thin_as_bytes(&self, len: usize) -> &[u8] {
        if len == 0 {
            Default::default()
        } else {
            let fat_ptr = RcDynBytesInner::<[u8]>::cast(self.ptr.as_ptr().cast::<u8>(), len);
            // Safety:
            // + We have access to `&self`, thus the pointer has not been deallocated.
            let ptr = unsafe { (*fat_ptr).data.as_ptr() };
            // Safety:
            // + We have access to `&self`, thus the bytes have not been deallocated.
            // + We require that the caller passes a valid length for the slice.
            unsafe { std::slice::from_raw_parts(ptr, len) }
        }
    }
}

fn array_layout<T>(len: usize) -> Layout {
    Layout::array::<T>(len).expect("A valid layout for a byte array")
}

fn arc_dyn_bytes_inner_layout(len: usize) -> Layout {
    Layout::new::<ArcDynBytesInner<()>>()
        .extend(array_layout::<u8>(len))
        .expect("A valid layout for a ArcDynBytesInner")
        .0
        .pad_to_align()
}

fn rc_dyn_bytes_inner_layout(len: usize) -> Layout {
    Layout::new::<RcDynBytesInner<()>>()
        .extend(array_layout::<u8>(len))
        .expect("A valid layout for a RcDynBytesInner")
        .0
        .pad_to_align()
}

#[cfg(test)]
mod tests {
    use super::{ArcDynBytes, BoxDynBytes, RcDynBytes, ThinAsBytes, ThinClone, ThinDrop};

    #[test]
    fn create_box_dyn_bytes_from_empty_slice() {
        let data = [];
        let boxed = BoxDynBytes::from(&data[..]);
        unsafe {
            assert_eq!(&data, boxed.thin_as_bytes(data.len()));
            boxed.thin_drop(data.len());
        }
    }

    #[test]
    fn create_box_dyn_bytes_from_non_empty_slice() {
        let data = b"hello world";
        let boxed = BoxDynBytes::from(&data[..]);
        unsafe {
            assert_eq!(&data[..], boxed.thin_as_bytes(data.len()));
            boxed.thin_drop(data.len());
        }
    }

    #[test]
    fn create_box_dyn_bytes_from_empty_vec() {
        let data = Vec::new();
        let boxed = BoxDynBytes::from(data.clone());
        unsafe {
            assert_eq!(&data, boxed.thin_as_bytes(data.len()));
            boxed.thin_drop(data.len());
        }
    }

    #[test]
    fn create_box_dyn_bytes_from_non_empty_vec() {
        let data = Vec::from(b"hello world");
        let boxed = BoxDynBytes::from(data.clone());
        unsafe {
            assert_eq!(&data, boxed.thin_as_bytes(data.len()));
            boxed.thin_drop(data.len());
        }
    }

    #[test]
    fn create_arc_dyn_bytes_from_empty_slice() {
        let data = [];
        let arc = ArcDynBytes::from(&data[..]);
        unsafe {
            assert_eq!(&data, arc.thin_as_bytes(data.len()));
            arc.thin_drop(data.len());
        }
    }

    #[test]
    fn create_arc_dyn_bytes_from_non_empty_slice() {
        let data = b"hello world";
        let arc = ArcDynBytes::from(&data[..]);
        unsafe {
            assert_eq!(&data[..], arc.thin_as_bytes(data.len()));
            arc.thin_drop(data.len());
        }
    }

    #[test]
    fn create_arc_dyn_bytes_from_empty_vec() {
        let data = Vec::new();
        let arc = ArcDynBytes::from(data.clone());
        unsafe {
            assert_eq!(&data, arc.thin_as_bytes(data.len()));
            arc.thin_drop(data.len());
        }
    }

    #[test]
    fn create_arc_dyn_bytes_from_non_empty_vec() {
        let data = Vec::from(b"hello world");
        let arc = ArcDynBytes::from(data.clone());
        unsafe {
            assert_eq!(&data, arc.thin_as_bytes(data.len()));
            arc.thin_drop(data.len());
        }
    }

    #[test]
    fn create_rc_dyn_bytes_from_empty_slice() {
        let data = [];
        let rc = RcDynBytes::from(&data[..]);
        unsafe {
            assert_eq!(&data, rc.thin_as_bytes(data.len()));
            rc.thin_drop(data.len());
        }
    }

    #[test]
    fn create_rc_dyn_bytes_from_non_empty_slice() {
        let data = b"hello world";
        let rc = RcDynBytes::from(&data[..]);
        unsafe {
            assert_eq!(&data[..], rc.thin_as_bytes(data.len()));
            rc.thin_drop(data.len());
        }
    }

    #[test]
    fn create_rc_dyn_bytes_from_empty_vec() {
        let data = Vec::new();
        let rc = RcDynBytes::from(data.clone());
        unsafe {
            assert_eq!(&data, rc.thin_as_bytes(data.len()));
            rc.thin_drop(data.len());
        }
    }

    #[test]
    fn create_rc_dyn_bytes_from_non_empty_vec() {
        let data = Vec::from(b"hello world");
        let rc = RcDynBytes::from(data.clone());
        unsafe {
            assert_eq!(&data, rc.thin_as_bytes(data.len()));
            rc.thin_drop(data.len());
        }
    }

    #[test]
    fn clone_box_dyn_bytes_empty() {
        let data = [];
        let boxed0 = BoxDynBytes::from(&data[..]);
        let boxed1 = unsafe { boxed0.thin_clone(data.len()) };
        unsafe {
            assert_eq!(&data, boxed0.thin_as_bytes(data.len()));
            assert_eq!(&data, boxed1.thin_as_bytes(data.len()));
            boxed0.thin_drop(data.len());
            boxed1.thin_drop(data.len());
        }
    }

    #[test]
    fn clone_box_dyn_bytes_non_empty() {
        let data = b"hello world";
        let boxed0 = BoxDynBytes::from(&data[..]);
        let boxed1 = unsafe { boxed0.thin_clone(data.len()) };
        unsafe {
            assert_eq!(&data[..], boxed0.thin_as_bytes(data.len()));
            assert_eq!(&data[..], boxed1.thin_as_bytes(data.len()));
            boxed0.thin_drop(data.len());
            boxed1.thin_drop(data.len());
        }
    }

    #[test]
    fn clone_arc_dyn_bytes_empty() {
        let data = [];
        let arc0 = ArcDynBytes::from(&data[..]);
        let arc1 = unsafe { arc0.thin_clone(data.len()) };
        unsafe {
            assert_eq!(&data, arc0.thin_as_bytes(data.len()));
            assert_eq!(&data, arc1.thin_as_bytes(data.len()));
            arc0.thin_drop(data.len());
            arc1.thin_drop(data.len());
        }
    }

    #[test]
    fn clone_arc_dyn_bytes_non_empty() {
        let data = b"hello world";
        let arc0 = ArcDynBytes::from(&data[..]);
        let arc1 = unsafe { arc0.thin_clone(data.len()) };
        unsafe {
            assert_eq!(&data[..], arc0.thin_as_bytes(data.len()));
            assert_eq!(&data[..], arc1.thin_as_bytes(data.len()));
            arc0.thin_drop(data.len());
            arc1.thin_drop(data.len());
        }
    }

    #[test]
    fn clone_rc_dyn_bytes_empty() {
        let data = [];
        let rc0 = RcDynBytes::from(&data[..]);
        let rc1 = unsafe { rc0.thin_clone(data.len()) };
        unsafe {
            assert_eq!(&data, rc0.thin_as_bytes(data.len()));
            assert_eq!(&data, rc1.thin_as_bytes(data.len()));
            rc0.thin_drop(data.len());
            rc1.thin_drop(data.len());
        }
    }

    #[test]
    fn clone_rc_dyn_bytes_non_empty() {
        let data = b"hello world";
        let rc0 = RcDynBytes::from(&data[..]);
        let rc1 = unsafe { rc0.thin_clone(data.len()) };
        unsafe {
            assert_eq!(&data[..], rc0.thin_as_bytes(data.len()));
            assert_eq!(&data[..], rc1.thin_as_bytes(data.len()));
            rc0.thin_drop(data.len());
            rc1.thin_drop(data.len());
        }
    }
}
