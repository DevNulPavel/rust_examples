use crate::{buffer_pool::SmartVector, error::CbltError};
use http::{Request, StatusCode, Version};
use httparse::Status;
use log::debug;
use std::str;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::instrument;

////////////////////////////////////////////////////////////////////////////////

/// Размер буфера
pub(super) const BUF_SIZE: usize = 8192;

// TODO: ???
/// Размер буфера на стеке?
pub(super) const STATIC_BUF_SIZE: usize = 1024;

/// Буфер для заголовков на стеке
pub(super) const HEADER_BUF_SIZE: usize = 32;

////////////////////////////////////////////////////////////////////////////////

#[cfg_attr(debug_assertions, instrument(level = "trace", skip_all))]
pub(super) async fn socket_to_request<S>(
    socket: &mut S,
    buffer: &mut SmartVector,
) -> Result<Request<Vec<u8>>, CbltError>
where
    S: AsyncReadExt + AsyncWriteExt + Unpin,
{
    // let mut buf = buffer.lock().await;
    loop {
        // Создаем небольшой буфер на стеке для удобства работы
        let mut temp_buf = [0; STATIC_BUF_SIZE];

        // Пробуем читать данные из сокета теперь
        let bytes_read = socket.read(&mut temp_buf).await.unwrap_or(0);

        // Если нам прилетело 0 данных, то это значит, что сокет уже закрыт
        if bytes_read == 0 {
            // Так что прервем работу
            break;
        }

        // Увеличиваем размер общего буффера на размер полученных данных
        buffer.extend_from_slice(&temp_buf[..bytes_read]);

        // Больше нам не нужен тот буфер стеке
        drop(temp_buf);

        // Создаем на стеке еще буфер для парсинга заголовков будущих
        let mut headers = [httparse::EMPTY_HEADER; HEADER_BUF_SIZE];

        // Создаем прилетевший запрос со слайсом в виде буфера
        // Теперь пробуем распарсить накопленные данные в виде запроса
        let request_parse_result = httparse::Request::new(&mut headers).parse(&buffer);

        // Смотрим на успешность парсинга
        match request_parse_result {
            // Завершился успешно парсинг,
            // прилетает смещение от начала буферов где
            // заканчиваются данные после всех заголовков
            Ok(Status::Complete(header_len)) => {
                let req_body_slice =
                    buffer
                        .get(..header_len)
                        .ok_or_else(|| CbltError::RequestError {
                            status_code: StatusCode::INTERNAL_SERVER_ERROR,
                            details: "Invalid slice".to_string(),
                        })?;

                // Теперь получаем уже тело запроса без заголовков
                let req_str = match str::from_utf8(req_body_slice) {
                    // Успешно распарсилось
                    Ok(v) => v,
                    // Ошибка парсинга тела, это не строка, а бинарные данные
                    Err(_) => {
                        return Err(CbltError::RequestError {
                            status_code: StatusCode::BAD_REQUEST,
                            details: "Bad request".to_string(),
                        });
                    }
                };

                // Пробуем распарсить заголовки запроса и получить `Content-Length`
                // заголовок.
                let (mut request, content_length) = match parse_request_headers(req_str) {
                    // Смогли распарсить запрос и длину
                    Some((req, content_length)) => (req, content_length),
                    // Не смогли распарсить
                    None => {
                        return Err(CbltError::RequestError {
                            status_code: StatusCode::BAD_REQUEST,
                            details: "Bad request".to_string(),
                        });
                    }
                };

                // Есть ли заголовок с конкретным размером контента?
                if let Some(content_length) = content_length {
                    // TODO: Зачем аллоцировать вектор?
                    // Получаем теперь тело непосредственно в виде аллоцированных данных
                    let mut body = buffer
                        .get(header_len..)
                        .ok_or_else(|| CbltError::RequestError {
                            status_code: StatusCode::INTERNAL_SERVER_ERROR,
                            details: "Invalid slice".to_string(),
                        })?
                        .to_vec();

                    // TODO: Какой-то временный буффер, зачем?
                    // TODO: Поменять на статический на стеке?
                    let mut temp_buf_inner = [0_u8; STATIC_BUF_SIZE];

                    // Пока не достигли ожидаемого размера тела
                    // продолжаем получать данные. Так может быть
                    // если тело запроса достаточно большое, поэтому нам
                    // надо поддгружать данные еще.
                    while body.len() < content_length {
                        // Вычитываем продолжение данных
                        let bytes_read = socket.read(&mut temp_buf_inner).await.unwrap_or(0);

                        // Если дальше не прочитать данные, то это значит,
                        // что сокет закрылся в процессе подгрузки
                        if bytes_read == 0 {
                            break;
                        }

                        // Получаем слайс на реальные данные подгруженные
                        // в этот раз
                        let read_slice =
                            temp_buf
                                .get(..bytes_read)
                                .ok_or_else(|| CbltError::RequestError {
                                    status_code: StatusCode::INTERNAL_SERVER_ERROR,
                                    details: "Invalid slice".to_string(),
                                })?;

                        // Добавляем их в общий буфер
                        body.extend_from_slice(read_slice);
                    }

                    // Получили теперь все данные тела, так что теперь можем эти
                    // данные сохранить у запроса.
                    *request.body_mut() = body;
                }

                #[cfg(debug_assertions)]
                debug!("{:?}", request);

                return Ok(request);
            }
            Ok(Status::Partial) => {
                // Need to read more data
                continue;
            }
            Err(_) => {
                return Err(CbltError::RequestError {
                    details: "Bad request".to_string(),
                    status_code: StatusCode::BAD_REQUEST,
                });
            }
        }
    }

    return Err(CbltError::ResponseError {
        details: "Bad request".to_string(),
        status_code: StatusCode::BAD_REQUEST,
    });
}

#[cfg_attr(debug_assertions, instrument(level = "trace", skip_all))]
pub(super) fn parse_request_headers(req_str: &str) -> Option<(Request<Vec<u8>>, Option<usize>)> {
    let mut headers = [httparse::EMPTY_HEADER; 32];
    let mut req = httparse::Request::new(&mut headers);

    match req.parse(req_str.as_bytes()) {
        Ok(Status::Complete(_)) => {
            let method = req.method?;
            let path = req.path?;
            let version = match req.version? {
                0 => Version::HTTP_10,
                1 => Version::HTTP_11,
                _ => return None,
            };

            let mut builder = Request::builder().method(method).uri(path).version(version);

            let mut content_length = None;

            for header in req.headers.iter() {
                let name = header.name;
                let value = header.value;
                builder = builder.header(name, value);

                if name.eq_ignore_ascii_case("Content-Length") {
                    if let Ok(s) = std::str::from_utf8(value) {
                        if let Ok(len) = s.trim().parse::<usize>() {
                            content_length = Some(len);
                        }
                    }
                }
            }

            builder
                .body(Vec::new())
                .ok()
                .map(|req| (req, content_length))
        }
        Ok(Status::Partial) => None, // Incomplete request
        Err(_) => None,              // Parsing failed
    }
}

#[cfg(test)]
mod tests {
    use crate::only_in_debug;
    use crate::request::parse_request_headers;
    use std::error::Error;

    #[test]
    fn test_simple() -> Result<(), Box<dyn Error>> {
        only_in_debug();

        let request_str = "POST /submit HTTP/1.1\r\n\
Host: example.com\r\n\
User-Agent: curl/7.68.0\r\n\
Accept: */*\r\n\
Content-Type: application/json\r\n\
Content-Length: 15\r\n\r\n\
{\"key\":\"value\"}";

        let req = parse_request_headers(request_str);
        println!("{:#?}", req);

        Ok(())
    }
}
