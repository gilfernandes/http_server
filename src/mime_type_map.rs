use std::collections::HashMap;
use std::str::from_utf8;

pub(crate) const TEXT_HTML: &'static str = "text/html";
const JPEG: &'static str = "image/jpeg";

#[derive(Clone)]
pub(crate) struct MimeTypeProperties {
    pub(crate) content_type: String,
    pub(crate) binary: bool,
    attachment: bool,
}

impl MimeTypeProperties {
    fn new(content_type: &str, binary: bool, attachment: bool) -> MimeTypeProperties {
        MimeTypeProperties {
            content_type: content_type.to_string(),
            binary: binary,
            attachment: attachment,
        }
    }

    fn default_extension() -> MimeTypeProperties {
        MimeTypeProperties {
            content_type: "application/octet-stream".to_string(),
            binary: true,
            attachment: true,
        }
    }
}

pub(crate) fn generate_mimetype_maps() -> HashMap<String, MimeTypeProperties> {
    let mut map: HashMap<String, MimeTypeProperties> = HashMap::new();
    map.insert(String::from("gif"), MimeTypeProperties::new("image/gif", true, false));
    map.insert(String::from("html"), MimeTypeProperties::new(TEXT_HTML, false, false));
    map.insert(String::from("htm"), MimeTypeProperties::new(TEXT_HTML, false, false));
    map.insert(String::from("jpe"), MimeTypeProperties::new(JPEG, true, false));
    map.insert(String::from("jpg"), MimeTypeProperties::new(JPEG, true, false));
    map.insert(String::from("jpeg"), MimeTypeProperties::new(JPEG, true, false));
    map.insert(String::from("pdf"), MimeTypeProperties::new("application/pdf", true, true));
    map.insert(String::from("png"), MimeTypeProperties::new("image/png", true, false));
    map.insert(String::from("svg"), MimeTypeProperties::new("image/svg+xml", true, false));
    map.insert(String::from("txt"), MimeTypeProperties::new("text/plain", false, false));
    map.insert(String::from("xlsx"), MimeTypeProperties::new("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet", true, true));
    let immutable = map.clone();
    return immutable;
}

pub(crate) fn extract_extension(file_name: &str) -> Option<String> {
    let bytes = file_name.as_bytes();
    let size = bytes.len();
    const DOT: u8 = 46;
    for (i, el) in bytes.iter().rev().enumerate() {
        if *el == DOT {
            let slice = &bytes[size - i..size];
            return Some(from_utf8(slice).unwrap().to_string());
        }
    }
    None
}

pub(crate) fn extract_mime_type(file_name: &str, mime_types: &HashMap<String, MimeTypeProperties>)
                                -> MimeTypeProperties {
    let option = extract_extension(file_name);
    match option {
        Some(extension) => {
            match mime_types.get(extension.as_str()) {
                Some(s) => { s.clone() }
                None => MimeTypeProperties::default_extension()
            }
        }
        None => MimeTypeProperties::default_extension()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn when_extract_extension_should_find_png() {
        test_extension("flower.png", "png");
    }

    #[test]
    fn when_extract_extension_should_find_jpg() {
        test_extension("flower.jpg", "jpg");
    }

    #[test]
    fn when_extract_mime_type_should_extract() {
        test_mime_conversion("flower.png", "image/png");
    }

    #[test]
    fn when_extract_mime_type_jpeg_should_extract() {
        test_mime_conversion("flower.jpeg", "image/jpeg");
    }

    fn test_extension(file_name: &str, expected: &str) {
        let mimetype_option = extract_extension(file_name);
        assert!(mimetype_option.is_some(), "Result should return some");
        let extension = mimetype_option.unwrap();
        assert_eq!(extension, expected.to_string());
    }

    fn test_mime_conversion(file_name: &str, expected: &str) {
        let map = generate_mimetype_maps();
        let mime_type = extract_mime_type(file_name, &map);
        assert_eq!(mime_type.content_type, expected);
    }
}
