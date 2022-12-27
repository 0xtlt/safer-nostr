use std::collections::HashMap;

pub fn parse_query_string(query_string: &str) -> HashMap<String, String> {
    form_urlencoded::parse(query_string.as_bytes())
        .into_owned()
        .collect()
}
