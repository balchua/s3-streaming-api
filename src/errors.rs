use aws_sdk_s3::error::SdkError;
use aws_sdk_s3::operation::put_object::PutObjectError;
use axum::extract::multipart::MultipartError;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use std::io::Error as IoError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StreamError {
    #[error("Error while uploading file: {0}")]
    UploadFileError(#[from] SdkError<PutObjectError>),

    #[error("Error while getting data from multipart: {0}")]
    Multipart(#[from] MultipartError),

    #[error("IO error: {0}")]
    IO(#[from] IoError),

    #[allow(dead_code)]
    #[error("Body is empty")]
    EmptyBody, // the user tried to send an empty body while uploading
}

impl IntoResponse for StreamError {
    fn into_response(self) -> Response {
        let response = match self {
            Self::UploadFileError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            Self::Multipart(e) => (StatusCode::BAD_REQUEST, e.to_string()),
            Self::IO(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            Self::EmptyBody => (StatusCode::BAD_REQUEST, self.to_string()),
        };

        response.into_response()
    }
}
