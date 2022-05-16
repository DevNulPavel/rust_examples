use eyre::{Context, ContextCompat};
use log::debug;
use std::env;
use wgpu::{
    Backends, Device, Features, Instance, PowerPreference, Queue, RequestAdapterOptions, Surface,
    SurfaceConfiguration,
};
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::*,
    event_loop::{ControlFlow, EventLoop, EventLoopProxy},
    window::{Window, WindowBuilder},
};

fn init_log() -> Result<(), eyre::Error> {
    const LOG_VAR: &str = "RUST_LOG";
    if env::var(LOG_VAR).is_err() {
        env::set_var(LOG_VAR, "debug");
    }
    pretty_env_logger::try_init().wrap_err("Logger setup failed")?;

    Ok(())
}

struct App {
    window: Window,
    loop_proxy: EventLoopProxy<()>,
    render_context: RenderContext,
    clear_color: wgpu::Color,
    previous_mouse_pos: PhysicalPosition<f64>,
    mouse_drag_active: bool,
}

impl App {
    fn process_window_event(&mut self, event: WindowEvent, control_flow: &mut ControlFlow) {
        match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::KeyboardInput {
                input,
                device_id,
                is_synthetic,
            } => {
                todo!()
            }
            WindowEvent::MouseInput {
                device_id,
                state,
                button,
                ..
            } => match button {
                MouseButton::Left => match state {
                    ElementState::Pressed => {
                        self.mouse_drag_active = true;
                    }
                    ElementState::Released => {
                        self.mouse_drag_active = false;
                    }
                },
                MouseButton::Right => match state {
                    ElementState::Pressed => {}
                    ElementState::Released => {}
                },
                _ => {}
            },
            WindowEvent::CursorMoved {
                device_id,
                position,
                ..
            } => {
                if self.mouse_drag_active {
                    let diff_x = position.x - self.previous_mouse_pos.x;
                    let diff_y = position.y - self.previous_mouse_pos.y;

                    // Перерисовкой будем заниматься только если двигаем мышку
                    self.window.request_redraw();

                    debug!("Mouse move diff: ({}, {})", diff_x, diff_y);
                }

                self.previous_mouse_pos = position;
            }
            WindowEvent::Resized(size) => {
                self.render_context.resize(size);
                self.window.request_redraw();
            }
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                // new_inner_size is &&mut so we have to dereference it twice
                self.render_context.resize(*new_inner_size);
                self.window.request_redraw();
            }
            _ => {}
        }
    }

    fn update(&self) {}

    fn render(&self) -> Result<(), eyre::Error> {
        let output = self
            .render_context
            .surface
            .get_current_texture()
            .wrap_err("No current texture in surface")?;

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder =
            self.render_context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });
        }

        // submit will accept anything that implements IntoIter
        self.render_context
            .queue
            .submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

struct RenderContext {
    surface: Surface,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    size: PhysicalSize<u32>,
}

impl RenderContext {
    // Создание рендеринга требует асинхронности
    async fn new(window: &Window) -> Result<Self, eyre::Error> {
        // Создаем инстас wgpu
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = Instance::new(Backends::all());

        // Создаем сурфейс
        let surface = unsafe { instance.create_surface(&window) };
        debug!("Surface created: {:?}", surface);

        // Содбираем адаптер для указанного сурфейса
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .wrap_err("Create adapter")?;
        debug!("Adapter received: {:?}", adapter);

        // Получаем девайс и очередь из адаптера
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    label: None,
                },
                None, // Trace path
            )
            .await
            .wrap_err("Device search")?;

        // Получаем размер окошка
        let size = window.inner_size();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_preferred_format(&adapter).unwrap(),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(&device, &config);

        Ok(RenderContext {
            surface,
            config,
            device,
            queue,
            size,
        })
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn rebuil_surface(&mut self) {
        self.resize(self.size);
    }

    // fn update(&mut self) {
    //     todo!()
    // }

    // fn render(&mut self) -> Result<(), SurfaceError> {
    //     todo!()
    // }
}

fn run_main_loop(event_loop: EventLoop<()>, mut app: App) -> ! {
    event_loop.run(move |event, _target, control_flow| match event {
        // Прилетело какое-то оконное событие к нам
        Event::WindowEvent { event, window_id } => {
            // Может быть событие было не для нашего окна?
            if window_id != app.window.id() {
                return;
            }

            app.process_window_event(event, control_flow);
        }
        Event::RedrawRequested(window_id) => {
            debug!("Redraw request");

            // Может быть событие было не для нашего окна?
            if window_id != app.window.id() {
                return;
            }

            // Пробуем рендерить
            if let Err(err) = app.render() {
                if let Some(err) = err.downcast_ref::<wgpu::SurfaceError>() {
                    match err {
                        // Потеряли контекст отрисовки, прото пробуем заново пересоздать через ресайз
                        wgpu::SurfaceError::Lost => {
                            app.render_context.rebuil_surface();
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

    let app = App {
        loop_proxy,
        window,
        render_context,
        clear_color: wgpu::Color {
            r: 0.1,
            g: 0.1,
            b: 0.1,
            a: 0.1,
        },
        mouse_drag_active: false,
        previous_mouse_pos: PhysicalPosition { x: 0.0, y: 0.0 },
    };

    // Запускаем цикл в работу и сохраняем все в приложени
    run_main_loop(event_loop, app);
}
