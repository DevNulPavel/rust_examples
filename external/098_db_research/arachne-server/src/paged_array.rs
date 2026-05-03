use std::sync::atomic::{AtomicPtr, AtomicU32, Ordering};

////////////////////////////////////////////////////////////////////////////////

const CHUNK_SIZE: usize = 1_000_000;
const MAX_CHUNKS: usize = 4096; // Покрывает до 4.096 млрд пользователей (почти весь u32)

////////////////////////////////////////////////////////////////////////////////

/// Многостраничный массив (плоский массив)
pub struct PagedArray {
    /// Статичный массив атомарных указателей (занимает 4096 * 8 байт = 32 КБ)
    chunks: [AtomicPtr<AtomicU32>; MAX_CHUNKS],
}

impl PagedArray {
    /// Выделяет память для индекса, если еще не аллоциорвана О(CHUNK_SIZE)
    fn ensure_allocated(&self, id: u32) {
        // Для указанного идентификатора мы получаем индекс самого чанка в массиве
        let chunk_idx = (id as usize) / CHUNK_SIZE;

        // Получаем указатель на этот самый чанк
        let ptr = self.chunks[chunk_idx].load(Ordering::Acquire);

        // Если чанк уже выделен, не делаем ничего
        if !ptr.is_null() {
            return;
        }

        // Выделяем 8 МБ ОЗУ в виде вектора атомик-нулей
        let mut vec = Vec::with_capacity(CHUNK_SIZE);
        for _ in 0..CHUNK_SIZE {
            vec.push(AtomicU32::new(0)); // std::alloc::alloc_zeroed?
        }

        // Забираем сырой указатель на данные, чтобы Rust не очистил их при выходе из скоупа
        let raw_ptr = vec.into_raw_parts().0;

        // Пытаемся атомарно сохранить указатель в массив чанков (Compare-And-Swap)
        let failed = self.chunks[chunk_idx]
            .compare_exchange(
                std::ptr::null_mut(),
                raw_ptr,
                Ordering::Release,
                Ordering::Relaxed,
            )
            .is_err();

        // Если два краулера одновременно сделали user_add на новой границе миллиона,
        // второй потерпит неудачу в CAS. Ему нужно просто удалить свой кусок памяти.
        if failed {
            unsafe {
                let _ = Vec::from_raw_parts(raw_ptr, CHUNK_SIZE, CHUNK_SIZE);
            }
        }
    }

    /// Получает элемент по индексу
    pub fn get(&self, id: u32) -> &AtomicU32 {
        // Убеждаемся, что память выделена
        self.ensure_allocated(id);

        // Вычисляем чанк
        let chunk_idx = (id as usize) / CHUNK_SIZE;

        // Вычисляем смещение внутри чанка теперь
        let offset = (id as usize) % CHUNK_SIZE;

        // Получаем теперь указатель на память этого самого чанка
        let ptr = self.chunks[chunk_idx].load(Ordering::Acquire);
        debug_assert!(!ptr.is_null(), "Попытка доступа к невыделенному чанку!");

        // TODO: Тут смещение же нужно по 4 байта именно (u32), а не по одному?
        // Адресная арифметика: сдвигаемся на нужный оффсет
        unsafe { &*ptr.add(offset) }
    }

    /// Дамп массива в вектор значений
    pub fn dump(&self, max_id: u32) -> Vec<u32> {
        let mut dump = vec![0u32; max_id as usize];
        if max_id == 0 {
            return dump;
        }

        let chunks_count = (max_id as usize).div_ceil(CHUNK_SIZE);
        for chunk_idx in 0..chunks_count {
            // Проверяем, что указатель не нулевой
            let ptr = self.chunks[chunk_idx].load(Ordering::Acquire);
            if !ptr.is_null() {
                let start = chunk_idx * CHUNK_SIZE;
                let end = std::cmp::min(start + CHUNK_SIZE, max_id as usize);
                let len = end - start;
                unsafe {
                    // Копируем весь чанк за одну инструкцию процессора
                    std::ptr::copy_nonoverlapping(
                        ptr as *const u32,
                        dump[start..].as_mut_ptr(),
                        len,
                    );
                }
            }
        }
        dump
    }

    /// Восстанавливает структуру из массива значений
    pub fn restore(dump: &[u32]) -> Self {
        let pa = Self::default();
        let chunks_count = dump.len().div_ceil(CHUNK_SIZE);

        for chunk_idx in 0..chunks_count {
            let start = chunk_idx * CHUNK_SIZE;
            let end = std::cmp::min(start + CHUNK_SIZE, dump.len());
            let slice = &dump[start..end];

            let mut vec = Vec::with_capacity(CHUNK_SIZE);
            // Rust соптимизирует этот цикл в memmove/memcpy
            for &v in slice {
                vec.push(AtomicU32::new(v));
            }
            // Заполняем остаток чанка нулями
            for _ in end..(start + CHUNK_SIZE) {
                vec.push(AtomicU32::new(0));
            }

            let raw_ptr = vec.into_raw_parts().0;
            pa.chunks[chunk_idx].store(raw_ptr, Ordering::Relaxed);
        }
        pa
    }

    /// Преаллоцирует память сразу для всего диапазона (убирает накладные расходы if ptr.is_null)
    pub fn preallocate_up_to(&self, max_id: u32) {
        if max_id == 0 {
            return;
        }
        let chunks_needed = (max_id as usize).div_ceil(CHUNK_SIZE);
        for chunk_idx in 0..chunks_needed {
            let ptr = self.chunks[chunk_idx].load(Ordering::Relaxed);
            if ptr.is_null() {
                let mut vec = Vec::with_capacity(CHUNK_SIZE);
                for _ in 0..CHUNK_SIZE {
                    vec.push(AtomicU32::new(0));
                }
                let raw_ptr = vec.into_raw_parts().0;
                self.chunks[chunk_idx].store(raw_ptr, Ordering::Release);
            }
        }
    }
}

impl Default for PagedArray {
    fn default() -> Self {
        Self {
            // При старте все чанки пустые (указывают на null)
            chunks: std::array::from_fn(|_| AtomicPtr::new(std::ptr::null_mut())),
        }
    }
}

// Чтобы память не утекла при штатном завершении программы
impl Drop for PagedArray {
    fn drop(&mut self) {
        for chunk in &self.chunks {
            let ptr = chunk.load(Ordering::Relaxed);
            if !ptr.is_null() {
                unsafe {
                    let _ = Vec::from_raw_parts(ptr, CHUNK_SIZE, CHUNK_SIZE);
                }
            }
        }
    }
}
