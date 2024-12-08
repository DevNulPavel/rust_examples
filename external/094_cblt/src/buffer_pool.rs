use std::sync::Arc;
use tokio::sync::Mutex;

////////////////////////////////////////////////////////////////////////////////

/// Удобный дефайн для вектора данных с блокировкой
pub(super) type SmartVector = Arc<Mutex<Vec<u8>>>;

////////////////////////////////////////////////////////////////////////////////

/// Пул буферов
pub(crate) struct BufferPool {
    /// Непосредственно сам пул
    pool: Arc<Mutex<Vec<SmartVector>>>,

    /// Размер буфера
    buffer_size: usize,
}

impl BufferPool {
    /// Создаем пул буферов данных
    pub(crate) fn new(buffer_count: usize, buffer_size: usize) -> Self {
        // Аллоцируем пул буферов нужного размера
        let pool = (0..buffer_count)
            .map(|_| {
                // Создаем теперь сразу же буфер нужного предаллоцированного
                // размера, делаем обертку над данными
                Arc::new(Mutex::new(Vec::with_capacity(buffer_size)))
            })
            .collect();

        BufferPool {
            pool: Arc::new(Mutex::new(pool)),
            buffer_size,
        }
    }

    /// Получаем буфер из пула, либо создаем новый если
    /// не нашлось никакого свободного
    pub(crate) async fn get_buffer(&self) -> SmartVector {
        // Берем блокировку на пуле
        let mut pool_lock = self.pool.lock().await;

        // Есть ли свободный какой-то?
        if let Some(buffer) = pool_lock.pop() {
            buffer
        } else {
            // Раз буфера нету, тогда можем просто снять блокировку
            drop(pool_lock);

            // Создаем тогда новый буфер
            Arc::new(Mutex::new(Vec::with_capacity(self.buffer_size)))
        }
    }

    /// Возврат назад буфера в общий пул
    pub(crate) async fn return_buffer(&self, buffer: SmartVector) {
        // Берем блокировку на пуле
        let mut pool_lock = self.pool.lock().await;

        // Берем блокировку на отдельном возвращаемом буфере
        let buffer_lock = buffer.lock().await;

        // Емкость возвращаемого буфера у нас сейчас больше,
        // чем рекомендуемый размер буфера
        if buffer_lock.capacity() > self.buffer_size {
            // TODO: Правильнее было бы просто урезать размер буфера, а не переаллоцировать

            // Уничтожаем поэтому возвращаемый буфер
            drop(buffer_lock);
            drop(buffer);

            // Создаем новый
            let new_buffer = Arc::new(Mutex::new(Vec::with_capacity(self.buffer_size)));

            // И новый буффер уже сохраняем
            pool_lock.push(new_buffer)
        } else {
            // Снимаем блокировку
            drop(buffer_lock);

            // Созвращаем буфер просто назад в пул
            pool_lock.push(buffer)
        }
    }
}
