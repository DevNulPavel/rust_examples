////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, thiserror::Error)]
enum ParseHostError {
    #[error("int parsing -> {0}")]
    PortParsing(#[from] std::num::ParseIntError),
}

////////////////////////////////////////////////////////////////////////////////

/// Структура с результатами парсинга строки конфига с хостом
pub(super) struct ParsedHost<'a> {
    pub(super) host: &'a str,
    pub(super) port: Option<u16>,
}

impl<'a> ParsedHost<'a> {
    pub(super) fn try_from_str(host_str: &'a str) -> Result<ParsedHost<'a>, ParseHostError> {
        ParsedHost::try_from(host_str)
    }
}

// TODO: Нельзя использовать FromStr из-за лайфтамов, которые трейт не поддерживает
impl<'a> TryFrom<&'a str> for ParsedHost<'a> {
    type Error = ParseHostError;

    fn try_from(host_str: &'a str) -> Result<ParsedHost<'a>, ParseHostError> {
        let res = if let Some((host_part, port_part)) = host_str.split_once(':') {
            let port = port_part.parse().ok();
            ParsedHost {
                host: host_part,
                port,
            }
        } else {
            ParsedHost {
                host: host_str,
                port: None,
            }
        };

        Ok(res)
    }
}
