use linked_hash_set::LinkedHashSet;

pub const STATUS_OK: &'static str = "HTTP/1.1 200 OK";
pub const STATUS_BAD_REQUEST: &'static str = "HTTP/1.1 400 Bad Request";
pub const STATUS_NOT_FOUND: &'static str = "HTTP/1.1 404 Not Found";
pub const STATUS_METHOD_NOT_ALLOWED: &'static str = "HTTP/1.1 405 Method Not Allowed";

const STATUS_NO_CONTENT: &'static str = "HTTP/1.1 204 No Content";
const STATUS_UNAUTHORIZED: &'static str = "HTTP/1.1 401 Unauthorized";
const SERVER_NAME: &'static str = "Gil HTTP";

const HEADER_AUTHENTICATE: &'static str =
    "WWW-Authenticate: Basic realm=\"User Visible Realm`\", charset=`\"UTF-8`\"";

pub fn generate_option_headers(_: &str, _: usize, _: &str, _: &bool) -> LinkedHashSet<String> {
    let allow = format!("Allow: OPTIONS, GET, HEAD\r\n");
    let (status_line, cache_control, server) = generate_status_with_common_headers(STATUS_NO_CONTENT);
    let mut status_headers_set = LinkedHashSet::new();
    status_headers_set.insert(status_line);
    status_headers_set.insert(allow);
    status_headers_set.insert(cache_control);
    status_headers_set.insert(server);
    return status_headers_set.clone();
}

pub(crate) fn generate_authenticate_response(_: &str, _: usize, _: &str, _: &bool) -> LinkedHashSet<String> {
    let status_line = format!("{STATUS_UNAUTHORIZED}\r\n");
    let mut status_headers_set = LinkedHashSet::new();
    status_headers_set.insert(status_line);
    status_headers_set.insert(HEADER_AUTHENTICATE.to_string());
    return status_headers_set.clone();
}

pub fn generate_status_with_common_headers(status_line: &str) -> (String, String, String) {
    let status_line = format!("{status_line}\r\n");
    let cache_control = format!("Cache-Control: public, max-age=120\r\n");
    let server = format!("Server: {SERVER_NAME}\r\n");
    return (status_line, cache_control, server);
}




