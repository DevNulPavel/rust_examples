use crate::audio::ExactStreamer;
use num_complex::Complex32;
use num_traits::identities::Zero;
use rustfft::FFT;
use std::time::Instant;


// Быстрое преобразование фурье здесь нужно для определения спектра выдываемого сигнала по частотам
// Затем результат отображается на пользовательском интерфейсе

pub struct FFTStreamer
{
    size: usize,                            // Размер потока
    stream: ExactStreamer<f32>,             // Поток входных данных
    sender: crossbeam::Sender<Vec<f32>>,    // Канал, куда мы выдаем данные
}

// impl<T> Drop for FFTStreamer<T>
// where
//     T: FnOnce()->()
// {
//     fn drop(&mut self) {
//         if let Some(thread) = self.thread{
//             thread.join();
//         }
//     }
// }

impl FFTStreamer
{
    pub fn new(size: usize, stream: ExactStreamer<f32>, sender: crossbeam::Sender<Vec<f32>>) -> Self {
        FFTStreamer {
            size,
            stream,
            sender
        }
    }

    // Данный код будет работать в потоке
    pub fn run(mut self) -> std::thread::JoinHandle<()>{
        let code = move || {
            // Создаем буффер
            let mut buf = vec![0.0f32; self.size];

            // Буфферы комплексных значений для FFT
            let mut complex_buf_input = vec![Complex32::zero(); self.size];
            let mut complex_buf_out = vec![Complex32::zero(); self.size];

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
                    // Коэффициент окна Ханна для использования прошлых значений
                    let window_fac = std::f32::consts::PI * 2.0 / self.size as f32;
                    let new_values_iter = buf
                        .iter()
                        .enumerate()
                        .map(|(i, sample)| {
                            // Окно нужно для того, чтобы делать плавный вход и выход для значений
                            // Чтобы преобразование Фурье могло опираться на старые значения тоже с перекрытием?
                            // Тогда график будет выглядеть как колокол
                            // 0.54 - 0.46 * cos(i * step)
                            let i = i as f32;
                            let window_coeff = 0.54 - 0.46 * (i * window_fac).cos();
                            let real_val = *sample * window_coeff;

                            Complex32::new(real_val, 0.0)
                        });
                    // TODO: Может не надо чистить из заново расширять?? Можно только заполнять значениями?
                    // хотя clear вроде как не деаллоцирует значения, может быть можно заменить так же как внизу с помощью zip
                    // Но скорее всего нужно для обновления нового размера
                    complex_buf_input.clear();
                    complex_buf_input.extend(new_values_iter);
                }

                // Обрабатываем быстрое преобразование фурье c нашей оконной функцией
                fft.process(&mut complex_buf_input, &mut complex_buf_out);

                {
                    // Итератор по модулям значений
                    // Модуль - это амплитуда конкретной частоты
                    let new_val_iter = complex_buf_out
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

                // Коэффициент 0.00005 в степени прошедшего времени длительности расчетов
                let elapsed_time = last_time.elapsed().as_secs_f32();
                let fac = 0.00005_f32.powf(elapsed_time);
                
                // Обновляем время с последнего отсчета
                last_time = Instant::now();

                // Домнажаем старые значения на коэффициент + обновляем значения новыми
                // Это делается из-за оконного преобразования Фурье выше
                // Сравнивая на максимум
                last_frequencies
                    .iter_mut()
                    .zip(frequencies.iter())
                    .for_each(|(old, new)| {
                        //(coefficient after one second).powf(time))
                        // TODO: ???
                        *old *= fac;
                        *old = ((*old) as f32).max(*new);
                    });
                
                // Формируем данные для отправки в канал
                // Обработка данных частот нужна для того, чтобы спектр был представлен в логарифмическом виде,
                // как во всех нормальных эквалайзерах и анализаторах спектра
                let send_values = last_frequencies
                    .iter()
                    .map(|x| {
                        // (exp(x * 0.008) - 1.0
                        let exp_val = (x * 0.008).exp();
                        ((exp_val - 1.0) * 0.7).powf(0.5) * 2.0
                    })
                    .collect::<Vec<f32>>();
            
                if self.sender.send(send_values).is_err(){
                    break;
                }  
            }          
        };

        let handle = std::thread::Builder::new()
            .name("FFT thread".into())
            .spawn(code)
            .unwrap();

        handle
    }
}
