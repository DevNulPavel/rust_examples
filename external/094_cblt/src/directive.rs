use crate::{
    buffer_pool::SmartVector,
    config::Directive,
    error::CbltError,
    file_server,
    helpers::matches_pattern,
    request::socket_to_request,
    response::{error_response, log_request_response, send_response},
    reverse_proxy,
    server::ServerConfig,
};
use http::{Response, StatusCode};
use log::{debug, info};
use reqwest::Client;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::instrument;

////////////////////////////////////////////////////////////////////////////////

// TODO: Не использовать Arc для вектора буфера
#[cfg_attr(debug_assertions, instrument(level = "trace", skip_all))]
pub async fn directive_process<S>(
    socket: &mut S,
    server: &ServerConfig,
    buffer: &mut SmartVector,
    client_reqwest: &Client,
) -> Result<(), CbltError>
where
    S: AsyncReadExt + AsyncWriteExt + Unpin,
{
    let request = {
        // Пробуем получить запрос теперь
        let convert_res = socket_to_request(socket, buffer).await;

        // Смотрим на прилетевший запрос
        match convert_res {
            // Есть какой-то конкретный запрос
            Ok(request) => request,
            // Была какая-то ошибка в процессе обработки
            Err(_) => {
                // Формируем теперь ответ с ошибкой
                let response = error_response(StatusCode::BAD_REQUEST);

                // Пробуем выдать ответ
                let ret = send_response(socket, response).await;

                // Смотрим на результаты отправки ответа
                match ret {
                    Ok(()) => {}
                    Err(err) => {
                        info!("Response send error: {}", err);
                        return Err(err);
                    }
                }

                return Err(CbltError::ParseRequestError {
                    details: "Parse request error".to_string(),
                });
            }
        }
    };

    // Пробуем из заголовков запроса заполучить заголовок Host,
    // может быть обращение к конкретному хосту там
    let host = match request.headers().get("Host") {
        Some(h) => h.to_str().unwrap_or(""),
        None => "",
    };

    // Пробуем получить конфиг теперь
    let host_config = {
        // Ищем нужный хост начиная со "*"
        let cfg_opt = server.hosts.iter().find(|(k, _)| k.starts_with("*"));

        // Смотрим, нашлось ли что-то общее среди хостов
        match cfg_opt {
            // Да, нашелся общий вариант конфига
            Some((_, cfg)) => cfg,
            // Нет, общего чего-то у нас не нашлось
            None => {
                // Пробуем тогда найти для конкретного хоста конфиг
                match server.hosts.get(host) {
                    // Нашлось что-то подходящее
                    Some(cfg) => cfg,
                    // Не нашлось
                    None => {
                        // Формируем тогда ответ с ошибкой
                        let response = error_response(StatusCode::FORBIDDEN);

                        // Пробуем назад выслать эту самую ошибку,
                        // смогли или нет - нас не волнует
                        let _ = send_response(socket, response).await;

                        // Ошибка передается на уровень вверх
                        return Err(CbltError::ResponseError {
                            details: "Forbidden".to_string(),
                            status_code: StatusCode::FORBIDDEN,
                        });
                    }
                }
            }
        }
    };

    // TODO: ???
    let mut root_path: Option<&str> = None;

    // Перебираем теперь директивы в конфиге
    for directive in host_config.iter() {
        match directive {
            Directive::Root { pattern, path } => {
                #[cfg(debug_assertions)]
                debug!("Root: {} -> {}", pattern, path);
                if matches_pattern(pattern, request.uri().path()) {
                    root_path = Some(path);
                }
            }
            Directive::FileServer => {
                #[cfg(debug_assertions)]
                debug!("File server");
                let ret = file_server::file_directive(root_path, &request, socket).await;
                match ret {
                    Ok(_) => {
                        log_request_response::<Vec<u8>>(&request, StatusCode::OK);
                        return Ok(());
                    }
                    Err(error) => match error {
                        CbltError::ResponseError {
                            details: _,
                            status_code,
                        } => {
                            let response = error_response(status_code);
                            match send_response(socket, response).await {
                                Ok(()) => {
                                    log_request_response::<Vec<u8>>(&request, status_code);
                                    return Ok(());
                                }
                                Err(err) => {
                                    log_request_response::<Vec<u8>>(
                                        &request,
                                        StatusCode::INTERNAL_SERVER_ERROR,
                                    );
                                    return Err(err);
                                }
                            }
                        }
                        CbltError::DirectiveNotMatched => {}
                        err => {
                            log_request_response::<Vec<u8>>(
                                &request,
                                StatusCode::INTERNAL_SERVER_ERROR,
                            );
                            return Err(err);
                        }
                    },
                }
                break;
            }
            Directive::ReverseProxy {
                pattern,
                destination,
            } => {
                #[cfg(debug_assertions)]
                debug!("Reverse proxy: {} -> {}", pattern, destination);
                match reverse_proxy::proxy_directive(
                    &request,
                    socket,
                    pattern,
                    destination,
                    client_reqwest.clone(),
                )
                .await
                {
                    Ok(status) => {
                        log_request_response::<Vec<u8>>(&request, status);
                        return Ok(());
                    }
                    Err(err) => match err {
                        CbltError::DirectiveNotMatched => {}
                        CbltError::ResponseError {
                            details: _,
                            status_code,
                        } => {
                            let response = error_response(status_code);
                            match send_response(socket, response).await {
                                Ok(()) => {
                                    log_request_response::<Vec<u8>>(&request, status_code);
                                    return Ok(());
                                }
                                Err(err) => {
                                    log_request_response::<Vec<u8>>(
                                        &request,
                                        StatusCode::INTERNAL_SERVER_ERROR,
                                    );
                                    return Err(err);
                                }
                            }
                        }
                        other => {
                            log_request_response::<Vec<u8>>(
                                &request,
                                StatusCode::INTERNAL_SERVER_ERROR,
                            );
                            return Err(other);
                        }
                    },
                }
            }
            Directive::Redir { destination } => {
                let dest = destination.replace("{uri}", request.uri().path());
                let response = Response::builder()
                    .status(StatusCode::FOUND)
                    .header("Location", &dest)
                    .body(Vec::new())?; // Empty body for redirects?
                match send_response(socket, response).await {
                    Ok(_) => {
                        log_request_response::<Vec<u8>>(&request, StatusCode::FOUND);
                        return Ok(());
                    }
                    Err(err) => {
                        log_request_response::<Vec<u8>>(
                            &request,
                            StatusCode::INTERNAL_SERVER_ERROR,
                        );
                        return Err(err);
                    }
                }
            }
            Directive::Tls { .. } => {}
        }
    }

    let response = error_response(StatusCode::NOT_FOUND);
    if let Err(err) = send_response(socket, response).await {
        log_request_response::<Vec<u8>>(&request, StatusCode::INTERNAL_SERVER_ERROR);
        return Err(err);
    }
    log_request_response::<Vec<u8>>(&request, StatusCode::NOT_FOUND);
    Ok(())
}
