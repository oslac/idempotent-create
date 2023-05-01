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

pub async fn get_user(
    Path(key): Path<String>,
    service: Extension<SharedService>,
) -> Result<Json<User>, GetUserErrors> {
    let db = service.read().await;
    let maybe = db.get(&key).await;

    match maybe {
        Ok(user) => {
            tracing::info!("User ``{}`` found", user.id);
            Ok(Json(user))
        }
        Err(ServiceError::UserNotFound(_)) => {
            tracing::info!("User ``{}`` not found", key);
            Err(GetUserErrors::UserNotFound(key))
        }

        Err(ServiceError::ValidationError(e)) => {
            tracing::warn!("{:#?}", e);
            Err(GetUserErrors::InvalidUserId(key))
        }

        Err(otherwise) => {
            tracing::warn!("{:#?}", otherwise);
            Err(GetUserErrors::Unexpected(color_eyre::eyre::eyre!(otherwise)))
        }
    }
}

#[derive(thiserror::Error)]
pub enum GetUserErrors {
    #[error("User {0} Not Found")]
    UserNotFound(String),
    #[error("Invalid User ID {0}")]
    InvalidUserId(String),
    #[error(transparent)]
    Unexpected(#[from] Report),
}

impl IntoResponse for GetUserErrors {
    fn into_response(self) -> axum::response::Response {
        use GetUserErrors::*;
        let (status, error) = match &self {
            UserNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            InvalidUserId(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            Unexpected(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };
        let body = Json(ErrorBody { error });
        (status, body).into_response()
    }
}

impl Debug for GetUserErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        get_error_cause(self, f)
    }
}
