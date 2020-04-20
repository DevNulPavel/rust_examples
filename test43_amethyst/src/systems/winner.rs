use amethyst::{
    assets::AssetStorage,
    core::Transform,
    derive::SystemDesc,
    ui::UiText,
    audio,
    ecs::prelude::{
        Entity, 
        Join, 
        Read, 
        ReadExpect, 
        System, 
        SystemData, 
        Write, 
        WriteStorage
    },
};
use crate::{
    audio::SoundsResource, 
    game_types::{
        ScoreBoard,
        BallComponent
    }
};


/// Структура хранит сущности, которые отображают счет игрока
pub struct ScoreTextResource {
    pub p1_score: Entity,
    pub p2_score: Entity,
}

/// Данная система ответственна за проверку, что шар ушел за левую или правую границу поля.
/// Очки выдаются игроку на обратной стороне, а шар при этом перезагружается.
#[derive(SystemDesc)]
pub struct WinnerSystem;

impl<'s> System<'s> for WinnerSystem {
    type SystemData = (
        // Хранилище компонентов шаров
        WriteStorage<'s, BallComponent>,
        // Хранилище на запись трансформов
        WriteStorage<'s, Transform>,
        // Хранилище компонентов отображения текста
        WriteStorage<'s, UiText>,
        // Ресурс счета игроков
        Write<'s, ScoreBoard>,
        // Ресурс хранилища звуков
        Read<'s, AssetStorage<audio::Source>>,
        // Ресурс звуков
        ReadExpect<'s, SoundsResource>,
        // Ресурс очков
        ReadExpect<'s, ScoreTextResource>,
        // Вывод звука
        Option<Read<'s, audio::output::Output>>,
    );

    fn run(&mut self, (mut balls,
                       mut transforms,
                       mut text,
                       mut score_board,
                       storage,
                       sounds,
                       score_text,
                       audio_output): Self::SystemData) {

        // Для каждого шара с его трансформомо                        
        for (ball, transform) in (&mut balls, &mut transforms).join() {
            use crate::ARENA_WIDTH;

            // Получаем X координату
            let ball_x = transform.translation().x;

            // Было или нет столкновение
            let did_hit = if ball_x <= ball.radius {
                // Счет игрока левой строны
                // Ограничение 999 очками
                score_board.score_right = (score_board.score_right + 1).min(999);
                // Получаем лейбл текста
                if let Some(text) = text.get_mut(score_text.p2_score) {
                    // Пишем текст
                    text.text = score_board.score_right.to_string();
                }
                true
            } else if ball_x >= ARENA_WIDTH - ball.radius {
                score_board.score_left = (score_board.score_left + 1).min(999);
                if let Some(text) = text.get_mut(score_text.p1_score) {
                    text.text = score_board.score_left.to_string();
                }
                true
            } else {
                false
            };

            if did_hit {
                // Было столкновение - сбрасываем направление на обратное
                ball.velocity[0] = -ball.velocity[0];
                // переносим в центр
                transform.set_translation_x(ARENA_WIDTH / 2.0);

                // Print the score board.
                println!("Score: | {:^3} | {:^3} |", score_board.score_left, score_board.score_right);

                // Play audio.
                if let Some(ref output) = audio_output {
                    if let Some(sound) = storage.get(&sounds.score_sfx) {
                        output.play_once(sound, 1.0);
                    }
                }
            }
        }
    }
}