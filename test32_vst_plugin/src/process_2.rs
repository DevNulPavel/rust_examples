fn process(&mut self, buffer: &mut AudioBuffer<f32>){
    let threshold = self.params.get_threshold();
    let volume = self.params.get_volume();

    const BUFFER_MUL: usize = 1;

    // Создаем итератор по парам элементов, входа и выхода
    for (input, output) in buffer.zip() {
        let window_size = input.len() * BUFFER_MUL;

        //let window = apodize::hanning_iter(window_size).collect::<Vec<f64>>();

        // Первая секция
        let mut input_fft_1: Vec<Complex32> = input
            .iter()
            .flat_map(|val|{
                std::iter::repeat(val).take(BUFFER_MUL)
            })
            //.zip(window.iter().map(|val| *val as f32))
            // .map(|(val, wind)|{
            //     Complex32::new(*val * wind, 0.0)
            // })
            .map(|val|{
                Complex32::new(*val, 0.0)
            })
            .collect();

        let mut output_fft_1: Vec<Complex32> = vec![Complex::zero(); window_size];

        // FFTplanner позволяет выбирать оптимальный алгоритм работы для входного размера данных
        // Создаем объект, который содержит в себе оптимальный алгоритм преобразования фурье
        // Обрабатываем данные
        // Входные данные мутабельные, так как они используются в качестве буффера
        // Как результат - там будет мусор после вычисления
        FFTplanner::new(false)
            .plan_fft(output_fft_1.len())
            .process(&mut input_fft_1, &mut output_fft_1);
        
        let inv_len = 1.0 / (output_fft_1.len() as f32).sqrt();
        output_fft_1
            .iter_mut()
            .for_each(|val|{
                *val *= inv_len;
            });                

        // FFTplanner позволяет выбирать оптимальный алгоритм работы для входного размера данных
        // Создаем объект, который содержит в себе оптимальный алгоритм преобразования фурье
        FFTplanner::new(true)
            .plan_fft(output_fft_1.len())
            .process(&mut output_fft_1, &mut input_fft_1);

        let inv_len = 1.0 / (input_fft_1.len() as f32).sqrt();
        input_fft_1
            .iter_mut()
            .for_each(|val|{
                *val *= inv_len;
            });

        let iter = input_fft_1
            .into_iter()
            .step_by(BUFFER_MUL)
            .map(|val1|{
                //val1.norm()
                val1.re
            })
            .zip(output.into_iter());

        for (in_sample, out_sample) in iter {
            let val = in_sample;

            *out_sample = val;

            // Эмулируем клиппинг значений
            *out_sample = if val > threshold {
                threshold
            } else if val < -threshold {
                -threshold
            } else {
                val
            };

            *out_sample *= volume;
        }
    }
} 