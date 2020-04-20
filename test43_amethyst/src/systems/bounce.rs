use amethyst::{
    audio,
    assets::AssetStorage,
    core::transform::Transform,
    derive::SystemDesc,    
    ecs::prelude::{
        Join, 
        Read, 
        ReadExpect, 
        ReadStorage, 
        System, 
        SystemData, 
        WriteStorage,
        Entities,
        Entity
    },
};
use crate::{
    game_types::{
        Side,
        PaddleComponent,
        BallComponent,
        BounceCountComponent
    },
    audio::{
        play_bounce, 
        SoundsResource
    }
};

// A point is in a box when its coordinates are smaller or equal than the top
// right and larger or equal than the bottom left.
fn point_in_rect(x: f32, y: f32, left: f32, bottom: f32, right: f32, top: f32) -> bool {
    x >= left && x <= right && y >= bottom && y <= top
}

/// Данная система отвечает за определение коллизий между шаром и ракетками, так же за стены тоже
#[derive(SystemDesc)]
pub struct BounceSystem;

impl<'s> System<'s> for BounceSystem {
    type SystemData = (
        // Ресурс непосредственно для получения сущности
        Entities<'s>,
        // Хранилище шаров в режиме чтения с блокировкой, так как мы меняем координаты
        WriteStorage<'s, BallComponent>,
        // Ракетки мы не изменяем - хранилище в режиме чтения, читать можно сразу из кучи потоков
        ReadStorage<'s, PaddleComponent>,
        // Хранилище счетчика касаний ракетки - режим записи, чтобы можно было менять при касании
        WriteStorage<'s, BounceCountComponent>,
        // Хранилище трансформов сущностей
        ReadStorage<'s, Transform>,
        // Ресурс, который реализует хранилище загруженных данных, обхекты должны поддерживать default
        Read<'s, AssetStorage<audio::Source>>,
        // Ресурс, где объекты не обязательно должны поддерживать default
        ReadExpect<'s, SoundsResource>,
        // Вывод звука
        Option<Read<'s, audio::output::Output>>,
    );

    fn run(&mut self, data: Self::SystemData) {

        // Развертываем наши данные
        let (ent,
            mut balls, 
             paddles,
             mut bounce_count,
             transforms, 
             storage, 
             sounds, 
             audio_output) = data;

        // Проверяем, что шарик соприкоснулся и отскочил правильно
        // Мы так же проверяем ускорение для шара каждый раз, чтобы избежать множественных коллизий 
        for (ball, transform) in (&mut balls, &transforms).join() {
            use crate::ARENA_HEIGHT;

            // Значения координат
            let ball_x = transform.translation().x;
            let ball_y = transform.translation().y;

            // Отскакиваем от верха и от низа области
            if (ball_y <= ball.radius && ball.velocity[1] < 0.0)
                || (ball_y >= ARENA_HEIGHT - ball.radius && ball.velocity[1] > 0.0)
            {
                // Меняем направление движения
                ball.velocity[1] = -ball.velocity[1];
                // Играем звук
                play_bounce(&*sounds, &storage, audio_output.as_deref());
            }

            // Отскакиваем от ракеток
            for (_e, paddle, paddle_transform, bounce_comp) in (&ent, &paddles, &transforms, &mut bounce_count).join() {
                let _e: Entity = _e;

                let paddle_x = paddle_transform.translation().x - (paddle.width * 0.5);
                let paddle_y = paddle_transform.translation().y - (paddle.height * 0.5);

                // Чтобы определить, что шарик соприкоснулся с ракеткой. Мы создаем больший ректангл
                // вокруг текущей ракетки, вычитая радиус шарика из наименьшей координаты.
                // Затем добавляем радиус шарика к наибольшей.
                // Шарик соприкосается с ракеткой если его центры внутри этого ректангла
                if point_in_rect(
                    ball_x,
                    ball_y,
                    paddle_x - ball.radius,
                    paddle_y - ball.radius,
                    paddle_x + (paddle.width + ball.radius),
                    paddle_y + (paddle.height + ball.radius),
                ) && ((paddle.side == Side::Left && ball.velocity[0] < 0.0)
                    || (paddle.side == Side::Right && ball.velocity[0] > 0.0))
                {
                    // Меняем направление у шара
                    ball.velocity[0] = -ball.velocity[0];
                    // Увеличиваем количество касаний
                    bounce_comp.count += 1;
                    // Воспроизводим звук
                    play_bounce(&*sounds, &storage, audio_output.as_deref());
                }
            }
        }
    }
}
