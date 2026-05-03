use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use tokio::sync::{Mutex, MutexGuard};

////////////////////////////////////////////////////////////////////////////////

/// Структура для шардированной блокировки (Lock Striping).
/// Позволяет блокировать операции по ключу, сводя вероятность коллизий к минимуму
/// и избегая глобальных блокировок.
pub struct StripedLock {
    /// Вектор из пустых Mutex
    locks: Vec<Mutex<()>>,
}

impl StripedLock {
    /// Создает массив из `shard_count` мьютексов.
    pub fn new(shard_count: usize) -> Self {
        // Создаем вектор сразу нужной емкости
        let mut locks = Vec::with_capacity(shard_count);

        // Наполняем его блокровками
        for _ in 0..shard_count {
            locks.push(Mutex::new(()));
        }

        Self { locks }
    }

    /// Асинхронно захватывает лок для конкретного ключа.
    #[inline]
    pub async fn lock<K: Hash>(&self, key: &K) -> MutexGuard<'_, ()> {
        // TODO: Нормально ли будет тут работать?
        // Хешер же должен каждый раз создаваться
        // с одними и теми же ключами инциализации.
        // Но вроде бы как все нормально - new вызов созадет с нулями.
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);

        // Получаем индекс блокировки теперь
        let shard_idx = (hasher.finish() as usize) % self.locks.len();

        // Проставляем блокировку и возвращаем guard
        self.locks[shard_idx].lock().await
    }
}

impl Default for StripedLock {
    fn default() -> Self {
        Self::new(1024)
    }
}
