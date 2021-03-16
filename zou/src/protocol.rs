use hyper::Url;

/// Supported protocols
#[derive(Debug)]
pub enum Protocol {
    HTTP,
    HTTPS,
}

/// Возвращает тип Option, который содержит Protocol enum
pub fn get_protocol(url: &str) -> Option<Protocol> {
    match Url::parse(url) {
        Ok(url) => match url.scheme() {
            "http" => Some(Protocol::HTTP),
            "https" => Some(Protocol::HTTPS),
            _ => None,
        },
        Err(error) => {
            warning!(&format!("Error extracting the protocol: {}", error));
            None
        }
    }
}