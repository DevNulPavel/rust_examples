use super::error::ParsingError;
use nom::{bytes::complete::tag, character::complete::alphanumeric1, error::context, sequence::terminated};

#[derive(Debug, PartialEq, Eq)]
pub enum Scheme<'a> {
    Http,
    Https,
    Custom(&'a str),
}

impl<'a> From<&'a str> for Scheme<'a> {
    fn from(scheme: &'a str) -> Self {
        match scheme {
            "http" => Scheme::Http,
            "https" => Scheme::Https,
            scheme => Scheme::Custom(scheme),
        }
    }
}

impl<'a> Scheme<'a> {
    pub fn try_parse(text: &str) -> Result<(&str, Scheme), ParsingError> {
        // Пишем комбинатор с контекстом
        let mut https_func = context("scheme", terminated(alphanumeric1, tag("://")));

        // Парсим ввод, затем пытаемся распарсить результат
        let (remind, result) = https_func(text)?;

        // Теперь уже парсим схему из строки
        let scheme = Scheme::from(result);

        Ok((remind, scheme))
    }

    pub fn as_str(&self) -> &str {
        match self {
            Scheme::Http => "http",
            Scheme::Https => "https",
            Scheme::Custom(scheme) => scheme,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn https_scheme_parsing() {
        let (remind, scheme) = Scheme::try_parse("https://www.rust-lang.org/en-US/").unwrap();
        assert_eq!(remind, "www.rust-lang.org/en-US/");
        assert_eq!(scheme, Scheme::Https);
        assert_eq!(scheme.as_str(), "https");
    }

    #[test]
    fn http_scheme_parsing() {
        let (remind, scheme) = Scheme::try_parse("http://www.rust-lang.org/en-US/").unwrap();
        assert_eq!(remind, "www.rust-lang.org/en-US/");
        assert_eq!(scheme, Scheme::Http);
        assert_eq!(scheme.as_str(), "http");
    }

    #[test]
    fn error_scheme_parsing() {
        match Scheme::try_parse("blabla:/www.rust-lang.org/en-US/").unwrap_err() {
            ParsingError::ParsingFailed(nom::Err::Error(nom::error::Error::<_> { input, .. })) => {
                assert_eq!(input, ":/www.rust-lang.org/en-US/");
                // assert_eq!(text, "blabla://www.rust-lang.org/en-US/")
            }
            err => panic!("Unexpected error: {}", err),
        }
    }
}
