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
};
use crate::game_types::BallComponent;

/// Данная системма ответственна за перемещение всех шаров в соответствии с их скоростью
#[derive(SystemDesc)]
pub struct MoveBallsSystem;

impl<'s> System<'s> for MoveBallsSystem {
    type SystemData = (
        ReadStorage<'s, BallComponent>,
        WriteStorage<'s, Transform>,
        Read<'s, Time>,
    );

    fn run(&mut self, (balls, mut locals, time): Self::SystemData) {
        // Перемещаем каждый шар в соответствии с его скоростью и временем
        let delta = time.delta_seconds();
        // Связываем шар и его трансформ
        for (ball, local) in (&balls, &mut locals).join() {
            local.prepend_translation_x(ball.velocity[0] * delta);
            local.prepend_translation_y(ball.velocity[1] * delta);
        }
    }
}
