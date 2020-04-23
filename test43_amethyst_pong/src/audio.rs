/*use std::{
    iter::Cycle, 
    vec::IntoIter
};
use amethyst::{
    assets::{
        AssetStorage, 
        Loader
    },
    audio::{
        self, // Так можно добавить себя, чтобы был доступ по audio::
        AudioSink, 
        OggFormat, 
        SourceHandle
    },
    ecs::{
        World, 
        WorldExt
    }
};

//////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Структура, содержащая загруженные звуки
pub struct SoundsResource {
    pub score_sfx: SourceHandle,
    pub bounce_sfx: SourceHandle,
}

// Структура, содержащая загруженную музыку
pub struct MusicResource {
    // Цикличный итератор по звуковым файликам
    pub music: Cycle<IntoIter<SourceHandle>>,
}

//////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Загружаем OGG аудио музыку
fn load_audio_track(loader: &Loader, world: &World, file: &str) -> SourceHandle {
    loader.load(file, OggFormat, (), &world.read_resource())
}

/// Инициализируем аудио в мире, сохраняя его в ресурсе
pub fn initialise_audio(world: &mut World) {
    use crate::{
        AUDIO_BOUNCE, 
        AUDIO_MUSIC, 
        AUDIO_SCORE
    };

    let (sound_effects, music) = {
        // Получаем ресурс загрузчика файликов в режиме толкьо на чтение
        let loader = world.read_resource::<Loader>();

        // Получаем ресурс аудио в режиме для чтения
        let mut sink = world.write_resource::<AudioSink>();
        // Снижаем громкость воспроизведения
        sink.set_volume(0.25);

        // Идем по списку файликов и загружаем треки
        let music = AUDIO_MUSIC
            .iter()
            .map(|file| {
                // Грузим стрек
                load_audio_track(&loader, &world, file)
            })
            // Собираем в вектор
            .collect::<Vec<_>>()
            // Вектор преобразуем во владеющий итератор
            .into_iter()
            // Делаем итератор цикличным, чтобы музыка играла одна за другой
            .cycle();
        
        // Муыкальный объект
        let music = MusicResource { music };

        // Звуки
        let sound = SoundsResource {
            bounce_sfx: load_audio_track(&loader, &world, AUDIO_BOUNCE),
            score_sfx: load_audio_track(&loader, &world, AUDIO_SCORE),
        };

        (sound, music)
    };

    // Добавляем звуки в мир в виде ресурсов, добавление происходит тут, так как
    // мир не позволяет добавлять новые ресурсы пока загрузчик заимствован
    world.insert(sound_effects);
    world.insert(music);
}

// Воспроизведение звука удара о границу
pub fn play_bounce(sounds: &SoundsResource, storage: &AssetStorage<audio::Source>, output: Option<&audio::output::Output>) {
    // Получаем ссылку на вывод звука
    if let Some(ref output) = output.as_ref() {
        // Получаем из хранилища исходник звука
        if let Some(sound) = storage.get(&sounds.bounce_sfx) {
            // Воспроизводим
            output.play_once(sound, 0.7);
        }
    }
}*/
