use eyre::WrapErr;
use hyper::{body::Body as BodyStruct, http::header, Response};
use mime::Mime;

pub fn get_content_length(response: &Response<BodyStruct>) -> Result<Option<usize>, eyre::Error> {
    let content_length: Option<usize> = match response.headers().get(header::CONTENT_LENGTH) {
        Some(val) => {
            let num = val
                .to_str()
                .wrap_err("Content-Length string convert failed")?
                .parse::<usize>()
                .wrap_err("Content Length parse failed")?;
            Some(num)
        }
        None => None,
    };
    Ok(content_length)
}

pub fn get_content_type(response: &Response<BodyStruct>) -> Result<Option<Mime>, eyre::Error> {
    let header_val = match response.headers().get(header::CONTENT_TYPE) {
        Some(val) => val,
        None => return Ok(None),
    };
    let content_type_mime: Mime = header_val
        .to_str()
        .wrap_err("Content type header to string convert failed")?
        .parse()
        .wrap_err("Mime parse failed")?;
    Ok(Some(content_type_mime))
}
