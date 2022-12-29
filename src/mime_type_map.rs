use std::collections::HashMap;

pub(crate) fn generate_mimetype_maps() -> HashMap<String, String> {
    let mut mime_type_map = HashMap::new();
    mime_type_map.insert(String::from("html"), String::from("text/html"));
    mime_type_map.insert(String::from("htm"), String::from("text/html"));
    mime_type_map.insert(String::from("png"), String::from("image/png"));
    mime_type_map.insert(String::from("jpg"), String::from("image/jpeg"));
    let immutable = mime_type_map.clone();
    return immutable;
}
