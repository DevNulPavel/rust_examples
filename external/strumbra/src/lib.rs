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
/// - `len` переменная всегда копируется, так как она на стеке у нас
/// - Аллоцированные в куче данные у нас являются `Send` и могут прыгать между потоками.
unsafe impl<B> Send for UmbraString<B> where B: ThinDrop + Send + Sync {}

/// # Safety:
///
/// - `len` переменная является неизменяемой и лежит на стеке
/// - Данные, которые у нас аллоцированы в куче являются `Sync` и позволяют доступ по ссылке
///   из разных потоков.
unsafe impl<B> Sync for UmbraString<B> where B: ThinDrop + Send + Sync {}

/// Кастомная реализация уничтожения строки для типов, которые реализуют у нас
/// специально трейт `ThinDrop`
impl<B> Drop for UmbraString<B>
where
    B: ThinDrop,
{
    fn drop(&mut self) {
        // Проверяем размер данных, если они у нас в куче, тогда
        // нам надо применять уничтожение
        if self.len_in_bytes_usize() > INLINED_LENGTH {
            // Safety:
            // - Мы знаем, что строка у нас аллоцирована в куче здесь из-за условия выше
            // - Мы никогда не модифицируем длину,
            //   так что она всегда отражает размер именно данных реальных аллоцированных
            unsafe {
                self.trailing.ptr.thin_drop(self.len_in_bytes_usize());
            }
        }
    }
}

/// Кастомная реализация клонирования строки если у нас внутренний тип реализует
/// `ThinDrop` и `ThinClone` трейты одновременно
impl<B> Clone for UmbraString<B>
where
    B: ThinDrop + ThinClone,
{
    fn clone(&self) -> Self {
        // Проверяем, что у нас размер меньше или равен данным, которые у нас на стеке точно лежат
        let trailing = if self.len_in_bytes_usize() <= INLINED_LENGTH {
            // Safety:
            // - Мы знаем, что строка встроена из-за проверки выше
            unsafe {
                Trailing {
                    buf: self.trailing.buf,
                }
            }
        } else {
            // Safety:
            // - Мы знаем, что строка у нас в куче из-за проверки выше
            unsafe {
                // Выполняем клонирование данных в куче через трейт.
                // Размер клонируемых данных передаем как параметр.
                let cloned_heap = self.trailing.ptr.thin_clone(self.len_in_bytes_usize());

                // Создаем здесь обертку для ручной деаллокации
                Trailing {
                    ptr: ManuallyDrop::new(cloned_heap),
                }
            }
        };

        // Создаем новый объект-клон
        Self {
            len: self.len,
            prefix: self.prefix,
            trailing,
        }
    }
}

/// Поддержка конвертации из обычной строки для типов, которые
/// поддерживают `ThinDrop` + умеют создаваться из слайса байт с помощью
/// реализации `for<'a> From<&'a [u8]>`
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

/// Поддержка конвертации из аллоцированной строки для типов, которые
/// поддерживают `ThinDrop` + умеют создаваться из слайса байт с помощью
/// реализации `for<'a> From<&'a [u8]>`
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

/// Поддержка конвертации из аллоцированной строки для типов, которые
/// поддерживают `ThinDrop` + умеют создаваться из байт в куче с помощью
/// реализации `From<Vec<u8>>`
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

/// Реализуем `Deref` для строки, но при условии,
/// что мы можемпредставить внутренний тип как байты
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

/// Реализуем `AsRef<str>` для строки, но при условии,
/// что мы можемпредставить внутренний тип как байты
impl<B> AsRef<str> for UmbraString<B>
where
    B: ThinDrop + ThinAsBytes,
{
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

/// Реализуем `Borrow<str>` для строки, но при условии,
/// что мы можемпредставить внутренний тип как байты
impl<B> Borrow<str> for UmbraString<B>
where
    B: ThinDrop + ThinAsBytes,
{
    #[inline]
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

/// Поддержка хеширования для строки, но при условии,
/// что мы можем представить внутренний тип как байты
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

/// Реализация равенсnва для строки, где внутренний тип у нас реализует
/// `ThinDrop` + `ThinAsBytes`
impl<B> Eq for UmbraString<B> where B: ThinDrop + ThinAsBytes {}

/// Реализация частичного равенства двух типов, которые реализуют
/// возможность `ThinDrop` + `ThinAsBytes` для представления
/// как байты
impl<B1, B2> PartialEq<UmbraString<B2>> for UmbraString<B1>
where
    B1: ThinDrop + ThinAsBytes,
    B2: ThinDrop + ThinAsBytes,
{
    fn eq(&self, other: &UmbraString<B2>) -> bool {
        // Получаем указатели на текущее значение и на
        // другое сравниваемое значение.
        //
        // После чего конвертируем эти самые указатели просто в указатель на 8 байт первых
        // у структуры.
        let lhs_first_qword = std::ptr::from_ref(self).cast::<u64>();
        let rhs_first_qword = std::ptr::from_ref(other).cast::<u64>();

        // Safety:
        // - Указатели, полученные из данных ссылок точно не нулевые + выровнены правильно, так как
        //   вся работа была в обычном безопасном коде до этого.
        // - Первые 4 байта содержат размер строки и префикс, они точно у нас представлены
        //   в памяти именно так за счет указания `#[repr(C)]` для строки. Поэтому там не
        //   происходит никакого перемешивания порядка полей.
        // - Ссылки являются иммутабельными и никто их не изменяет в безопасном коде из потоков других
        //
        // Здесь мы просто сравниваем первые 8 байт и вторые 8 байт.
        // Если они не равны - значит точно не совпадают значения.
        if unsafe { *lhs_first_qword != *rhs_first_qword } {
            return false;
        }

        // На всякий случай дальше сверяем размер, что он меньше или равен тому,
        // что влезает у нас на стек
        if self.len_in_bytes_usize() <= INLINED_LENGTH {
            // Safety:
            // - Строка точно на стеке, проверка была выше
            //
            // Сравниваем здесь стековые буфферы тогда уже
            return unsafe { self.trailing.buf == other.trailing.buf };
        }

        // TODO: Проверить
        // Если мы в куче - тогда сравниваем уже данные в куче
        self.suffix() == other.suffix()
    }
}

/// Частичное сравнение просто со строкой,
/// но если текущий тип у нас реализует представление в виде байт.
impl<B> PartialEq<str> for UmbraString<B>
where
    B: ThinDrop + ThinAsBytes,
{
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

/// Сравнение локальной строки и слайса на строку, то есть в обратную сторону
impl<B> PartialEq<UmbraString<B>> for str
where
    B: ThinDrop + ThinAsBytes,
{
    #[inline]
    fn eq(&self, other: &UmbraString<B>) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

/// Сравнение со строкой в куче
impl<B> PartialEq<String> for UmbraString<B>
where
    B: ThinDrop + ThinAsBytes,
{
    #[inline]
    fn eq(&self, other: &String) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

/// Сравнение со строкой в куче обратное
impl<B> PartialEq<UmbraString<B>> for String
where
    B: ThinDrop + ThinAsBytes,
{
    #[inline]
    fn eq(&self, other: &UmbraString<B>) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

/// Реализация сортировки
impl<B> Ord for UmbraString<B>
where
    B: ThinDrop + ThinAsBytes,
{
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        Self::cmp(self, other)
    }
}

/// Реализация сортировки
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

/// Реализация сортировки
impl<B> PartialOrd<str> for UmbraString<B>
where
    B: ThinDrop + ThinAsBytes,
{
    #[inline]
    fn partial_cmp(&self, other: &str) -> Option<cmp::Ordering> {
        PartialOrd::partial_cmp(self.as_bytes(), other.as_bytes())
    }
}

/// Реализация сортировки
impl<B> PartialOrd<UmbraString<B>> for str
where
    B: ThinDrop + ThinAsBytes,
{
    #[inline]
    fn partial_cmp(&self, other: &UmbraString<B>) -> Option<cmp::Ordering> {
        PartialOrd::partial_cmp(self.as_bytes(), other.as_bytes())
    }
}

/// Реализация сортировки
impl<B> PartialOrd<String> for UmbraString<B>
where
    B: ThinDrop + ThinAsBytes,
{
    #[inline]
    fn partial_cmp(&self, other: &String) -> Option<cmp::Ordering> {
        PartialOrd::partial_cmp(self.as_bytes(), other.as_bytes())
    }
}

/// Реализация сортировки
impl<B> PartialOrd<UmbraString<B>> for String
where
    B: ThinDrop + ThinAsBytes,
{
    #[inline]
    fn partial_cmp(&self, other: &UmbraString<B>) -> Option<cmp::Ordering> {
        PartialOrd::partial_cmp(self.as_bytes(), other.as_bytes())
    }
}

/// Форматирование
impl<B> std::fmt::Display for UmbraString<B>
where
    B: ThinDrop + ThinAsBytes,
{
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Отладочный вывод
impl<B> std::fmt::Debug for UmbraString<B>
where
    B: ThinDrop + ThinAsBytes,
{
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.as_str())
    }
}

/// Непосредственно реализация нашего типа уже
impl<B> UmbraString<B>
where
    B: ThinDrop,
{
    /// Создание новой строки с помощью определенногоа аллокатора, который
    fn new<S, A>(s: S, alloc: A) -> Result<Self, Error>
    where
        S: AsRef<str>,
        A: FnOnce(S) -> B,
    {
        // Байты исходной строки
        let bytes = s.as_ref().as_bytes();

        // Размер в байтах у исходной строки
        let len = bytes.len();

        // Поддерживаются строки размером не более 32-х бит
        if len > u32::MAX as usize {
            return Err(Error::TooLong);
        }

        // Формируем на стеке префикс сначала
        let mut prefix = [0u8; PREFIX_LENGTH];

        // Формируем буфер для непосредственно байт
        // У нас буфер влезает на стек?
        let trailing = if len <= INLINED_LENGTH {
            // Раз буффер влезает на стек, то формируем буффер-суффикс
            let mut buf = [0u8; SUFFIX_LENGTH];

            // Если размер исходных данных меньше, чем размерность префикса даже
            if len <= PREFIX_LENGTH {
                // То мы можем просто в префикс скопировать входящие данные
                prefix[..len].copy_from_slice(&bytes[..len]);
            } else {
                // Раз у нас не влезает в префикс, тогда в префикс мы копируем начало данных
                prefix.copy_from_slice(&bytes[..PREFIX_LENGTH]);
                // После чего мы в буффер записываем оставшуюся часть данных.
                //
                // Таким образом, у нас исходные данные пусть и разделены на 2 части, но все еще
                // идут подряд одна за другой.
                buf[..len - PREFIX_LENGTH].copy_from_slice(&bytes[PREFIX_LENGTH..]);
            }

            // Результат
            Trailing { buf }
        } else {
            // Здесь нам ничего не остается как уже скопировать сначала данные в префикс сколько влезает
            prefix.copy_from_slice(&bytes[..PREFIX_LENGTH]);

            // TODO: Проверить
            // После чего уже аллоцируем оставшуюся часть в куче?
            Trailing {
                ptr: ManuallyDrop::new(alloc(s)),
            }
        };

        #[allow(clippy::cast_possible_truncation)]
        Ok(Self {
            // Здесь безопасно кастить размерность буффера,
            // так как проверки были выше
            len: len as u32,
            prefix,
            trailing,
        })
    }

    /// Выдает размерность текущего `self`.
    ///
    /// Данный размер выдается в количестве байт, не в количестве символов или графем.
    ///
    /// Другими словами, это побайтовая длина, а не длина самой строки, поэтому может возвращаться размер
    /// больше, чем количество символов строки.
    #[inline]
    pub const fn len_in_bytes_usize(&self) -> usize {
        self.len as usize
    }

    /// Выдает размерность текущего `self`.
    ///
    /// Данный размер выдается в количестве байт, не в количестве символов или графем.
    ///
    /// Другими словами, это побайтовая длина, а не длина самой строки, поэтому может возвращаться размер
    /// больше, чем количество символов строки.
    #[inline]
    pub const fn len_in_bytes_u32(&self) -> u32 {
        self.len
    }

    /// Пустая ли данная строка?
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl<B> UmbraString<B>
where
    B: ThinDrop + ThinAsBytes,
{
    continue

    /// Представляем данную строку как слайс байт.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        if self.len_in_bytes_usize() <= INLINED_LENGTH {
            // Note: If we cast from a reference to a pointer, we can only access memory that was
            // within the bounds of the reference. This is done to satisfied miri when we create a
            // slice starting from the pointer of self.prefix to access data beyond it.
            let ptr = std::ptr::from_ref(self);
            // Safety:
            // + We know that the string is inlined because len <= INLINED_LENGTH.
            // + We can create a slice starting from the pointer to self.prefix with a length of at
            // most PREFIX_LENGTH because we have an inlined suffix of 8 bytes after the prefix.
            unsafe {
                std::slice::from_raw_parts(
                    std::ptr::addr_of!((*ptr).prefix).cast(),
                    self.len_in_bytes_usize(),
                )
            }
        } else {
            // Safety:
            // + We know that the string is heap-allocated because len > INLINED_LENGTH.
            // + We never modify `len`, thus it always equals to the number of allocated bytes.
            unsafe { self.trailing.ptr.thin_as_bytes(self.len_in_bytes_usize()) }
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
        if self.len_in_bytes_usize() <= INLINED_LENGTH {
            // Safety:
            // + We know that the string is inlined because len <= INLINED_LENGTH.
            let suffix_len = self.len_in_bytes_usize().saturating_sub(PREFIX_LENGTH);
            unsafe { self.trailing.buf.get_unchecked(..suffix_len) }
        } else {
            // Safety:
            // + We know that the string is heap-allocated because len > INLINED_LENGTH.
            // + We never modify `len`, thus it always equals to the number of allocated bytes.
            // + We can slice into the bytes without bound checks because len > PREFIX_LENGTH.
            unsafe {
                self.trailing
                    .ptr
                    .thin_as_bytes(self.len_in_bytes_usize())
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
        if lhs.len_in_bytes_usize() <= PREFIX_LENGTH && rhs.len_in_bytes_usize() <= PREFIX_LENGTH {
            return Ord::cmp(&lhs.len, &rhs.len);
        }
        if lhs.len_in_bytes_usize() <= INLINED_LENGTH && rhs.len_in_bytes_usize() <= INLINED_LENGTH
        {
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
