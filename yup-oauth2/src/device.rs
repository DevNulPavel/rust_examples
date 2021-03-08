use std::{
    borrow::{
        Cow
    },
    time::{
        Duration
    }
};
use hyper::{
    header
};
use url::{
    form_urlencoded
};
use crate::{
    authenticator_delegate::{
        DefaultDeviceFlowDelegate, 
        DeviceAuthResponse, 
        DeviceFlowDelegate,
    },
    error::{
        AuthError, 
        Error
    },
    types::{
        ApplicationSecret, 
        TokenInfo
    }
};

pub const GOOGLE_DEVICE_CODE_URL: &str = "https://accounts.google.com/o/oauth2/device/code";

// https://developers.google.com/identity/protocols/OAuth2ForDevices#step-4:-poll-googles-authorization-server
pub const GOOGLE_GRANT_TYPE: &str = "http://oauth.net/grant_type/device/1.0";

/// Implements the [Oauth2 Device Flow](https://developers.google.com/youtube/v3/guides/authentication#devices)
/// Работает в 2 этапа:
/// * заполучает код, чтобы показать пользователю
/// * (неоднократно) опрашивает пользователя для аутентификации в приложении
pub struct DeviceFlow {
    pub(crate) app_secret: ApplicationSecret,
    pub(crate) device_code_url: Cow<'static, str>,
    pub(crate) flow_delegate: Box<dyn DeviceFlowDelegate>, // делегат потока auth
    pub(crate) grant_type: Cow<'static, str>,
}

impl DeviceFlow {
    /// Создает новый DeviceFlow. Стандартный FlowDelegate будет использован,
    /// стандартное время ожидания - 120 секунд
    pub(crate) fn new(app_secret: ApplicationSecret) -> Self {
        DeviceFlow {
            app_secret,
            device_code_url: GOOGLE_DEVICE_CODE_URL.into(),
            flow_delegate: Box::new(DefaultDeviceFlowDelegate),
            grant_type: GOOGLE_GRANT_TYPE.into(),
        }
    }

    /// Запрос токена
    pub(crate) async fn token<C, T>(&self, hyper_client: &hyper::Client<C>, scopes: &[T]) -> Result<TokenInfo, Error>
    where
        T: AsRef<str>,
        C: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
    {
        // Запрос кода с сервера
        let device_auth_resp: DeviceAuthResponse = Self::request_code(&self.app_secret,
                                                  hyper_client,
                                                  &self.device_code_url,
                                                  scopes)
            .await?;

        log::debug!("Presenting code to user");

        // Предоставить пользователю код
        self
            .flow_delegate
            .present_user_code(&device_auth_resp)
            .await;

        // Дождаться токена
        self.wait_for_device_token(hyper_client,
                                   &self.app_secret,
                                   &device_auth_resp,
                                   &self.grant_type)
            .await
    }

    /// Дожидаемся окончания работы токена
    async fn wait_for_device_token<C>(&self,
                                      hyper_client: &hyper::Client<C>,
                                      app_secret: &ApplicationSecret,
                                      device_auth_resp: &DeviceAuthResponse,
                                      grant_type: &str) -> Result<TokenInfo, Error>
    where
        C: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
    {
        // интервал ожидания ответа от сервера, изначально нулевое ожидание?
        let mut interval = device_auth_resp.interval;
        log::debug!("Polling every {:?} for device token", interval);
        loop {
            // Асинхронно ждем интервал времени
            tokio::time::sleep(interval).await;
            let token_response = Self::poll_token(&app_secret, 
                                                hyper_client,
                                                &device_auth_resp.device_code,
                                                grant_type)
                .await;

            interval = match token_response {
                // Получили токен
                Ok(token) => {
                    return Ok(token)
                },
                // Если ошибка авторизации, то выдаем снова интервал ожидания
                Err(Error::AuthError(AuthError { error, .. })) if error.as_str() == "authorization_pending" => {
                    log::debug!("still waiting on authorization from the server");
                    interval
                }
                // Если ошибка авторизации, то выдаем снова НОВЫЙ интервал ожидания 
                Err(Error::AuthError(AuthError { error, .. })) if error.as_str() == "slow_down" => {
                    let interval = interval + Duration::from_secs(5);
                    log::debug!("server requested slow_down. Increasing polling interval to {:?}", interval);
                    interval
                }
                Err(err) => return Err(err),
            }
        }
    }

    /// Первый шаг включает в себя запрос у сервера код, который пользователь может печатать в поле в указанный URL
    /// Он вызывается лишь раз, предполагая, что не было ошибки соединения
    /// Иначе он может быть вызван снова пока не получите результат OK
    /// # Arguments
    /// * `client_id` & `client_secret` - заполучается при регистрации приложения (https://developers.google.com/youtube/registering_an_application)
    /// * `scopes` - an iterator yielding String-like objects which are URLs defining what your
    ///              application is able to do. It is considered good behaviour to authenticate
    ///              only once, with all scopes you will ever require.
    ///              However, you can also manage multiple tokens for different scopes, if your
    ///              application is providing distinct read-only and write modes.
    /// # Panics
    /// * If called after a successful result was returned at least once.
    /// # Examples
    /// See test-cases in source code for a more complete example.
    async fn request_code<C, T>(application_secret: &ApplicationSecret,
                                client: &hyper::Client<C>,
                                device_code_url: &str,
                                scopes: &[T]) -> Result<DeviceAuthResponse, Error>
    where
        T: AsRef<str>,
        C: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
    {
        let req = form_urlencoded::Serializer::new(String::new())
            .extend_pairs(&[ ("client_id", application_secret.client_id.as_str()),
                             ("scope", crate::helper::join(scopes, " ").as_str()) ])
            .finish();

        // Создаем запрос
        // note: works around bug in rustlang
        // https://github.com/rust-lang/rust/issues/22252
        let req = hyper::Request::post(device_code_url)
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(hyper::Body::from(req))
            .unwrap();

        log::debug!("requesting code from server: {:?}", req);

        // Выполняем запрос к серверу
        let (head, body) = client
            .request(req)
            .await?
            .into_parts();

        // Тело конвертируем в байты
        let body = hyper::body::to_bytes(body)
            .await?;

        log::debug!("received response; head: {:?}, body: {:?}", head, body);

        DeviceAuthResponse::from_json(&body)
    }

    /// Если первый вызов был успешный, данный метод может быть вызван.
    /// Пока мы ожидаем аутентификации, он будет возвращать `Ok(None)`
    /// Вы должны вызывать этот метод с интервалом, выданным в поле `DeviceAuthResponse.interval` до этого
    /// 
    /// Операция была успешной когда будет выдан Ok(Some(Token)) в первый раз.
    /// Последующие вызовы так же могут возвращать ошибочное состояние.
    ///
    /// Не нужно вызывать после получения PollError::Expired|PollError::AccessDenied
    /// Таким образом в случае ошибки кроме `PollError::HttpError`, мы должны заново начать цепочку действий.
    ///
    /// > ⚠️ **Warning**: We assume the caller doesn't call faster than `interval` and are not
    /// > protected against this kind of mis-use.
    ///
    /// # Examples
    /// See test-cases in source code for a more complete example.
    async fn poll_token<'a, C>(application_secret: &ApplicationSecret,
                               client: &hyper::Client<C>,
                               device_code: &str,
                               grant_type: &str) -> Result<TokenInfo, Error>
    where
        C: hyper::client::connect::Connect + Clone + Send + Sync + 'static 
    {
        // We should be ready for a new request
        let req = form_urlencoded::Serializer::new(String::new())
            .extend_pairs(&[
                ("client_id", application_secret.client_id.as_str()),
                ("client_secret", application_secret.client_secret.as_str()),
                ("code", device_code),
                ("grant_type", grant_type),
            ])
            .finish();

        let request = hyper::Request::post(&application_secret.token_uri)
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(hyper::Body::from(req))
            .unwrap(); // TODO: Error checking
        log::debug!("polling for token: {:?}", request);
        let (head, body) = client.request(request).await?.into_parts();
        let body = hyper::body::to_bytes(body).await?;
        log::debug!("received response; head: {:?} body: {:?}", head, body);
        TokenInfo::from_json(&body)
    }
}
