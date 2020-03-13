use sdl2::audio::AudioCallback;

// Добавляем от корня с помощью crate::, а не super::
use crate::audio::streamer::ExactStreamer;

pub struct StreamingPlayer {
    pub stream: ExactStreamer<f32>,
}

impl AudioCallback for StreamingPlayer {
    type Channel = f32;

    /// takes buffered audio from the channel and stores excess data inside `self.samples_remainder`
    fn callback(&mut self, out: &mut [f32]) {
        self.stream
            .fill(out)
            .expect("channel broken in audio callback");
    }
}
