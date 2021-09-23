use hyper::StatusCode;
use std::{borrow::Cow, error::Error as StdError, fmt::Display};

/////////////////////////////////////////////////////////////////////////////////////////////////////////

pub trait WrapErrorWithStatusAndDesc<T> {
    fn wrap_err_with_status(self: Self, status: StatusCode, desc: Cow<'static, str>) -> Result<T, eyre::Error>;
    fn wrap_err_with_status_fn_desc<F>(self: Self, status: StatusCode, f: F) -> Result<T, eyre::Error>
    where
        F: FnOnce() -> Cow<'static, str>;
}

impl<T, E> WrapErrorWithStatusAndDesc<T> for Result<T, E>
where
    E: StdError + Send + Sync + 'static,
{
    fn wrap_err_with_status(self: Self, status: StatusCode, desc: Cow<'static, str>) -> Result<T, eyre::Error> {
        self.map_err(|e| ErrorWithStatusAndDesc::from_error(e, status, desc.into()))
            .map_err(|e| eyre::Error::new(e))
    }

    fn wrap_err_with_status_fn_desc<F>(self: Self, status: StatusCode, f: F) -> Result<T, eyre::Error>
    where
        F: FnOnce() -> Cow<'static, str>,
    {
        self.map_err(|e| {
            let desc = f();
            ErrorWithStatusAndDesc::from_error(e, status, desc.into())
        })
        .map_err(|e| eyre::Error::new(e))
    }
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ErrorWithStatusAndDesc {
    // Время жизни распространяется лишь на ссылки в подтипе, они должны иметь время жизни 'static
    // На обычные переменные не распространяется
    source: Option<Box<dyn StdError + Send + Sync + 'static>>,
    pub status: StatusCode,
    pub desc: Cow<'static, str>,
}
impl ErrorWithStatusAndDesc {
    pub fn from_error<E: StdError + Send + Sync + 'static>(e: E, status: StatusCode, desc: Cow<'static, str>) -> Self {
        ErrorWithStatusAndDesc {
            source: Some(Box::new(e)),
            status,
            desc: desc,
        }
    }

    pub fn new(status: StatusCode, desc: Cow<'static, str>) -> Self {
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
            writeln!(f, "Status: {}, Description: {}, Source: {}", self.status, self.desc, source)
        } else {
            writeln!(f, "Status: {}, Description: {}", self.status, self.desc)
        }
    }
}
impl StdError for ErrorWithStatusAndDesc {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.source.as_ref().map(|e| e.as_ref() as &(dyn StdError + 'static))
    }
}
