use reqwest::StatusCode;
use std::collections::HashMap;

#[derive(Debug)]
pub struct HttpResponse {
    pub status_code: StatusCode,
    pub response_body: String,
    pub headers: HashMap<String, String>,
}