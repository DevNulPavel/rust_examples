use std::sync::Arc;
use parking_lot::RwLock;
use sdl2::{
    self,
    audio::{AudioDevice, AudioSpecDesired},
};

// Добавляем от корня с помощью crate::, а не super::
use crate::gen::Generator;
use crate::audio::{
    ExactStreamer,
    StreamingPlayer
};

// Как много сгенерированных буфферов будет хранить в себе канал
pub const GENERATOR_CHANNEL_SIZE: usize = 12;
pub const GENERATOR_BUFFER_SIZE: usize = 1024;

pub struct Audio {
    /// dropping this stops the stream
    #[allow(unused)]
    player: AudioDevice<StreamingPlayer>,
}

// Запускает поддержку потокового воспроизведения аудио из генератора
pub fn init(gen: Arc<RwLock<Generator>>, sample_rate: u32) -> Result<(Audio, crossbeam::Receiver<Vec<f32>>), String> {
    // Инициализируем SDL2
    let sdl_context = sdl2::init()?;
    // Получаем подсистему аудио
    let audio_subsystem = sdl_context.audio()?;

    // Структура, описывающая звуковые настройки
    let desired_spec = AudioSpecDesired {
        // Частота дискретизации 48 кГц
        freq: Some(sample_rate as i32),
        // 1 канал
        channels: Some(1),
        // 1024 семпла на коллбек
        samples: Some(crate::SAMPLES_PER_CALLBACK as u16),
    };

    // Создаем канал отправитель из генератора в потоковое воспроизведение
    let (generator_sender, device_receiver) = crossbeam::channel::bounded(GENERATOR_CHANNEL_SIZE);
    
    // Создаем канал отправитель в FFT для преобразования фурье и отображения спектра
    let (generator_fft_sender, fft_receiver) = crossbeam::channel::bounded(GENERATOR_CHANNEL_SIZE);

    // Запускаем воспроизведение звука в системе
    let out_device = audio_subsystem.open_playback(None, &desired_spec, |spec| {
        // Если частота семплирования нормальная
        if sample_rate == spec.freq as u32 {
            // Тогда создаем потоковый плеер с буффером и каналом получателем данных
            StreamingPlayer {
                // Создаем специальный буфферизующий поток, который буфферизирует данные из канала в себе
                // для быстрой отгрузки в поток вопроизведения
                stream: ExactStreamer::new(GENERATOR_BUFFER_SIZE, device_receiver),
            }
        } else {
            panic!(
                "Sample rate {} is not provided by the audio system",
                sample_rate
            );
        }
    })?;

    // Запускаем
    out_device.resume();

    // Получаем конфиги
    let spec = out_device.spec();

    // Выводим всю нашу информацию
    println!(
        "Audio driver: {:?}\nAudioSpecDesired: Channels: {:?}, Samplerate: {:?}, Samples: {:?}\nActual spec     : Channels: {:?}, Samplerate: {:?}, Samples: {:?}",
        out_device.subsystem().current_audio_driver(),
        desired_spec.channels,
        desired_spec.freq,
        desired_spec.samples,
        spec.channels,
        spec.freq,
        spec.samples
    );

    // Стартуем поток, который будет выдывать данные
    std::thread::spawn(move || {
        // Поток содержит буффер
        let mut buf = [0.0f32; GENERATOR_BUFFER_SIZE];

        loop {
            // contains lock
            {
                // Создаем блокировку записи и генерируем данные
                gen.write().generate(&mut buf);
            }

            // Отправляем все наши данные по каналу FFT для спектра без блокировки
            let _ = generator_fft_sender.try_send(buf.to_vec());

            // Отправляем данные на воспроизведение уже с блокировкой
            if generator_sender.send(buf.to_vec()).is_err() {
                break;
            }
        }
    });

    Ok((Audio { player: out_device }, fft_receiver))
}