use eyre::{Context, ContextCompat};
use log::{debug, info, LevelFilter};
use std::{borrow::Cow, future::Future};
use wgpu::{
    Adapter, Backends, CommandEncoderDescriptor, Device, DeviceDescriptor, Features, FragmentState, Instance, Limits, LoadOp,
    MultisampleState, Operations, PipelineLayoutDescriptor, PowerPreference, PresentMode, PrimitiveState, Queue, RenderPassColorAttachment,
    RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, RequestAdapterOptions, RequestDeviceError, ShaderModuleDescriptor,
    ShaderSource, SurfaceConfiguration, TextureUsages, TextureViewDescriptor, VertexState,
};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Настойка уровня логирования
fn setup_logging() -> Result<(), eyre::Error> {
    pretty_env_logger::formatted_timed_builder()
        .filter_level(LevelFilter::Trace)
        .try_init()?;
    Ok(())
}

/////////////////////////////////////////////////////////////////////////////////////

fn print_adapters_info(instance: &Instance, backends: Backends) {
    instance.enumerate_adapters(backends).for_each(|adapter| {
        info!(
            "{:?}, downlevel props: {:?}, features: {:?}",
            adapter.get_info(),
            adapter.get_downlevel_properties(),
            adapter.features()
        );
    });
}

fn request_adapter(instance: &Instance, surface: &wgpu::Surface) -> impl Future<Output = Option<Adapter>> + Send {
    instance.request_adapter(&RequestAdapterOptions {
        // Мощный или слабый адаптер?
        power_preference: PowerPreference::HighPerformance,
        // Разрешаем софтварный рендер если нету другого
        force_fallback_adapter: false,
        // Передаем наш сурфейс для рендеринга
        compatible_surface: Some(surface),
    })
}

fn request_device(adapter: &wgpu::Adapter) -> impl Future<Output = Result<(Device, Queue), RequestDeviceError>> + Send {
    adapter.request_device(
        &DeviceDescriptor {
            // Отладочное имя девайса
            label: Some("Rendering device"),
            // Список фич девайса, которые должны поддерживаться обязательно
            features: Features::empty(),
            // Дополнительные ограничения для девайса
            // Указываем стандартные ограничения максимального размера текстуры, поддерживаемые адаптером
            limits: Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits()),
        },
        None,
    )
}

fn create_render_pipeline(
    device: &Device,
    pipeline_layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
    swapchain_format: wgpu::TextureFormat,
) -> RenderPipeline {
    device.create_render_pipeline(&RenderPipelineDescriptor {
        // Отладочное имя
        label: Some("Red plain triangle rendering"),
        // Лаяут рендеринга
        layout: Some(pipeline_layout),
        // Обработка вершин, указываем имя функции и шейдер
        vertex: VertexState {
            // Шейдер в виде ссылки
            module: shader,
            // Имя  функции
            entry_point: "vs_main",
            // TODO: Формат вершинных буфферов данных
            buffers: &[],
        },
        // Обработка фрагментов
        fragment: Some(FragmentState {
            // Шейдер
            module: shader,
            // Имя функции
            entry_point: "fs_main",
            // TODO: Формат выходной текстуры?
            targets: &[swapchain_format.into()],
        }),
        // Формат наших данных для рендеринга
        primitive: PrimitiveState::default(),
        // Тест глубины
        depth_stencil: None,
        // Режим мультисемплинга
        multisample: MultisampleState::default(),
        // Мультивью с рендерингом в разные таргеты
        multiview: None,
    })
}

fn perform_render(surface: &wgpu::Surface, device: &Device, render_pipeline: &RenderPipeline, queue: &Queue) {
    let frame = surface.get_current_texture().expect("Failed to acquire next swap chain texture");
    let view = frame.texture.create_view(&TextureViewDescriptor::default());
    let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
        label: Some("Triangle render encoder"),
    });
    {
        // Создаем ренде-проход
        let mut rpass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Render pass for triangle"),
            // Куда именно рендерим
            color_attachments: &[RenderPassColorAttachment {
                // В какую текстуру сурфейса
                view: &view,
                resolve_target: None,
                // Что делаем с текстурой сурфейса, когда начинаем рендеринг
                ops: Operations {
                    load: LoadOp::Clear(wgpu::Color::BLACK),
                    store: true,
                },
            }],
            // Аттачмент для глубины
            depth_stencil_attachment: None,
        });

        // Установка пайплайнов
        rpass.set_pipeline(&render_pipeline);

        // Рисуем 3 вершины, 1 треугольник
        rpass.draw(0..3, 0..1);
    }
    queue.submit(Some(encoder.finish()));
    frame.present();
}

async fn execute_app() -> Result<(), eyre::Error> {
    // Создаем инстанс WGPU
    let backends = Backends::all();
    let instance = Instance::new(backends);

    // Создаем окно (нативный хендл) и event loop для этого самого окошка
    let event_loop = EventLoop::new();
    let window = Window::new(&event_loop).wrap_err("Create window failed")?;
    let render_size = window.inner_size();

    // Создаем сурфейс рендеринга для нашего окошка, данный вызов небезопасный
    let surface = unsafe { instance.create_surface(&window) };

    // Выводим список адаптеров в системе
    print_adapters_info(&instance, backends);

    // Получаем адаптер, который мы хотим
    let adapter = request_adapter(&instance, &surface)
        .await
        .wrap_err("Failed to find an appropriate adapter")?;

    // Создаем девайс и очередь команд для этого девайса
    let (device, queue) = request_device(&adapter).await.wrap_err("Failed to create device")?;
    info!("Device info: {:?}", device);
    info!("Queue info: {:?}", queue);

    // Загружаем шейдер, текст шейдера мы включаем сразу в наш бинарник
    let shader = device.create_shader_module(&ShaderModuleDescriptor {
        label: Some("Triangle shader"),
        source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("../shaders/triangle.wgsl"))),
    });
    info!("Shader data: {:?}", shader);

    // TODO: Создаем описание пайплайна данных для шейдера?
    let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some("Red plain render pipeline layout"),
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });

    // Для нашего сурфейса выбираем удобный формат
    let swapchain_format = surface.get_preferred_format(&adapter).wrap_err("Preffered format fetch failed")?;

    // Создаем пайплайн рендеринга, передавая туда все то, что нам надо для рендеринга треугольника
    let render_pipeline = create_render_pipeline(&device, &pipeline_layout, &shader, swapchain_format);
    info!("Pipeline info: {:?}", render_pipeline);

    // Конфиг сурфейса для рендеринга
    // Мутабельный конфиг передается в event-loop потом
    let mut surface_config = SurfaceConfiguration {
        // Сурфейс нужен для рендеринга в окне
        usage: TextureUsages::RENDER_ATTACHMENT,
        // Формат свапчейна
        format: swapchain_format,
        // Ширина и высота рендеринга
        width: render_size.width,
        height: render_size.height,
        // Режим отображения опирающийся на вертикальную синхронизацию, но кадры можно вкидывать не дожидаясь
        // вертикальной синхронизации.
        // Оптимально для настольных девайсов.
        // Для мобил лучше fifo
        present_mode: PresentMode::Mailbox,
    };
    surface.configure(&device, &surface_config);

    // Стартуем event-loop рендеринга и обработки событий окна
    event_loop.run(move |event, _loop, control_flow| {
        // Данное замыкание принимает во владение переданные ресурсы
        // Так как `event_loop.run` никогда не завершается, тогда нам
        // надо упомянуть все объекты в данном замыкании, чтобы они благополучно уничтожились
        let _ = (&instance, &adapter, &shader, &pipeline_layout);

        // Говорим EventLoop, что надо подождать до очередной итерации в штатном режиме
        // Хотя удобнее было бы, чтобы коллбек просто возвращал статус
        *control_flow = ControlFlow::Wait;

        // Обработка прилетевших ивентов
        match event {
            // Был запрос перерисовки изображения
            // Вызывается при сбросе контекста рендеринга окна
            Event::RedrawRequested(_) => {
                // TODO: А можем ли мы тут как-то обработать ошибку потери текстуры рендеринга?
                // Получаем текстуру рендер-таргета
                perform_render(&surface, &device, &render_pipeline, &queue);
            }

            // Событие окна на ресайз
            Event::WindowEvent {
                // Обрабатываем только ресайз
                event: WindowEvent::Resized(size),
                ..
            } => {
                // Обновляем значения в конфиге, заново перестраиваем сурфейс
                surface_config.width = size.width;
                surface_config.height = size.height;
                surface.configure(&device, &surface_config);
            }

            // Событие закрытия окна
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,

            // Все остальные события никак не обрабатываем
            _ => {}
        }
    });
}

fn main() {
    // Настройка color eyre для ошибок
    color_eyre::install().expect("Error setup failed");

    // Настройка логирования на основании количества флагов verbose
    setup_logging().expect("Logging setup");

    // Запуск приложения
    if let Err(err) = pollster::block_on(execute_app()) {
        // При ошибке не паникуем, а спокойно выводим сообщение и завершаем приложение с кодом ошибки
        eprint!("Error! Failed with: {:?}", err);
        std::process::exit(1);
    }
}
