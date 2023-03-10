use std::path::PathBuf;

use crate::remove_double_slash;

const DEFAULT_FILE_NAME: &'static str = "unknown";

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
    </head>
    <body>
        <h1>{folder_name}</h1>
");
    for dir_res in files {
        match dir_res {
            Ok(dir_entry) => {
                let f_name = dir_entry.file_name();
                let file_name = f_name.to_str().unwrap_or(DEFAULT_FILE_NAME);
                buffered += format!("<p><a href='{folder_name}/{file_name}'>{file_name}</a></p>").as_str();
            }
            Err(_) => {}
        }
    }
    buffered += "</body></html>";
    buffered
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