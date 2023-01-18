use std::{
    {fmt, str},
};

use nom::{
    bytes::streaming::{tag, take_while},
    character::streaming::one_of,
    IResult,
};
use nom::character::is_alphanumeric;

// Primitives

fn is_token_char(i: u8) -> bool {
    is_alphanumeric(i) || b"!#$%&'*+-.^_`|~".contains(&i)
}

fn token(i: &[u8]) -> IResult<&[u8], &[u8]> {
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
    Custom(String),
}

impl Method {
    pub fn new(s: &[u8]) -> Method {
        if compare_no_case(s, b"GET") {
            Method::Get
        } else {
            Method::Custom(String::from(unsafe { str::from_utf8_unchecked(s) }))
        }
    }
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Method::Get => write!(f, "GET"),
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

    let raw_request_line = RawRequestLine {
        method,
        uri,
        version,
    };

    Ok((
        i,
        RequestLine::from_raw_request(raw_request_line)
    ))
}


