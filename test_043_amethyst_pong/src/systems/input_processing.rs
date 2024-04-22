extern crate nalgebra_glm as glm;
use amethyst::{
    //prelude::*,
    derive::SystemDesc,
    winit::MouseButton,
    window::ScreenDimensions,
    core::{
        Transform,
    },
    renderer::Camera,
    ecs::{
        //Join,
        Read,
        ReadExpect,
        ReadStorage,
        WriteStorage,
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
use crate::{
    //constants::*,
    game_types::{
        PointerComponent,
        //CameraEntity
    }
};

/// Данная системма ответственна за перемещение всех шаров в соответствии с их скоростью
#[derive(SystemDesc)]
pub struct InputProcessingSystem{
    //last_mouse_pos: Option<(f32, f32)>
}

impl Default for InputProcessingSystem{
    fn default() -> Self {
        InputProcessingSystem{
            //last_mouse_pos: None
        }
    }
}

impl<'s> System<'s> for InputProcessingSystem {
    // Данные о входных ивентах
    type SystemData = (
        ReadExpect<'s, ScreenDimensions>,
        ReadStorage<'s, Camera>,
        Read<'s, InputHandler<StringBindings>>,
        ReadStorage<'s, PointerComponent>,
        WriteStorage<'s, Transform>
    );

    fn run(&mut self, (_screen, _cameras, input, _pointers, mut _transform): Self::SystemData) {
        //let screen: ReadExpect<'s, ScreenDimensions> = screen;
        //let cameras: ReadStorage<'s, Camera> = cameras;

        // Получаем дельту мышки
        /*let mouse_delta = self.get_mouse_delta(&input);
        if let Some((delta_x, delta_y)) = mouse_delta{
        }*/

        /*let matrices: Vec<glm::Mat4> = (&cameras, &transform)
            .join()
            .map(|(camera, cam_transform)|{
                let cam: &Camera = camera;
                let t: &Transform = cam_transform;

                // let cam_full_matrix: glm::Mat4 = (*cam.as_matrix()) * t.matrix();
                // let inverse: glm::Mat4 = cam_full_matrix.try_inverse().unwrap();

                let inverse: glm::Mat4 = *cam.as_inverse_matrix();

                inverse
            })
            .collect();

        matrices
            .into_iter()
            .for_each(|inverse_proj_matrix|{
                if let Some((x, y)) = input.mouse_position() {
                    // Указатель и трансформ указателя
                    for ( _, t) in (&pointers, &mut transform).join() {
                        //let pos_x = x / screen.width() * ARENA_HEIGHT;
                        //let pos_y = x / screen.height() * ARENA_HEIGHT;
        
                        let screen_pos_x = x / screen.width() * 2.0 - 1.0;
                        let screen_pos_y = y / screen.height() * 2.0 - 1.0;
        
                        let vector: glm::Vec4 = glm::vec4(screen_pos_x, screen_pos_y, 0.0, 0.0);
                        let real_pos: glm::Vec4 = inverse_proj_matrix * vector;
        
                        t.set_translation_x(real_pos.x);
                        t.set_translation_y(real_pos.y);
        
                        println!("Screen pos: {} {}", screen_pos_x, screen_pos_y);
                        println!("Mouse pos: {} {}", x, y);
                        println!("New pos: {} {}", real_pos.x, real_pos.y);
                    }
                }
            });*/

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
    /*fn get_mouse_delta<'s>(&mut self, input: &Read<'s, InputHandler<StringBindings>>) -> Option<(f32, f32)>{

        let (x, y) = match input.mouse_position() {
            Some(pos) => {
                pos
            },
            None => {
                self.last_mouse_pos.take();
                return None;
            }
        };

        let last_mouse_pos = self.last_mouse_pos.replace((x, y));
        //println!("Mouse pos: {} {}", x, y);

        // Получаем информацию об указателе мышки
        let delta= match last_mouse_pos {
            Some((last_x, last_y)) => {
                let delta = (x - last_x, y - last_y);
                self.last_mouse_pos.replace((x, y));
                delta
            },
            None => {
                return None;
            }
        };
        
        //println!("Mouse delta: {} {}", delta.0, delta.1);

        Some(delta)
    }*/
}