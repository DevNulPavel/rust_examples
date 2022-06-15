mod app;
mod render_context;

use self::{app::App, render_context::RenderContext};
use eyre::Context;
use log::{debug, warn};
use std::env;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn init_log() -> Result<(), eyre::Error> {
    const LOG_VAR: &str = "RUST_LOG";
    if env::var(LOG_VAR).is_err() {
        env::set_var(LOG_VAR, "debug");
    }
    pretty_env_logger::try_init().wrap_err("Logger setup failed")?;

    Ok(())
}

macro_rules! check_window_id {
    ($received_window_id: expr, $real_window_id: expr) => {
        if $received_window_id != $real_window_id {
            warn!("Redraw request for wrong window");
            return;
        }
    };
}

fn run_main_loop(event_loop: EventLoop<()>, mut app: App) -> ! {
    event_loop.run(move |event, _target, control_flow| match event {
        // Прилетело какое-то оконное событие к нам
        Event::WindowEvent { event, window_id } => {
            // Может быть событие было не для нашего окна?
            check_window_id!(window_id, app.get_window().id());

            app.process_window_event(event, control_flow);
        }
        Event::RedrawRequested(window_id) => {
            debug!("Redraw request");

            // Может быть событие было не для нашего окна?
            check_window_id!(window_id, app.get_window().id());

            // Пробуем рендерить
            if let Err(err) = app.render() {
                if let Some(err) = err.downcast_ref::<wgpu::SurfaceError>() {
                    match err {
                        // Потеряли контекст отрисовки, прото пробуем заново пересоздать через ресайз
                        wgpu::SurfaceError::Lost => {
                            app.rebuil_surface();
                        }
                        // Памяти не осталось, выходим
                        wgpu::SurfaceError::OutOfMemory => {
                            *control_flow = ControlFlow::Exit;
                        }
                        _ => {
                            eprintln!("Render error: {:?}", err);
                        }
                    }
                } else {
                    eprintln!("Render error: {:?}", err);
                }
            }
        }
        Event::RedrawEventsCleared => {
            debug!("Redraw clear");
        }
        Event::Suspended => {
            debug!("Suspended");
        }
        Event::Resumed => {
            debug!("Resumed");
        }
        Event::MainEventsCleared => {
            // RedrawRequested will only trigger once, unless we manually
            // request it.
            //app.window.request_redraw();

            // Чтобы перерисовка не происходила постоянно, а только тогда, когда нам это надо - тогда выставляем флаг ожидания события
            *control_flow = ControlFlow::Wait;
        }
        _ => {}
    })
}

fn main() -> Result<(), eyre::Error> {
    // Поддержка подробной инфы по ошибкам
    color_eyre::install()?;

    // Инициализируем log
    init_log()?;

    // Создаем Event-Loop
    let event_loop = EventLoop::new();

    // Окно рендеринга
    let window = WindowBuilder::new()
        .with_resizable(true)
        .with_title("WGPU")
        .build(&event_loop)
        .wrap_err("Window create failed")?;

    // Блокируемся на создании контекста
    let render_context = pollster::block_on(RenderContext::new(&window))?;

    // Прокси для основного лупера
    let loop_proxy = event_loop.create_proxy();

    // Создаем приложение
    let app = app::App::new(loop_proxy, window, render_context);

    // Запускаем цикл в работу и сохраняем все в приложени
    run_main_loop(event_loop, app);
}
