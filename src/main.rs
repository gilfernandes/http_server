use std::{fs, io};
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;

use clap::Parser;
use linked_hash_set::LinkedHashSet;

use generate_headers::{STATUS_BAD_REQUEST, STATUS_METHOD_NOT_ALLOWED, STATUS_NOT_FOUND, STATUS_OK};
use http_server::ThreadPool;

use crate::args::{HttpServerArgs, Mode, RunCommand};
use crate::basic_auth::process_basic_auth;
use crate::folder_operations::{build_path, is_folder, list_folder, transform_uri};
use crate::http_parser::{BasicCredentials, decode_user_name_password, find_basic_authorization_header, Method, request_line};
use crate::http_struct::HttpData;
use crate::mime_type_map::{extract_extension, extract_mime_type, MimeTypeProperties, TEXT_HTML};
use crate::string_operations::{extract_file_name, remove_double_slash, replace_slash};

mod http_parser;
mod mime_type_map;
mod string_operations;
mod http_struct;
mod args;
mod header_parser;
mod folder_operations;
mod basic_auth;
mod generate_headers;

const STATUS_METHOD_NOT_ALLOWED_RESPONSE: &'static str = "<!DOCTYPE html>
<html lang=\"en\">
<head>
    <meta charset=\"utf-8\">
    <title>Method Not Allowed!</title>
</head>
<body>
<h1>Method not allowed!</h1>
<p>400 - The method you tried is not allowed</p>
</body>
</html>";

fn main() {
    let args = HttpServerArgs::parse();
    let mode = args.mode;
    match mode {
        Mode::Run(run_args) => {
            println!("Running on {} {}", run_args.host, run_args.port);
            run_server(&run_args);
        }
        Mode::Info(_) => {}
    }
}

fn run_server(run_args: &RunCommand) {
    let listener = TcpListener::bind(format!("{}:{}",
                                             &run_args.host,
                                             &run_args.port)).unwrap();
    let pool = ThreadPool::new(run_args.pool_size);

    for stream_result in listener.incoming() {
        let stream = stream_result.unwrap();
        let run_args_clone = run_args.clone();
        pool.execute(move || {
            handle_connection(stream, &run_args_clone);
        });
        println!("Connection established");
    }

    println!("Shutting down");

    fn handle_connection(mut stream: TcpStream, run_args: &RunCommand) {
        let buf_reader = BufReader::new(&mut stream);
        let http_request: Vec<String> = buf_reader
            .lines()
            .map(|result| result.unwrap())
            .take_while(|line| !line.is_empty())
            .collect();

        if http_request.is_empty() {
            send_bad_request(&mut stream, &run_args.root_folder);
            return;
        }
        let rl = http_request[0].clone();
        let (_, request_line_option) = request_line(rl.as_bytes()).unwrap();

        for header in http_request.iter() {
            println!(":: {:#?}", header);
        }

        let root_folder = &run_args.root_folder;

        match request_line_option {
            Some(request_line_content) => {
                let uri = request_line_content.uri.clone();
                if let Some(use_basic_auth) = process_basic_auth(&uri, run_args) {
                    let credentials_option = process_basic_authentication(http_request);
                    if let None = credentials_option {
                        stream_headers_only(&mut stream,
                                            generate_headers::generate_authenticate_response);
                        return;
                    }
                    let credentials = credentials_option.unwrap();
                    if credentials.username != *use_basic_auth.username && credentials.password != *use_basic_auth.password {
                        stream_headers_only(&mut stream,
                                            generate_headers::generate_authenticate_response);
                        return;
                    }
                }
                match request_line_content.method {
                    Method::Get | Method::Head => {
                        let built_path = transform_uri(uri.clone(), root_folder);
                        let extension_option = extract_extension(built_path.as_str());
                        let folder_option = is_folder(built_path.clone());
                        let mime_type_map = extract_mime_type(extension_option);
                        println!("Requested resource: {:#?}. Mime type: {}", built_path.clone(), mime_type_map.content_type);
                        let is_head = request_line_content.method == Method::Head;
                        let http_data = HttpData {
                            stream: &mut stream,
                            uri: built_path,
                            mime_type_map: &mime_type_map,
                            is_head: &is_head,
                            root_folder: root_folder,
                        };
                        match folder_option {
                            Some(folder) => {
                                process_folder_response(http_data, folder);
                            }
                            None => {
                                if mime_type_map.binary {
                                    process_binary_content(http_data);
                                } else {
                                    process_text_content(http_data);
                                }
                            }
                        }
                    }
                    Method::Options => {
                        let uri = replace_slash(request_line_content.uri);
                        println!("Requested resource: {:#?}", uri);
                        stream_headers_only(&mut stream,
                                            generate_headers::generate_option_headers);
                    }
                    _ => {
                        send_error_response(HttpData {
                            stream: &mut stream,
                            uri: "".to_string(),
                            mime_type_map: &MimeTypeProperties::default_extension(),
                            is_head: &false,
                            root_folder: &run_args.root_folder,
                        }, "method_not_allowed.html", STATUS_METHOD_NOT_ALLOWED,
                                            STATUS_METHOD_NOT_ALLOWED_RESPONSE);
                    }
                }
            }
            None => {
                send_bad_request(&mut stream, &run_args.root_folder);
            }
        }
    }

    fn process_basic_authentication(http_request: Vec<String>) -> Option<BasicCredentials> {
        let authentication_option = find_basic_authorization_header(http_request);
        match authentication_option {
            Some(authentication) => {
                let credentials_option = decode_user_name_password(&authentication);
                match credentials_option {
                    Some(credentials) => {
                        return Some(credentials);
                    }
                    None => {}
                }
            }
            None => {}
        }
        None
    }
}

fn process_text_content(http_data: HttpData) {
    let HttpData {
        stream,
        uri,
        mime_type_map,
        is_head,
        root_folder
    } = http_data;
    let path = uri.clone();
    let result_file = File::open(path);
    match result_file {
        Ok(path) => {
            let res = io::read_to_string(BufReader::new(path));
            match res {
                Ok(contents) => {
                    stream_text(stream,
                                STATUS_OK,
                                contents.as_str(), mime_type_map.content_type.as_str(),
                                is_head);
                }
                Err(e) => {
                    println!("Error: {:?}", e.to_string());
                    not_found(HttpData { stream, uri: uri.clone(), mime_type_map, is_head, root_folder });
                }
            }
        }
        Err(_) => {
            not_found(HttpData { stream, uri: uri.clone(), mime_type_map, is_head, root_folder });
        }
    }
}

fn process_folder_response(http_data: HttpData, dir: PathBuf) {
    let stream = http_data.stream;
    let is_head = http_data.is_head;
    let folder_response = list_folder(dir, &http_data.root_folder);
    stream_text(stream,
                STATUS_OK,
                folder_response.as_str(), TEXT_HTML,
                is_head);
}

fn send_error_response(http_data: HttpData, html_file: &str,
                       status: &str, missing_html: &str) {
    let HttpData { stream, is_head, .. } = http_data;
    let request_file = format!("./{}/{html_file}", http_data.root_folder);
    let result_file = File::open(request_file);
    match result_file {
        Ok(file) => {
            let buf_result = io::read_to_string(BufReader::new(file));
            match buf_result {
                Ok(contents) => {
                    stream_text(stream, status, contents.as_str(), TEXT_HTML, is_head);
                }
                Err(e) => {
                    println!("Cannot find file {html_file}: {:?}", e);
                    stream_text(stream, status, missing_html, TEXT_HTML, is_head);
                }
            }
        }
        Err(e) => {
            println!("Cannot find file {html_file}: {:?}", e);
            stream_text(stream, status, missing_html, TEXT_HTML, is_head);
        }
    }
}

fn not_found(http_data: HttpData) {
    let uri = http_data.uri.clone();
    send_error_response(http_data, "not_found.html", STATUS_NOT_FOUND,
                        format!("<!DOCTYPE html>
<html lang=\"en\">
<head>
    <meta charset=\"utf-8\">
    <title>Not found!</title>
</head>
<body>
<h1>File {} Not Found!</h1>
<p>404 - Cannot find resource in folder</p>
</body>
</html>", uri).as_str());
}

fn process_binary_content(http_data: HttpData) {
    let HttpData {
        stream,
        uri,
        mime_type_map: mime_type_properties,
        is_head,
        root_folder
    } = http_data;
    let res = fs::read(uri.clone());
    let mime_type = &mime_type_properties.content_type;

    match res {
        Ok(content) => {
            let bytes = &content[..];
            let length = bytes.len();
            let header_map =
                generate_status_headers(STATUS_OK, length, mime_type.as_str(), &mime_type_properties.binary);
            let response = generate_binary_status_line(
                uri.clone(),
                header_map,
                mime_type_properties,
            );
            let header_bytes = response.as_bytes();
            if *is_head {
                stream.write_all(header_bytes).unwrap();
            } else {
                let concat_vec = [header_bytes, bytes].concat();
                let concat_bytes = &concat_vec[..];
                stream.write_all(concat_bytes).unwrap();
            }
        }
        Err(_) => {
            not_found(HttpData {
                stream,
                uri: uri.clone(),
                mime_type_map: mime_type_properties,
                is_head,
                root_folder,
            });
        }
    }
}

fn generate_binary_status_line(uri: String, header_map: LinkedHashSet<String>,
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


fn send_bad_request(mut stream: &mut TcpStream, root_folder: &String) {
    let bad_request = build_path("./{}/bad_request.html".to_string(), root_folder);
    let result = fs::read_to_string(bad_request.as_str());
    match result {
        Ok(contents) => {
            stream_text(&mut stream, STATUS_BAD_REQUEST, contents.as_str(), TEXT_HTML, &false);
        }
        Err(_) => {
            let contents = "<!DOCTYPE html>
<html lang=\"en\">
<head>
    <meta charset=\"utf-8\">
    <title>Bad request!</title>
</head>
<body>
<h1>Bad Request!</h1>
<p>400 - Your request could not be understood by the server</p>
</body>
</html>";
            stream_text(&mut stream, STATUS_BAD_REQUEST, contents, TEXT_HTML, &false);
        }
    }
}

fn stream_text(stream: &mut TcpStream,
               status_line: &str,
               contents: &str,
               mime_type: &str,
               is_head: &bool,
) {
    stream_text_function(stream, status_line, contents, mime_type, is_head, generate_status_headers);
}

fn stream_headers_only(stream: &mut TcpStream,
                       generate_status_headers: fn(status_line: &str, length: usize,
                                                   mime_type: &str,
                                                   is_binary: &bool) -> LinkedHashSet<String>) {
    stream_text_function(stream, "", "", "",
                         &true, generate_status_headers)
}

fn stream_text_function(stream: &mut TcpStream,
                        status_line: &str,
                        contents: &str,
                        mime_type: &str,
                        is_head: &bool,
                        generate_status_headers: fn(status_line: &str, length: usize,
                                                    mime_type: &str,
                                                    is_binary: &bool) -> LinkedHashSet<String>,
) {
    let length = contents.len();
    let header_map = generate_status_headers(status_line, length, mime_type, is_head);

    let concatenated_headers_str = concatenate_headers(&header_map);

    let response = if *is_head { format!("{concatenated_headers_str}\r\n") } else { format!("{concatenated_headers_str}\r\n{contents}") };
    let bytes = response.as_bytes();
    stream.write_all(bytes).unwrap();
}

fn concatenate_headers(header_map: &LinkedHashSet<String>) -> String {
    return header_map.iter()
        .map(|x| (*x).to_string())
        .collect::<Vec<_>>().join("");
}

fn generate_status_headers(status_line: &str, length: usize, mime_type: &str, is_binary: &bool) -> LinkedHashSet<String> {
    let content_length = format!("Content-Length: {length}\r\n");
    let content_type = if *is_binary { format!("Content-Type: {mime_type}\r\n") } else { format!("Content-Type: {mime_type}; charset=utf-8\r\n") };
    let (status_line, cache_control, server) = generate_headers::generate_status_with_common_headers(status_line);

    let mut status_headers_set = LinkedHashSet::new();
    status_headers_set.insert(status_line);
    status_headers_set.insert(content_length);
    status_headers_set.insert(content_type);
    status_headers_set.insert(cache_control);
    status_headers_set.insert(server);

    return status_headers_set.clone();
}
