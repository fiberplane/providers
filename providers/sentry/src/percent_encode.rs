use percent_encoding::{utf8_percent_encode, AsciiSet, NON_ALPHANUMERIC};

const URI_COMPONENT_SET: AsciiSet = create_uri_component_ascii_set();

const fn create_uri_component_ascii_set() -> AsciiSet {
    // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/encodeURIComponent#description
    NON_ALPHANUMERIC
        .remove(b'-')
        .remove(b'_')
        .remove(b'.')
        .remove(b'!')
        .remove(b'~')
        .remove(b'*')
        .remove(b'\'')
        .remove(b'(')
        .remove(b')')
}

/// Encodes a string using the same algorithm as JavaScript's
/// `encodeURIComponent()`:
///   https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/encodeURIComponent
pub fn encode_uri_component(string: &str) -> String {
    utf8_percent_encode(string, &URI_COMPONENT_SET).collect()
}
