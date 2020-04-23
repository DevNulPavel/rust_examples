// use std::sync::{
//     Mutex,
//     Arc
// };
use amethyst::{
    prelude::*,
    ecs::prelude::World,
    assets::{
        AssetStorage, 
        Handle, 
        Loader,
        ProgressCounter,
        // Progress
    },
    core::{
        // timing::Time, 
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
    before_start_state::BeforeStartState,
    game_types::{
        Side,
        PaddleComponent,
        //BallComponent,
        BounceCountComponent,
        PointerComponent
    }, 
    constants::{
        ARENA_HEIGHT, 
        ARENA_WIDTH
    }
};

///////////////////////////////////////////////////////////////////////////////////////////////////

/// Инициализация камеры
fn initialise_camera(world: &mut World) {
    // Настраиваем камеру так, чтобы наш экран покрывал все иговое поле и (0,0) был нижним левым углом
    let mut transform = Transform::default();
    transform.set_translation_xyz(ARENA_WIDTH * 0.5, ARENA_HEIGHT * 0.5, 1.0);

    let camera_entity = world
        .create_entity()
        .with(Camera::standard_2d(ARENA_WIDTH, ARENA_HEIGHT))
        .with(transform)
        .build();

    world.insert(camera_entity);
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

fn initialize_mouse_pointer(world: &mut World, sprite_sheet_handle: Handle<SpriteSheet>){
    // Создаем трансформ для указателя мыши
    let mut local_transform = Transform::default();
    local_transform.set_translation_xyz(0.0, 0.0, 0.0);

    // Assign the sprite for the ball
    let sprite_render = SpriteRender {
        sprite_sheet: sprite_sheet_handle,
        sprite_number: 1, // Будем использовать второй спрайт из атласа
    };

    // Создаем сущность указателя
    world
        .create_entity()
        // Добавляем компонент рендера
        .with(sprite_render)
        // Компонент трансформа
        .with(local_transform)
        // Дабавляем компонент указателя
        .with(PointerComponent::default())
        .build();
}

///////////////////////////////////////////////////////////////////////////////////////////////////

pub struct LoadingState {
    loading_progress: ProgressCounter,
    sprite_sheet_handle: Option<Handle<SpriteSheet>>,
}

impl Default for LoadingState{
    fn default() -> Self{
        LoadingState{
            loading_progress: ProgressCounter::new(),
            sprite_sheet_handle: None
        }
    }
}

// Реализуем простое игровое состояние
impl SimpleState for LoadingState {
    // Старт игрового состояния
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        //use crate::audio::initialise_audio;

        // Разворачиваем структуру для получения мира
        let StateData{ world, .. } = data;

        // Прогружаем страйты необходимые для рендеринга, загрузка будет асинхронная
        // `spritesheet` - это лаяут спрайтов на картинке (атлас)
        // `texture` - это картинка
        self.load_sprite_sheet(world);

        // Инициализируем спрайты
        initialise_paddles(world, self.sprite_sheet_handle.clone().unwrap());
        // Камера
        initialise_camera(world);
        // Аудио
        //initialise_audio(world);
        // Счет
        initialise_score(world);
        // Указатели на мышку
        initialize_mouse_pointer(world, self.sprite_sheet_handle.clone().unwrap());
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

    fn update(&mut self, _data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        if self.loading_progress.is_complete() {
            let new_state = Box::new(BeforeStartState::new(self.sprite_sheet_handle.take().unwrap()));
            Trans::Switch(new_state)
        } else {
            Trans::None
        }
    }
}

impl LoadingState{
    /// Функция загрузки атласа
    fn load_sprite_sheet(&mut self, world: &mut World) {
        // Загружаем спрайт атлас необходимый для рендеринга графики
        // `sprite_sheet` - это атлас
        // `texture_handle` - это клонируемый референс на текстуру

        // Получаем "ресурс" загрузки ресурсов
        let loader = world.read_resource::<Loader>();

        // Получаем текстуру, но загрузка будет происходить в фоне, а мы имеем только хендл на текстуру
        let texture_handle = {
            // Получаем хранилище ассетов текстур
            let texture_storage = world.read_resource::<AssetStorage<Texture>>();
            // Получаем текстуру и сохраняем в хранилище
            loader.load(
                "texture/pong_spritesheet.png",
                ImageFormat::default(),
                &mut self.loading_progress,
                &texture_storage,
            )
        };

        // Получаем хранилище для развертки атласа
        let sprite_sheet_store = world.read_resource::<AssetStorage<SpriteSheet>>();
        
        // Получаем развертку с сохранением в хранилище, но загрузка будет происходить в фоне, а мы имеем только хендл на текстуру
        let atlas_handle = loader.load(
            "texture/pong_spritesheet.ron", // Файл развертки
            SpriteSheetFormat(texture_handle), // Формат на основании текстуры
            &mut self.loading_progress,
            &sprite_sheet_store,
        );

        self.sprite_sheet_handle.replace(atlas_handle);
    }

}