use super::system::InfallibleSystem;
use std::thread;
use std::time::{Duration, Instant};

// Конфигурация частоты кадров
pub struct Config {
    pub timestep: f32,
}

// Комер тика с начала работы приложения
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct TickIndex(pub u64);

pub struct Tick {
    // Время с последнего кадра
    timestep: f32,
    // Индекс тика со старта приложения
    index: TickIndex,

    // Отклонение во времени
    drift: f32,
    slept: f32,
    last_time: Option<Instant>,
    need_render_frame: bool,
}

// Реализация структуры тика
impl Tick {
    #[inline]
    pub fn need_render_frame(&self) -> bool {
        self.need_render_frame
    }

    #[inline]
    pub fn timestep(&self) -> f32 {
        self.timestep
    }

    #[inline]
    pub fn index(&self) -> TickIndex {
        self.index
    }

    #[inline]
    pub fn drift(&self) -> f32 {
        self.drift
    }

    #[inline]
    pub fn slept(&self) -> f32 {
        self.slept
    }

    #[inline]
    pub fn seconds_since_tick(&self, index: TickIndex) -> f32 {
        if index.0 < self.index.0 {
            (self.index.0 - index.0) as f32 * self.timestep
        } else {
            (index.0 - self.index.0) as f32 * (-self.timestep)
        }
    }
}

// Реализация методов системного компонента для Tick
impl<'context> InfallibleSystem<'context> for Tick {
    type Dependencies = &'context Config;

    // Отладочное имя
    fn debug_name() -> &'static str {
        "tick"
    }

    // Создание объекта из конфига, вызывается при конструировании системы
    fn create(config: &Config) -> Self {
        Tick {
            timestep: config.timestep,
            index: TickIndex(0),

            drift: 0.0,
            slept: 0.0,
            last_time: None,
            need_render_frame: true,
        }
    }

    // Вызов очередного обновления кадра
    fn update(&mut self, _: &Config) {
        // Получаем текущее время
        let mut current_time = Instant::now();
        // Получаем прошлое время если есть, иначе не надо ничего обновлять
        let last_time = if let Some(instant) = self.last_time {
            instant
        } else {
            self.last_time = Some(current_time);
            return;
        };

        // Накапливаем ошибку, Accumulate drift: real_time - simulation_time
        let real_delta = duration_to_seconds(current_time.duration_since(last_time));
        self.drift += real_delta - self.timestep;

        // Если мы отрендерили кадр, но симуляция все еще опережает реальное время более чем на время очередного шага
        // тогда мы спим до момента синхронизации
        if self.drift < -self.timestep {
            let sleep_duration = duration_from_seconds(-self.drift - self.timestep + 1e-3);
            let sleep_until = current_time + sleep_duration;

            // Спим указанное время, минус 1 мСек
            thread::sleep(sleep_duration - Duration::from_millis(1));

            // Для точности активным образом ждем указанное время в цикле
            let new_current_time = loop {
                let new_current_time = Instant::now();
                if new_current_time >= sleep_until {
                    break new_current_time;
                }
                thread::yield_now();
            };

            // Обновляем значенияя
            self.slept = duration_to_seconds(new_current_time.duration_since(current_time));
            self.drift += self.slept;

            // Report the amount slept.
            current_time = new_current_time;
        } else {
            self.slept = 0.0;
        }
        self.last_time = Some(current_time);

        // Рендерим кадр, если отклонение меньше, чем один временной шаг
        self.need_render_frame = self.drift <= self.timestep;

        // Обновляем индекс тика
        self.index.0 += 1;
    }
}

fn duration_to_seconds(duration: Duration) -> f32 {
    duration.as_secs() as f32 + (duration.subsec_nanos() as f32 * 1e-9f32)
}

fn duration_from_seconds(seconds: f32) -> Duration {
    Duration::new(seconds as u64, ((seconds - seconds.floor()) * 1e9) as u32)
}
