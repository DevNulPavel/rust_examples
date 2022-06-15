use eyre::{Context, ContextCompat};
use log::debug;
use wgpu::{
    Backends, Device, Features, Instance, PowerPreference, Queue, RequestAdapterOptions, Surface,
    SurfaceConfiguration,
};
use winit::dpi::PhysicalSize;
use winit::window::Window;

pub struct RenderContext {
    pub surface: Surface,
    pub device: Device,
    pub queue: Queue,
    pub config: SurfaceConfiguration,
    pub size: PhysicalSize<u32>,
}

impl RenderContext {
    // Создание рендеринга требует асинхронности
    pub async fn new(window: &Window) -> Result<Self, eyre::Error> {
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

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn rebuil_surface(&mut self) {
        self.resize(self.size);
    }
}
