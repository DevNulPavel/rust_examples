use super::errors::{Error, ErrorKind, Result};
use super::platform;
use super::system::System;
use glium::glutin::{Api, ContextBuilder, EventsLoop, GlProfile, GlRequest, WindowBuilder};
use glium::{Display, Frame, Surface};

const OPENGL_DEPTH_SIZE: u8 = 24;

// Конфигурация окна
pub struct WindowConfig {
    pub width: u32,
    pub height: u32,
    pub title: String,
}

// Структура, описывающее окно
pub struct Window {
    display: Display,   // glium окно
    events: EventsLoop, // Ивент луп окна
    width: u32,
    height: u32,
}

impl Window {
    // Геттер ширины
    pub fn width(&self) -> u32 {
        self.width
    }

    // Геттер высоты
    pub fn height(&self) -> u32 {
        self.height
    }

    // Соотношение сторон
    pub fn aspect_ratio(&self) -> f32 {
        self.width as f32 / self.height as f32
    }
    
    // Вызов отрисовки, возвращает окно отрисовки
    pub fn draw(&self) -> Frame {
        let mut frame = self.display.draw();
        frame.clear_all_srgb((0.06, 0.07, 0.09, 0.0), 1.0, 0);
        frame
    }

    // Получаем список ивентов
    pub fn events(&mut self) -> &mut EventsLoop {
        &mut self.events
    }

    // Получаем окошко отрисовки
    pub fn facade(&self) -> &Display {
        &self.display
    }
}

// Реализация трейта системы приложения для окна
impl<'context> System<'context> for Window {
    // Данная система зависит от ссылкип конфига окна
    type Dependencies = &'context WindowConfig;
    // Ошибка стандартная
    type Error = Error;

    // Создание окна
    fn create(config: &'context WindowConfig) -> Result<Self> {
        // Создаем ивент луп
        let events = EventsLoop::new();

        // Создание окна
        let window = WindowBuilder::new()
            .with_dimensions((config.width, config.height).into()) // Можно создать переменную нужного типа с помощью трейта into
            .with_title(config.title.clone());

        // Создаем контекст
        let context = ContextBuilder::new()
            .with_gl_profile(GlProfile::Core) // Core профиль OpenGL
            .with_gl(GlRequest::Specific(
                Api::OpenGl,
                (platform::GL_MAJOR_VERSION, platform::GL_MINOR_VERSION), // Создаем контекст нужного типа
            ))
            .with_vsync(true) // Вертикальная синхронизация
            .with_depth_buffer(OPENGL_DEPTH_SIZE); // 24х битный буффер глубины

        // Создаем окно
        let display = Display::new(window, context, &events)
            .map_err(ErrorKind::create_window(config.width, config.height))?;
        
        // Возврачаем окно
        Ok(Window {
            display,
            events,
            width: config.width,
            height: config.height,
        })
    }

    fn debug_name() -> &'static str {
        "window"
    }
}
