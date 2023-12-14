use reqwest::Error as ReqwestError;
use serde_json::Error as SerdeJsonError;

#[derive(Debug)]
pub enum HttpRequestError {
    RequestBuilderError(ReqwestError),
    JsonError(SerdeJsonError),
}

impl From<ReqwestError> for HttpRequestError {
    fn from(error: ReqwestError) -> Self {
        HttpRequestError::RequestBuilderError(error)
    }
}

impl From<SerdeJsonError> for HttpRequestError {
    fn from(error: SerdeJsonError) -> Self {
        HttpRequestError::JsonError(error)
    }
}