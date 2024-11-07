use crate::{
    constants::KB,
    helpers::{executable_path, get_payload, ExecutionResult},
};
use raw_sync::{
    events::{BusyEvent, EventImpl, EventInit, EventState},
    Timeout,
};
use shared_memory::{Shmem, ShmemConf};
use std::{
    process::{Child, Command},
    thread::sleep,
    time::{Duration, Instant},
};

////////////////////////////////////////////////////////////////////////////////

/// Обертка для удобной работы с общей памятью
pub struct ShmemWrapper {
    /// Непосредственно указатель на общие данные
    pub shmem: Shmem,

    /// Являемся ли мы владельцем текущей общей памяти?
    pub owner: bool,

    /// Флаги в общей памяти для блокировок
    pub our_event: Box<dyn EventImpl>,

    /// Флаги в общей памяти для блокировок
    pub their_event: Box<dyn EventImpl>,

    /// С какого момента начинаются пользовательские данные
    pub data_start: usize,

    // Замер общей памяци в целом, включая начальные блокировки
    pub data_size: usize,
}

impl ShmemWrapper {
    /// Создаем
    pub fn new(handle: Option<String>, data_size: usize) -> ShmemWrapper {
        // Размер данных + 4 байта для мета-информации и для блокировок
        let data_size = data_size + 4;

        // Если нам передан внешний хендл для общей памяти, то мы присоединяемся
        // к нему, а если нет - то просто создаем новый
        let (is_owner, mut shmem) = match handle {
            None => {
                // Создаем блок общей памяти с конфигом
                let shmem = ShmemConf::new().size(data_size).create().unwrap();
                (true, shmem)
            }
            Some(h) => {
                // Общая память
                let shmem = ShmemConf::new()
                    .size(data_size)
                    .os_id(&h)
                    .open()
                    .unwrap_or_else(|_| panic!("Unable to open the shared memory at {}", h));

                (false, shmem)
            }
        };

        // Получаем теперь слайс на байты общей памяти
        let bytes = unsafe { shmem.as_slice_mut() };

        // Два события блокировки - по одному для каждой стороны.
        // Каждая сторона активирует блокировку перед записью, а потом снимает блокировку
        // когда данные могут быть прочитаны.
        let ((our_event, lock_bytes_ours), (their_event, lock_bytes_theirs)) = unsafe {
            // Являемся ли мы владельцами текущей памяти изначально?
            if is_owner {
                (
                    // Создаем блокировку на 0-м байте данных
                    BusyEvent::new(bytes.get_mut(0).unwrap(), true).unwrap(),
                    // Создаем блокировку на 2-м байте данных
                    BusyEvent::new(bytes.get_mut(2).unwrap(), true).unwrap(),
                )
            } else {
                (
                    // Раз мы не являемся исходным владельцем памяти, то события могут быть уже созданы
                    //
                    // Создаем блокировку здесь уже на 2-м байте данных
                    BusyEvent::from_existing(bytes.get_mut(2).unwrap()).unwrap(),
                    // Создаем блокировку здесь уже на 0-м байте данных
                    BusyEvent::from_existing(bytes.get_mut(0).unwrap()).unwrap(),
                )
            }
        };

        // Отвалидируем, что у нас корректный список блокировок для каждой из сторон
        assert!(lock_bytes_ours <= 2);
        assert!(lock_bytes_theirs <= 2);

        // Если мы являемся владельцем, тогда
        // для каждой из блокировок устанавливаем
        // событие очистки
        if is_owner {
            our_event.set(EventState::Clear).unwrap();
            their_event.set(EventState::Clear).unwrap();
        }

        // Создаем обертку
        ShmemWrapper {
            // Непосредственно указатель на общие данные
            shmem,
            // Являемся ли мы владельцем текущей общей памяти?
            owner: is_owner,
            // Флаги в общей памяти для блокировок
            our_event,
            their_event,
            // С какого момента начинаются уже посльзовательские данные в общей памяти
            data_start: 4,
            // Замер общей памяци в целом, включая начальные блокировки
            data_size,
        }
    }

    /// Устанавливаем сигнал сброса для текущей стороны
    pub fn signal_start(&mut self) {
        self.our_event.set(EventState::Clear).unwrap()
    }

    /// Сбрасываем сигнал сброса для текущей стороны
    pub fn signal_finished(&mut self) {
        self.our_event.set(EventState::Signaled).unwrap()
    }

    /// Записываем теперь данные в общую память
    pub fn write(&mut self, data: &[u8]) {
        // Получаем сырой слайс на общие данные
        let bytes = unsafe { self.shmem.as_slice_mut() };

        // Записываем теперь выходные данные по определенному смещению
        // TODO: Здесь лучше бы сразу проверять выход за границы памяти, а не падать с ошибкой
        for i in 0..data.len() {
            bytes[i + self.data_start] = data[i];
        }
    }

    /// Получаем слайс на все пользовательские данные
    pub fn read(&self) -> &[u8] {
        unsafe { &self.shmem.as_slice()[self.data_start..self.data_size] }
    }
}

////////////////////////////////////////////////////////////////////////////////

// #[derive(Debug)]
pub struct ShmemRunner {
    child_proc: Option<Child>,
    wrapper: ShmemWrapper,
    data_size: usize,
    request_data: Vec<u8>,
    response_data: Vec<u8>,
}

impl ShmemRunner {
    pub fn new(start_child: bool, data_size: usize) -> ShmemRunner {
        let wrapper = ShmemWrapper::new(None, data_size);

        let id = wrapper.shmem.get_os_id();

        let exe = executable_path("shmem_consumer");

        let child_proc = if start_child {
            let res = Some(
                Command::new(exe)
                    .args(&[id.to_string(), data_size.to_string()])
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

        ShmemRunner {
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
                format!("Shared memory - {}KB", self.data_size / KB),
                elapsed,
                n,
            );
            res.print_info();
        }
    }
}

impl Drop for ShmemRunner {
    fn drop(&mut self) {
        if let Some(ref mut child) = self.child_proc {
            child.kill().expect("Unable to kill child process")
        }
    }
}
