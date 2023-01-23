use std::{env::var, fs, io};
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};

use lazy_static::lazy_static;
use linked_hash_map::LinkedHashMap;

use http_server::ThreadPool;

use crate::http_parser::{Method, request_line};
use crate::mime_type_map::{extract_mime_type, MimeTypeProperties, TEXT_HTML};
use crate::string_operations::{extract_file_name, remove_double_slash, replace_slash};

mod http_parser;
mod mime_type_map;
mod string_operations;

const STATUS_OK: &'static str = "HTTP/1.1 200 OK";
const STATUS_BAD_REQUEST: &'static str = "HTTP/1.1 400 Bad Request";
const STATUS_NOT_FOUND: &'static str = "HTTP/1.1 404 Not Found";
const STATUS_METHOD_NOT_ALLOWED: &'static str = "HTTP/1.1 405 Method Not Allowed";
const SERVER_NAME: &'static str = "Gil HTTP";

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

        for header in http_request.iter() {
            println!(":: {:#?}", header);
        }

        match request_line_option {
            Some(request_line_content) => {
                match request_line_content.method {
                    Method::Get | Method::Head => {
                        let uri = replace_slash(request_line_content.uri);
                        let mime_type_map = extract_mime_type(uri.as_str());
                        println!("Requested resource: {:#?}. Mime type: {}", uri, mime_type_map.content_type);
                        let is_head = request_line_content.method == Method::Head;
                        if mime_type_map.binary {
                            process_binary_content(&mut stream, uri, &mime_type_map, &is_head);
                        } else {
                            process_text_content(&mut stream, uri, mime_type_map.content_type, &is_head);
                        }
                    }
                    _ => {
                        send_error_response(&mut stream, "method_not_allowed.html", STATUS_METHOD_NOT_ALLOWED);
                    }
                }
            }
            None => {
                send_bad_request(&mut stream);
            }
        }
    }
}

fn process_text_content(mut stream: &mut TcpStream, uri: String, mime_type: String, is_head: &bool) {
    let path = build_path(uri);
    let res = fs::read_to_string(path);

    match res {
        Ok(contents) => {
            stream_text(&mut stream,
                        STATUS_OK,
                        contents.as_str(), mime_type.as_str(),
                        is_head);
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

fn send_error_response(mut stream: &mut TcpStream, html_file: &str, status: &str) {
    let request_file = format!("./{}/{html_file}", *ROOT_FOLDER);
    let result_file = File::open(request_file);
    match result_file {
        Ok(file) => {
            let reader = BufReader::new(file);
            let buf_result = io::read_to_string(reader);
            match buf_result {
                Ok(contents) => {
                    stream_text(&mut stream, status, contents.as_str(), TEXT_HTML, &false);
                }
                Err(e) => {
                    println!("Cannot find file {html_file}: {:?}", e);
                }
            }
        }
        Err(e) => {
            println!("Cannot find file {html_file}: {:?}", e);
        }
    }
}

fn not_found(mut stream: &mut &mut TcpStream) {
    send_error_response(&mut stream, "not_found.html", STATUS_NOT_FOUND);
}

fn process_binary_content(mut stream: &mut TcpStream, uri: String, mime_type_properties: &MimeTypeProperties,
                          is_head: &bool) {
    let res = fs::read(format!("./{}/{}", *ROOT_FOLDER, uri).as_str());
    let mime_type = &mime_type_properties.content_type;

    match res {
        Ok(content) => {
            let bytes = &content[..];
            let length = bytes.len();
            let header_map =
                generate_status_headers(STATUS_OK, length, mime_type.as_str());
            let response = generate_binary_status_line(
                uri,
                header_map,
                mime_type_properties,
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

fn generate_binary_status_line(uri: String, header_map: LinkedHashMap<String, String>,
                               mime_type_properties: &MimeTypeProperties) -> String {
    let attachment = &mime_type_properties.attachment;
    let concatenated_headers_str = concatenate_headers(&header_map);
    if *attachment {
        let file_name = extract_file_name(uri);
        let content_disposition = format!("Content-Disposition: attachment; filename=\"{file_name}\"\r\n");
        format!("{concatenated_headers_str}{content_disposition}\r\n").to_string()
    } else {
        format!("{concatenated_headers_str}\r\n").to_string()
    }
}


fn send_bad_request(mut stream: &mut TcpStream) {
    let bad_request = format!("./{}/bad_request.html", *ROOT_FOLDER);
    let contents = fs::read_to_string(bad_request.as_str()).expect("Cannot find bad request html.");
    stream_text(&mut stream, STATUS_BAD_REQUEST, contents.as_str(), TEXT_HTML, &false);
}

fn stream_text(stream: &mut TcpStream,
               status_line: &str,
               contents: &str,
               mime_type: &str,
               is_head: &bool,
) {
    let length = contents.len();
    let header_map = generate_status_headers(status_line, length, mime_type);

    let concatenated_headers_str = concatenate_headers(&header_map);

    let response = if *is_head { format!("{concatenated_headers_str}\r\n") }
        else { format!("{concatenated_headers_str}\r\n{contents}") };
    let bytes = response.as_bytes();
    stream.write_all(bytes).unwrap();
}

fn concatenate_headers(header_map: &LinkedHashMap<String, String>) -> String {
    let header_vec = Vec::from_iter(header_map.values());
    return header_vec.iter()
        .map(|x| (*x).to_string())
        .collect::<Vec<_>>().join("");
}

fn generate_status_headers(status_line: &str, length: usize, mime_type: &str) -> LinkedHashMap<String, String> {
    let status_line = format!("{status_line}\r\n");
    let content_length = format!("Content-Length: {length}\r\n");
    let content_type = format!("Content-Type: {mime_type}; charset=utf-8\r\n");
    let cache_control = format!("Cache-Control: public, max-age=120\r\n");
    let server = format!("Server: {SERVER_NAME}\r\n");

    let mut status_headers_map = LinkedHashMap::new();
    status_headers_map.insert(String::from("status_line"), status_line);
    status_headers_map.insert(String::from("content_length"), content_length);
    status_headers_map.insert(String::from("content_type"), content_type);
    status_headers_map.insert(String::from("cache_control"), cache_control);
    status_headers_map.insert(String::from("server"), server);

    return status_headers_map.clone();
}

