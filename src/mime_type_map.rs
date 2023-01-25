use std::str::from_utf8;
use phf::{Map, phf_map};

pub(crate) const TEXT_HTML: &'static str = "text/html";
pub(crate) const JPEG: &'static str = "image/jpeg";

static MIME_TYPES: Map<&'static str, (&'static str, bool, bool)> = phf_map! {
    "css" => ("text/css", false, false),
    "doc" => ("application/msword", true, true),
    "gif" => ("image/gif", true, false),
    "ico" => ("image/x-icon", true, false),
    "html" => (TEXT_HTML, false, false),
    "htm" => (TEXT_HTML, false, false),
    "jpe" => (JPEG, true, false),
    "jpg" => (JPEG, true, false),
    "jpeg" => (JPEG, true, false),
    "pdf" => ("application/pdf", true, true),
    "png" => ("image/png", true, false),
    "svg" => ("image/svg+xml", false, false),
    "txt" => ("text/plain", false, false),
    "xlsx" => ("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet", true, true)
};

#[derive(Clone)]
pub(crate) struct MimeTypeProperties {
    pub(crate) content_type: String,
    pub(crate) binary: bool,
    pub(crate) attachment: bool,
}

impl MimeTypeProperties {
    pub(crate) fn new(content_type: &str, binary: bool, attachment: bool) -> MimeTypeProperties {
        MimeTypeProperties {
            content_type: content_type.to_string(),
            binary: binary,
            attachment: attachment,
        }
    }

    pub(crate) fn default_extension() -> MimeTypeProperties {
        MimeTypeProperties {
            content_type: "application/octet-stream".to_string(),
            binary: true,
            attachment: true,
        }
    }
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

pub(crate) fn extract_mime_type(file_name: &str)
                                -> MimeTypeProperties {
    let option = extract_extension(file_name);
    match option {
        Some(extension) => {
            match MIME_TYPES.get(extension.as_str()) {
                Some(s) => {
                    let (mime, binary, attachment) = s;
                    MimeTypeProperties::new(mime, *binary, *attachment)
                }
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
        let mime_type = extract_mime_type(file_name);
        assert_eq!(mime_type.content_type, expected);
    }
}
