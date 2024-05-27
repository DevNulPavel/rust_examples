use amethyst::{
    derive::SystemDesc,
    core::{
        timing::Time, 
        transform::Transform
    },
    ecs::prelude::{
        Join, 
        Read, 
        ReadStorage, 
        System, 
        SystemData, 
        WriteStorage
    },
    input::{
        InputHandler, 
        StringBindings
    },
};
use crate::game_types::PaddleComponent;


/// Данная система отвечает за перемещение всех ракеток в соответствии с вводом пользователя
#[derive(SystemDesc)]
pub struct PaddleSystem;

impl<'s> System<'s> for PaddleSystem {
    type SystemData = (
        ReadStorage<'s, PaddleComponent>,
        WriteStorage<'s, Transform>,
        Read<'s, Time>,
        Read<'s, InputHandler<StringBindings>>,
    );

    fn run(&mut self, (paddles, mut transforms, time, input): Self::SystemData) {
        use crate::game_types::Side;

        // Итерируемся по всем ракеткам и двигаем их в соответствии с вводом пользователя
        // Связываем каждую ракетку с трансформом ракетки
        for (paddle, transform) in (&paddles, &mut transforms).join() {
            // В зависимости от стороны определяем, был ли ввод
            let opt_movement = match paddle.side {
                Side::Left => input.axis_value("left_paddle"),
                Side::Right => input.axis_value("right_paddle"),
            };

            // Если был
            if let Some(movement) = opt_movement {
                use crate::ARENA_HEIGHT;

                // Выполняем перемещение
                transform.prepend_translation_y(
                    paddle.velocity * time.delta_seconds() * movement as f32,
                );

                // Ограничиваем перемещение ракетки в пределах области игры
                let paddle_y = transform
                    .translation()
                    .y
                    .max(paddle.height * 0.5)
                    .min(ARENA_HEIGHT - paddle.height * 0.5);
                transform.set_translation_y(paddle_y);
            }
        }
    }
}
