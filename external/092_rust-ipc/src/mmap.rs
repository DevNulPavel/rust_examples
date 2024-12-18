use crate::{
    constants::KB,
    helpers::{executable_path, get_payload, ExecutionResult},
};
use memmap2::MmapMut;
use raw_sync::{
    events::{BusyEvent, EventImpl, EventInit, EventState},
    Timeout,
};
use std::io::Write;
use std::{
    fs::OpenOptions,
    path::PathBuf,
    process::{Child, Command},
    thread::sleep,
    time::{Duration, Instant},
};

////////////////////////////////////////////////////////////////////////////////

/// Тестовая обертка над кодом mmap
pub struct MmapWrapper {
    /// Непосредственно мутабельная область памяти
    pub mmap: MmapMut,

    /// Являемся ли владельцами замапленной памяти?
    pub owner: bool,

    /// Флаг для уведомлений с нашей стороны
    pub our_event: Box<dyn EventImpl>,

    /// Флаг для уведомлений с их стороны
    pub their_event: Box<dyn EventImpl>,

    /// С какого места в памяти у нас начинаются данные непосредственно
    pub data_start: usize,

    /// Размер полезных данных
    pub data_size: usize,
}

impl MmapWrapper {
    /// Создание новой обертки над замапленной памятью
    pub fn new(owner: bool, data_size: usize) -> Self {
        // Полный размер данных, включая место для событий
        let data_size = data_size + 4;

        // Путь к замапленному файлику
        let path: PathBuf = "/tmp/mmap_data.txt".into();

        // Теперь пробуем создать или открыть файлик если его еще нету там
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)
            .unwrap();

        // Раз файлик открылся, то можем теперь указать
        // конкретный размер файлика полный
        file.set_len(data_size as u64).unwrap();

        // И вот теперь мы можем замаппить данный файлик в адресное пространство
        let mut mmap = unsafe { MmapMut::map_mut(&file).unwrap() };

        // Торжественно получаем теперь указатель на замапленную память
        let bytes = mmap.as_mut();

        // Создаем события-блокировки для нашей и для противоположной стороны.
        // Они симметричные с точки зрения смещений в зависимости от того кто владеет
        // данными.
        let ((our_event, lock_bytes_ours), (their_event, lock_bytes_theirs)) = unsafe {
            if owner {
                (
                    BusyEvent::new(bytes.get_mut(0).unwrap(), true).unwrap(),
                    BusyEvent::new(bytes.get_mut(2).unwrap(), true).unwrap(),
                )
            } else {
                (
                    // Так как мы не владельцы, то события-блокировки скорее всего есть уже,
                    // так что их мы и получим.
                    BusyEvent::from_existing(bytes.get_mut(2).unwrap()).unwrap(),
                    BusyEvent::from_existing(bytes.get_mut(0).unwrap()).unwrap(),
                )
            }
        };

        // Отвалидируем размерность теперь
        assert!(lock_bytes_ours <= 2);
        assert!(lock_bytes_theirs <= 2);

        // Если мы сейчас являемся владельцем, тогда сбрасываем флаг,
        // тем самым разрешаем запись
        if owner {
            our_event.set(EventState::Clear).unwrap();
            their_event.set(EventState::Clear).unwrap();
        }

        // Создаем обертку теперь
        Self {
            mmap,
            owner,
            our_event,
            their_event,
            data_start: 4,
            data_size,
        }
    }

    /// Проставляем сигнал начала записи
    pub fn signal_start(&mut self) {
        self.our_event.set(EventState::Clear).unwrap()
    }

    /// Проставляем сигнал завершения записи данных
    pub fn signal_finished(&mut self) {
        self.our_event.set(EventState::Signaled).unwrap()
    }

    /// Записываем данные в нужное место
    pub fn write(&mut self, data: &[u8]) {
        // TODO: Правильно ошибку кидать, но это тестовый пример
        assert!(data.len() == self.data_size);

        // Пишем данные с нужным нам смещением
        let writtend = (&mut self.mmap[self.data_start..]).write(data).unwrap();

        // TODO: Тоже лучше кадать ошибку
        assert_eq!(writtend, data.len());
    }

    /// Читаем данные
    pub fn read(&self) -> &[u8] {
        // Вычитываем буфер данных
        &self.mmap.as_ref()[self.data_start..self.data_size]
    }
}

////////////////////////////////////////////////////////////////////////////////

/// Запускатель тестов
pub struct MmapRunner {
    /// Дочерний процесс
    child_proc: Option<Child>,

    /// Тестовая обертка
    wrapper: MmapWrapper,

    /// Размер тестовых данных
    data_size: usize,

    /// Буфер запроса
    request_data: Vec<u8>,

    /// Буфер ответа
    response_data: Vec<u8>,
}

impl MmapRunner {
    pub fn new(start_child: bool, data_size: usize) -> Self {
        let wrapper = MmapWrapper::new(true, data_size);

        let exe = executable_path("mmap_consumer");
        let child_proc = if start_child {
            let res = Some(
                Command::new(exe)
                    .args(&[data_size.to_string()])
                    .spawn()
                    .unwrap(),
            );
            // Clumsy sleep here but it allows the child proc to spawn without it having to offer
            // us a ready event
            sleep(Duration::from_secs(2));
            res
        } else {
            None
        };

        let (request_data, response_data) = get_payload(data_size);

        Self {
            child_proc,
            wrapper,
            data_size,
            request_data,
            response_data,
        }
    }

    pub fn run(&mut self, n: usize, print: bool) {
        let instant = Instant::now();
        for _ in 0..n {
            // Activate our lock in preparation for writing
            self.wrapper.signal_start();
            self.wrapper.write(&self.request_data);
            // Unlock after writing
            self.wrapper.signal_finished();
            // Wait for their lock to be released so we can read
            if self.wrapper.their_event.wait(Timeout::Infinite).is_ok() {
                let str = self.wrapper.read();

                #[cfg(debug_assertions)]
                if str.ne(&self.response_data) {
                    panic!("Sent request didn't get response")
                }
            }
        }
        let elapsed = instant.elapsed();

        if print {
            let res = ExecutionResult::new(
                format!("Memory mapped file - {}KB", self.data_size / KB),
                elapsed,
                n,
            );
            res.print_info();
        }
    }
}

impl Drop for MmapRunner {
    fn drop(&mut self) {
        if let Some(ref mut child) = self.child_proc {
            child.kill().expect("Unable to kill child process")
        }
    }
}
