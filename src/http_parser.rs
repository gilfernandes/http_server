use std::{
    {fmt, str},
};

use base64::{Engine as _, engine::general_purpose};
use nom::{
    bytes::streaming::{tag, take, take_while},
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

pub(crate) fn token_eof(i: &[u8]) -> IResult<&[u8], &[u8]> {
    take(i.len())(i)
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
        } else {
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
    pub encoded_credentials: String,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct BasicCredentials {
    pub username: String,
    pub password: String,
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
    println!("=== '{}'", str::from_utf8(i).unwrap());
    let (i, _) = tag("Authorization:")(i)?;
    let (i, _) = space(i)?;
    let (i, _) = tag("Basic")(i)?;
    let (i, _) = space(i)?;
    let (i, credentials) = token_eof(i)?;

    Ok((
        i,
        if let Ok(encoded_credentials) = str::from_utf8(credentials) {
            Some(Authentication {
                encoded_credentials: encoded_credentials.to_string(),
                method: Basic,
            })
        } else { None }
    ))
}

pub fn find_basic_authorization_header(http_request: Vec<String>) -> Option<Authentication> {
    for header in http_request.iter() {
        if let Ok(auth_option) = basic_authorization_header(header.as_bytes()) {
            if let Some(auth) = auth_option.1 {
                return Some(auth);
            }
        }
    }
    return None;
}

pub fn decode_user_name_password(authentication: &Authentication) -> Option<BasicCredentials> {
    let encoded_credentials = &authentication.encoded_credentials;
    let res = &general_purpose::STANDARD.decode(encoded_credentials).ok()?;
    let decoded = str::from_utf8(res).ok()?;
    let splits = decoded.split(":");
    let splits_vec = splits.collect::<Vec<&str>>();
    Some(
        BasicCredentials {
            username: splits_vec[0].to_string(),
            password: splits_vec[1].to_string()
        }
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn when_is_basic_auth_header_should_succeed() {
        let header = "Authorization: Basic QWxhZGRpbjpvcGVuIHNlc2FtZQ==";
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
    }

    #[test]
    fn when_find_basic_authorization_header_should_find_header() {
        let headers_str = provide_headers("Authorization: Basic QWxhZGRpbjpvcGVuIHNlc2FtZQ==\r\n".to_string());
        let strings = split_to_vec(headers_str.as_str());
        let found = find_basic_authorization_header(strings);
        assert!(found.is_some());
        let authentication = found.unwrap();
        assert_eq!(authentication.encoded_credentials, "QWxhZGRpbjpvcGVuIHNlc2FtZQ==");
    }

    #[test]
    fn when_find_basic_authorization_header_should_not_find_header() {
        let headers_str = provide_headers("".to_string());
        let strings = split_to_vec(headers_str.as_str());
        let found = find_basic_authorization_header(strings);
        assert!(found.is_none())
    }

    fn provide_headers(extra_header: String) -> String {
        let headers_str = format!("GET /favicon.ico HTTP/1.1\r\n\
Host: localhost:7879\r\n\
Connection: keep-alive\r\n\
sec-ch-ua: \"Google Chrome\";v=\"111\", \"Not(A:Brand\";v=\"8\", \"Chromium\";v=\"111\"\r\n\
sec-ch-ua-mobile: ?0\r\n\
User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/111.0.0.0 Safari/537.36\r\n\
sec-ch-ua-platform: \"Windows\"\r\n\
Accept: image/avif,image/webp,image/apng,image/svg+xml,image/*,*/*;q=0.8\r\n\
Sec-Fetch-Site: same-origin\r\n\
Sec-Fetch-Mode: no-cors\r\n\
Sec-Fetch-Dest: image\r\n\
Referer: http://localhost:7879/\r\n\
Accept-Encoding: gzip, deflate, br\r\n\
Accept-Language: en-GB,en;q=0.9,en-US;q=0.8\r\n\
{}Cookie: Idea-34adbe16=ae2204d1-dba5-490c-adae-34e47f4bf087; _xsrf=2|17f5ef96|d61a2b35aeab101730fd39f8a33cf464|1678705197; username-localhost-8888=\"2|1:0|10:1678954766|23:username-localhost-8888|44:ZmExNDc2ZDA1ZTg2NDk5YjhiNjZmNzM2ZTcxZWE2NmM=|1abf8d69b057d953d3755cd90273b2e962e40ff295174aa65b3da8c86ffae56d\"\r\n
", extra_header);
        return headers_str;
    }

    #[test]
    fn when_decode_user_name_password_should_decode () {
        let authentication = Authentication {
            method: Basic,
            encoded_credentials: "QWxhZGRpbjpvcGVuIHNlc2FtZQ==".to_string()
        };
        let cred_option = decode_user_name_password(&authentication);
        assert!(cred_option.is_some());
        let creds = cred_option.unwrap();
        assert_eq!(creds.username, "Aladdin".to_string());
        assert_eq!(creds.password, "open sesame".to_string());
    }

    fn split_to_vec(headers_str: &str) -> Vec<String> {
        let splits = headers_str.split("\r\n");
        let splits_vec = splits.collect::<Vec<&str>>();
        assert!(splits_vec.len() > 0);
        splits_vec.iter().map(|s| s.to_string()).collect::<Vec<String>>()
    }
}


