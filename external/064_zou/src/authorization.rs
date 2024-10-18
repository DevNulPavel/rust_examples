use hyper::{
    header::{
        Authorization, 
        Basic, 
        Headers, 
        Scheme
    }
};
use std::{
    fmt::{
        Display, 
        Formatter, 
        Result
    }
};

/// Перечисление для разных типов авторизации
#[derive(Clone, Debug)]
pub enum AuthorizationType {
    Basic,
    Digest,
    Unknown,
}

/// Специальный трейт для расширения функциональности типа Headers из `hyper`
pub trait GetAuthorizationType {
    /// Функция для получения типа авторизации remote аккаунта
    fn get_authorization_type(&self) -> Option<AuthorizationType>;
}

/// Реализация трейта выше для заголовков из `hyper`
impl GetAuthorizationType for Headers {
    /// Функция для получения `WWW-Authenticate` контейнера из данного заголовка
    /// Данная функция возвращает Option, который содержит `AuthorizationType`
    fn get_authorization_type(&self) -> Option<AuthorizationType> {
        // Получаем значение в сыром виде (массив векторов байт)
        match self.get_raw("WWW-Authenticate") {
            Some(raw) => {
                // Перегоняем в строку данные
                let header_content = String::from_utf8(raw
                                                        .get(0)
                                                        .unwrap()
                                                        .clone())
                    .unwrap();
                // Разделяем по пробелам
                let mut header_parts = header_content.split(" ");
                
                // Берем первый элемент и смотрим значение
                let auth_type = match header_parts.next() {
                    Some(part) => {
                        match part {
                            "Basic" => AuthorizationType::Basic,
                            "Digest" => AuthorizationType::Digest,
                            _ => AuthorizationType::Unknown,
                        }
                    }
                    None => return None,
                };
                Some(auth_type)
            }
            None => None,
        }
    }
}

impl Display for AuthorizationType {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            &AuthorizationType::Basic => write!(f, "Basic"),
            &AuthorizationType::Digest => write!(f, "Digest"),
            _ => write!(f, "Unknown"),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct AuthorizationHeaderFactory {
    authorization_type: AuthorizationType,
    username: String,
    password: Option<String>,
}

impl AuthorizationHeaderFactory {
    pub fn new(authorization_type: AuthorizationType,
               username: String,
               password: Option<String>) -> AuthorizationHeaderFactory {
        AuthorizationHeaderFactory {
            authorization_type: authorization_type,
            username: username,
            password: password,
        }
    }

    pub fn build_header(&self) -> Authorization<String> {
        match self.authorization_type {
            AuthorizationType::Basic => {
                Authorization(format!("Basic {}", self))
            },
            _ => {
                epanic!(&format!("{} Authorization is not supported!",
                                 self.authorization_type))
            }
        }
    }
}

impl Display for AuthorizationHeaderFactory {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self.authorization_type {
            AuthorizationType::Basic => {
                let basic_auth = Basic {
                    username: self.username.clone(),
                    password: self.password.clone(),
                };
                basic_auth.fmt_scheme(f)
            }
            _ => {
                if let Some(ref password) = self.password {
                    write!(f, "{}:{}", self.username, password)
                } else {
                    write!(f, "{}", self.username)
                }
            }
        }
    }
}
