use cargo_helper::RemoteServerInformations;
use Bytes;
use client::{Config, GetResponse};
use hyper::client::Client;
use hyper::error::Error;
use hyper::header::{ByteRangeSpec, Headers, Range};
use pbr::{MultiBar, Pipe, ProgressBar, Units};
use response::CheckResponseStatus;
use std::cmp::min;
use std::io::Read;
use std::thread;
use std::time::{Instant, Duration};
use write::{OutputFileWriter, OutputChunkWriter};

/// Constant to represent the length of the buffer to download
/// the remote content
const DOWNLOAD_BUFFER_BYTES: usize = 1024 * 64;

/// Constant to represent the refresh interval (in milliseconds)
/// for the CLI
const PROGRESS_UPDATE_INTERVAL_MILLIS: u64 = 500;

/// Represents a range between two Bytes types
#[derive(Debug, PartialEq)]
struct RangeBytes(Bytes, Bytes);

macro_rules! initbar {
    ($mp:ident,$mpb:ident,$length:expr,$index:expr,$server:expr) => {
        let mut $mp = $mpb.create_bar($length);
        $mp.tick_format("▏▎▍▌▋▊▉██▉▊▋▌▍▎▏");
        $mp.format("|#--|");
        $mp.show_tick = true;
        $mp.show_speed = true;
        $mp.show_percent = true;
        $mp.show_counter = false;
        $mp.show_time_left = true;
        $mp.set_units(Units::Bytes);
        $mp.message(&format!("Chunk {} (from {}) ", $index, $server));
    }
}

/// Функция получения длины чанка, основанная на индексе чанка
fn get_chunk_length(chunk_index: u64,
                    content_length: Bytes,
                    global_chunk_length: Bytes) -> Option<RangeBytes> {
    // Если размер контента нулевой - выход
    if content_length == 0 || global_chunk_length == 0 {
        return None;
    }

    // Получаем диапазон значений как индекс, умноженный раз размер чанка
    let b_range: Bytes = chunk_index * global_chunk_length;

    // Если размер превышен, тогда None
    if b_range >= (content_length - 1) {
        return None;
    }

    // Ограничиваем диапазон размером файлика
    let e_range: Bytes = min(content_length - 1,
                             ((chunk_index + 1) * global_chunk_length) - 1);

    Some(RangeBytes(b_range, e_range))
}


/// Функция получения HTTP заголовка для отправки на файловый сервер для конкретного чанка, определенного индексом
fn get_header_from_index(chunk_index: u64,
                         content_length: Bytes,
                         global_chunk_length: Bytes) -> Option<(Headers, RangeBytes)> {
    get_chunk_length(chunk_index, content_length, global_chunk_length)
        .map(|range| {
            // На исновании диапазона создаем заголовок
            let mut header = Headers::new();
            // Устанавливаем диапазон на основаниии полученного диапазона
            header.set(Range::Bytes(vec![ByteRangeSpec::FromTo(range.0, range.1)]));
            // Возвращаем хедер, и диапазон в байтах (позиция + размер)
            (header, RangeBytes(range.0, range.1 - range.0))
        })
}


/// Функция для получения с сервера содержимого чанка
/// Данная функция возвращает Result - Bytes, если контень хедера доступен
/// Error - иначе
fn download_a_chunk(http_client: &Client,
                    http_header: Headers,
                    mut chunk_writer: OutputChunkWriter,
                    url: &str,
                    mpb: &mut ProgressBar<Pipe>,
                    monothreading: bool) -> Result<Bytes, Error> {
    // Получаем HTTP ответ c хедерами
    let mut body = http_client
        .get_http_response_using_headers(url, http_header)
        .unwrap();
    
    // Если однопоточно и не поддерживается загрузка по частям - ошибка
    if monothreading && !body.check_partialcontent_status() {
        return Err(Error::Status);
    }

    // Буффер на стеке в 64 килобайта
    let mut bytes_buffer = [0; DOWNLOAD_BUFFER_BYTES];
    let mut sum_bytes = 0;

    // Интервал обновления прогресса
    let progress_update_interval = Duration::from_millis(PROGRESS_UPDATE_INTERVAL_MILLIS);

    // Последний прогресс в байтах
    let mut last_progress_bytes = 0;
    // Последнее время прогресса
    let mut last_progress_time = Instant::now() - progress_update_interval;

    // Читаем из body в буффер
    while let Ok(n) = body.read(&mut bytes_buffer) {
        // Если не прочитали ничего, значит возвращаем
        if n == 0 {
            return Ok(sum_bytes);
        }

        chunk_writer.write(sum_bytes, &bytes_buffer[0..n]);
        sum_bytes += n as u64;

        // Update the CLI
        if Instant::now().duration_since(last_progress_time) > progress_update_interval {
            last_progress_time = Instant::now();
            let progress_bytes_delta = sum_bytes - last_progress_bytes;
            last_progress_bytes = sum_bytes;
            mpb.add(progress_bytes_delta);
        }
    }
    mpb.add(sum_bytes - last_progress_bytes);
    return Ok(0u64);
}

/// Функция для загрузки каждого чанка удаленного контента с сервера, данного через URL
/// Данная функция принимает в виде параметров:
/// * размера контента удаленного
/// * мутабельную ссылку писателя, чтобы шарить между потоками, которые содержат каждый чанк
/// * число чанков, которые содержат удаленный контент
/// * урл удаленного контент-сервера
/// * кастомную авторизацию для доступа и загрузки удаленного контента
pub fn download_chunks<'a>(cargo_info: RemoteServerInformations<'a>,
                           mut out_file: OutputFileWriter,
                           nb_chunks: u64,
                           ssl_support: bool) -> bool {
    // Разворачиваем переменные в переменные
    let (content_length, auth_header_factory) = (cargo_info.file.content_length, cargo_info.auth_header);
    // Расчитываем максимальный размер чанка
    let global_chunk_length: u64 = (content_length / nb_chunks) + 1;

    // Массив с джобами
    let mut jobs = vec![];

    // Создаем экземпляр терминального прогрессбара
    let mut mpb = MultiBar::new();
    mpb.println(&format!("Downloading {} chunk(s): ", nb_chunks));

    // Идем по массиву чанков
    for chunk_index in 0..nb_chunks {
        // Получаем урл для загрузки
        let server_url = cargo_info.url.clone();
        // Создаем клон урла загрузки
        let url_clone = String::from(server_url);

        // HTTP заголовок + диапазон байтов
        let (mut http_header, RangeBytes(chunk_offset, chunk_length)) = 
            get_header_from_index(chunk_index, content_length, global_chunk_length)
                .unwrap();

        // Создаем конфиг
        let current_config = Config { enable_ssl: ssl_support };
        // Создаем клиент на основе конфига
        let hyper_client = current_config.get_hyper_client();

        // Клонируем заголовок аутентификации
        if let Some(auth_header_factory) = auth_header_factory.clone() {
            http_header.set(auth_header_factory.build_header());
        }

        // Один поток или нет?
        let monothreading = cargo_info.accept_partialcontent;

        // Инициализируем прогрессбар для данного чанка
        initbar!(mp, mpb, chunk_length, chunk_index, server_url);

        // Создаем писателя в файлик для диапазона
        let chunk_writer = out_file.get_chunk_writer(chunk_offset);

        // Создаем поток загрузки, в нем мы пушим булевское значение, чтобы знать, что чанк в порядке
        jobs.push(thread::spawn(move || {
            // Грузим наш чанк
            let res = download_a_chunk(&hyper_client,
                                       http_header,
                                       chunk_writer,
                                       &url_clone,
                                       &mut mp,
                                       monothreading);
            match res {
                Ok(bytes_written) => {
                    mp.finish();
                    if bytes_written == 0 {
                        error!(&format!("The downloaded chunk {} is empty", chunk_index));
                    }
                    return true;
                }
                Err(error) => {
                    mp.finish();
                    error!(&format!(
                        "Cannot download the chunk {}, due to error {}",
                        chunk_index,
                        error
                    ));
                    return false;
                }
            }
        }));
    }

    mpb.listen();

    // Contain the result state for chunks
    let mut child_results: Vec<bool> = Vec::with_capacity(nb_chunks as usize);

    for child in jobs {
        match child.join() {
            Ok(b) => child_results.push(b),
            Err(_) => child_results.push(false),
        }
    }

    // Check if all chunks are OK
    return child_results.iter().all(|x| *x);
}

#[cfg(test)]
mod test_chunk_length {

    use super::get_chunk_length;
    use super::RangeBytes;

    #[test]
    fn wrong_content_length_parameter_should_return_none() {
        assert_eq!(None, get_chunk_length(0, 15, 0));
    }

    #[test]
    fn wrong_global_chunk_length_parameter_should_return_none() {
        assert_eq!(None, get_chunk_length(0, 0, 15));
    }

    #[test]
    fn wrong_length_parameters_should_return_none() {
        assert_eq!(None, get_chunk_length(0, 0, 0));
    }

    #[test]
    fn get_the_first_range_in_chunk() {
        assert_eq!(Some(RangeBytes(0, 249)), get_chunk_length(0, 1000, 250));
    }

    #[test]
    fn get_the_last_range_in_chunk() {
        assert_eq!(Some(RangeBytes(750, 999)), get_chunk_length(3, 1000, 250));
    }

    #[test]
    fn get_the_last_range_in_shorten_chunk() {
        assert_eq!(Some(RangeBytes(750, 997)), get_chunk_length(3, 998, 250));
    }

    #[test]
    fn wrong_index_parameter_should_return_none() {
        assert_eq!(None, get_chunk_length(4, 1000, 250));
    }

}

#[cfg(test)]
mod test_header {

    use super::{get_header_from_index, RangeBytes};
    use hyper::header::{ByteRangeSpec, Headers, Range};

    #[test]
    fn wrong_chunk_length_should_return_none() {
        assert_eq!(None, get_header_from_index(0, 0, 0));
    }

    #[test]
    fn good_chunk_length_should_return_a_good_header() {
        let mut test_header = Headers::new();
        test_header.set(Range::Bytes(vec![ByteRangeSpec::FromTo(750, 997)]));
        assert_eq!(
            Some((test_header, RangeBytes(750, 247))),
            get_header_from_index(3, 998, 250)
        );
    }

}
