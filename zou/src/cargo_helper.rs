use authorization::{AuthorizationHeaderFactory, AuthorizationType, GetAuthorizationType};
use Bytes;
use client::{Config, GetResponse};
use contentlength::GetContentLength;
use hyper::header::{ByteRangeSpec, Headers, Range};
use std::error;
use std::fmt;
use std::result::Result;
use util::prompt_user;

/// Contains informations about the remote server
#[derive(Debug)]
pub struct RemoteServerInformations<'a> {
    pub accept_partialcontent: bool,
    pub auth_header: Option<AuthorizationHeaderFactory>,
    pub file: RemoteFileInformations,
    pub url: &'a str,
}

/// Contains informations about the remote file
#[derive(Debug)]
pub struct RemoteFileInformations {
    pub content_length: Bytes,
}

/// Some enumeration to display accurate errors
#[derive(Debug)]
pub enum RemoteServerError {
    /// Error throwed when too much connection has been connected, in order to
    /// create connection with the server
    TooMuchAttempting(usize),
    /// Error throwed when an Authorization type can't be deal with Zou
    UnknownAuthorizationType(AuthorizationType),
}

impl fmt::Display for RemoteServerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            RemoteServerError::TooMuchAttempting(ref attempts) => write!(f, "{} attempts failed", attempts),
            RemoteServerError::UnknownAuthorizationType(ref unknown_type) => write!(f, "{} is not supporting by Zou.\
                                                                                        You can create a new issue to report this problem \
                                                                                        at https://github.com/k0pernicus/zou/issues/new", unknown_type) 
        }
    }
}

impl error::Error for RemoteServerError {
    fn description(&self) -> &str {
        match *self {
            RemoteServerError::TooMuchAttempting(_) => "Many attempts failed",
            RemoteServerError::UnknownAuthorizationType(_) => "Authorization type not supported"
        }
    }
}

type RemoteServerInformationsResult<'a> = Result<RemoteServerInformations<'a>, RemoteServerError>;

/// Get Rust structure that contains network benchmarks
pub fn get_remote_server_informations<'a>(url: &'a str, ssl_support: bool) -> RemoteServerInformationsResult<'a> {
    // Создаем конфигурацию для Hyper
    let current_config = Config{ enable_ssl: ssl_support };
    // Создаем клиента для Hyper
    let hyper_client = current_config.get_hyper_client();
    // Запрашиваем у данного URL информацию чисто по хедерам
    let client_response = hyper_client.get_head_response(url).unwrap();
    
    // Вытягиваем тип авторизации (аутентификации)
    // затем создаем фабрику аутентификации
    let auth_type = client_response.headers.get_authorization_type();
    let auth_header_factory = match auth_type {
        Some(a_type) => {
            match a_type {
                // Если у нас basic - просим ввод пароля и логина
                AuthorizationType::Basic => {
                    warning!("The remote content is protected by Basic Auth.");
                    warning!("Please to enter below your credential informations.");
                    let username = prompt_user("Username:");
                    let password = prompt_user("Password:");
                    Some(AuthorizationHeaderFactory::new(AuthorizationType::Basic,
                                                         username,
                                                         Some(password)))
                }
                _ => {
                    return Err(RemoteServerError::UnknownAuthorizationType(a_type));
                }
            }
        }
        None => None,
    };

    let client_response = match auth_header_factory.clone() {
        Some(header_factory) => {
            // Если у нас уже есть фабрика аутентификации,
            // тогда заново выполняем запрос заголовком, но уже с этими данными
            // авторизации, которые ввел пользователь
            let mut headers = Headers::new();
            headers.set(header_factory.build_header());
            hyper_client
                .get_head_response_using_headers(url, headers)
                .unwrap()
        }
        None => {
            // Иначе используем старый ответ просто
            client_response
        },
    };

    // Определям размер возвращаемого контента
    let remote_content_length = match client_response.headers.get_content_length() {
        Some(remote_content_length) => {
            remote_content_length
        },
        None => {
            warning!("Cannot get the remote content length, using an \
                                 HEADER request.");
            warning!("Trying to send an HTTP request, to get the remote \
                                 content length...");
            // Заставляем сервер отправлять нам размер контента
            let mut custom_http_header = Headers::new();
            // HTTP заголовок, чтобы получить весь удаленный контент с 0 позиции,
            // если ответ OK, то ContentLength отправится назад серверу
            custom_http_header.set(Range::Bytes(vec![ByteRangeSpec::AllFrom(0)]));
            // Получаем ответ от сервера, используя кастомный HTTP запрос
            let client_response = hyper_client
                .get_http_response_using_headers(url, custom_http_header)
                .unwrap();
            // Попробовать снова получить размер контента, если нету - ошибка
            match client_response.headers.get_content_length() {
                Some(remote_content_length) => remote_content_length,
                None => {
                    return Err(RemoteServerError::TooMuchAttempting(2));
                }
            }
        }
    };

    Ok(RemoteServerInformations {
        accept_partialcontent: true,
        auth_header: auth_header_factory,
        file: RemoteFileInformations {
            content_length: remote_content_length,
        },    
        url: url,
    })
}
