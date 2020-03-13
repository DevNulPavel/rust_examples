use std::sync::Arc;
use parking_lot::RwLock;
use sdl2::{
    self,
    audio::{AudioDevice, AudioSpecDesired},
};

// Добавляем от корня с помощью crate::, а не super::
use crate::gen::Generator;
use crate::audio::GENERATOR_CHANNEL_SIZE;
use crate::audio::GENERATOR_BUFFER_SIZE;
use crate::audio::streamer::ExactStreamer;
use crate::audio::player::StreamingPlayer;

pub struct Audio {
    /// dropping this stops the stream
    #[allow(unused)]
    player: AudioDevice<StreamingPlayer>,
}

/// starts audio streaming to an audio device and also steps the generator with a fixed buffer of size `GENERATOR_BUFFER_SIZE`
pub fn init(gen: Arc<RwLock<Generator>>, sample_rate: u32) -> Result<(Audio, crossbeam::Receiver<Vec<f32>>), String> {
    let sdl_context = sdl2::init()?;
    let audio_subsystem = sdl_context.audio()?;

    let desired_spec = AudioSpecDesired {
        freq: Some(sample_rate as i32),
        channels: Some(1),
        samples: Some(crate::SAMPLES_PER_CALLBACK as u16),
    };

    let (generator_sender, device_receiver) = crossbeam::channel::bounded(GENERATOR_CHANNEL_SIZE);
    let (generator_fft_sender, fft_receiver) = crossbeam::channel::bounded(GENERATOR_CHANNEL_SIZE);

    let out_device = audio_subsystem.open_playback(None, &desired_spec, |spec| {
        if sample_rate == spec.freq as u32 {
            StreamingPlayer {
                stream: ExactStreamer::new(GENERATOR_BUFFER_SIZE, device_receiver),
            }
        } else {
            panic!(
                "Sample rate {} is not provided by the audio system",
                sample_rate
            );
        }
    })?;

    out_device.resume();

    let spec = out_device.spec();

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

    std::thread::spawn({
        move || {
            let mut buf = [0.0f32; GENERATOR_BUFFER_SIZE];

            loop {
                // contains lock
                {
                    gen.write().generate(&mut buf);
                }

                let _ = generator_fft_sender.try_send(buf.to_vec());

                if generator_sender.send(buf.to_vec()).is_err() {
                    break;
                }
            }
        }
    });

    Ok((Audio { player: out_device }, fft_receiver))
}