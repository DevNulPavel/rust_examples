use crate::audio::ExactStreamer;
use num_complex::Complex32;
use num_traits::identities::Zero;
use rustfft::FFT;
use std::time::Instant;

pub struct FFTStreamer {
    size: usize,                            // Размер потока
    stream: ExactStreamer<f32>,             // Поток входных данных
    sender: crossbeam::Sender<Vec<f32>>,    // Канал, куда мы выдаем данные
}

impl FFTStreamer {
    pub fn new(
        size: usize,
        stream: ExactStreamer<f32>,
        sender: crossbeam::Sender<Vec<f32>>,
    ) -> Self {
        FFTStreamer {
            size,
            stream,
            sender,
        }
    }

    // Данный код будет работать в потоке
    pub fn run(&mut self) {
        // Создаем буффер
        let mut buf = vec![0.0f32; self.size];
        // Буфферы комплексных значений для FFT
        let mut complex_buf = vec![Complex32::zero(); self.size];
        let mut complex_buf2 = vec![Complex32::zero(); self.size];

        // Частоты
        let mut frequencies = vec![0.0; self.size];
        let mut last_frequencies = vec![0.0; self.size];
        let mut last_time = Instant::now();

        // Создаем алгоритм быстрого преобразования фурье
        let fft = rustfft::algorithm::Radix4::new(self.size, false);

        // Бесконечный цикл
        loop {
            // Заполняем буффер входными данными
            // Ошибка записи потока - прерываем цикл
            if self.stream.fill(&mut buf).is_err() {
                break;
            }

            // Заполняем данные начальными значениями на основании входного
            {
                let window_fac = std::f32::consts::PI * 2.0 / self.size as f32;
                let new_values_iter = buf
                    .iter()
                    .enumerate()
                    .map(|(i, sample)| {
                        let real_val = *sample * (0.54 - 0.46 * (i as f32 * window_fac).cos());
                        Complex32::new(real_val, 0.0)
                    });
                // TODO: Может не надо чистить из заново расширять?? Можно только заполнять значениями?
                // хотя clear вроде как не деаллоцирует значения, может быть можно заменить так же как внизу с помощью zip
                // Но скорее всего нужно для обновления нового размера
                complex_buf.clear();
                complex_buf.extend(new_values_iter);
            }

            // Обрабатываем быстрое преобразование фурье
            fft.process(&mut complex_buf, &mut complex_buf2);

            {
                // Итератор по нормализованным комплексным значением
                let new_val_iter = complex_buf2
                    .iter()
                    .map(|complex| {
                        complex.norm()
                    });

                // Заменяем значения частот с помощью объединения с другим итератором
                frequencies
                    .iter_mut()
                    .zip(new_val_iter)
                    .for_each(|(old, new)| {
                        // Заменяем на новое значение
                        *old = new
                    });
            }

            let fac = 0.00005f32.powf(last_time.elapsed().as_secs_f32());
            last_time = Instant::now();
            last_frequencies
                .iter_mut()
                .zip(frequencies.iter())
                .for_each(|(old, new)| {
                    //(coefficient after one second).powf(time))
                    // TODO: ???
                    *old *= fac;
                    *old = old.max(*new);
                });
            
            // Формируем данные для отправки в канал
            let send_values = last_frequencies
                .iter()
                .map(|x| {
                    (((x * 0.008).exp() - 1.0) * 0.7).powf(0.5) * 2.0
                })
                .collect::<Vec<f32>>();
            if self.sender.send(send_values).is_err(){
                break;
            }
        }
    }
}
