use test32_vst_plugin;

fn main() {
    let mut plugin = test32_vst_plugin::BasicPlugin::default();

    let prev_input: Vec<f32> = vec![
        -9.0,
        -7.0,
        -6.0,
        -5.0,
        -4.0,
        -3.0,
        -2.0,
        -1.0,
    ];
    let input: [f32; 8] = [
        1.0,
        2.0,
        3.0,
        4.0,
        5.0,
        6.0,
        7.0,
        9.0
    ];
    let mut output: [f32; 8] = [0.0; 8];
    plugin.handle_data(0, &prev_input, &mut output, 100.0, 1.0);
    plugin.handle_data(0, &input, &mut output, 100.0, 1.0);

    println!("{:?}", output);
}