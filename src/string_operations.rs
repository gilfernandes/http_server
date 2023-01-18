use fancy_regex::Regex;

pub(crate) fn replace_slash(uri: String) -> String {
    return extract_from_str(&Regex::new("(.*)/$").expect("Slash regex is not correct"),
                            uri.to_string(), "${1}/index.html".to_string());
}

pub(crate) fn extract_file_name(uri: String) -> String {
    return extract_from_str(&Regex::new(".*/(.+)$").expect("File name regex is not correct"),
                            uri.to_string(), "${1}".to_string());
}

pub(crate) fn remove_double_slash(uri: &str) -> String {
    let regex = Regex::new("/{2,}").unwrap();
    regex.replace_all(uri, "/").to_string()
}

fn extract_from_str(regex: &Regex, uri: String, rep: String) -> String {
    let result = regex.replace(uri.as_str(), rep);
    return result.to_string();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn when_replace_slash_should_include_index_html() {
        test_res(String::from("/"), String::from("/index.html"));
    }

    #[test]
    fn when_replace_subfolder_slash_should_include_index_html() {
        test_res(String::from("/test/"), String::from("/test/index.html"));
    }

    #[test]
    fn when_replace_noslash_should_not_replace() {
        test_res(String::from("/test/test.html"), String::from("/test/test.html"));
    }

    fn test_res(res: String, expected: String) {
        let res = replace_slash(res);
        assert_eq!(res, expected);
    }

    #[test]
    fn when_extract_file_name_should_extract_index_html() {
        assert_eq!(extract_file_name(String::from("/index.html")),
                   String::from("index.html"));
    }

    #[test]
    fn when_extract_pdf_name_should_extract_test_pdf() {
        assert_eq!(extract_file_name(String::from("/test/test.pdf")),
                   String::from("test.pdf"));
    }
}