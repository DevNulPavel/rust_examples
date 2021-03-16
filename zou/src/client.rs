use hyper::{
    client::{
        response::{
            Response
        },
        Client
    },
    error::{
        Error
    },
    header::{
        Headers
    },
    method::{
        Method
    },
    net::{
        HttpsConnector
    }
};
use hyper_openssl::{
    OpensslClient
};

//////////////////////////////////////////////////////////////////////

/// Структура для сохранения необходимости SSL
/// и для реализации стандартного HTTP/HTTPS клиента
pub struct ClientBuilder {
    pub enable_ssl: bool,
}

impl ClientBuilder {
    /// Создаем HTTP/HTTPS Hyper клиента
    pub fn build_hyper_client(&self) -> Client {
        if !self.enable_ssl {
            return Client::default();
        }
        Client::default_ssl()
    }
}

//////////////////////////////////////////////////////////////////////

/// Трейт для создания Hyper клиента с поддержкой SSL
trait SSLSupport {
    /// Функция создания клиента с SSL
    fn default_ssl() -> Client;
}

impl SSLSupport for Client {
    fn default_ssl() -> Client {
        let ssl = OpensslClient::new().unwrap();
        let connector = HttpsConnector::new(ssl);
        Client::with_connector(connector)
    }
}

//////////////////////////////////////////////////////////////////////

/// Трейт представляет собой некоторые методы для отправки специфичных запросов
pub trait GetResponse {
    /// Учитывая специфичный URL адрес, получем заголовок без тела контента
    /// Полезно для того, чтобы не тратить время и ресурсы на тело
    fn get_head_response(&self, url: &str) -> Result<Response, Error>;

    /// Давая специфичный URL и заголовок, получаем заголовок ответа без тела контента
    fn get_head_response_using_headers(&self,
                                       url: &str,
                                       header: Headers) -> Result<Response, Error>;

    /// Для конкретного URL, получаем ответ от целевого сервера
    fn get_http_response(&self, url: &str) -> Result<Response, Error>;

    /// Given a specific URL and an header, get the response from the target server
    /// 
    fn get_http_response_using_headers(&self,
                                       url: &str,
                                       header: Headers) -> Result<Response, Error>;
}

impl GetResponse for Client {
    fn get_head_response(&self, url: &str) -> Result<Response, Error> {
        self.request(Method::Head, url).send()
    }

    fn get_head_response_using_headers(
        &self,
        url: &str,
        custom_header: Headers,
    ) -> Result<Response, Error> {
        self.request(Method::Head, url)
            .headers(custom_header)
            .send()
    }

    fn get_http_response(&self, url: &str) -> Result<Response, Error> {
        self.get_http_response_using_headers(url, Headers::new())
    }

    fn get_http_response_using_headers(
        &self,
        url: &str,
        custom_header: Headers,
    ) -> Result<Response, Error> {
        self.request(Method::Get, url).headers(custom_header).send()
    }
}
