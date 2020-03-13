mod player;
mod audio;
mod streamer;

pub use audio::init;
pub use audio::Audio;
pub use player::StreamingPlayer;
pub use streamer::ExactStreamer;

pub const GENERATOR_BUFFER_SIZE: usize = 256;
pub const GENERATOR_CHANNEL_SIZE: usize = 6;