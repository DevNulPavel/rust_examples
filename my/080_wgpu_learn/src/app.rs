use crate::{
    pipeline_builders::build_color_triangle_pipeline, render_context::RenderContext,
    figures::{VERTICES, INDICES},
};
use eyre::Context;
use log::debug;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, BufferUsages, Color, RenderPipeline,
};
use winit::{
    dpi::PhysicalPosition,
    event::*,
    event_loop::{ControlFlow, EventLoopProxy},
    window::Window,
};

pub struct App {
    window: Window,
    //loop_proxy: EventLoopProxy<()>,
    render_context: RenderContext,
    clear_color: wgpu::Color,
    previous_mouse_pos: PhysicalPosition<f64>,
    mouse_drag_active: bool,
    // Render pipeline
    color_triangle_pipeline: RenderPipeline,
    // Triangle buffer
    triangle_vertex_buffer: Buffer,
    //triangle_vertex_len: u32,
    triangle_index_buffer: Buffer,
    triangle_index_len: u32,
}

impl App {
    pub fn new(
        _loop_proxy: EventLoopProxy<()>,
        window: Window,
        render_context: RenderContext,
    ) -> Self {
        // Создаем разные пайплайны заранее
        let color_triangle_pipeline = build_color_triangle_pipeline(&render_context);

        // Создаем буффер для вершин
        let triangle_vertex_buffer =
            render_context
                .device
                .create_buffer_init(&BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    // Ссылка на данные, которые будут загружаться на видеокарту
                    contents: bytemuck::cast_slice(VERTICES),
                    // Описываем как будет использоваться буффер
                    usage: BufferUsages::VERTEX,
                });
        // Создаем буффер для индексов
        let triangle_index_buffer = render_context
            .device
            .create_buffer_init(&BufferInitDescriptor {
                label: Some("Index Buffer"),
                // Ссылка на данные, которые будут загружаться на видеокарту
                contents: bytemuck::cast_slice(INDICES),
                // Описываем как будет использоваться буффер
                usage: BufferUsages::INDEX,
            });

        App {
            // loop_proxy,
            window,
            render_context,
            clear_color: Color {
                r: 0.1,
                g: 0.1,
                b: 0.1,
                a: 0.1,
            },
            mouse_drag_active: false,
            previous_mouse_pos: PhysicalPosition { x: 0.0, y: 0.0 },
            color_triangle_pipeline,
            triangle_vertex_buffer,
            //triangle_vertex_len: VERTICES.len() as u32,
            triangle_index_buffer,
            triangle_index_len: INDICES.len() as u32,
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
            WindowEvent::KeyboardInput { input, .. } => match input {
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
                    self.window.request_redraw();
                }

                // Все остальное
                _ => {}
            },

            // Событие мышки
            WindowEvent::MouseInput { state, button, .. } => match button {
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
            WindowEvent::CursorMoved { position, .. } => {
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

            // Для рендер прохода выставляем наш пайплайн для рендеринга
            render_pass.set_pipeline(&self.color_triangle_pipeline);
            // Выставляем вертекс буффер
            render_pass.set_vertex_buffer(0, self.triangle_vertex_buffer.slice(..));
            // Выставляем буфер индексов
            render_pass.set_index_buffer(self.triangle_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            // Рисуем один раз вершины от 0 до 3
            render_pass.draw_indexed(0..self.triangle_index_len, 0, 0..1);
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
