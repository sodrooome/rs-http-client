// mod exception;

use base64::{engine::general_purpose, Engine as _};
use core::panic;
// use base64::Engine;
use reqwest::{Body, Client, RequestBuilder, Response, Url};
use serde_json::Value;
use serde_urlencoded::to_string;
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};

use crate::{HttpRequestError, RequestHook, ResponseHook};
// use exception::HttpRequestError;

pub struct HttpRequest {
    client: Client,
    base_url: Url,
    timeout: Duration,
    logging: bool,
    request_hook: Option<Box<dyn RequestHook>>,
    response_hook: Option<Box<dyn ResponseHook>>,
}

#[allow(dead_code)]
impl HttpRequest {
    pub fn new(base_url: &str, logging: bool) -> Self {
        let client = Client::new();
        let base_url = Url::parse(base_url).expect("Given the invalid argument for Base URL");
        let timeout = Duration::from_secs(30);

        HttpRequest {
            client,
            base_url,
            timeout,
            logging,
            request_hook: None,
            response_hook: None,
        }
    }

    fn build_request(&self, method: reqwest::Method, path: &str) -> RequestBuilder {
        let url = self
            .base_url
            .join(path)
            .expect("Given the invalid argument for URL");
        let url_schema = url.scheme();
        if url_schema == "http" {
            panic!("HTTP scheme currently is not allowed");
        }
        let request = self.client.request(method, url).timeout(self.timeout);

        if let Some(request_hook) = &self.request_hook {
            request_hook.apply(&request);
        }

        request
    }

    pub fn set_request_hook(&mut self, request_hook: Box<dyn RequestHook>) {
        self.request_hook = Some(request_hook);
    }

    pub fn set_response_hook(&mut self, response_hook: Box<dyn ResponseHook>) {
        self.response_hook = Some(response_hook);
    }

    pub fn get(&self, path: &str) -> RequestBuilder {
        self.build_request(reqwest::Method::GET, path)
    }

    pub fn post(&self, path: &str) -> RequestBuilder {
        self.build_request(reqwest::Method::POST, path)
    }

    pub fn delete(&self, path: &str) -> RequestBuilder {
        self.build_request(reqwest::Method::DELETE, path)
    }

    pub fn patch(&self, path: &str) -> RequestBuilder {
        self.build_request(reqwest::Method::PATCH, path)
    }

    pub fn put(&self, path: &str) -> RequestBuilder {
        self.build_request(reqwest::Method::PUT, path)
    }

    pub async fn send_request(
        &self,
        request: RequestBuilder,
    ) -> Result<Response, HttpRequestError> {
        if self.logging {
            log::info!("HTTP request: {:?}", request);
        }

        let response = request
            .send()
            .await
            .map_err(HttpRequestError::RequestBuilderError)?;

        if self.logging {
            log::info!("HTTP response: {:?}", response);
        }

        if let Some(response_hook) = &self.response_hook {
            response_hook.apply(&response);
        }

        Ok(response)
    }

    #[deprecated(note = "prepare_get rarely used, please use the get instead")]
    pub fn prepare_get(&self, path: &str) -> RequestBuilder {
        self.get(path)
    }

    #[deprecated(note = "prepare_post rarely used, please use the post instead")]
    pub fn prepare_post(&self, path: &str) -> RequestBuilder {
        self.post(path)
    }

    #[deprecated(note = "prepare_delete rarely used, please use the delete instead")]
    pub fn prepare_delete(&self, path: &str) -> RequestBuilder {
        self.delete(path)
    }

    #[deprecated(note = "prepare_patch rarely used, please use the patch instead")]
    pub fn prepare_patch(&self, path: &str) -> RequestBuilder {
        self.patch(path)
    }

    #[deprecated(note = "prepare_put rarely used, please use the put instead")]
    pub fn prepare_put(&self, path: &str) -> RequestBuilder {
        self.put(path)
    }

    pub fn basic_auth(
        &self,
        method: reqwest::Method,
        username: &str,
        password: &str,
        path: &str,
    ) -> RequestBuilder {
        let authorization = reqwest::header::AUTHORIZATION;
        let base64format = general_purpose::STANDARD.encode(&format!("{}:{}", username, password));
        let auth_format = format!("Basic {}", base64format);
        self.build_request(method, path)
            .header(authorization, auth_format)
    }

    pub fn bearer_token(&self, method: reqwest::Method, token: &str, path: &str) -> RequestBuilder {
        let authorization = reqwest::header::AUTHORIZATION;
        let bearer_format = format!("Bearer {:?}", token);
        self.build_request(method, path)
            .header(authorization, bearer_format)
    }

    pub async fn prepare_request(
        &self,
        method: reqwest::Method,
        path: &str,
        query_params: Option<&HashMap<&str, &str>>,
        json_body: Option<&Value>,
        headers: Option<&HashMap<&str, &str>>,
        form_data: Option<&bool>,
        filename: Option<&str>,
    ) -> Result<RequestBuilder, Box<dyn std::error::Error>> {
        let mut prepared_request: RequestBuilder = self.build_request(method, path);

        if let Some(params) = query_params {
            let query_string = to_string(params).expect("Failed to encode the query parameters");
            prepared_request = prepared_request.query(&query_string);
        }

        if let Some(body) = json_body {
            // prepared_request = prepared_request.body(serde_json::to_vec(body)?);
            prepared_request = prepared_request.json(body);
        }

        if let Some(headers) = headers {
            for (key, value) in headers {
                prepared_request = prepared_request.header(*key, *value);
            }
        }

        // let mut form = Form::new();
        // hacky-way, seems unstable since when it ran the program,
        // it also get the error message although the form-data option
        // was set to False
        if let (Some(_), Some(filename)) = (form_data, filename) {
            let form_data = File::open(filename).await?;
            prepared_request = prepared_request.body(self.convert_file_to_body(form_data));
        }

        Ok(prepared_request)
    }

    fn convert_file_to_body(&self, file: File) -> Body {
        let body_stream = FramedRead::new(file, BytesCodec::new());
        let body = Body::wrap_stream(body_stream);
        body
    }

    pub async fn retry_request_builder<F>(
        &self,
        request_build_provider: F,
        max_retries: Option<usize>,
        backoff: Option<Duration>,
    ) -> Result<Response, HttpRequestError>
    where
        F: Fn() -> RequestBuilder,
    {
        let mut retries = 0;
        let max_retries = max_retries.unwrap_or(3);
        let backoff = backoff.unwrap_or(Duration::from_secs(1));

        loop {
            // again, hacky-way. i actually don't know why the clone method
            // it doesn't appears since i need to moved the closure of RequestBuilder
            match self.send_request(request_build_provider()).await {
                Ok(response) => return Ok(response),
                Err(error) => {
                    retries += 1;
                    if retries >= max_retries {
                        return Err(error);
                    }

                    tokio::time::sleep(backoff).await;
                }
            }
        }
    }

    // getter method looks-like that will be returned
    // each piece of HTTP response information
    pub async fn status_code(
        &self,
        request: RequestBuilder,
    ) -> Result<reqwest::StatusCode, HttpRequestError> {
        let response = self.send_request(request).await?;
        Ok(response.status())
    }

    pub async fn headers(
        &self,
        request: RequestBuilder,
    ) -> Result<reqwest::header::HeaderMap, HttpRequestError> {
        let response = self.send_request(request).await?;
        Ok(response.headers().clone())
    }

    pub async fn elapsed_time(
        &self,
        request: RequestBuilder,
    ) -> Result<Duration, HttpRequestError> {
        let start_time = Instant::now();
        let request_time = self.send_request(request).await?;
        Ok(start_time.elapsed())
    }

    pub async fn json(
        &self,
        request: RequestBuilder,
    ) -> Result<Value, HttpRequestError> {
        let response = self.send_request(request).await?;
        let resp_body = response.text().await?;
        serde_json::from_str(&resp_body).map_err(HttpRequestError::JsonError)
    }
}
