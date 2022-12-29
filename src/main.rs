mod http_parser;
mod mime_type_map;

use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use crate::http_parser::request_line;
use crate::mime_type_map::generate_mimetype_maps;

const STATUS_OK: &'static str = "HTTP/1.1 200 OK";
const STATUS_BAD_REQUEST: &'static str = "HTTP/1.1 400 Bad Request";
const STATUS_NOT_FOUND: &'static str = "HTTP/1.1 404 Not Found";

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let mime_types = &generate_mimetype_maps();

    for stream_result in listener.incoming() {
        let stream = stream_result.unwrap();

        handle_connection(stream);
        println!("Connection established");
    }

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
                let uri = request_line_content.uri;
                println!("Requested resource: {:#?}", uri);
                let res = fs::read_to_string(format!("./root/{}", uri).as_str());
                match res {
                    Ok(contents) => {
                        let status_line = STATUS_OK;
                        stream_html(&mut stream, status_line, contents.as_str());
                    }
                    Err(e) => {
                        let contents = fs::read_to_string("./root/not_found.html")
                            .expect("Cannot find bad request html.");
                        stream_html(&mut stream, STATUS_NOT_FOUND, contents.as_str());
                    }
                }

            }
            None => {
                send_bad_request(&mut stream);
            }
        }
    }

    fn send_bad_request(mut stream: &mut TcpStream) {
        let contents = fs::read_to_string("./root/bad_request.html")
            .expect("Cannot find bad request html.");
        stream_html(&mut stream, STATUS_BAD_REQUEST, contents.as_str());
    }

    fn stream_html(stream: &mut TcpStream, status_line: &str, contents: &str) {
        let length = contents.len();
        let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");
        stream.write_all(response.as_bytes()).unwrap();
    }
}
