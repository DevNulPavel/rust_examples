

/// Используется как разновидность BufReader для канала crossbeam, чтобы читать данные
/// Позволяет получать данные из канала и буфферизировать их, а затем постепенно быстро отдавать данные маленькими порциями
pub struct ExactStreamer<T> {
    // Хранит данные если размер слайса коллбека не множитель GENERATOR_BUFFER_SIZE
    remainder: Vec<T>,
    remainder_len: usize,
    // Канал данных из которого буфферизуется
    receiver: crossbeam::Receiver<Vec<T>>,
}

impl<T> ExactStreamer<T>
where
    T: Copy + Default,
{
    // Создание нового канала
    pub fn new(remainder_buffer_size: usize, receiver: crossbeam::Receiver<Vec<T>>) -> ExactStreamer<T> {
        ExactStreamer {
            remainder: vec![T::default(); remainder_buffer_size],
            remainder_len: 0,
            receiver,
        }
    }

    // Вызывается из Player для получения маленькой порции даннных
    pub fn fill(&mut self, out: &mut [T]) -> Result<(), crossbeam::crossbeam_channel::RecvError> {
        // Получаем количество данных, которое можем скопировать в выход
        // Либо это оставшееся значение, либо длина выходного буфера
        let mut i = self.remainder_len.min(out.len());

        // Заполняем выходной буфер нашими буферизованными данными
        out[..i].copy_from_slice(&self.remainder[..i]);

        // TODO: Может быть можно задействовать кольцевой буфер
        // Смещаем данные в нашем буфере к началу для следующего чтения
        self.remainder.copy_within(i..self.remainder_len, 0);

        // Оставшееся количество данных уменьшаем
        self.remainder_len -= i;

        // Если мы не полностью записали выходной буфер
        'while_loop: while i < out.len() {
            // Получаем из канала новую порцию данных
            let generated = self.receiver.recv()?;

            // Если полученное значение покрывает недостающее значение
            if generated.len() > out.len() - i {
                // Узнаем сколько нам еще осталось
                let left = out.len() - i;

                // Дозаполняем наши значения в выходном буффере
                out[i..].copy_from_slice(&generated[..left]);

                // Оставшийся непрочитанный размер данных
                self.remainder_len = generated.len() - left;
                
                // Суммарная длина
                let vec_len = self.remainder.len();
                // Если суммарная длина меньше непрочитанных данных
                if vec_len < self.remainder_len {
                    // Дозаполняем начальными значениями наш буффер
                    self.remainder
                        .extend(std::iter::repeat(T::default()).take(self.remainder_len - vec_len));
                }

                // Заполняем начало буфера новыми данными, которые не попали в выход
                self.remainder[..self.remainder_len].copy_from_slice(&generated[left..]);

                break 'while_loop;
            } else {
                // Если новые значения норм, значит выгружаем из сгенерированных
                out[i..(i + generated.len())].copy_from_slice(&generated);
                i += generated.len();
            }
        }

        Ok(())
    }
}