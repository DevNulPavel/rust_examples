use sdl2::audio::AudioCallback;

// Добавляем от корня с помощью crate::, а не super::
use crate::audio::streamer::ExactStreamer;

// Потоковый плеер нужен для того, чтобы у него вызывался коллбек, который заполняет данные для воспроизведения звука
pub struct StreamingPlayer {
    pub stream: ExactStreamer<f32>,
}

impl AudioCallback for StreamingPlayer {
    type Channel = f32;

    // Коллбек вызывается из аудио системы, при этом он берет данные из канала и заполняет их в выходной буффер
    fn callback(&mut self, out: &mut [f32]) {
        // Специальный буфферизирующий поток
        self.stream
            .fill(out)
            .expect("channel broken in audio callback");
    }
}
