use amethyst::{
    //prelude::*,
    derive::SystemDesc,
    winit::MouseButton,
    ecs::{
        Read,
        System,
        SystemData
    },
    input::{
        InputHandler,
        ControllerButton,
        VirtualKeyCode,
        StringBindings,
    },
};

/// Данная системма ответственна за перемещение всех шаров в соответствии с их скоростью
#[derive(SystemDesc)]
pub struct InputProcessingSystem{
    last_mouse_pos: Option<(f32, f32)>
}

impl Default for InputProcessingSystem{
    fn default() -> Self {
        InputProcessingSystem{
            last_mouse_pos: None
        }
    }
}

impl<'s> System<'s> for InputProcessingSystem {
    // Данные о входных ивентах
    type SystemData = Read<'s, InputHandler<StringBindings>>;

    fn run(&mut self, input: Self::SystemData) {
        // Получаем дельту мышки
        let mouse_delta = self.get_mouse_delta(&input);
        if let Some((delta_x, delta_y)) = mouse_delta{
            println!("Mouse delta: {} {}", delta_x, delta_y);
        }

        // Обработка подключенных контроллеров
        for controller in input.connected_controllers(){
            let pressed = input.controller_button_is_down(controller, ControllerButton::Back);
            if pressed {
                println!("Controller button pressed");
            }
        }

        // Обработка нажатых кнопок клавиатуры
        let up_pressed = input.key_is_down(VirtualKeyCode::Up);
        if up_pressed {
            println!("Up pressed");
        }

        // Левая кнопка мыши
        let left_mouse_pressed = input.mouse_button_is_down(MouseButton::Left);
        if left_mouse_pressed {
            println!("Left mouse pressed")
        }
    }
}

impl InputProcessingSystem{
    fn get_mouse_delta<'s>(&mut self, input: &Read<'s, InputHandler<StringBindings>>) -> Option<(f32, f32)>{

        let (x, y) = match input.mouse_position() {
            Some(pos) => {
                pos
            },
            None => {
                self.last_mouse_pos.take();
                return None;
            }
        };

        println!("Mouse pos: {} {}", x, y);

        // Получаем информацию об указателе мышки
        let delta= match self.last_mouse_pos.take(){
            Some((last_x, last_y)) => {
                let delta = (x - last_x, y - last_y);
                self.last_mouse_pos.replace((x, y));
                delta
            },
            None => {
                return None;
            }
        };
        
        println!("Mouse delta: {} {}", delta.0, delta.1);

        Some(delta)
    }
}