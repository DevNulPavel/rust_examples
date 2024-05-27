use crate::{
    error::{
        AuthErrorOr, 
        Error
    }
};
use chrono::{
    DateTime, 
    Utc
};
use serde::{
    Deserialize, 
    Serialize
};

////////////////////////////////////////////////////////////////////////////////////////////////

/// Представляет из себя токен, возвращаемый после аутентификации. Поддерживаются только Bearer токены
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub struct AccessToken {
    value: String,
    expires_at: Option<DateTime<Utc>>, // Время завершения работоспособности токена
}

impl AccessToken {
    /// Строковое представление токена
    pub fn as_str(&self) -> &str {
        &self.value
    }

    /// Время, когда токен будет просрочен
    pub fn expiration_time(&self) -> Option<DateTime<Utc>> {
        self.expires_at
    }

    /// Определяем, не просрочен ли токен.
    /// Информирование о завершении работы токена будет за 1 минуту до его фактического окончания,
    /// чтобы гарантировать, что токен который еще в работе все еще валидный.
    pub fn is_expired(&self) -> bool {
        // Считаем токен просроченым если его время жизни истекает через 1ну минуту
        self.expires_at
            .map(|expiration_time| expiration_time - chrono::Duration::minutes(1) <= Utc::now())
            .unwrap_or(false)
    }
}

/// Интерпретация данного токена как ссылку на строку
impl AsRef<str> for AccessToken {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

/// Конвертация данного токена из десереализованной информации
impl From<TokenInfo> for AccessToken {
    fn from(value: TokenInfo) -> Self {
        AccessToken {
            value: value.access_token,
            expires_at: value.expires_at,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////

/// Представляет из себя токен, возвращаемый OAuth2 сервером
///
/// Создается всеми потоками аутентификации
/// Он аутентифицирует определенные операции и должен быть сброшен когда достигается дата окончания
#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub(crate) struct TokenInfo {
    /// Используется когда вызываются oauth2 сервисы
    pub(crate) access_token: String,
    /// Данный токен используется для переполучения завершенного токена
    pub(crate) refresh_token: Option<String>,
    /// Дата, когда токен сбрасывается
    pub(crate) expires_at: Option<DateTime<Utc>>,
}

impl TokenInfo {
    /// Парсинг токена из уже готовых данных
    pub(crate) fn from_json(json_data: &[u8]) -> Result<TokenInfo, Error> {
        #[derive(Deserialize)]
        struct RawToken {
            access_token: String,
            refresh_token: Option<String>,
            token_type: String,
            expires_in: Option<i64>,
        }

        let RawToken {
            access_token,
            refresh_token,
            token_type,
            expires_in,
        } = serde_json::from_slice::<AuthErrorOr<RawToken>>(json_data)?.into_result()?;

        // Если у нас не Bearer токен, тогда выдаем ошибку наружу
        if token_type.to_lowercase().as_str() != "bearer" {
            use std::io;
            return Err(io::Error::new(io::ErrorKind::InvalidData,
                                      format!(r#"unknown token type returned; expected "bearer" found {}"#, token_type))
                .into());
        }

        // Конкретное время истечения вместо длительности жизни
        let expires_at = expires_in
            .map(|seconds_from_now| {
                Utc::now() + chrono::Duration::seconds(seconds_from_now)
            });

        Ok(TokenInfo {
            access_token,
            refresh_token,
            expires_at,
        })
    }

    /// Возвращает true если уже истечен
    pub fn is_expired(&self) -> bool {
        self.expires_at
            .map(|expiration_time| {
                (expiration_time - chrono::Duration::minutes(1)) <= Utc::now()
            })
            .unwrap_or(false)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////

/// Представляет собой 'installed' или 'web' приложения в json файлике
/// See `ConsoleApplicationSecret` for more information
#[derive(Deserialize, Serialize, Clone, Default, Debug)]
pub struct ApplicationSecret {
    /// The client ID.
    pub client_id: String,
    /// The client secret.
    pub client_secret: String,
    /// The token server endpoint URI.
    pub token_uri: String,
    /// The authorization server endpoint URI.
    pub auth_uri: String,
    /// The redirect uris.
    pub redirect_uris: Vec<String>,
    /// Name of the google project the credentials are associated with
    pub project_id: Option<String>,
    /// The service account email associated with the client.
    pub client_email: Option<String>,
    /// The URL of the public x509 certificate, used to verify the signature on JWTs, such
    /// as ID tokens, signed by the authentication provider.
    pub auth_provider_x509_cert_url: Option<String>,
    ///  The URL of the public x509 certificate, used to verify JWTs signed by the client.
    pub client_x509_cert_url: Option<String>,
}

/// Тип, помогающий чтению и записи json файликов
/// возвращается из [google developer console](https://code.google.com/apis/console)
#[derive(Deserialize, Serialize, Default, Debug)]
pub struct ConsoleApplicationSecret {
    /// web app secret
    pub web: Option<ApplicationSecret>,
    /// installed app secret
    pub installed: Option<ApplicationSecret>,
}

#[cfg(test)]
pub mod tests {
    use super::*;

    pub const SECRET: &'static str =
        "{\"installed\":{\"auth_uri\":\"https://accounts.google.com/o/oauth2/auth\",\
         \"client_secret\":\"UqkDJd5RFwnHoiG5x5Rub8SI\",\"token_uri\":\"https://accounts.google.\
         com/o/oauth2/token\",\"client_email\":\"\",\"redirect_uris\":[\"urn:ietf:wg:oauth:2.0:\
         oob\",\"oob\"],\"client_x509_cert_url\":\"\",\"client_id\":\
         \"14070749909-vgip2f1okm7bkvajhi9jugan6126io9v.apps.googleusercontent.com\",\
         \"auth_provider_x509_cert_url\":\"https://www.googleapis.com/oauth2/v1/certs\"}}";

    #[test]
    fn console_secret() {
        use serde_json as json;
        match json::from_str::<ConsoleApplicationSecret>(SECRET) {
            Ok(s) => assert!(s.installed.is_some() && s.web.is_none()),
            Err(err) => panic!(
                "Encountered error parsing ConsoleApplicationSecret: {}",
                err
            ),
        }
    }
}
