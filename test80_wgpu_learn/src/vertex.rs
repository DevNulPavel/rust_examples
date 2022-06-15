use bytemuck::{Pod, Zeroable};
use const_field_offset::FieldOffsets;
use std::mem::size_of;
use wgpu::{BufferAddress, VertexAttribute, VertexBufferLayout, VertexStepMode};

#[repr(C)]
#[derive(FieldOffsets, Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
}

impl Vertex {
    pub fn get_layout() -> VertexBufferLayout<'static> {
        /*
        use wgpu::VertexFormat;
        const ATTRIBUTES: &[VertexAttribute] = &[
            // Аттрибует координаты вершины
            VertexAttribute {
                offset: Vertex::FIELD_OFFSETS.position.get_byte_offset() as BufferAddress,
                shader_location: 0,
                format: VertexFormat::Float32x3, // Размерность в байтах
            },
            // Аттрибут цвета вершины
            VertexAttribute {
                offset: Vertex::FIELD_OFFSETS.color.get_byte_offset() as BufferAddress,
                shader_location: 1,
                format: VertexFormat::Float32x3,
            },
        ];*/

        // Укороченный вариант записи с помощью макроса
        const ATTRIBUTES: &[VertexAttribute] =
            &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];

        VertexBufferLayout {
            // Вычисляем размер отдельной вершины
            array_stride: size_of::<Vertex>() as BufferAddress,
            // Режим обхода - повершинно
            step_mode: VertexStepMode::Vertex,
            // Описание аттрибутов вершин
            attributes: ATTRIBUTES,
        }
    }
}

pub const VERTICES: &[Vertex] = &[
    Vertex {
        position: [0.0, 0.5, 0.0],
        color: [1.0, 0.0, 0.0],
    },
    Vertex {
        position: [-0.5, -0.5, 0.0],
        color: [0.0, 1.0, 0.0],
    },
    Vertex {
        position: [0.5, -0.5, 0.0],
        color: [0.0, 0.0, 1.0],
    },
];
