use crate::error;
use crate::error::ErrorBody;
use crate::error::OpaqueError;
use crate::service::ServiceError;
use crate::service::SharedService;
use crate::user::NewUser;
use crate::user::User;

use axum::response::IntoResponse;
use axum::Extension;
use axum::Json;
use color_eyre::eyre;
use hyper::StatusCode;
use std::fmt::Debug;

pub async fn create_user(
    service: Extension<SharedService>,
    Json(new_user): Json<NewUser>,
) -> Result<Json<User>, CreateUserError> {
    let mut service = service.write().await;
    let res = service.create(&new_user).await;

    match res {
        Ok(new_user) => {
            tracing::info!("New User Created");
            Ok(Json(new_user))
        }
        Err(error) => {
            tracing::info!("{:#?}", error);
            Err(error.into())
        }
    }
}

#[derive(thiserror::Error)]
pub enum CreateUserError {
    /// Email  might be taken.
    #[error("{0}")]
    EmailTaken(#[source] ServiceError),
    /// Email might be malformed
    #[error("{0}")]
    Validation(#[source] ServiceError),
    /// Internal (!Business & !Use-Case Errors)
    #[error(transparent)]
    Internal(#[from] OpaqueError),
}

impl IntoResponse for CreateUserError {
    fn into_response(self) -> axum::response::Response {
        use CreateUserError::*;
        let (status, error) = match self {
            EmailTaken(e) => (StatusCode::CONFLICT, e.to_string()),
            Validation(e) => (StatusCode::BAD_REQUEST, e.to_string()),
            Internal(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        };

        let body = Json(ErrorBody { error });
        (status, body).into_response()
    }
}

impl From<ServiceError> for CreateUserError {
    fn from(value: ServiceError) -> Self {
        match value {
            ServiceError::EmailTaken(_) => Self::EmailTaken(value),
            ServiceError::ValidationError(_) => Self::Validation(value),
            ServiceError::UnexpectedError(internal) => Self::Internal(eyre::eyre!(internal)),
            _ => unreachable!(),
        }
    }
}

impl Debug for CreateUserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error::get_error_cause(self, f)
    }
}
