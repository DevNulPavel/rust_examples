fn process(&mut self, buffer: &mut AudioBuffer<f32>){
    let threshold = self.params.get_threshold();
    let volume = self.params.get_volume();

    const BUFFER_MUL: usize = 1;

    // Создаем итератор по парам элементов, входа и выхода
    let mut i = 0;
    for (input, output) in buffer.zip() {
        if self.last_input_buffer[i].len() < input.len() {
            self.last_input_buffer[i].resize(input.len(), 0.0_f32);
        }

        let last_in_buf: &mut Vec<f32> = &mut self.last_input_buffer[i];
        let window_size = input.len() * (BUFFER_MUL as f32 * 1.5) as usize;

        let window = apodize::hanning_iter(window_size).collect::<Vec<f64>>();

        // Первая секция
        let mut input_fft_1: Vec<Complex32> = last_in_buf
            .iter()
            .chain(input
                .iter()
                .take(input.len() / 2))
            .flat_map(|val|{
                std::iter::repeat(val).take(BUFFER_MUL)
            })
            .zip(window.iter().map(|val| *val as f32))
            .map(|(val, wind)|{
                Complex32::new(*val * wind, 0.0)
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
        
        //let inv_len = 1.0 / (output_fft_1.len() as f32);
        //let sqrt_len = 1.0 / inv_len.sqrt();
        // output_fft
        //     .iter_mut()
        //     .for_each(|val|{
        //         *val *= sqrt_len;
        //     });

        // FFTplanner позволяет выбирать оптимальный алгоритм работы для входного размера данных
        // Создаем объект, который содержит в себе оптимальный алгоритм преобразования фурье
        FFTplanner::new(true)
            .plan_fft(output_fft_1.len())
            .process(&mut output_fft_1, &mut input_fft_1);

        let inv_len = 1.0 / (input_fft_1.len() as f32);
            input_fft_1
                .iter_mut()
                .for_each(|val|{
                    //val.norm();
                    *val *= inv_len;
                });

        // Вторая секция
        let mut input_fft_2: Vec<Complex32> = last_in_buf
            .iter()
            .skip(last_in_buf.len() / 2)
            .chain(input
                .iter())
            .flat_map(|val|{
                std::iter::repeat(val).take(BUFFER_MUL)
            })
            .zip(window.iter().map(|val| *val as f32))
            .map(|(val, wind)|{
                Complex32::new(*val * wind, 0.0)
            })
            .collect();

        let mut output_fft_2: Vec<Complex32> = vec![Complex::zero(); window_size];
        
        // FFTplanner позволяет выбирать оптимальный алгоритм работы для входного размера данных
        // Создаем объект, который содержит в себе оптимальный алгоритм преобразования фурье
        FFTplanner::new(true)
            .plan_fft(output_fft_2.len())
            .process(&mut output_fft_2, &mut input_fft_2);

        // Сохраняем текущие данные из нового буффера в старый
        last_in_buf.copy_from_slice(input);

        let inv_len = 1.0 / (input_fft_2.len() as f32);
        input_fft_2
            .iter_mut()
            .for_each(|val|{
                //val.norm();
                *val *= inv_len;
            });

        // Для каждого входного и выходного семпла в буфферах
        //let len_1 = input_fft_1.len();
        //let len_2= input_fft_2.len();

        let iter = input_fft_1
            .into_iter()
            .skip(output.len()/2)
            .zip(input_fft_2
                .into_iter()
                .take(output.len()))
            .step_by(BUFFER_MUL)
            .map(|(val1, val2)|{
                val1.norm() + val2.norm()
            })
            .zip(output.into_iter());

        for (in_sample, out_sample) in iter {
            // let val = if in_sample.re < 0.0_f32{
            //     in_sample.re + min_abs
            // }else{
            //     in_sample.re + min_abs
            // };
            //let val = in_sample.re;
            //let val: Complex32 = in_sample;
            //let val = val.norm();
            // let val = val.re;
            // let val = val.norm() + const_val;

            /*let index = ii+last_out_buf.len()/2;
            if index < last_out_buf.len(){
                *out_sample += last_out_buf[ii+last_out_buf.len()/2];
            }*/

            let val = in_sample;

            *out_sample = val;
            //let val = *in_sample;

            // Эмулируем клиппинг значений
            *out_sample = if val > threshold {
                threshold
            } else if val < -threshold {
                -threshold
            } else {
                val
            };

            *out_sample *= volume;

            //last_out_buf[ii] = *out_sample;
        }

        i += 1;
    }
} 