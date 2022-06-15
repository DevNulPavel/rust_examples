// Структурка с выходными значениями
struct VertexOutput {
    // Описываем непосредственно координаты выходные, то есть аналог gl_Position
    [[builtin(position)]] clip_position: vec4<f32>;
};

// Описываем наш вершинный шейдер
// В качестве параметра мы имеем индекс вершины в отрисуемом списке
[[stage(vertex)]]
fn vs_main([[builtin(vertex_index)]] in_vertex_index: u32) -> VertexOutput {
    // Создаем переменную с выходными значениями
    var out: VertexOutput;
    
    // Создаем координаты треугольника на основании индексов вершины от 0 до 2
    let x = f32(1 - i32(in_vertex_index)) * 0.5;
    let y = f32(i32(in_vertex_index & 1u) * 2 - 1) * 0.5;

    // В качестве выходных значений выдаем наружу координаты
    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);

    return out;
}
 
// Помечаем функцию как фрагментный шейдер
// возвращаемое значение в [[location(0)]] значит, что значение пишем в первый таргет цвета
[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    return vec4<f32>(0.3, 0.2, 0.1, 1.0);
}