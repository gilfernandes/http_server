use std::{env::var, fs};
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use http_server::ThreadPool;
use lazy_static::lazy_static;

use crate::http_parser::request_line;
use crate::mime_type_map::{extract_mime_type, MimeTypeProperties, TEXT_HTML};
use crate::string_operations::{extract_file_name, remove_double_slash, replace_slash};

mod http_parser;
mod mime_type_map;
mod string_operations;

const STATUS_OK: &'static str = "HTTP/1.1 200 OK";
const STATUS_BAD_REQUEST: &'static str = "HTTP/1.1 400 Bad Request";
const STATUS_NOT_FOUND: &'static str = "HTTP/1.1 404 Not Found";

lazy_static! {
    static ref ROOT_FOLDER: String = var("ROOT_FOLDER").unwrap_or("root".to_string());
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(10);

    for stream_result in listener.incoming() {
        let stream = stream_result.unwrap();
        pool.execute(|| {
            handle_connection(stream);
        });
        println!("Connection established");
    }

    println!("Shutting down");

    fn handle_connection(mut stream: TcpStream) {
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
                let mime_type_map = extract_mime_type(uri.as_str());
                println!("Requested resource: {:#?}. Mime type: {}", uri, mime_type_map.content_type);
                if mime_type_map.binary {
                    process_binary_content(&mut stream, uri, &mime_type_map);
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
        let path = build_path(uri);
        let res = fs::read_to_string(path);

        match res {
            Ok(contents) => {
                stream_text(&mut stream, STATUS_OK, contents.as_str()
                            , mime_type.as_str());
            }
            Err(_) => {
                not_found(&mut stream);
            }
        }
    }

    fn build_path(uri: String) -> String {
        let root_folder = ROOT_FOLDER.as_str().to_string();
        let path_str = if root_folder.starts_with("/") { format!("/{}/{}", root_folder, uri) } else {
            format!("./{}/{}", root_folder, uri)
        };
        let path = remove_double_slash(path_str.as_str());
        path
    }

    fn not_found(mut stream: &mut &mut TcpStream) {
        let not_found_file = format!("./{}/not_found.html", *ROOT_FOLDER);
        let contents = fs::read_to_string(not_found_file.as_str())
            .expect(format!("Cannot find {}", not_found_file).as_str());
        stream_text(&mut stream, STATUS_NOT_FOUND, contents.as_str()
                    , extract_mime_type(not_found_file.as_str()).content_type.as_str());
    }

    fn process_binary_content(mut stream: &mut TcpStream, uri: String, mime_type_properties: &MimeTypeProperties) {
        let res = fs::read(format!("./{}/{}", *ROOT_FOLDER, uri).as_str());
        let mime_type = &mime_type_properties.content_type;

        match res {
            Ok(content) => {
                let bytes = &content[..];
                let length = bytes.len();
                let (status_line, content_length, content_type) =
                    generate_status_headers(STATUS_OK, length, mime_type.as_str());
                let response = generate_binary_status_line(
                    uri,
                    status_line,
                    content_length,
                    content_type,
                    mime_type_properties
                );
                let header_bytes = response.as_bytes();
                let concat_vec = [header_bytes, bytes].concat();
                let concat_bytes = &concat_vec[..];
                stream.write_all(concat_bytes).unwrap();
            }
            Err(_) => {
                not_found(&mut stream);
            }
        }
    }

    fn generate_binary_status_line(uri: String, status_line: String, content_length: String, content_type: String,
                                   mime_type_properties: &MimeTypeProperties) -> String {
        let attachment = &mime_type_properties.attachment;
        if *attachment {
            let file_name = extract_file_name(uri);
            let content_disposition = format!("Content-Disposition: attachment; filename=\"{file_name}\"\r\n");
            format!("{status_line}{content_disposition}{content_length}{content_type}\r\n").to_string()
        } else {
            format!("{status_line}{content_length}{content_type}\r\n").to_string()
        }
    }
}

fn send_bad_request(mut stream: &mut TcpStream) {
    let bad_request = format!("./{}/bad_request.html", *ROOT_FOLDER);
    let contents = fs::read_to_string(bad_request.as_str())
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
