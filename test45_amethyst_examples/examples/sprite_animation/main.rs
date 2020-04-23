//! Demonstrates how to load and render sprites.
//!
//! Sprites are from <https://opengameart.org/content/bat-32x32>.

use amethyst::{
    animation::{
        get_animation_set, 
        AnimationBundle, 
        AnimationCommand, 
        AnimationControlSet, 
        AnimationSet,
        AnimationSetPrefab, 
        EndControl,
    },
    assets::{
        PrefabData, 
        PrefabLoader, 
        PrefabLoaderSystemDesc, 
        ProgressCounter, 
        RonFormat
    },
    core::transform::{
        Transform, 
        TransformBundle
    },
    derive::PrefabData,
    ecs::{
        prelude::Entity, 
        Entities, 
        Join, 
        ReadStorage, 
        WriteStorage
    },
    error::Error,
    prelude::{
        Builder, 
        World, 
        WorldExt
    },
    renderer::{
        camera::Camera,
        plugins::{
            RenderFlat2D, 
            RenderToWindow
        },
        sprite::{
            prefab::SpriteScenePrefab, 
            SpriteRender
        },
        types::DefaultBackend,
        RenderingBundle,
    },
    utils::application_root_dir,
    window::ScreenDimensions,
    Application, 
    GameData, 
    GameDataBuilder, 
    SimpleState,
    SimpleTrans, 
    StateData, 
    Trans,
};
use serde::{
    Deserialize, 
    Serialize
};

/// Инициализируем камеру
fn initialise_camera_entity(world: &mut World) {
    // Получаем размер окошка
    let (width, height) = {
        let dim = world.read_resource::<ScreenDimensions>();
        (dim.width(), dim.height())
    };

    // Устанавливаем трансформ с Z
    let mut camera_transform = Transform::default();
    camera_transform.set_translation_z(1.0);

    // Создаем новую сущность с камерой и трансформом
    world
        .create_entity()
        .with(camera_transform)
        .with(Camera::standard_2d(width, height))
        .build();
}

/// Id анимации используется в AnimationSet
#[derive(Eq, PartialOrd, PartialEq, Hash, Debug, Copy, Clone, Deserialize, Serialize)]
enum AnimationId {
    Fly,
}

/// Загрузка данных для сущностей
#[derive(Debug, Clone, Deserialize, PrefabData)]
struct MyPrefabData {
    /// Информация для рендеринга сцены со спрайтами
    sprite_scene: SpriteScenePrefab,
    /// Все анимации, которые могут быть запущены на сущности
    animation_set: AnimationSetPrefab<AnimationId, SpriteRender>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Главное состояние
#[derive(Default)]
struct MainGameState {
    /// Переменная для отслеживания прогресса загрузки
    pub progress_counter: Option<ProgressCounter>,
}

impl SimpleState for MainGameState {
    // Вызывается при запуске состояния
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let StateData { world, .. } = data;
        
        // Создаем новый объект прогресса
        self.progress_counter = Some(Default::default());

        // Запускаем загрузку ассетов
        // exec - выполняет переданную функцию прямо сейчас c загрузчиком, но с доступом к систем-дате
        let bat_prefab = world.exec(|loader: PrefabLoader<'_, MyPrefabData>| {
            let handle = loader.load("prefab/sprite_animation.ron", RonFormat, self.progress_counter.as_mut().unwrap());
            handle
        });
        let arrow_test_prefab = world.exec(|loader: PrefabLoader<'_, MyPrefabData>| {
            let handle = loader.load("prefab/sprite_animation_test.ron", RonFormat, self.progress_counter.as_mut().unwrap());
            handle
        });
        
        // Создает новые сущности с компонентами из префабов
        world
            .create_entity()
            .with(bat_prefab)
            .build();
        world
            .create_entity()
            .with(arrow_test_prefab)
            .build();
        
        // Создаем новую камеру
        initialise_camera_entity(world);
    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        // Проверяем, что мы еще грузимся
        if let Some(ref progress_counter) = self.progress_counter {
            // Если мы загрузились
            if progress_counter.is_complete() {
                let StateData { world, .. } = data;

                // Можно определить собственный класс системных данных
                /*#[derive(SystemData)]
                pub struct CurSystemData<'a> {
                    e: Entities, 
                    anim_id: ReadStorage<AnimationSet<AnimationId, SpriteRender>>,
                    anim_control: WriteStorage<AnimationControlSet<AnimationId, SpriteRender>>
                }*/

                // Выполняем что-то в мире, но с доступом к систем-дате
                world.exec( |(entities, animation_sets, mut control_sets): (Entities, 
                                                                            ReadStorage<AnimationSet<AnimationId, SpriteRender>>,
                                                                            WriteStorage<AnimationControlSet<AnimationId, SpriteRender>>)| {
                        // Обходим все сущности, которые имеют AnimationSet
                        for (entity, animation_set) in (&entities, &animation_sets).join() {
                            // Создаем новый компонент AnimationControlSet для сущности
                            let control_set = get_animation_set(&mut control_sets, entity).unwrap();
                            // Добавляем анимацию `Fly` в AnimationControlSet и запускаем бесконечно
                            control_set.add_animation(AnimationId::Fly,
                                                      &animation_set.get(&AnimationId::Fly).unwrap(),
                                                      EndControl::Loop(None),
                                                      1.0,
                                                      AnimationCommand::Start);
                        }
                    },
                );

                // Данные загрузили - больше не надо хранить счетчик
                self.progress_counter = None;
            }
        }
        Trans::None
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////

fn main() -> amethyst::Result<()> {
    // Запускаем логирование
    amethyst::start_logger(Default::default());

    // Путь к корню приложения
    let app_root = application_root_dir()?;
    // Путь к ассетам и конфигу отображения
    let assets_dir = app_root.join("examples/assets/");
    let display_config_path = app_root.join("examples/sprite_animation/config/display.ron");

    // Система загрузки префабов
    let prefab_loader_system_desc = PrefabLoaderSystemDesc::<MyPrefabData>::default();

    // Бандл систем для анимации
    let animation_bundle = AnimationBundle::<AnimationId, SpriteRender>::new("sprite_animation_control", "sprite_sampler_interpolation");

    // Бандл систем трансформа, который зависит от системы анимации спрайтов и интерполяции
    let transform_bundle =  TransformBundle::new()
        .with_dep(&["sprite_animation_control", "sprite_sampler_interpolation"]);

    // Рендеринг в окно
    let render_to_window_plugin = RenderToWindow::from_config_path(display_config_path)?
        .with_clear([0.34, 0.36, 0.52, 1.0]);
    // Плагин для рендеринга 2D
    let render_2d_plugin = RenderFlat2D::default();
    // Создаем бандл из систем для рендеринга
    let render_bundle = RenderingBundle::<DefaultBackend>::new()
        .with_plugin(render_to_window_plugin)
        .with_plugin(render_2d_plugin);

    // Создаем билдер из систем
    let game_data = GameDataBuilder::default()
        .with_system_desc(prefab_loader_system_desc, "scene_loader", &[])
        .with_bundle(animation_bundle)?
        .with_bundle(transform_bundle)?
        .with_bundle(render_bundle)?;

    let mut game = Application::new(assets_dir, MainGameState::default(), game_data)?;
    game.run();

    Ok(())
}
