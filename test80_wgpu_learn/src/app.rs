use crate::render_context::RenderContext;
use eyre::Context;
use log::debug;
use std::rc::Rc;
use wgpu::{include_wgsl, RenderPipeline};
use winit::{
    dpi::PhysicalPosition,
    event::*,
    event_loop::{ControlFlow, EventLoop, EventLoopProxy},
    window::{Window, WindowBuilder},
};

enum RenderMode {
    Simple,
    Color,
}

pub struct App {
    window: Window,
    loop_proxy: EventLoopProxy<()>,
    render_context: RenderContext,
    clear_color: wgpu::Color,
    previous_mouse_pos: PhysicalPosition<f64>,
    mouse_drag_active: bool,
    simple_triangle_pipeline: RenderPipeline,
    color_triangle_pipeline: RenderPipeline,
    render_mode: RenderMode,
}

impl App {
    pub fn new(
        loop_proxy: EventLoopProxy<()>,
        window: Window,
        render_context: RenderContext,
    ) -> Self {
        // Создаем разные пайплайны заранее
        let simple_triangle_pipeline = create_simple_triangle_pipeline(&render_context);
        let color_triangle_pipeline = create_color_triangle_pipeline(&render_context);

        App {
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
            simple_triangle_pipeline,
            color_triangle_pipeline,
            render_mode: RenderMode::Simple,
        }
    }

    /// Получение ссылки для окна
    pub fn get_window(&self) -> &Window {
        &self.window
    }

    /// После ресайза надо заново пересоздать контекст
    pub fn rebuil_surface(&mut self) {
        self.render_context.rebuil_surface();
    }

    /// Выполняем рендеринг
    pub fn process_window_event(&mut self, event: WindowEvent, control_flow: &mut ControlFlow) {
        match event {
            // Событие закрытия приложения
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,

            // Клавиатурное событие
            WindowEvent::KeyboardInput {
                input,
                device_id,
                is_synthetic,
            } => match input {
                // Клавиша escape
                KeyboardInput {
                    virtual_keycode: Some(VirtualKeyCode::Escape),
                    ..
                } => *control_flow = ControlFlow::Exit,

                // Клавиша пробел
                KeyboardInput {
                    virtual_keycode: Some(VirtualKeyCode::Space),
                    state: ElementState::Pressed,
                    ..
                } => {
                    self.render_mode = match self.render_mode {
                        RenderMode::Simple => RenderMode::Color,
                        RenderMode::Color => RenderMode::Simple,
                    };
                    self.window.request_redraw();
                }

                // Все остальное
                _ => {}
            },

            // Событие мышки
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

    /// Обновление состояния
    // pub fn update(&self) {}

    /// Рендеринг
    pub fn render(&self) -> Result<(), eyre::Error> {
        // Получаем текстуру для рендеринга в нее в будущем
        let output = self
            .render_context
            .surface
            .get_current_texture()
            .wrap_err("No current texture in surface")?;

        // Вьюшка для рендеринга в нее
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Создаем энкодер команд для ренедринга
        let mut encoder =
            self.render_context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        {
            // Создаем рендер-проход
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                // Прицепляем к рендер проходу текстуру для рендеринга
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    // Вьюшка для рендеринга
                    view: &view,
                    // Вьюшка, которая будет получать рендеринг если включен мультисемплинг
                    resolve_target: None,
                    // Что мы делаем при загрузке и надо ли сохранять результат
                    ops: wgpu::Operations {
                        // Чистим текстуру при начале работы цветом очистки
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        // Сохраняем результат
                        store: true,
                    },
                }],
                // Аттачмент для глубины и трафарета
                depth_stencil_attachment: None,
            });

            // Выбираем нужный пайплайн
            let pipeline = match self.render_mode {
                RenderMode::Simple => &self.simple_triangle_pipeline,
                RenderMode::Color => &self.color_triangle_pipeline,
            };

            // Для рендер прохода выставляем наш пайплайн для рендеринга
            render_pass.set_pipeline(pipeline);
            // Рисуем один раз вершины от 0 до 3
            render_pass.draw(0..3, 0..1);
        }

        // Ставим в очередь команды рендеринга
        self.render_context
            .queue
            .submit(std::iter::once(encoder.finish()));

        // Вызываем после submit запрос показа текстуры
        output.present();

        Ok(())
    }
}

/// Создаем простой пайплайн для рендеринга простого треугольника
fn create_simple_triangle_pipeline(render_context: &RenderContext) -> RenderPipeline {
    // Шейдер
    // let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
    //     label: Some("Shader"),
    //     source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
    // });
    let shader = render_context
        .device
        .create_shader_module(&include_wgsl!("shaders/simple_triangle.wgsl")); // Кототкий вариант записи

    // Лаяут нашего пайплайна
    let render_pipeline_layout =
        render_context
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

    // Непосредственно сам лаяут рендеринга
    let pipeline = render_context
        .device
        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            // Описание обработки вершин
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main", // Функция в шейдере
                buffers: &[], // Буфферы для отрисовки, пока испольузются лишь индексы, так что буффер пустой
            },
            // Описание обработки пикселей, она опциональная
            fragment: Some(wgpu::FragmentState {
                module: &shader,        // Указываем имя фрагментного шейдера
                entry_point: "fs_main", // Имя функции в шейдере
                targets: &[wgpu::ColorTargetState {
                    format: render_context.config.format,
                    blend: Some(wgpu::BlendState::REPLACE), // Полность покрашиваем пиксели
                    write_mask: wgpu::ColorWrites::ALL, // Пишем полностью все цвета в буффер цвета
                }],
            }),
            // Описываем способ интерпритации вершин из входного буфера
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // Используем список треугольников
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // Обход против часовой стрелки будет
                cull_mode: Some(wgpu::Face::Back), // Задняя грань отбрасыается
                polygon_mode: wgpu::PolygonMode::Fill, // Полигоны заполняем при рендеринге
                // Если мы выходим за границы от 0 до 1 по глубине, нужно ли отбрасывать пиксель?
                unclipped_depth: false, // Requires Features::DEPTH_CLIP_CONTROL
                // Заполняется каждый пиксель при рендеринге если режим Fill
                // Иначе можно было бы использовать оптимизации в духе
                // Наверное тут речь про включенный режим отрисовки пискелей с двух сторон?
                conservative: false, // Requires Features::CONSERVATIVE_RASTERIZATION
            },
            // Пока не используем никак буффер трафарета
            depth_stencil: None,
            // Режим мультисемплинга
            multisample: wgpu::MultisampleState {
                count: 1,                         // Пока не используем никак
                mask: !0_u64, // Сейчас используем все пиксели, поэтому маска полная
                alpha_to_coverage_enabled: false, //
            },
            // Не рендерим пока в массив буфферов
            multiview: None, // 5.
        });

    pipeline
}

/// Создаем пайплайн для рендеринга цветного треугольника
fn create_color_triangle_pipeline(render_context: &RenderContext) -> RenderPipeline {
    // Шейдер
    // let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
    //     label: Some("Shader"),
    //     source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
    // });
    let shader = render_context
        .device
        .create_shader_module(&include_wgsl!("shaders/color_triangle.wgsl")); // Кототкий вариант записи

    // Лаяут нашего пайплайна
    let render_pipeline_layout =
        render_context
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

    // Непосредственно сам лаяут рендеринга
    let pipeline = render_context
        .device
        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            // Описание обработки вершин
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main", // Функция в шейдере
                buffers: &[], // Буфферы для отрисовки, пока испольузются лишь индексы, так что буффер пустой
            },
            // Описание обработки пикселей, она опциональная
            fragment: Some(wgpu::FragmentState {
                module: &shader,        // Указываем имя фрагментного шейдера
                entry_point: "fs_main", // Имя функции в шейдере
                targets: &[wgpu::ColorTargetState {
                    format: render_context.config.format,
                    blend: Some(wgpu::BlendState::REPLACE), // Полность покрашиваем пиксели
                    write_mask: wgpu::ColorWrites::ALL, // Пишем полностью все цвета в буффер цвета
                }],
            }),
            // Описываем способ интерпритации вершин из входного буфера
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // Используем список треугольников
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // Обход против часовой стрелки будет
                cull_mode: Some(wgpu::Face::Back), // Задняя грань отбрасыается
                polygon_mode: wgpu::PolygonMode::Fill, // Полигоны заполняем при рендеринге
                // Если мы выходим за границы от 0 до 1 по глубине, нужно ли отбрасывать пиксель?
                unclipped_depth: false, // Requires Features::DEPTH_CLIP_CONTROL
                // Заполняется каждый пиксель при рендеринге если режим Fill
                // Иначе можно было бы использовать оптимизации в духе
                // Наверное тут речь про включенный режим отрисовки пискелей с двух сторон?
                conservative: false, // Requires Features::CONSERVATIVE_RASTERIZATION
            },
            // Пока не используем никак буффер трафарета
            depth_stencil: None,
            // Режим мультисемплинга
            multisample: wgpu::MultisampleState {
                count: 1,                         // Пока не используем никак
                mask: !0_u64, // Сейчас используем все пиксели, поэтому маска полная
                alpha_to_coverage_enabled: false, //
            },
            // Не рендерим пока в массив буфферов
            multiview: None, // 5.
        });

    pipeline
}
