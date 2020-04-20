use amethyst::{
    core::bundle::SystemBundle,
    ecs::prelude::{
        DispatcherBuilder, 
        World
    },
    error::Error,
};
use crate::systems::{
    BounceSystem, 
    MoveBallsSystem, 
    PaddleSystem, 
    WinnerSystem
};


/// Бандл - это удобный способ инициализировать связанные ресурсы, компоненты и системы в мире.
/// Бандл подготавливает мир для игры
pub struct PongBundle;

// Реализуем бандл
impl<'a, 'b> SystemBundle<'a, 'b> for PongBundle {
    fn build(self, _world: &mut World, builder: &mut DispatcherBuilder<'a, 'b>) -> Result<(), Error> {
        // Добавляем системы, описывая их зависимости для работы в многопоточной среде
        builder.add(PaddleSystem{}, "paddle_system", &["input_system"]);
        builder.add(MoveBallsSystem{}, "ball_system", &[]);
        builder.add(BounceSystem{},"collision_system",&["paddle_system", "ball_system"]);
        builder.add(WinnerSystem{},"winner_system",&["paddle_system", "ball_system"]);
        Ok(())
    }
}
