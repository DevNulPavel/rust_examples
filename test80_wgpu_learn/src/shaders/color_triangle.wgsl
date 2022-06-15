struct VertexInput {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] color: vec3<f32>;
};

// Структурка с выходными значениями
struct VertexOutput {
    // Описываем непосредственно координаты выходные, то есть аналог gl_Position
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] color: vec3<f32>;
};


// Описываем наш вершинный шейдер
// В качестве параметра мы имеем индекс вершины в отрисуемом списке: [[builtin(vertex_index)]] in_vertex_index: u32, 
[[stage(vertex)]]
fn vs_main(input: VertexInput) -> VertexOutput {
    // Создаем переменную с выходными значениями
    var out: VertexOutput;
    
    // В качестве выходных значений выдаем наружу координаты
    out.clip_position = vec4<f32>(input.position, 1.0);
    out.color = input.color;

    return out;
}
 
// Помечаем функцию как фрагментный шейдер
// возвращаемое значение в [[location(0)]] значит, что значение пишем в первый таргет цвета
[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}