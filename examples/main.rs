// use reqwest::Response;
use rs_http_request::HttpRequest;
// use rs_http_request::HttpResponse;
use serde_json::json;

// fn main() {
//     println!("Hello, world!");
// }
// #[tokio::main]

#[tokio::main]
async fn main() {
    let base_url = "https://jsonplaceholder.typicode.com";
    let http_request = HttpRequest::new(base_url, false);
    
    // use the request builder and prepare request
    let prepare_request = http_request.prepare_get("/posts/1");
    let response = http_request.send_request(prepare_request).await;
    match response {
        Ok(resp) => {
            println!("Status: {}", resp.status());
            println!("Headers: {:?}", resp.headers());
            println!("Response Body: {:?}", resp.text().await);
        }
        Err(err) => eprintln!("Error: {:?}", err),
    }

    // directly call the HTTP method
    let get_request = http_request.get("/posts");
    let resp = get_request.send().await;
    match resp {
        Ok(resp) => {
            println!("Status: {}", resp.status());
        }
        Err(err) => eprintln!("Error: {:?}", err),
    }

    // making the POST request with JSON body as the payload
    let request_body_as_json = json!({"title": "foobar"});
    let post_response = http_request
        .prepare_request(
            reqwest::Method::POST,
            "/posts",
            None,
            Some(&request_body_as_json),
            None,
            Some(&false),
            Some(""),
        )
        .await
        .and_then(|req| Ok(http_request.send_request(req)));

    match post_response {
        Ok(post_response) => {
            println!("Status: {:?}", post_response.await);
        }
        // i need to do this since it will raise the `NotFoundDirectory`
        // error message when i passed the argument on here
        Err(_) => {}
    }

    // usage of the HttpResponse structure
    let delete_request = http_request.prepare_delete("/posts/1");
    let struct_resp = http_request.send_request(delete_request).await;
    match struct_resp {
        Ok(http_response) => {
            println!("Status Code: {}", http_response.status());
            println!("Response Body: {:?}", http_response.text().await);
            // println!("Headers: {:?}", HttpResponse.headers());
        }
        Err(err) => eprintln!("Error: {:?}", err),
    }

    // prompt the authentication with bearer token
    let base_auth_url = "https://httpbin.org";
    // let auth_request = http_request.get("/bearer");
    let http_auth_request = HttpRequest::new(base_auth_url, false);
    let auth_response = http_auth_request
        .bearer_token(reqwest::Method::GET, "1234", "/bearer")
        .send()
        .await;
    match auth_response {
        Ok(auth_response) => {
            let status_code = auth_response.status();
            if status_code.is_success() {
                let body = auth_response
                    .text()
                    .await
                    .unwrap_or_else(|_| String::from("Failed to read the response body"));
                println!("Successfully Authenticated");
                println!("Status Code: {}", status_code);
                println!("Response Body: {:?}", body);
            } else {
                eprintln!(
                    "Failed to Authenticate: {:?}",
                    auth_response.error_for_status()
                );
                eprintln!("Status Code: {}", status_code);
            }
        }
        Err(err) => eprintln!("Error: {:?}", err),
    }
}
