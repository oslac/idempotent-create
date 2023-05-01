//! Middleware to extract [IKey] from request headers.
use crate::ikey::IKey;

use axum::extract::FromRequestParts;
use hyper::http::request::Parts;
use hyper::StatusCode;

#[axum::async_trait]
impl<X> FromRequestParts<X> for IKey
where
    X: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _state: &X) -> Result<Self, Self::Rejection> {
        let header = parts.headers.get(IKey::HEADER);
        let value = header.and_then(|value| value.to_str().ok());
        let Some(value) = value else {
            return Err((StatusCode::INTERNAL_SERVER_ERROR, "MISSING KEY".to_string()));
        };
        let ikey = IKey::try_from(value.to_string())
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
        Ok(ikey)
    }
}
