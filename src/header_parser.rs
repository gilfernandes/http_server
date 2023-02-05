use nom::bytes::complete::take_while;
use nom::bytes::streaming::tag;
use nom::character::is_space;
use nom::IResult;
use nom::sequence::tuple;

use crate::http_parser::token;

#[derive(PartialEq, Eq, Debug)]
pub(crate) struct Header {
    pub(crate) name: Vec<u8>,
    pub(crate) value: Vec<u8>,
}

fn crlf(i: &[u8]) -> IResult<&[u8], &[u8]> {
    tag("\r\n")(i)
}

fn is_header_value_char(i: u8) -> bool {
    i == 9 || (i >= 32 && i <= 126) || i >= 160
}

// #[cfg(feature = "tolerant http1-parser")]
pub(crate) fn message_header(i: &[u8]) -> IResult<&[u8], Header> {
    let (i, (name, _, _, value, _)) = tuple((
        token,
        tag(":"),
        take_while(is_space),
        take_while(is_header_value_char),
        crlf,
    ))(i)?;

    Ok((i,
        Header { name: name.to_owned(), value: value.to_owned() }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn when_message_header_should_produce_header() {
        let header = "Accept: text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8";
        let bytes = header.as_bytes();
        let result = message_header(bytes);
        match result {
            Ok(header) => {
                assert_eq!(header.1.name, "Accept".to_string().into_bytes());
                assert_eq!(header.1.value, "Accept: text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8".to_string().into_bytes())
            }
            Err(_) => {}
        }
    }
}