use std::io::Error;
use std::path::PathBuf;
use std::time::SystemTime;
use fancy_regex::Regex;

use chrono::{Datelike, DateTime, Local, TimeZone, Utc};

use crate::{HttpData, remove_double_slash};

const DEFAULT_FILE_NAME: &'static str = "unknown";

struct FileData {
    file_name: String,
    file_size: u64,
    is_dir: bool,
    create_date: String,
    modified_date: String
}

pub(crate) fn build_path(uri: String, root_folder: &String) -> String {
    let path_str = if root_folder.starts_with("/") { format!("/{}/{}", root_folder, uri) } else {
        format!("./{}/{}", root_folder, uri)
    };
    remove_double_slash(path_str.as_str())
}

pub(crate) fn is_folder(built_path: String) -> Option<PathBuf> {
    let path = PathBuf::from(built_path);
    return if path.is_dir() { Some(path) } else { None };
}

pub(crate) fn list_folder(pb: PathBuf, root_folder: &String) -> String {
    let files = pb.read_dir().unwrap();
    let root_path_buf = PathBuf::from(root_folder);
    let folder_name = if pb != root_path_buf { pb.file_name().and_then(|name| name.to_str()).unwrap_or(DEFAULT_FILE_NAME) }
        else { "" };
    let mut buffered = format!("
<html>
    <head>
        <title>{folder_name}</title>
        <meta name='viewport' content='width=device-width'/>
    </head>
    <body>
        <h1>{folder_name}</h1>
");
    let mut files_vec = Vec::new();
    for dir_res in files {
        match dir_res {
            Ok(dir_entry) => {
                let f_name = dir_entry.file_name();
                let file_name = f_name.to_str().unwrap_or(DEFAULT_FILE_NAME);
                let file_data_option = adapt_file_data(&dir_entry.path(), file_name);
                match file_data_option {
                    Some(file_date) => { files_vec.push(file_date) }
                    None => {}
                }
            }
            Err(_) => {}
        }
    }
    buffered += format!("<h4>total {}</h4>", files_vec.len()).as_str();
    buffered += "<table>";
    files_vec.sort_by(|a, b| create_key(a).cmp(&create_key(b)));
    for f in files_vec {
        buffered += print_table_row(folder_name, &f).as_str();
    }
    buffered += "</table>";
    buffered += "</body></html>";
    buffered
}

fn create_key(file_data: &FileData) -> String {
    let marker = if file_data.is_dir { "d" } else { "f" };
    return format!("{}_{}", marker, file_data.file_name);
}

fn print_table_row(folder_name: &str, file_data: &FileData) -> String {
    match file_data {
        FileData { file_name, file_size, create_date, is_dir, modified_date } => {
            let folder_char = if *is_dir { "&#x1F4C1;" } else { "&#128196;" };
            format!("\
                <tr>\
                    <td>{folder_char}</td>\
                    <td align='right'>{file_size}</td>\
                    <td align='right'>{create_date}</td>\
                    <td align='right'>{modified_date}</td>\
                    <td><a href='{folder_name}/{file_name}'>{file_name}</a></td>\
                </tr>")
        }
    }
}

fn adapt_file_data(path_buf: &PathBuf, file_name: &str) -> Option<FileData> {
    let metadata = path_buf.metadata().ok()?;

    let created_str = convert_time(metadata.created());
    let modified_str = convert_time(metadata.modified());
    return Some(FileData {
        file_size: metadata.len(),
        file_name: file_name.to_string(),
        is_dir: path_buf.is_dir(),
        create_date: created_str,
        modified_date: modified_str
    });
}

fn convert_time(created_res: Result<SystemTime, Error>) -> String {
    let mut created_str = "".to_string();

    match created_res {
        Ok(system_time) => {
            let date_time: DateTime<Local> = system_time.into();

            let format = if date_time.year() == Utc::now().year() { "%b %e %T" } else { "%b %e %Y" };
            created_str = date_time.format(format).to_string();
        }
        Err(_) => {}
    }
    created_str
}

pub(crate) fn transform_uri(uri: String, root_folder: &String) -> String {
    println!("uri: {uri} root_folder: {root_folder}");
    let path_str = build_path(uri, root_folder);

    let path_buf = PathBuf::from(path_str);
    if path_buf.is_dir() {
        // Check if it has index.htm or index.html
        let paths = vec!["index.html", "index.htm"];
        for path in paths {
            let mut path_index_html = path_buf.clone();
            path_index_html.push(path);
            if path_index_html.exists() {
                return path_index_html.to_str().unwrap().to_string();
            }
        }
    }
    return path_buf.to_str().unwrap().to_string();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn when_build_path_should_build_path() {
        let built = build_path(String::from("info.txt"), &String::from("root"));
        assert_eq!(built, String::from("./root/info.txt"));
    }

    #[test]
    fn when_is_folder_should_be_folder() {
        let built_path = build_path(String::from("css"), &String::from("root"));
        let folder = is_folder(built_path);
        assert_eq!(folder.is_some(), true);
    }

    #[test]
    fn when_is_folder_should_be_false() {
        let built_path = build_path(String::from("index.html"), &String::from("root"));
        let folder = is_folder(built_path);
        assert_eq!(folder.is_some(), false);
    }

    #[test]
    fn when_does_not_exist_should_be_false() {
        let built_path = build_path(String::from("index1.html"), &String::from("root"));
        let folder = is_folder(built_path);
        assert_eq!(folder.is_some(), false);
    }

    #[test]
    fn when_transform_uri_should_produce_index_html() {
        let res = transform_uri("".to_string(), &String::from("root"));
        assert_eq!(res.contains("index.html"), true);
    }

    #[test]
    fn when_transform_uri_should_produce_index_htm() {
        let res = transform_uri("".to_string(), &String::from("root/pdf"));
        assert_eq!(res.contains("index.htm"), true);
    }

    #[test]
    fn when_transform_uri_should_produce_folder() {
        let res = transform_uri("".to_string(), &String::from("root/pdf/test"));
        assert_eq!(res.contains("root/pdf/test"), true);
    }
}