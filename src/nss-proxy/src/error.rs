use axum::body::Body;
use axum::response::{IntoResponse, Response};
use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq)]
pub enum ErrorType {
    BadRequest,
    Connection,
    Generic,
    Internal,
}

#[derive(Debug)]
pub struct Error {
    pub error: ErrorType,
    pub message: String,
}

impl Display for ErrorType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "error: {} message: {}", self.error, self.message)
    }
}

impl Error {
    pub fn new<M>(error: ErrorType, message: M) -> Self
    where
        M: ToString,
    {
        Self {
            error: error.into(),
            message: message.to_string(),
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let status = match self.error {
            ErrorType::BadRequest => 400,
            ErrorType::Connection => 500,
            ErrorType::Generic => 400,
            ErrorType::Internal => 500,
        };

        Response::builder()
            .status(status)
            .body(Body::from(self.message))
            .unwrap()
            .into_response()
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::new(ErrorType::BadRequest, format!("IO Error: {}", value))
    }
}

impl From<std::fmt::Error> for Error {
    fn from(value: std::fmt::Error) -> Self {
        Error::new(ErrorType::Internal, value.to_string())
    }
}

impl From<Error> for std::io::Error {
    fn from(value: Error) -> Self {
        Self::other(value.message)
    }
}

impl From<axum::Error> for Error {
    fn from(value: axum::Error) -> Self {
        Self::new(ErrorType::Connection, value.to_string())
    }
}

impl From<reqwest::header::ToStrError> for Error {
    fn from(value: reqwest::header::ToStrError) -> Self {
        Error::new(
            ErrorType::BadRequest,
            format!(
                "Request headers contained non ASCII characters: {:?}",
                value
            ),
        )
    }
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Error::new(
            ErrorType::Connection,
            format!("Cannot send out HTTP request: {:?}", value),
        )
    }
}
