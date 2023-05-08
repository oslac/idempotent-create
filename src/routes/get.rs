use crate::error::get_error_cause;
use crate::error::ErrorBody;
use crate::service::ServiceError;
use crate::service::SharedService;
use crate::user::User;

use axum::extract::Path;
use axum::response::IntoResponse;
use axum::Extension;
use axum::Json;
use color_eyre::Report;
use hyper::StatusCode;
use std::fmt::Debug;

#[tracing::instrument(skip(service))]
pub async fn get_user(
    Path(key): Path<String>,
    service: Extension<SharedService>,
) -> Result<Json<User>, GetUserErrors> {
    let service = service.read().await;
    let user = service.get(&key).await?;
    tracing::info!("User `{user}` found");
    Ok(Json(user))
}

#[derive(thiserror::Error)]
pub enum GetUserErrors {
    #[error("User {0} Not Found")]
    UserNotFound(#[source] ServiceError),
    #[error("Invalid User ID {0}")]
    InvalidUserId(#[source] ServiceError),
    #[error(transparent)]
    Internal(#[from] Report),
}

impl GetUserErrors {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::UserNotFound(_) => StatusCode::NOT_FOUND,
            Self::InvalidUserId(_) => StatusCode::BAD_REQUEST,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for GetUserErrors {
    fn into_response(self) -> axum::response::Response {
        let (status, error) = (self.status_code(), self.to_string());
        let body = Json(ErrorBody { error });
        (status, body).into_response()
    }
}

impl From<ServiceError> for GetUserErrors {
    fn from(value: ServiceError) -> Self {
        use ServiceError::*;
        match value {
            UserNotFound(_) => Self::UserNotFound(value),
            ValidationError(_) => Self::InvalidUserId(value),
            Internal(e) => Self::Internal(e),
            _ => unreachable!(),
        }
    }
}

impl Debug for GetUserErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        get_error_cause(self, f)
    }
}
