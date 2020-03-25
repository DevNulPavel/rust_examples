// Vorbis decoder written in Rust
//
// This example file is licensed
// under the CC-0 license:
// https://creativecommons.org/publicdomain/zero/1.0/

extern crate alto;
extern crate lewton;
extern crate byteorder;

use std::env;
use lewton::VorbisError;
use lewton::inside_ogg::OggStreamReader;
use std::fs::File;
use std::thread::sleep;
use std::time::{Instant, Duration};
use alto::{Alto, Mono, Stereo, Source};

fn main() {
	match run() {
		Ok(_) =>(),
		Err(err) => println!("Error: {}", err),
	}
}

fn run() -> Result<(), VorbisError> {
	// Получаем имя файлика из параметров
	let file_path = env::args()
		.nth(1)
		.expect("No arg found. Please specify a file to open.");
	println!("Opening file: {}", file_path);

	// Открываем файлик
	let f = File::open(file_path).expect("Can't open file");

	// Создаем reader
	let mut srr = OggStreamReader::new(f)?;

	// Подготавливаем воспроизведение
	let al = Alto::load_default().expect("Could not load alto");
	// Открываем устройство
	let device = al.open(None).expect("Could not open device");
	// Получаем контекст воспроизведения
	let cxt = device.new_context(None).expect("Could not create context");
	// Получаем объект для потокового воспроизведения
	let mut player_streaming = cxt.new_streaming_source()
		.expect("could not create streaming src");
	// Получаем частоту семплирования
	let sample_rate = srr.ident_hdr.audio_sample_rate as i32;

	// Если много каналов - выводим сообщение
	if srr.ident_hdr.audio_channels > 2 {
		println!("Stream error: {} channels are too many!", srr.ident_hdr.audio_channels);
	}

	println!("Sample rate: {}", srr.ident_hdr.audio_sample_rate);

	// Now the fun starts..
	// Номер пакета декодированного
	let mut n = 0;
	let mut len_play = 0.0;
	let mut start_play_time = None;
	// Время начала воспроизведения
	let start_decode_time = Instant::now();
	// Частота семплирования * количество каналов
	let sample_channels = srr.ident_hdr.audio_channels as f32 * srr.ident_hdr.audio_sample_rate as f32;
	// Пока можем читать данные - итерируемся
	while let Some(pck_samples) = srr.read_dec_packet_itl()? {
		// Информация и данных
		println!("Decoded packet no {}, with {} samples.", n, pck_samples.len());

		n += 1;

		// Определяем, сколкьо из каналов декодировали, получаем его буффер
		let buf = match srr.ident_hdr.audio_channels {
			1 => cxt.new_buffer::<Mono<i16>,_>(&pck_samples, sample_rate),
			2 => cxt.new_buffer::<Stereo<i16>,_>(&pck_samples, sample_rate),
			n => panic!("unsupported number of channels: {}", n),
		}.unwrap();

		// Вкидываем буффер на воспроизведение
		player_streaming.queue_buffer(buf).unwrap();

		// Добавляем количество семплов к общей сумме
		len_play += pck_samples.len() as f32 / sample_channels;

		// Если мы быстрее реалтайма, то мы начинаем воспроизведение сейчас
		if n == 100 {
			// Берем текущее время
			let cur = Instant::now();
			let diff = cur - start_decode_time;
			// Если фактического времени прошло меньше, чем накодировано
			if diff < Duration::from_millis((len_play * 1000.0) as u64) {
				// Тогда начинаем воспроизведение
				start_play_time = Some(cur);
				player_streaming.play();
			}
		}
	}
	// Все декодировали - можно воспроизвести до конца
	let total_duration = Duration::from_millis((len_play * 1000.0) as u64);
	let sleep_duration = total_duration - match start_play_time {
			None => {
				player_streaming.play();
				Duration::from_millis(0)
			},
			Some(t) => (Instant::now() - t)
		};
	println!("The piece is {} s long.", len_play);
	sleep(sleep_duration);

	Ok(())
}
