use std::net::TcpStream;

use crate::MimeTypeProperties;

pub(crate) struct HttpData<'a> {
    pub(crate) stream: &'a mut TcpStream,
    pub(crate) uri: String,
    pub(crate) mime_type_map: &'a MimeTypeProperties,
    pub(crate) is_head : &'a bool,
    pub(crate) root_folder: &'a String
}