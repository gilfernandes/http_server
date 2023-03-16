use std::path::PathBuf;

use crate::remove_double_slash;

const DEFAULT_FILE_NAME: &'static str = "unknown";

struct FileData {
    file_name: String,
    file_size: u64,
    is_dir: bool
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

pub(crate) fn list_folder(pb: PathBuf) -> String {
    let files = pb.read_dir().unwrap();
    let folder_name = pb.file_name().and_then(|name| name.to_str()).unwrap_or(DEFAULT_FILE_NAME);
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
    return format!("{}_{}", marker, file_data.file_name)
}

fn print_table_row(folder_name: &str, file_data: &FileData) -> String {
    let f = &file_data.file_name;
    let file_size = &file_data.file_size;
    format!("<tr><td>{file_size}</td><td><a href='{folder_name}/{f}'>{f}</a></td></tr>")
}

fn adapt_file_data(path_buf: &PathBuf, file_name: &str) -> Option<FileData> {
    let metadata = path_buf.metadata().ok()?;
    return Some(FileData {
        file_size: metadata.len(),
        file_name: file_name.to_string(),
        is_dir: path_buf.is_dir()
    });
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
}