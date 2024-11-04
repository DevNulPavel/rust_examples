use crate::{
    constants::KB,
    helpers::{executable_path, get_payload, ExecutionResult},
};
use std::{
    io::{Read, Write},
    process::{Child, Command, Stdio},
    time::Instant,
};

////////////////////////////////////////////////////////////////////////////////

pub struct PipeRunner {
    pipe_proc: Child,
    data_size: usize,
    request_data: Vec<u8>,
    response_data: Vec<u8>,
}

impl PipeRunner {
    pub fn new(data_size: usize) -> PipeRunner {
        // let output_dir = PathBuf::from(env::var("CARGO_TARGET_DIR").unwrap());
        // let output_dir = PathBuf::new();
        // let exe = output_dir.join("pipes_consumer.exe");

        // Получаем путь к исполняемому файлику какому-то дополнительному
        let exe = executable_path("pipes_consumer");

        // Формируем Буфер
        let (request_data, response_data) = get_payload(data_size);

        // Запускаем процесс,
        // передаем ему размер данных через аргументы
        let pipe_proc = Command::new(exe)
            .args(&[data_size.to_string()])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        PipeRunner {
            pipe_proc,
            data_size,
            request_data,
            response_data,
        }
    }

    /// Запускаем
    fn inner_run(&mut self, n: usize) {
        // Получаем ссылку на stdin на дочерний процесс
        let Some(ref mut pipes_input) = self.pipe_proc.stdin else {
            return;
        };

        // Получаем ссылку на stdoud на дочерний процесс
        let Some(ref mut pipes_output) = self.pipe_proc.stdout else {
            return;
        };

        // Создаем буфер теперь для данных
        let mut buf = vec![0; self.data_size];

        // Делаем нужное количество итераций
        for _ in 0..n {
            // Пишем данные в пайп
            pipes_input.write(&self.request_data).unwrap();

            // Вычитываем в буфер данных теперь те же самые данные в ответ
            pipes_output.read_exact(&mut buf).unwrap();

            // Делаем дополнительную валидацию совпадения данных
            #[cfg(debug_assertions)]
            if buf.ne(&self.response_data) {
                panic!("Unexpected response {}", String::from_utf8_lossy(&buf))
            }
        }
    }

    /// Непосредственно запуск теста
    pub fn run(&mut self, n: usize, print: bool) {
        // Время старта
        let instant = Instant::now();

        // Запускаем тест
        self.inner_run(n);

        // Делаем подсчет времени
        let elapsed = instant.elapsed();

        if print {
            // Формируем результат
            let res = ExecutionResult::new(
                format!("Stdin/stdout - {}KB", self.data_size / KB),
                elapsed,
                n,
            );

            // Делаем вывод данных
            res.print_info();
        }
    }
}

// Реализация уничтожения теста
impl Drop for PipeRunner {
    fn drop(&mut self) {
        // Здесь мы уничтодаем дочерний тестовый процесс
        self.pipe_proc.kill().unwrap();
    }
}
