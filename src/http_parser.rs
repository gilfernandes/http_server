use std::{
    {fmt, str},
};

use nom::{
    bytes::streaming::{tag, take_while},
    character::streaming::one_of,
    IResult,
};
use nom::character::is_alphanumeric;
use crate::http_parser::AuthMethod::Basic;

// Primitives

fn is_token_char(i: u8) -> bool {
    is_alphanumeric(i) || b"!#$%&'*+-.^_`|~=".contains(&i)
}

pub(crate) fn token(i: &[u8]) -> IResult<&[u8], &[u8]> {
    take_while(is_token_char)(i)
}

fn space(i: &[u8]) -> IResult<&[u8], char> {
    nom::character::streaming::char(' ')(i)
}

fn is_vchar(i: u8) -> bool {
    i > 32 && i < 126
}

fn vchar_i(i: &[u8]) -> IResult<&[u8], &[u8]> {
    take_while(is_vchar)(i)
}

// fn crlf(i: &[u8]) -> IResult<&[u8], &[u8]> {
//     tag("\r\n")(i)
// }

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Method {
    Get,
    Head,
    Options,
    Custom(String),
}

impl Method {
    pub fn new(s: &[u8]) -> Method {
        if compare_no_case(s, b"GET") {
            Method::Get
        } else if compare_no_case(s, b"HEAD") {
            Method::Head
        } else if compare_no_case(s, b"OPTIONS") {
            Method::Options
        }else {
            Method::Custom(String::from(unsafe { str::from_utf8_unchecked(s) }))
        }
    }
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Method::Get => write!(f, "GET"),
            Method::Head => write!(f, "HEAD"),
            Method::Options => write!(f, "OPTIONS"),
            Method::Custom(s) => write!(f, "{}", s),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum Version {
    V10,
    V11,
}

pub fn compare_no_case(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    left.iter().zip(right).all(|(a, b)| match (*a, *b) {
        (0..=64, 0..=64) | (91..=96, 91..=96) | (123..=255, 123..=255) => a == b,
        (65..=90, 65..=90) | (97..=122, 97..=122) | (65..=90, 97..=122) | (97..=122, 65..=90) => {
            *a | 0b00_10_00_00 == *b | 0b00_10_00_00
        }
        _ => false,
    })
}

#[derive(PartialEq, Eq, Debug)]
pub struct RawRequestLine<'a> {
    pub method: &'a [u8],
    pub uri: &'a [u8],
    pub version: Version,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct RequestLine {
    pub method: Method,
    pub uri: String,
    pub version: Version,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum AuthMethod {
    Basic
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Authentication {
    pub method: AuthMethod,
    pub encoded_credentials: String
}

impl RequestLine {
    pub fn from_raw_request(r: RawRequestLine) -> Option<RequestLine> {
        if let Ok(uri) = str::from_utf8(r.uri) {
            Some(RequestLine {
                method: Method::new(r.method),
                uri: String::from(uri),
                version: r.version,
            })
        } else { None }
    }
}

fn http_version(i: &[u8]) -> IResult<&[u8], Version> {
    let (i, _) = tag("HTTP/1.")(i)?;
    let (i, minor) = one_of("01")(i)?;
    Ok((
        i,
        if minor == '0' {
            Version::V10
        } else {
            Version::V11
        }
    ))
}

/// Parse first line into RawRequestLine
pub fn request_line(i: &[u8]) -> IResult<&[u8], Option<RequestLine>> {
    let (i, method) = token(i)?;
    let (i, _) = space(i)?;
    let (i, uri) = vchar_i(i)?;
    let (i, _) = space(i)?;
    let (i, version) = http_version(i)?;

    let raw_request_line = RawRequestLine { method, uri, version };

    Ok((
        i,
        RequestLine::from_raw_request(raw_request_line)
    ))
}

pub fn basic_authorization_header(i: &[u8]) -> IResult<&[u8], Option<Authentication>> {
    let (i, _) = tag("Authorization:")(i)?;
    let (i, _) = space(i)?;
    let (i, _) = tag("Basic")(i)?;
    let (i, _) = space(i)?;
    let (i, credentials) = token(i)?;

    Ok((
        i,
        if let Ok(encoded_credentials) = str::from_utf8(credentials) {
            Some(Authentication {
                encoded_credentials: encoded_credentials.to_string(),
                method: Basic
            })
        } else { None }
    ))
}

#[cfg(test)]
mod tests {
    use nom::Finish;
    use super::*;

    #[test]
    fn when_is_basic_auth_header_should_succeed() {
        let header = "Authorization: Basic QWxhZGRpbjpvcGVuIHNlc2FtZQ==\r\n";
        let res = basic_authorization_header(header.as_bytes());
        assert!(res.is_ok());
        let (_, option) = res.unwrap();
        assert!(option.is_some());
        let authentication = option.unwrap();
        assert_eq!(authentication.encoded_credentials, "QWxhZGRpbjpvcGVuIHNlc2FtZQ==");
    }

    #[test]
    fn when_is_basic_auth_header_should_err() {
        let header = "accept-language: en-GB,en;q=0.9,en-US;q=0.8\r\n";
        let res = basic_authorization_header(header.as_bytes());
        assert!(res.is_err());
    } }


