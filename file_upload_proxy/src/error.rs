use hyper::StatusCode;
use std::{borrow::Cow, error::Error as StdError, fmt::Display};

/////////////////////////////////////////////////////////////////////////////////////////////////////////

pub trait WrapErrorWithStatusAndDesc<T> {
    /// Оборачиваем ошибку в 500й статус
    fn wrap_err_with_500(self: Self) -> Result<T, ErrorWithStatusAndDesc>;

    /// Оборачиваем ошибку в конкретный статус
    fn wrap_err_with_status(self: Self, status: StatusCode) -> Result<T, ErrorWithStatusAndDesc>;

    /// Оборачиваем ошибку в статус и описание
    fn wrap_err_with_status_desc(self: Self, status: StatusCode, desc: Cow<'static, str>) -> Result<T, ErrorWithStatusAndDesc>;

    /// Оборачиваем ошибку в статус и описание (отложенное)
    fn wrap_err_with_status_fn_desc<F>(self: Self, status: StatusCode, desc_fn: F) -> Result<T, ErrorWithStatusAndDesc>
    where
        F: FnOnce() -> Cow<'static, str>;
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////

/*impl<T, E> WrapErrorWithStatusAndDesc<T> for Result<T, E>
where
    E: StdError + Send + Sync + 'static,
{
    /// Оборачиваем ошибку в 500й статус
    fn wrap_err_with_500(self: Self) -> Result<T, ErrorWithStatusAndDesc> {
        self.map_err(|e| eyre::Error::new(e))
            .map_err(|e| ErrorWithStatusAndDesc::from_error_with_status_desc(e, StatusCode::INTERNAL_SERVER_ERROR, "Internal error".into()))
    }

    /// Оборачиваем ошибку в конкретный статус
    fn wrap_err_with_status(self: Self, status: StatusCode) -> Result<T, ErrorWithStatusAndDesc> {
        self.map_err(|e| eyre::Error::new(e))
            .map_err(|e| ErrorWithStatusAndDesc::from_error_with_status_desc(e, status, "".into()))
    }

    /// Оборачиваем ошибку в статус и описание
    fn wrap_err_with_status_desc(self: Self, status: StatusCode, desc: Cow<'static, str>) -> Result<T, ErrorWithStatusAndDesc> {
        self.map_err(|e| eyre::Error::new(e))
            .map_err(|e| ErrorWithStatusAndDesc::from_error_with_status_desc(e, status, desc.into()))
    }

    /// Оборачиваем ошибку в статус и описание (отложенное)
    fn wrap_err_with_status_fn_desc<F>(self: Self, status: StatusCode, desc_fn: F) -> Result<T, ErrorWithStatusAndDesc>
    where
        F: FnOnce() -> Cow<'static, str>,
    {
        self.map_err(|e| eyre::Error::new(e)).map_err(|e| {
            let desc = desc_fn();
            ErrorWithStatusAndDesc::from_error_with_status_desc(e, status, desc.into())
        })
    }
}*/

/////////////////////////////////////////////////////////////////////////////////////////////////////////

impl<T, E> WrapErrorWithStatusAndDesc<T> for Result<T, E>
where
    eyre::Error: From<E>, // Альтернативных синтаксис
                          //E: Into<eyre::Error>,
{
    /// Оборачиваем ошибку в 500й статус
    fn wrap_err_with_500(self: Self) -> Result<T, ErrorWithStatusAndDesc> {
        self.map_err(|e| eyre::Error::from(e))
            .map_err(|e| ErrorWithStatusAndDesc::from_error_with_status_desc(e, StatusCode::INTERNAL_SERVER_ERROR, "Internal error".into()))
    }

    /// Оборачиваем ошибку в конкретный статус
    fn wrap_err_with_status(self: Self, status: StatusCode) -> Result<T, ErrorWithStatusAndDesc> {
        self.map_err(|e| eyre::Error::from(e))
            .map_err(|e| ErrorWithStatusAndDesc::from_error_with_status_desc(e, status, "".into()))
    }

    /// Оборачиваем ошибку в статус и описание
    fn wrap_err_with_status_desc(self: Self, status: StatusCode, desc: Cow<'static, str>) -> Result<T, ErrorWithStatusAndDesc> {
        self.map_err(|e| eyre::Error::from(e))
            .map_err(|e| ErrorWithStatusAndDesc::from_error_with_status_desc(e, status, desc.into()))
    }

    /// Оборачиваем ошибку в статус и описание (отложенное)
    fn wrap_err_with_status_fn_desc<F>(self: Self, status: StatusCode, desc_fn: F) -> Result<T, ErrorWithStatusAndDesc>
    where
        F: FnOnce() -> Cow<'static, str>,
    {
        self.map_err(|e| eyre::Error::from(e)).map_err(|e| {
            let desc = desc_fn();
            ErrorWithStatusAndDesc::from_error_with_status_desc(e, status, desc.into())
        })
    }
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////

impl<T> WrapErrorWithStatusAndDesc<T> for Option<T> {
    /// Оборачиваем ошибку в 500й статус
    fn wrap_err_with_500(self: Self) -> Result<T, ErrorWithStatusAndDesc> {
        self.ok_or_else(|| eyre::eyre!("Option is None"))
            .map_err(|e| ErrorWithStatusAndDesc::from_error_with_status_desc(e, StatusCode::INTERNAL_SERVER_ERROR, "Internal error".into()))
    }

    /// Оборачиваем ошибку в конкретный статус
    fn wrap_err_with_status(self: Self, status: StatusCode) -> Result<T, ErrorWithStatusAndDesc> {
        self.ok_or_else(|| eyre::eyre!("Option is None"))
            .map_err(|e| ErrorWithStatusAndDesc::from_error_with_status_desc(e, status, "".into()))
    }

    /// Оборачиваем ошибку в статус и описание
    fn wrap_err_with_status_desc(self: Self, status: StatusCode, desc: Cow<'static, str>) -> Result<T, ErrorWithStatusAndDesc> {
        self.ok_or_else(|| eyre::eyre!("Option is None"))
            .map_err(|e| ErrorWithStatusAndDesc::from_error_with_status_desc(e, status, desc.into()))
    }

    /// Оборачиваем ошибку в статус и описание (отложенное)
    fn wrap_err_with_status_fn_desc<F>(self: Self, status: StatusCode, desc_fn: F) -> Result<T, ErrorWithStatusAndDesc>
    where
        F: FnOnce() -> Cow<'static, str>,
    {
        self.ok_or_else(|| eyre::eyre!("Option is None")).map_err(|e| {
            let desc = desc_fn();
            ErrorWithStatusAndDesc::from_error_with_status_desc(e, status, desc.into())
        })
    }
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ErrorWithStatusAndDesc {
    // Время жизни распространяется лишь на ссылки в подтипе, они должны иметь время жизни 'static
    // На обычные переменные не распространяется
    source: Option<eyre::Error>,
    pub status: StatusCode,
    pub desc: Cow<'static, str>,
}
impl ErrorWithStatusAndDesc {
    pub fn from_error_with_status_desc(e: eyre::Error, status: StatusCode, desc: Cow<'static, str>) -> Self {
        ErrorWithStatusAndDesc {
            source: Some(e),
            status,
            desc: desc,
        }
    }

    pub fn new_with_status_desc(status: StatusCode, desc: Cow<'static, str>) -> Self {
        ErrorWithStatusAndDesc {
            source: None,
            status,
            desc: desc,
        }
    }
}
impl Display for ErrorWithStatusAndDesc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(source) = self.source.as_ref() {
            write!(f, "Status: {}, Description: {}, Source: {}", self.status, self.desc, source)
        } else {
            write!(f, "Status: {}, Description: {}", self.status, self.desc)
        }
    }
}
impl StdError for ErrorWithStatusAndDesc {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.source.as_ref().map(|e| e.as_ref() as &(dyn StdError + 'static))
    }
}
