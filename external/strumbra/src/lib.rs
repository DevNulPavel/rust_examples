//! Реализация для строковой структуры данных, описанной в статье [Umbra: A Disk-Based System with In-Memory Performance].
//!
//! [Umbra: A Disk-Based System with In-Memory Performance]: https://www.cidrdb.org/cidr2020/papers/p29-neumann-cidr20.pdf

////////////////////////////////////////////////////////////

#![warn(
    rustdoc::all,
    clippy::cargo,
    clippy::pedantic,
    clippy::nursery,
    missing_debug_implementations
)]
#![deny(clippy::all, missing_docs, rust_2018_idioms, rust_2021_compatibility)]

////////////////////////////////////////////////////////////

#[cfg(feature = "serde")]
pub mod serde;

////////////////////////////////////////////////////////////

mod heap;

////////////////////////////////////////////////////////////

use heap::{ArcDynBytes, BoxDynBytes, RcDynBytes, ThinAsBytes, ThinClone, ThinDrop};
use std::{borrow::Borrow, cmp, mem::ManuallyDrop};

////////////////////////////////////////////////////////////

const INLINED_LENGTH: usize = 12;
const PREFIX_LENGTH: usize = 4;
const SUFFIX_LENGTH: usize = 8;

////////////////////////////////////////////////////////////

/// Тип для всех возможных ошибок, которые могут возникнуть при использовании [`UmbraString`].
#[derive(Debug)]
pub enum Error {
    /// Ошибка возникает при конвертации из строки, чья длина превышает максимальное значение
    /// 32-х битного беззнакоаого значения.
    TooLong,
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooLong => write!(f, "string is too long"),
        }
    }
}

////////////////////////////////////////////////////////////

/// Реальный юнион для представления в в памяти либо данных на стеке,
/// либо указателя, который будет вручную уничтожаться
#[repr(C)]
union Trailing<B> {
    /// Буфер данных на стеке
    buf: [u8; SUFFIX_LENGTH],

    /// Либо указатель для уничтожения данных вручную
    ptr: ManuallyDrop<B>,
}

/// # Safety:
///
/// - Встраиваемый контент на стеке всегда копируется
/// - Аллоцированные в куче данные у нас реализуют `Send` если данные `Send` + `Sync`
unsafe impl<B> Send for Trailing<B> where B: Send + Sync {}

/// # Safety:
///
/// - Встраиваемый контент у нас является неизменяемым из-за `Sync` (?)
/// - Аллоцированные в куче данные у нас реализуют `Sync` если данные `Send` + `Sync`
unsafe impl<B> Sync for Trailing<B> where B: Send + Sync {}

////////////////////////////////////////////////////////////

/// Umbra-style строка, которая владеет принадлежащими
/// ей байтами и не шарит данные среди разных инстансов.
pub type BoxString = UmbraString<BoxDynBytes>;

/// Umbra-style строка, которая шарит управляемые данные между потоками и
/// отслеживает количество атомарных ссылок
/// на эту самую строку.
pub type ArcString = UmbraString<ArcDynBytes>;

/// Umbra-style строка, которая шарит управляемые данные между владельцами в пределах
/// одного потока и отслеживает количество ссылок
/// на эту самую строку.
pub type RcString = UmbraString<RcDynBytes>;

/// Umbra-style строка, которая владеет принадлежащими
/// ей байтами и не шарит данные среди разных инстансов.
#[deprecated(since = "0.5.0", note = "please use `BoxString` instead")]
pub type UniqueString = BoxString;

/// Umbra-style строка, которая шарит управляемые данные между потоками и
/// отслеживает количество атомарных ссылок
/// на эту самую строку.
#[deprecated(since = "0.5.0", note = "please use `ArcString` instead")]
pub type SharedString = ArcString;

////////////////////////////////////////////////////////////

/// Строковая структура данных, оптимизированная для аналитической обработки.
///
/// В отличие от обычной [`String`], которая использует 24 байта на стеке, данная структура данных
/// хранит на стеке лишь только 16 байт и является неизменяемой.
#[repr(C)]
pub struct UmbraString<B: ThinDrop> {
    /// Размер строки, 4 байта
    len: u32,

    /// Данные на стеке для префикса, 4 байта
    prefix: [u8; PREFIX_LENGTH],

    /// Конец данных, 8 байт
    trailing: Trailing<B>,
}



/// # Safety:
///
/// + `len` is always copied.
/// + The heap-allocated bytes are `Send`.
unsafe impl<B> Send for UmbraString<B> where B: ThinDrop + Send + Sync {}

/// # Safety:
///
/// + `len` is immutable.
/// + The heap-allocated bytes are `Sync`.
unsafe impl<B> Sync for UmbraString<B> where B: ThinDrop + Send + Sync {}

impl<B> Drop for UmbraString<B>
where
    B: ThinDrop,
{
    fn drop(&mut self) {
        if self.len() > INLINED_LENGTH {
            // Safety:
            // + We know that the string is heap-allocated because len > INLINED_LENGTH.
            // + We never modify `len`, thus it always equals to the number of allocated bytes.
            unsafe {
                self.trailing.ptr.thin_drop(self.len());
            }
        }
    }
}

impl<B> Clone for UmbraString<B>
where
    B: ThinDrop + ThinClone,
{
    fn clone(&self) -> Self {
        let trailing = if self.len() <= INLINED_LENGTH {
            // Safety:
            // + We know that the string is inlined because len <= INLINED_LENGTH.
            unsafe {
                Trailing {
                    buf: self.trailing.buf,
                }
            }
        } else {
            // Safety:
            // + We know that the string is heap-allocated because len > INLINED_LENGTH.
            unsafe {
                Trailing {
                    ptr: ManuallyDrop::new(self.trailing.ptr.thin_clone(self.len())),
                }
            }
        };
        Self {
            len: self.len,
            prefix: self.prefix,
            trailing,
        }
    }
}

impl<B> TryFrom<&str> for UmbraString<B>
where
    B: ThinDrop + for<'a> From<&'a [u8]>,
{
    type Error = Error;

    #[inline]
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Self::new(s, |s| B::from(s.as_bytes()))
    }
}

impl<B> TryFrom<&String> for UmbraString<B>
where
    B: ThinDrop + for<'a> From<&'a [u8]>,
{
    type Error = Error;

    #[inline]
    fn try_from(s: &String) -> Result<Self, Self::Error> {
        Self::new(s, |s| B::from(s.as_bytes()))
    }
}

impl<B> TryFrom<String> for UmbraString<B>
where
    B: ThinDrop + From<Vec<u8>>,
{
    type Error = Error;

    #[inline]
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::new(s, |s| B::from(s.into_bytes()))
    }
}

impl<B> std::ops::Deref for UmbraString<B>
where
    B: ThinDrop + ThinAsBytes,
{
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl<B> AsRef<str> for UmbraString<B>
where
    B: ThinDrop + ThinAsBytes,
{
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<B> Borrow<str> for UmbraString<B>
where
    B: ThinDrop + ThinAsBytes,
{
    #[inline]
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl<B> std::hash::Hash for UmbraString<B>
where
    B: ThinDrop + ThinAsBytes,
{
    #[inline]
    fn hash<H>(&self, hasher: &mut H)
    where
        H: std::hash::Hasher,
    {
        self.as_str().hash(hasher);
    }
}

impl<B> Eq for UmbraString<B> where B: ThinDrop + ThinAsBytes {}
impl<B1, B2> PartialEq<UmbraString<B2>> for UmbraString<B1>
where
    B1: ThinDrop + ThinAsBytes,
    B2: ThinDrop + ThinAsBytes,
{
    fn eq(&self, other: &UmbraString<B2>) -> bool {
        let lhs_first_qword = std::ptr::from_ref(self).cast::<u64>();
        let rhs_first_qword = std::ptr::from_ref(other).cast::<u64>();
        // Safety:
        // + The pointers are obtained from the given references and guaranteed to be non-null and
        // properly aligned.
        // + The first QWORD contains the string length and prefix based on the layout, guaranteed
        // by `#[repr(C)]`.
        // + The referenced objects are immutable and are not changed concurrently.
        if unsafe { *lhs_first_qword != *rhs_first_qword } {
            return false;
        }
        if self.len() <= INLINED_LENGTH {
            // Safety:
            // + We know that the string is inlined because len <= INLINED_LENGTH.
            return unsafe { self.trailing.buf == other.trailing.buf };
        }
        self.suffix() == other.suffix()
    }
}

impl<B> PartialEq<str> for UmbraString<B>
where
    B: ThinDrop + ThinAsBytes,
{
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

impl<B> PartialEq<UmbraString<B>> for str
where
    B: ThinDrop + ThinAsBytes,
{
    #[inline]
    fn eq(&self, other: &UmbraString<B>) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

impl<B> PartialEq<String> for UmbraString<B>
where
    B: ThinDrop + ThinAsBytes,
{
    #[inline]
    fn eq(&self, other: &String) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

impl<B> PartialEq<UmbraString<B>> for String
where
    B: ThinDrop + ThinAsBytes,
{
    #[inline]
    fn eq(&self, other: &UmbraString<B>) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

impl<B> Ord for UmbraString<B>
where
    B: ThinDrop + ThinAsBytes,
{
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        Self::cmp(self, other)
    }
}

impl<B1, B2> PartialOrd<UmbraString<B2>> for UmbraString<B1>
where
    B1: ThinDrop + ThinAsBytes,
    B2: ThinDrop + ThinAsBytes,
{
    #[inline]
    fn partial_cmp(&self, other: &UmbraString<B2>) -> Option<cmp::Ordering> {
        Some(Self::cmp(self, other))
    }
}

impl<B> PartialOrd<str> for UmbraString<B>
where
    B: ThinDrop + ThinAsBytes,
{
    #[inline]
    fn partial_cmp(&self, other: &str) -> Option<cmp::Ordering> {
        PartialOrd::partial_cmp(self.as_bytes(), other.as_bytes())
    }
}

impl<B> PartialOrd<UmbraString<B>> for str
where
    B: ThinDrop + ThinAsBytes,
{
    #[inline]
    fn partial_cmp(&self, other: &UmbraString<B>) -> Option<cmp::Ordering> {
        PartialOrd::partial_cmp(self.as_bytes(), other.as_bytes())
    }
}

impl<B> PartialOrd<String> for UmbraString<B>
where
    B: ThinDrop + ThinAsBytes,
{
    #[inline]
    fn partial_cmp(&self, other: &String) -> Option<cmp::Ordering> {
        PartialOrd::partial_cmp(self.as_bytes(), other.as_bytes())
    }
}

impl<B> PartialOrd<UmbraString<B>> for String
where
    B: ThinDrop + ThinAsBytes,
{
    #[inline]
    fn partial_cmp(&self, other: &UmbraString<B>) -> Option<cmp::Ordering> {
        PartialOrd::partial_cmp(self.as_bytes(), other.as_bytes())
    }
}

impl<B> std::fmt::Display for UmbraString<B>
where
    B: ThinDrop + ThinAsBytes,
{
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl<B> std::fmt::Debug for UmbraString<B>
where
    B: ThinDrop + ThinAsBytes,
{
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.as_str())
    }
}

impl<B> UmbraString<B>
where
    B: ThinDrop,
{
    /// Returns the length of `self`.
    ///
    /// This length is in bytes, not [`char`]s or graphemes. In other words,
    /// it might not be what a human considers the length of the string.
    #[inline]
    pub const fn len(&self) -> usize {
        self.len as usize
    }

    /// Returns `true` if `self` has a length of zero bytes.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    fn new<S, A>(s: S, alloc: A) -> Result<Self, Error>
    where
        S: AsRef<str>,
        A: FnOnce(S) -> B,
    {
        let bytes = s.as_ref().as_bytes();
        let len = bytes.len();
        if len > u32::MAX as usize {
            return Err(Error::TooLong);
        }
        let mut prefix = [0u8; PREFIX_LENGTH];
        let trailing = if len <= INLINED_LENGTH {
            let mut buf = [0u8; SUFFIX_LENGTH];
            if len <= PREFIX_LENGTH {
                prefix[..len].copy_from_slice(&bytes[..len]);
            } else {
                prefix.copy_from_slice(&bytes[..PREFIX_LENGTH]);
                buf[..len - PREFIX_LENGTH].copy_from_slice(&bytes[PREFIX_LENGTH..]);
            }
            Trailing { buf }
        } else {
            prefix.copy_from_slice(&bytes[..PREFIX_LENGTH]);
            Trailing {
                ptr: ManuallyDrop::new(alloc(s)),
            }
        };
        #[allow(clippy::cast_possible_truncation)]
        Ok(Self {
            len: len as u32,
            prefix,
            trailing,
        })
    }
}
impl<B> UmbraString<B>
where
    B: ThinDrop + ThinAsBytes,
{
    /// Converts `self` to a byte slice.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        if self.len() <= INLINED_LENGTH {
            // Note: If we cast from a reference to a pointer, we can only access memory that was
            // within the bounds of the reference. This is done to satisfied miri when we create a
            // slice starting from the pointer of self.prefix to access data beyond it.
            let ptr = std::ptr::from_ref(self);
            // Safety:
            // + We know that the string is inlined because len <= INLINED_LENGTH.
            // + We can create a slice starting from the pointer to self.prefix with a length of at
            // most PREFIX_LENGTH because we have an inlined suffix of 8 bytes after the prefix.
            unsafe {
                std::slice::from_raw_parts(std::ptr::addr_of!((*ptr).prefix).cast(), self.len())
            }
        } else {
            // Safety:
            // + We know that the string is heap-allocated because len > INLINED_LENGTH.
            // + We never modify `len`, thus it always equals to the number of allocated bytes.
            unsafe { self.trailing.ptr.thin_as_bytes(self.len()) }
        }
    }

    /// Extracts a string slice containing the entire [`UmbraString`].
    #[inline]
    pub fn as_str(&self) -> &str {
        // Safety:
        // + We always construct the string using valid UTF-8 bytes.
        unsafe { std::str::from_utf8_unchecked(self.as_bytes()) }
    }

    #[inline]
    fn suffix(&self) -> &[u8] {
        if self.len() <= INLINED_LENGTH {
            // Safety:
            // + We know that the string is inlined because len <= INLINED_LENGTH.
            let suffix_len = self.len().saturating_sub(PREFIX_LENGTH);
            unsafe { self.trailing.buf.get_unchecked(..suffix_len) }
        } else {
            // Safety:
            // + We know that the string is heap-allocated because len > INLINED_LENGTH.
            // + We never modify `len`, thus it always equals to the number of allocated bytes.
            // + We can slice into the bytes without bound checks because len > PREFIX_LENGTH.
            unsafe {
                self.trailing
                    .ptr
                    .thin_as_bytes(self.len())
                    .get_unchecked(PREFIX_LENGTH..)
            }
        }
    }

    fn cmp<BB>(lhs: &Self, rhs: &UmbraString<BB>) -> cmp::Ordering
    where
        BB: ThinDrop + ThinAsBytes,
    {
        let prefix_ordering = Ord::cmp(&lhs.prefix, &rhs.prefix);
        if prefix_ordering != cmp::Ordering::Equal {
            return prefix_ordering;
        }
        if lhs.len() <= PREFIX_LENGTH && rhs.len() <= PREFIX_LENGTH {
            return Ord::cmp(&lhs.len, &rhs.len);
        }
        if lhs.len() <= INLINED_LENGTH && rhs.len() <= INLINED_LENGTH {
            // Safety:
            // + We know that the string is inlined because len <= INLINED_LENGTH.
            let suffix_ordering = unsafe { Ord::cmp(&lhs.trailing.buf, &rhs.trailing.buf) };
            if suffix_ordering != cmp::Ordering::Equal {
                return suffix_ordering;
            }
            return Ord::cmp(&lhs.len, &rhs.len);
        }
        Ord::cmp(lhs.suffix(), rhs.suffix())
    }
}
