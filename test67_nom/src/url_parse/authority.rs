use super::error::ParsingError;
use nom::{
    bytes::complete::tag,
    character::complete::alphanumeric1,
    combinator::opt,
    error::context,
    sequence::{separated_pair, terminated},
};

#[derive(Debug)]
pub struct Authority<'a> {
    pub username: &'a str,
    pub password: Option<&'a str>,
}

impl<'a> Authority<'a> {
    pub fn try_parse(text: &str) -> Result<(&str, Option<Authority>), ParsingError> {
        // Пишем комбинатор с контекстом
        let mut parse_func = context(
            "authority",
            opt(terminated(separated_pair(alphanumeric1, tag(":"), opt(alphanumeric1)), tag("@"))),
        );

        // Парсим ввод, затем пытаемся распарсить результат
        let (remind, result) = parse_func(text)?;

        // Теперь уже парсим схему из строки
        let auth = result.map(|result| Authority {
            username: result.0,
            password: result.1,
        });

        Ok((remind, auth))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_authority() {
        let parse_res = Authority::try_parse("user:pass@www.google.com").unwrap();
        assert_eq!(parse_res.0, "www.google.com");

        let authority = parse_res.1.unwrap();
        assert_eq!(authority.username, "user");
        assert_eq!(authority.password, Some("pass"));
    }

    #[test]
    fn test_partial_authority() {
        let parse_res = Authority::try_parse("user:@www.google.com").unwrap();
        assert_eq!(parse_res.0, "www.google.com");

        let authority = parse_res.1.unwrap();
        assert_eq!(authority.username, "user");
        assert_eq!(authority.password, None);
    }

    #[test]
    fn test_empty_authority() {
        let parse_res = Authority::try_parse("www.google.com").unwrap();
        assert_eq!(parse_res.0, "www.google.com");
        assert!(parse_res.1.is_none());
    }

    #[test]
    fn test_invalid_authority() {
        // Из-за опциональности мы не падаем на данном правиле, падать будем потом при парсинге адреса хоста
        let parse_res = Authority::try_parse("qwe;13r2@www.google.com").unwrap();
        assert_eq!(parse_res.0, "qwe;13r2@www.google.com");
        assert!(parse_res.1.is_none());
    }
}
