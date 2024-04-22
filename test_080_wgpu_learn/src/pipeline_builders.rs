use crate::{render_context::RenderContext, vertex::Vertex};
use wgpu::{
    include_wgsl, BlendState, ColorTargetState, ColorWrites, Face, FragmentState, FrontFace,
    MultisampleState, PipelineLayoutDescriptor, PolygonMode, PrimitiveState, PrimitiveTopology,
    RenderPipeline, RenderPipelineDescriptor, VertexState,
};

/// Создаем пайплайн для рендеринга цветного треугольника
pub fn build_color_triangle_pipeline(render_context: &RenderContext) -> RenderPipeline {
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
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

    // Непосредственно сам лаяут рендеринга
    render_context
        .device
        .create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            // Описание обработки вершин
            vertex: VertexState {
                module: &shader,
                entry_point: "vs_main",           // Функция в шейдере
                buffers: &[Vertex::get_layout()], // Буфферы для отрисовки, пока испольузются лишь индексы, так что буффер пустой
            },
            // Описание обработки пикселей, она опциональная
            fragment: Some(FragmentState {
                module: &shader,        // Указываем имя фрагментного шейдера
                entry_point: "fs_main", // Имя функции в шейдере
                targets: &[ColorTargetState {
                    format: render_context.config.format,
                    blend: Some(BlendState::REPLACE), // Полность покрашиваем пиксели
                    write_mask: ColorWrites::ALL,     // Пишем полностью все цвета в буффер цвета
                }],
            }),
            // Описываем способ интерпритации вершин из входного буфера
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList, // Используем список треугольников
                strip_index_format: None,
                front_face: FrontFace::Ccw, // Обход против часовой стрелки будет
                cull_mode: Some(Face::Back), // Задняя грань отбрасыается
                polygon_mode: PolygonMode::Fill, // Полигоны заполняем при рендеринге
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
            multisample: MultisampleState {
                count: 1,                         // Пока не используем никак
                mask: !0_u64, // Сейчас используем все пиксели, поэтому маска полная
                alpha_to_coverage_enabled: false, //
            },
            // Не рендерим пока в массив буфферов
            multiview: None, // 5.
        })
}
