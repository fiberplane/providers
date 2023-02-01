pub mod request_state {
    /// A state marker for Canonical Requests
    pub type State = u8;
    /// The request has all API Action-specific information.
    /// It lacks only the mandatory information to sign the request
    pub const STEM: State = 0;
    /// The request has all information necessary to be signed with V4 algorithm and sent.
    pub const READY_TO_SIGN_V4: State = 1;
}

use super::sigv4;
use bytes::Bytes;
use fiberplane_pdk::prelude::Timestamp;
use http::Method;
use itertools::Itertools;
use secrecy::SecretString;
use std::collections::BTreeMap;

/// A valid Canonical Request for AWS API.
///
/// A good Canonical Request must specify the Action it wants to do:
/// - either as an 'Action' query parameter (in the query_params member), or
/// - in the header as a 'X-Amz-Target' header.
/// Note that in both cases, the action ends up being signed.
#[derive(Debug, Clone)]
pub struct CanonicalRequest<const S: request_state::State> {
    pub method: Method,
    /// Must include the initial '/'.
    /// The '/' is necessary to build the payload to sign,
    /// and [ClientCommon::send]() assumes the initial / is present
    pub uri: String,
    pub query_params: BTreeMap<String, String>,
    /// Note: The request can include any headers; canonical_headers and
    /// signed_headers lists those that you want to be included in the
    /// hash of the request. "host" and "x-amz-date" are always required.
    /// Those headers are guaranteed to be present if the request is in the
    /// [Ready to sign](RequestState::ReadyToSignV4) state
    pub headers: BTreeMap<String, String>,
    pub body: Option<Bytes>,
    date: Option<Timestamp>,
}

impl CanonicalRequest<{ request_state::STEM }> {
    /// Builds a Stem canonical request. This request still needs
    /// to be "prepared" (with [prepare](Self::prepare)) to be
    /// augmented with the necessary data for a valid signature
    pub fn new(
        method: Method,
        uri: String,
        query_params: BTreeMap<String, String>,
        headers: BTreeMap<String, String>,
        body: Option<Bytes>,
    ) -> Self {
        Self {
            method,
            uri,
            query_params,
            headers,
            body,
            date: None,
        }
    }

    pub fn add_signed_header<K, V>(&mut self, key: K, value: V)
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.headers.insert(key.into(), value.into());
    }

    pub fn prepare(
        mut self,
        host: &str,
        date: &Timestamp,
    ) -> CanonicalRequest<{ request_state::READY_TO_SIGN_V4 }> {
        self.add_signed_header("host", host);
        self.add_signed_header("x-amz-date", sigv4::amzdate(date));
        self.date = Some(*date);
        CanonicalRequest::<{ request_state::READY_TO_SIGN_V4 }> {
            method: self.method,
            uri: self.uri,
            query_params: self.query_params,
            headers: self.headers,
            body: self.body,
            date: self.date,
        }
    }
}

impl CanonicalRequest<{ request_state::READY_TO_SIGN_V4 }> {
    pub fn date(&self) -> &Timestamp {
        self.date
            .as_ref()
            .expect("A ReadyToSignV4 request always have its date set.")
    }

    pub fn signed_headers(&self) -> String {
        Itertools::intersperse(self.headers.keys().cloned(), ";".to_string()).collect()
    }

    pub fn querystring(&self) -> String {
        let mut serializer = form_urlencoded::Serializer::new(String::new());
        serializer.extend_pairs(self.query_params.iter());
        serializer.finish()
    }

    pub fn to_sigv4_auth_header(
        &self,
        access_key_id: &SecretString,
        secret_access_key: &SecretString,
        region: &str,
        service: &str,
    ) -> (String, String) {
        sigv4::format_auth_header_value(self, access_key_id, secret_access_key, region, service)
    }
}

impl std::fmt::Display for CanonicalRequest<{ request_state::READY_TO_SIGN_V4 }> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let method = match self.method {
            Method::GET => "GET",
            Method::POST => "POST",
            Method::OPTIONS => "OPTIONS",
            Method::PUT => "OPTIONS",
            Method::PATCH => "PATCH",
            Method::DELETE => "DELETE",
            _ => unimplemented!("unknown method"),
        };

        let querystring = self.querystring();
        let headers: String = self
            .headers
            .iter()
            .map(|(k, v)| format!("{k}:{v}\n"))
            .collect::<String>()
            .trim_end()
            .to_string();
        let signed_headers = self.signed_headers();

        let payload_hash =
            super::sigv4::hash_bytes_to_hexstring(&self.body.clone().unwrap_or_default());

        write!(
            f,
            "{}\n{}\n{}\n{}\n\n{}\n{}",
            method, self.uri, querystring, headers, signed_headers, payload_hash
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::datetime;

    #[test]
    fn test_serialization() {
        let expected = r#"GET
/
Param1=value1&Param2=value2
host:example.amazonaws.com
x-amz-date:20150830T123600Z

host;x-amz-date
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"#;

        let actual_creq = CanonicalRequest {
            method: Method::GET,
            uri: "/".to_string(),
            query_params: [
                ("Param2".to_string(), "value2".to_string()),
                ("Param1".to_string(), "value1".to_string()),
            ]
            .into(),
            headers: [
                ("host".to_string(), "example.amazonaws.com".to_string()),
                ("x-amz-date".to_string(), "20150830T123600Z".to_string()),
            ]
            .into(),
            body: None,
            date: Some(Timestamp(datetime!(2015-08-30 12:36:00 UTC))),
        };

        assert_eq!(actual_creq.to_string(), expected);
    }
}
