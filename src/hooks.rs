use reqwest::RequestBuilder;
use reqwest::Response;

pub trait RequestHook {
    fn apply(&self, request: &RequestBuilder);
}

pub trait ResponseHook {
    fn apply(&self, response: &Response);
}

pub struct HttpRequestHook;

pub struct HttpResponseHook;

impl RequestHook for HttpRequestHook {
    fn apply(&self, request: &RequestBuilder) {
        println!("Request hook before sending the request: {:?}", request);
    }
}


// dispatches the response hook after sending the request
// the design system is referred to the httpx package
impl ResponseHook for HttpResponseHook {
    fn apply(&self, response: &Response) {
        println!("Response hook after sending the request: {:?}", response);
    }
}
