use fancy_regex::Regex;

pub(crate) fn replace_slash(uri: String) -> String {
    let parse_result = Regex::new("(.*)/$").expect("Slash regex is not correct");
    let result = parse_result.replace(uri.as_str(), "${1}/index.html");
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

    fn test_res(res: String, expected: String) {
        let res = replace_slash(res);
        assert_eq!(res, expected);
    }
}