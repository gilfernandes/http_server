use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};

use crate::http_parser::request_line;
use crate::mime_type_map::{extract_mime_type, generate_mimetype_maps, MimeTypeProperties, TEXT_HTML};
use crate::string_operations::replace_slash;

mod http_parser;
mod mime_type_map;
mod string_operations;

const STATUS_OK: &'static str = "HTTP/1.1 200 OK";
const STATUS_BAD_REQUEST: &'static str = "HTTP/1.1 400 Bad Request";
const STATUS_NOT_FOUND: &'static str = "HTTP/1.1 404 Not Found";

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let mime_map = generate_mimetype_maps();

    for stream_result in listener.incoming() {
        let stream = stream_result.unwrap();

        handle_connection(stream, &mime_map);
        println!("Connection established");
    }

    fn handle_connection(mut stream: TcpStream, mime_map: &HashMap<String, MimeTypeProperties>) {
        let buf_reader = BufReader::new(&mut stream);
        let http_request: Vec<_> = buf_reader
            .lines()
            .map(|result| result.unwrap())
            .take_while(|line| !line.is_empty())
            .collect();

        if http_request.is_empty() {
            send_bad_request(&mut stream);
            return;
        }
        let rl = http_request[0].clone();
        let (_, request_line_option) = request_line(rl.as_bytes()).unwrap();
        println!("Request: {:#?}", rl);
        match request_line_option {
            Some(request_line_content) => {
                let uri = replace_slash(request_line_content.uri);
                let mime_type_map = extract_mime_type(uri.as_str(),
                                                                &mime_map);
                println!("Requested resource: {:#?}. Mime type: {}", uri, mime_type_map.content_type);
                if mime_type_map.binary {
                    process_binary_content(&mut stream, uri, mime_type_map.content_type);
                }
                else {
                    process_text_content(&mut stream, uri, mime_type_map.content_type);
                }
            }
            None => {
                send_bad_request(&mut stream);
            }
        }
    }

    fn process_text_content(mut stream: &mut TcpStream, uri: String, mime_type: String) {
        let res = fs::read_to_string(format!("./root/{}", uri).as_str());
        match res {
            Ok(contents) => {
                stream_text(&mut stream, STATUS_OK, contents.as_str()
                            , mime_type.as_str());
            }
            Err(_) => {
                not_found(&mut stream, mime_type);
            }
        }
    }

    fn not_found(mut stream: &mut &mut TcpStream, mime_type: String) {
        let contents = fs::read_to_string("./root/not_found.html")
            .expect("Cannot find bad request html.");
        stream_text(&mut stream, STATUS_NOT_FOUND, contents.as_str()
                    , mime_type.as_str());
    }

    fn process_binary_content(mut stream: &mut TcpStream, uri: String, mime_type: String) {
        let res = fs::read(format!("./root/{}", uri).as_str());
        match res {
            Ok(content) => {
                let bytes = &content[..];
                let length = bytes.len();
                let (status_line, content_length, content_type) =
                    generate_status_headers(STATUS_OK, length, mime_type.as_str());
                let response = format!("{status_line}{content_length}{content_type}\r\n");
                let header_bytes = response.as_bytes();
                let concat_vec = [header_bytes, bytes].concat();
                let concat_bytes = &concat_vec[..];
                stream.write_all(concat_bytes).unwrap();
            }
            Err(_) => {
                not_found(&mut stream, mime_type);
            }
        }
    }
}

fn send_bad_request(mut stream: &mut TcpStream) {
    let contents = fs::read_to_string("./root/bad_request.html")
        .expect("Cannot find bad request html.");
    stream_text(&mut stream, STATUS_BAD_REQUEST, contents.as_str(),
                TEXT_HTML);
}

fn stream_text(stream: &mut TcpStream,
               status_line: &str,
               contents: &str,
               mime_type: &str
) {
    let length = contents.len();
    let (status_line, content_length, content_type) = generate_status_headers(status_line, length, mime_type);
    let response = format!("{status_line}{content_length}{content_type}\r\n{contents}");
    let bytes = response.as_bytes();
    stream.write_all(bytes).unwrap();
}

fn generate_status_headers(status_line: &str, length: usize, mime_type: &str) -> (String, String, String) {
    let status_line = format!("{status_line}\r\n");
    let content_length = format!("Content-Length: {length}\r\n");
    let content_type = format!("Content-Type: {mime_type}; charset=utf-8\r\n");
    return (status_line, content_length, content_type);
}
