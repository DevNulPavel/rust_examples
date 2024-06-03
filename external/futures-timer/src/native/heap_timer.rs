use super::{Node, ScheduledTimer};
use std::{cmp::Ordering, sync::Arc, time::Instant};

////////////////////////////////////////////////////////////////////////////////

/// Специальная структура для сортировки в двоичной куче.
pub(crate) struct HeapTimer {
    /// Время пробуждения
    pub(crate) at: Instant,

    /// Номер итерации
    pub(crate) gen: usize,

    /// Поставленный в очередь таймер
    pub(crate) node: Arc<Node<ScheduledTimer>>,
}

/// Реализация сравнения
impl PartialEq for HeapTimer {
    fn eq(&self, other: &HeapTimer) -> bool {
        self.at == other.at
    }
}

impl Eq for HeapTimer {}

/// Реализация сортировки от меньшего к большему
impl PartialOrd for HeapTimer {
    fn partial_cmp(&self, other: &HeapTimer) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Реализация сортировки от меньшего к большему
impl Ord for HeapTimer {
    fn cmp(&self, other: &HeapTimer) -> Ordering {
        self.at.cmp(&other.at)
    }
}
