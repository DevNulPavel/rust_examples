use nom::{error::Error as NomError, Err as NomErr};
// use nom::error::VerboseError as NomVerboseError;

////////////////////////////////////////////////////////////////////////////////////////////////

// #[derive(thiserror::Error, Debug, PartialEq)]
// pub enum NomInternalError<'a> {
//     #[error("Simple error: {0}")]
//     Simple(NomErr<NomError<&'a str>>),

//     #[error("Verbose error: {0}")]
//     Verbose(NomErr<NomVerboseError<&'a str>>),
// }

// Реализуем руками from из-за того, что при макросе #[from] в thiserror
// у нас source ошибка может содержать лишь 'static ссылки

// impl<'a> From<NomErr<NomError<&'a str>>> for NomInternalError<'a> {
//     fn from(err: NomErr<NomError<&'a str>>) -> Self {
//         NomInternalError::Simple(err)
//     }
// }

// impl<'a> From<NomErr<NomVerboseError<&'a str>>> for NomInternalError<'a> {
//     fn from(err: NomErr<NomVerboseError<&'a str>>) -> Self {
//         NomInternalError::Verbose(err)
//     }
// }

////////////////////////////////////////////////////////////////////////////////////////////////

// Определяем наш собственный тип ошибки для парсера, но с подробным описанием ошибки
#[derive(thiserror::Error, Debug, PartialEq)]
pub enum ParsingError<'a> {
    // #[error("Invalid scheme `{0}`")]
    // InvalidScheme(&'a str),

    #[error("Parsing fail with `{0}`")]
    ParsingFailed(NomErr<NomError<&'a str>>),
}

// Реализуем руками from из-за того, что при макросе #[from] в thiserror
// у нас source ошибка может содержать лишь 'static ссылки
impl<'a> From<NomErr<NomError<&'a str>>> for ParsingError<'a> {
    fn from(err: NomErr<NomError<&'a str>>) -> Self {
        ParsingError::ParsingFailed(err)
    }
}
