use amethyst::{
    prelude::*,
    ecs::prelude::World,
    assets::{
        Handle, 
    },
    core::{
        timing::Time, 
        transform::Transform
    },
    renderer::{
        SpriteRender, 
        SpriteSheet
    }
};
use crate::{
    pong_state::PongState,
    game_types::{
        BallComponent
    }, 
    constants::{
        ARENA_HEIGHT, 
        ARENA_WIDTH
    }
};

///////////////////////////////////////////////////////////////////////////////////////////////////

/// Initialises one ball in the middle-ish of the arena.
fn initialise_ball(world: &mut World, sprite_sheet_handle: Handle<SpriteSheet>) {
    use crate::constants::{
        BALL_RADIUS, 
        BALL_VELOCITY_X, 
        BALL_VELOCITY_Y
    };

    // Create the translation.
    let mut local_transform = Transform::default();
    local_transform.set_translation_xyz(ARENA_WIDTH / 2.0, ARENA_HEIGHT / 2.0, 0.0);

    // Assign the sprite for the ball
    let sprite_render = SpriteRender {
        sprite_sheet: sprite_sheet_handle,
        sprite_number: 1, // ball is the second sprite on the sprite_sheet
    };

    world
        .create_entity()
        .with(sprite_render)
        .with(BallComponent {
            radius: BALL_RADIUS,
            velocity: [BALL_VELOCITY_X, BALL_VELOCITY_Y],
        })
        .with(local_transform)
        .build();
}

pub struct BeforeStartState {
    ball_spawn_timer: Option<f32>,
    sprite_sheet_handle: Option<Handle<SpriteSheet>>,
}

impl BeforeStartState{
    pub fn new(sprite_sheet_handle: Handle<SpriteSheet>)-> Self {
        // Устанавливаем время ожидания в 3 секунды до старта мячика
        BeforeStartState{
            ball_spawn_timer: Some(3.0),
            sprite_sheet_handle: Some(sprite_sheet_handle)
        }
    }
}

// Реализуем простое игровое состояние
impl SimpleState for BeforeStartState {
    // Старт игрового состояния
    fn on_start(&mut self, _data: StateData<'_, GameData<'_, '_>>) {
        // Разворачиваем структуру для получения мира
        // let StateData{ world, .. } = data;
    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        // Получаем оставшееся время обнуляя Option
        if let Some(mut timer) = self.ball_spawn_timer.take() {
            // Если время не истекло, тогда отнимаем время прошедшее с прошлого кадра
            {
                let time = data.world.fetch::<Time>();
                timer -= time.delta_seconds();
            }

            if timer <= 0.0 {
                // Время истекло - создаем мяч
                initialise_ball(data.world, self.sprite_sheet_handle.clone().unwrap());
                let new_state = Box::new(PongState::new(self.sprite_sheet_handle.take().unwrap()));
                return Trans::Switch(new_state);
            } else {
                // Если время не истекло - возвращаем значение в Option
                self.ball_spawn_timer.replace(timer);
            }
        }

        // Состояние менять не надо
        Trans::None
    }
}
