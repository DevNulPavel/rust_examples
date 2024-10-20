use std::mem;
pub use heap_size_derive::*;

pub trait HeapSize {
    /// Вычисляем потраченную память в куче(???) для данного типа 
    /// Не включает размер `self` потому что неизвестно в куче или нет self объект
    /// Включает только детей
    fn heap_size_of_children(&self) -> usize;
}

// Реализации для основных типов

impl HeapSize for u8 {
    /// A `u8` does not own any heap memory.
    fn heap_size_of_children(&self) -> usize {
        0
    }
}

impl HeapSize for String {
    /// A `String` owns enough heap memory to hold its reserved capacity.
    fn heap_size_of_children(&self) -> usize {
        self.capacity()
    }
}

impl<T> HeapSize for Box<T>
where
    T: ?Sized + HeapSize,
{
    /// A `Box` owns however much heap memory was allocated to hold the value of
    /// type `T` that we placed on the heap, plus transitively however much `T`
    /// itself owns.
    fn heap_size_of_children(&self) -> usize {
        mem::size_of_val(&**self) + (**self).heap_size_of_children()
    }
}

impl<T> HeapSize for [T]
where
    T: HeapSize,
{
    /// Sum of heap memory owned by each element of a dynamically sized slice of
    /// `T`.
    fn heap_size_of_children(&self) -> usize {
        self.iter().map(HeapSize::heap_size_of_children).sum()
    }
}

impl<'a, T> HeapSize for &'a T
where
    T: ?Sized,
{
    /// A shared reference does not own heap memory.
    fn heap_size_of_children(&self) -> usize {
        0
    }
}