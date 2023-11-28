use reqwest::Error as ReqwestError;

#[derive(Debug)]
pub enum HttpRequestError {
    RequestBuilderError(ReqwestError),
}

impl From<ReqwestError> for HttpRequestError {
    fn from(error: ReqwestError) -> Self {
        HttpRequestError::RequestBuilderError(error)
    }
}