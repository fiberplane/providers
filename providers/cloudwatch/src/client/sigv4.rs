//! Implemenatation of the version 4 signing protocol for AWS API

use std::ops::Deref;

use super::{canonical_request::request_state, CanonicalRequest};
use bytes::Bytes;
use fiberplane_pdk::prelude::Timestamp;
use hmac::{Hmac, Mac};
use secrecy::{ExposeSecret, SecretString};
use sha2::{Digest, Sha256};
use time::{
    format_description::well_known::Iso8601,
    macros::{format_description, offset},
};

pub static ALGORITHM: &str = "AWS4-HMAC-SHA256";
static AWS4_REQUEST: &str = "aws4_request";

pub fn format_auth_header_value(
    request: &CanonicalRequest<{ request_state::READY_TO_SIGN_V4 }>,
    access_key_id: &SecretString,
    secret_access_key: &SecretString,
    region: &str,
    service: &str,
) -> (String, String) {
    (
        "Authorization".to_string(),
        format!(
            "{} Credential={}/{}, SignedHeaders={}, Signature={}",
            ALGORITHM,
            access_key_id.expose_secret(),
            credential_scope(region, request.date(), service),
            request.signed_headers(),
            signature(secret_access_key, region, service, request)
        ),
    )
}

fn datestamp(date: &Timestamp) -> String {
    date.deref()
        .to_offset(offset!(UTC))
        .format(format_description!("[year][month][day]"))
        .unwrap()
}

pub fn amzdate(date: &Timestamp) -> String {
    use time::format_description::well_known::iso8601::{Config, TimePrecision};
    // The date that is used to create the signature. The format must be ISO 8601 basic format (YYYYMMDD'T'HHMMSS'Z').
    // For example, the following date time is a valid X-Amz-Date value: 20120325T120000Z.
    // https://docs.aws.amazon.com/AWSSimpleQueueService/latest/APIReference/CommonParameters.html
    const AMZCONF: u128 = Config::DEFAULT
        .set_use_separators(false)
        .set_time_precision(TimePrecision::Second {
            decimal_digits: None,
        })
        .encode();

    date.deref()
        .to_offset(offset!(UTC))
        .format(&Iso8601::<AMZCONF>)
        .unwrap()
}

fn sign(key: &[u8], message: String) -> Vec<u8> {
    let mut mac = Hmac::<Sha256>::new_from_slice(key).expect("HMAC can take key of any size");
    mac.update(&message.into_bytes());
    mac.finalize().into_bytes()[..].to_vec()
}

fn signature_key(
    secret_access_key: &SecretString,
    region: &str,
    date: &Timestamp,
    service: &str,
) -> Vec<u8> {
    let k_date = sign(
        &format!("AWS4{}", secret_access_key.expose_secret()).into_bytes(),
        datestamp(date),
    );
    let k_region = sign(k_date.as_slice(), region.to_string());
    let k_service = sign(k_region.as_slice(), service.to_string());
    sign(k_service.as_slice(), AWS4_REQUEST.to_string())
}

fn credential_scope(region: &str, date: &Timestamp, service: &str) -> String {
    format!(
        "{}/{}/{}/{}",
        datestamp(date),
        region,
        service,
        AWS4_REQUEST
    )
}

pub fn signature(
    secret_access_key: &SecretString,
    region: &str,
    service: &str,
    request: &CanonicalRequest<{ request_state::READY_TO_SIGN_V4 }>,
) -> String {
    let date = request.date();
    let key = signature_key(secret_access_key, region, date, service);

    let credential_scope = credential_scope(region, date, service);
    let canonical_payload = request.to_string();
    // log(format!(
    //     "CloudWatch: Canonical payload being hashed\n{}",
    //     canonical_payload
    // ));
    let request_digest = hash_to_hexstring(canonical_payload);
    let message = format!(
        "{}\n{}\n{}\n{}",
        ALGORITHM,
        amzdate(date),
        credential_scope,
        request_digest
    );
    // log(format!("CloudWatch: message being signed\n{}", message));

    hex::encode(sign(key.as_slice(), message))
}

pub fn hash_to_hexstring(message: impl ToString) -> String {
    let mut hasher = Sha256::new();
    hasher.update(message.to_string().into_bytes());
    hex::encode(hasher.finalize())
}

pub fn hash_bytes_to_hexstring(message: &Bytes) -> String {
    let mut hasher = Sha256::new();
    hasher.update(message);
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::datetime;

    #[test]
    fn test_signature() {
        let message = r#"AWS4-HMAC-SHA256
20150830T123600Z
20150830/us-east-1/iam/aws4_request
f536975d06c0309214f805bb90ccff089219ecd68b2577efef23edd43b7e1a59"#;

        let secret_access_key = "wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY".parse().unwrap();
        let sign_time = datetime!(2015-08-30 12:36:00 UTC).into();

        let key = signature_key(&secret_access_key, "us-east-1", &sign_time, "iam");

        let expected = "5d672d79c15b13162d9279b0855cfba6789a8edb4c82c400e06b5924a6f2b5d7";
        let actual = hex::encode(sign(key.as_slice(), message.to_string()));

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_empty_hash() {
        let expected = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        let actual = hash_to_hexstring("");
        assert_eq!(actual, expected);
    }
}
