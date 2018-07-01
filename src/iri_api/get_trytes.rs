use failure::Error;
use reqwest::header::{ContentType, Headers};
use crate::utils::input_validator;

pub fn get_trytes(uri: &str, hashes: &[String]) -> Result<GetTrytesResponse, Error> {
    assert!(input_validator::is_array_of_hashes(hashes));

    let client = reqwest::Client::new();
    let mut headers = Headers::new();
    headers.set(ContentType::json());
    headers.set_raw("X-IOTA-API-Version", "1");

    let body = json!({
        "command": "getTrytes",
        "hashes": hashes,
    });

    Ok(client
        .post(uri)
        .headers(headers)
        .body(body.to_string())
        .send()?
        .json()?)
}

#[derive(Deserialize, Debug)]
pub struct GetTrytesResponse {
    duration: i64,
    trytes: Vec<String>,
}