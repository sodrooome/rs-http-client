use serde_json::json;

#[cfg(test)]
mod tests {
    use rs_http_request::HttpRequest;
    use rs_http_request::HttpRequestHook;
    use rs_http_request::HttpResponseHook;

    use super::*;
    use std::collections::HashMap;
    use std::time::Duration;

    #[tokio::test]
    async fn test_send_request() {
        let base_url = "https://jsonplaceholder.typicode.com";
        let request = HttpRequest::new(base_url, false);
        let get_request = request.get("/posts/1");
        let response = get_request.send().await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_get_status_code() {
        let base_url = "https://jsonplaceholder.typicode.com";
        let request = HttpRequest::new(base_url, false);
        let get_request = request.get("/posts/1");
        let status_code = request.status_code(get_request).await;
        status_code.expect("Successfully to get the status code");
        // let expected_status_code = reqwest::StatusCode::OK;
        // assert_eq!(status_code, expected_status_code);
    }

    #[tokio::test]
    async fn test_failure_request() {
        let base_url = "https://jsonplaceholder.typicode.com";
        let request = HttpRequest::new(base_url, false);
        let req_payload = json!({"title": "foobar"});
        let post_request = request
            .prepare_request(
                reqwest::Method::POST,
                "/post",
                None,
                Some(&req_payload),
                None,
                Some(&false),
                Some(""),
            )
            .await
            .and_then(|req| Ok(request.send_request(req)));
        assert!(post_request.is_err());
    }

    #[tokio::test]
    async fn test_custom_headers() {
        let base_url = "https://jsonplaceholder.typicode.com";
        let request = HttpRequest::new(base_url, true);
        let mut custom_headers = HashMap::new();
        custom_headers.insert("key", "value1");
        let response = request
            .prepare_request(
                reqwest::Method::GET,
                "/posts",
                None,
                None,
                Some(&custom_headers),
                Some(&false),
                Some(""),
            )
            .await
            .and_then(|req| Ok(request.send_request(req)));

        assert!(response.is_err());
    }

    #[tokio::test]
    async fn test_delete_existing_records() {
        let base_url = "https://jsonplaceholder.typicode.com";
        let del_request = HttpRequest::new(base_url, false);
        let response = del_request.delete("/posts/1").send().await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_basic_auth() {
        let base_url = "https://httpbin.org/basic-auth/user/passwd";
        let request = HttpRequest::new(base_url, false);
        let username = "user";
        let password = "passwd";
        let timeout = Duration::new(5, 0);
        let response = request
            .basic_auth(reqwest::Method::GET, username, password, "")
            .timeout(timeout)
            .send()
            .await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_request_retry() {
        let base_url = "https://httpbin.org/status/";
        let request = HttpRequest::new(base_url, true);
        let max_retries = 3;
        let backoff = Duration::new(3, 0);

        // got weird issue, if the prefix contains "/"
        // it will be removed the previous path
        let response = request
            .retry_request_builder(|| request.get("500"), Some(max_retries), Some(backoff))
            .await;
        let resp_body = response.unwrap();
        assert_eq!(resp_body.status(), 500);
        // assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_hooks() {
        let base_url = "https://jsonplaceholder.typicode.com";
        let mut request = HttpRequest::new(base_url, false);

        // initiate the instances of request-response hook first
        let request_hook = Box::new(HttpRequestHook);
        let response_hook = Box::new(HttpResponseHook);

        request.set_request_hook(request_hook);

        let get_request = request.get("/posts/1");
        let response = request.send_request(get_request).await;

        request.set_response_hook(response_hook);

        assert!(response.is_ok());
    }
}
