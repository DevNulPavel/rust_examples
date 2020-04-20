use amethyst::{
    prelude::*,
    ecs::prelude::World,
    assets::{
        AssetStorage, 
        Handle, 
        Loader
    },
    core::{
        timing::Time, 
        transform::Transform
    },
    renderer::{
        Camera, 
        ImageFormat, 
        SpriteRender, 
        SpriteSheet, 
        SpriteSheetFormat, 
        Texture
    },
    ui::{
        Anchor, 
        TtfFormat, 
        UiText, 
        UiTransform
    }
};
use crate::{
    systems::ScoreTextResource, 
    game_types::{
        Side,
        PaddleComponent,
        BallComponent,
        BounceCountComponent
    }, 
    constants::{
        ARENA_HEIGHT, 
        ARENA_WIDTH
    }
};

///////////////////////////////////////////////////////////////////////////////////////////////////

/// Функция загрузки атласа
fn load_sprite_sheet(world: &mut World) -> Handle<SpriteSheet> {
    // Загружаем спрайт атлас необходимый для рендеринга графики
    // `sprite_sheet` - это атлас
    // `texture_handle` - это клонируемый референс на текстуру

    // Получаем "ресурс" загрузки ресурсов
    let loader = world.read_resource::<Loader>();

    // Получаем текстуру
    let texture_handle = {
        // Получаем хранилище ассетов текстур
        let texture_storage = world.read_resource::<AssetStorage<Texture>>();
        // Получаем текстуру и сохраняем в хранилище
        loader.load(
            "texture/pong_spritesheet.png",
            ImageFormat::default(),
            (),
            &texture_storage,
        )
    };

    // Получаем хранилище для развертки атласа
    let sprite_sheet_store = world.read_resource::<AssetStorage<SpriteSheet>>();
    // Получаем развертку с сохранением в хранилище
    loader.load(
        "texture/pong_spritesheet.ron", // Файл развертки
        SpriteSheetFormat(texture_handle), // Формат на основании текстуры
        (),
        &sprite_sheet_store,
    )
}

///////////////////////////////////////////////////////////////////////////////////////////////////

/// Инициализация камеры
fn initialise_camera(world: &mut World) {
    // Настраиваем камеру так, чтобы наш экран покрывал все иговое поле и (0,0) был нижним левым углом
    let mut transform = Transform::default();
    transform.set_translation_xyz(ARENA_WIDTH * 0.5, ARENA_HEIGHT * 0.5, 1.0);

    world
        .create_entity()
        .with(Camera::standard_2d(ARENA_WIDTH, ARENA_HEIGHT))
        .with(transform)
        .build();
}

///////////////////////////////////////////////////////////////////////////////////////////////////

/// Инициализируем ракетки слева и справа
fn initialise_paddles(world: &mut World, sprite_sheet_handle: Handle<SpriteSheet>) {
    use crate::constants::{
        PADDLE_WIDTH
    };

    // Позиционируем ракетки
    let y = ARENA_HEIGHT / 2.0;

    // Создаем компонент левой ракетки
    let left_paddle_component = PaddleComponent::new(Side::Left);

    // Компонент правой ракетки
    let right_paddle_component = PaddleComponent::new(Side::Right);

    // Создаем трансформ
    let left_transform_component = {
        let mut t = Transform::default();
        t.set_translation_xyz(PADDLE_WIDTH * 0.5, y, 0.0);
        t
    };
    let right_transform_component = {
        let mut t = Transform::default();
        t.set_translation_xyz(ARENA_WIDTH - PADDLE_WIDTH * 0.5, y, 0.0);
        t
    };

    // Компоненты количества касаний слева и справа
    let bounce_count_component_left = BounceCountComponent::default();
    let bounce_count_component_right = BounceCountComponent::default();

    // Создаем рендер спрайта
    let sprite_render_component = SpriteRender {
        sprite_sheet: sprite_sheet_handle,
        sprite_number: 0, // paddle is the first sprite in the sprite_sheet
    };

    // Создаем сущность левой ракетки
    world
        .create_entity()
        .named("Left paddle")
        .with(sprite_render_component.clone())
        .with(left_paddle_component)
        .with(left_transform_component)
        .with(bounce_count_component_left)
        .build();

    // Создаем сущность правой ракетки
    world
        .create_entity()
        .named("Right paddle")
        .with(sprite_render_component)
        .with(right_paddle_component)
        .with(right_transform_component)
        .with(bounce_count_component_right)
        .build();
}

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

///////////////////////////////////////////////////////////////////////////////////////////////////

fn initialise_score(world: &mut World) {
    // С помощью загрузчика мы грузим шрифт
    let font = world.read_resource::<Loader>().load(
        "font/square.ttf",
        TtfFormat,
        (),
        &world.read_resource(),
    );

    // Трансформация левого лейбла
    let p1_transform = UiTransform::new(
        "P1".to_string(),
        Anchor::TopMiddle,
        Anchor::Middle,
        -50.,
        -50.,
        1.,
        200.,
        50.,
    );

    // Трансформация правого лейбла
    let p2_transform = UiTransform::new(
        "P2".to_string(),
        Anchor::TopMiddle,
        Anchor::Middle,
        50.,
        -50.,
        1.,
        200.,
        50.,
    );

    let p1_score = world
        .create_entity()
        .with(p1_transform)
        .with(UiText::new(
            font.clone(),
            "0".to_string(),
            [1.0, 1.0, 1.0, 1.0],
            50.0,
        ))
        .build();

    let p2_score = world
        .create_entity()
        .with(p2_transform)
        .with(UiText::new(
            font,
            "0".to_string(),
            [1.0, 1.0, 1.0, 1.0],
            50.0,
        ))
        .build();

    // Добавляем ресурс, который будет хранить структуру с сущностями
    world.insert(ScoreTextResource{ 
        p1_score, 
        p2_score 
    });
}

///////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
pub struct PongState {
    ball_spawn_timer: Option<f32>,
    sprite_sheet_handle: Option<Handle<SpriteSheet>>,
}

// Реализуем простое игровое состояние
impl SimpleState for PongState {
    // Старт игрового состояния
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        use crate::audio::initialise_audio;

        // Разворачиваем структуру для получения мира
        let StateData{ world, .. } = data;

        // Устанавливаем время ожидания в 3 секунды до старта мячика
        self.ball_spawn_timer.replace(3.0);

        // Прогружаем страйты необходимые для рендеринга
        // `spritesheet` - это лаяут спрайтов на картинке (атлас)
        // `texture` - это картинка
        self.sprite_sheet_handle.replace(load_sprite_sheet(world));

        // Инициализируем спрайты
        initialise_paddles(world, self.sprite_sheet_handle.clone().unwrap());
        // Камера
        initialise_camera(world);
        // Аудио
        initialise_audio(world);
        // Счет
        initialise_score(world);
    }

    /// Executed when the game state exits.
    fn on_stop(&mut self, _data: StateData<'_, GameData<'_, '_>>) {
    }

    /// Executed when a different game state is pushed onto the stack.
    fn on_pause(&mut self, _data: StateData<'_, GameData<'_, '_>>) {
    }

    /// Executed when the application returns to this game state once again.
    fn on_resume(&mut self, _data: StateData<'_, GameData<'_, '_>>) {
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
            } else {
                // Если время не истекло - возвращаем значение в Option
                self.ball_spawn_timer.replace(timer);
            }
        }

        // Состояние менять не надо
        Trans::None
    }
}
