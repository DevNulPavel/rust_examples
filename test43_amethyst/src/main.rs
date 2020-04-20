mod audio;
mod pong_bundle;
mod pong_state;
mod systems;
mod game_types;
mod constants;

use std::time::Duration;
use amethyst::{
    prelude::*,
    utils::application_root_dir,
    audio::{
        AudioBundle, 
        DjSystemDesc
    },
    core::{
        frame_limiter::FrameRateLimitStrategy, 
        transform::TransformBundle
    },
    input::{
        InputBundle, 
        StringBindings
    },
    renderer::{
        RenderingBundle,
        types::DefaultBackend,
        plugins::{
            RenderFlat2D, 
            RenderToWindow
        },
    },
    ui::{
        RenderUi, 
        UiBundle
    }
};
use crate::{
    audio::MusicResource, 
    pong_bundle::PongBundle,
    constants::*,
};

fn main() -> amethyst::Result<()> {
    // Запускаем систему логирования со стандартным конфигом (вызывая default метод трейта)
    amethyst::start_logger(Default::default());

    // Получаем коренную папку приложения
    let app_root = application_root_dir()?;

    // Путь к конфигу приложения
    let display_config_path = app_root.join("src/config/display.ron");

    // Путь к конфигу кнопок
    let key_bindings_path = {
        if cfg!(feature = "sdl_controller") {
            app_root.join("src/config/input_controller.ron")
        } else {
            app_root.join("src/config/input.ron")
        }
    };

    // Директория ассетов
    let assets_dir = app_root.join("assets/");

    // Бандл систем обработки ввода
    let input_systems_bundle = InputBundle::<StringBindings>::new()
        .with_bindings_from_file(key_bindings_path)?;

    let game_data = GameDataBuilder::default()
        // Добавляем бандл трансформов, который обрабатывает позиции сущностей
        .with_bundle(TransformBundle::new())?
        // Бандл обработки ввода пользователя
        .with_bundle(input_systems_bundle)?
        // Бандл работы со звуком
        .with_bundle(AudioBundle::default())?
        // Добавляем уже систему работы с музыкой
        .with_system_desc(
            DjSystemDesc::new(|music: &mut MusicResource| { 
                music.music.next()
            }),
            "dj_system", // Имя системы
            &[],         // Зависимости
        )
        // Бандл непосредственно игры
        .with_bundle(PongBundle{})?
        // Бандл с системами интерфейса
        .with_bundle(UiBundle::<StringBindings>::new())?
        // Система рендеринга
        .with_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(
                    // RenderToWindow плагин предоставляет все строительные леса для открытия окна
                    // и отрисовки в нем
                    RenderToWindow::from_config_path(display_config_path)?
                        .with_clear([0.34, 0.36, 0.52, 1.0]),
                )
                .with_plugin(RenderFlat2D::default())
                .with_plugin(RenderUi::default()),
        )?;

    // Создаем непосредственно игру
    let initial_state = pong_state::PongState::default();
    let frame_limit_stategy = FrameRateLimitStrategy::SleepAndYield(Duration::from_millis(1));
    let mut game = Application::build(assets_dir, initial_state)?
        .with_frame_limit(frame_limit_stategy,30)
        .build(game_data)?;

    game.run();

    Ok(())
}

