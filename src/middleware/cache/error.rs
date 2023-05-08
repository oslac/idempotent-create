use crate::error::get_error_cause;

use axum::response::IntoResponse;
use axum::response::Response;
use color_eyre::Report;
use hyper::StatusCode;
use std::fmt::Debug;

#[derive(thiserror::Error)]
pub enum CacheLayerError {
    #[error("Error While Processing Uncached Request: {0}")]
    Internal(#[from] Report),
}

impl Debug for CacheLayerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        get_error_cause(self, f)
    }
}

impl IntoResponse for CacheLayerError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
    }
}
