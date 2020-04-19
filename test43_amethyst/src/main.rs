//! Pong

mod audio;
mod bundle;
mod pong;
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
    ecs::{
        Component, 
        DenseVecStorage
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
    audio::Music, 
    bundle::PongBundle,
    constants::*,
    game_types::{
        Ball,
        Side,
        ScoreBoard
    }
};

fn main() -> amethyst::Result<()> {
    use crate::pong::Pong;

    // Запускаем систему логирования
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

    let game_data = GameDataBuilder::default()
        // Добавляем бандл трансформов, который обрабатывает позиции сущностей
        .with_bundle(TransformBundle::new())?
        // Бандл обработки ввода пользователя
        .with_bundle(InputBundle::<StringBindings>::new()
            .with_bindings_from_file(key_bindings_path)?)?
        // Бандл непосредственно игры
        .with_bundle(PongBundle)?
        // Бандл работы со звуком
        .with_bundle(AudioBundle::default())?
        // Добавляем уже систему работы с музыкой
        .with_system_desc(
            DjSystemDesc::new(|music: &mut Music| { 
                music.music.next()
            }),
            "dj_system", // Имя системы
            &[],         // Зависимости
        )
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
    let mut game = Application::build(assets_dir, Pong::default())?
        .with_frame_limit(
            FrameRateLimitStrategy::SleepAndYield(Duration::from_millis(2)),
            144,
        )
        .build(game_data)?;

    game.run();
    Ok(())
}

