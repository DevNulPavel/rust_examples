pub mod error;
pub mod authority;
pub mod scheme;
pub mod host;

use self::{authority::Authority, host::Host, scheme::Scheme};
use eyre::{Context, WrapErr};
use nom::{
    branch::alt,
    bytes::complete::{tag_no_case, take_till},
    error::{context, make_error, Error, ErrorKind, VerboseError},
    IResult,
};

/////////////////////////////////////////////////////////////////////////////////


#[derive(Debug)]
struct QueryParam<'a> {
    name: &'a str,
    value: Option<&'a str>,
}

#[derive(Debug)]
struct QueryParams<'a> {
    params: Vec<QueryParam<'a>>,
}

#[derive(Debug)]
struct URI<'a> {
    scheme: Scheme<'a>,
    authority: Option<Authority<'a>>,
    host: Host<'a>,
    path: Option<Vec<&'a str>>,
    query: Option<QueryParams<'a>>,
    fragment: Option<&'a str>, // Символы после #
}

#[allow(dead_code)]
pub fn parse_url() -> Result<(), eyre::Error> {
    // let url = "https://www.rust-lang.org/en-US/";
    // let (input, url) = parse_url_str(url)?;
    // println!("{:?}", url);

    Ok(())
}
