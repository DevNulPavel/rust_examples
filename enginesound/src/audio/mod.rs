mod player;
mod audio;
mod streamer;

pub use audio::init;
pub use audio::Audio;
pub use player::StreamingPlayer;
pub use streamer::ExactStreamer;

// Размер одного буффера, выдаваемого генератором
pub const GENERATOR_BUFFER_SIZE: usize = 256;
