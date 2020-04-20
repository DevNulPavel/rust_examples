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
        WriteStorage,
        Entities,
        Entity
    },
};
use crate::game_types::BallComponent;

/// Данная системма ответственна за перемещение всех шаров в соответствии с их скоростью
#[derive(SystemDesc)]
pub struct MoveBallsSystem;

impl<'s> System<'s> for MoveBallsSystem {
    type SystemData = (
        Entities<'s>,
        ReadStorage<'s, BallComponent>,
        WriteStorage<'s, Transform>,
        Read<'s, Time>,
    );

    fn run(&mut self, (entities, balls, mut locals, time): Self::SystemData) {
        // Перемещаем каждый шар в соответствии с его скоростью и временем
        let delta = time.delta_seconds();
        // Связываем шар и его трансформ
        for (_e, ball, local) in (&entities, &balls, &mut locals).join() {
            let _e: Entity = _e;

            local.prepend_translation_x(ball.velocity[0] * delta);
            local.prepend_translation_y(ball.velocity[1] * delta);
        }
    }
}
