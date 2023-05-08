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
use hyper::StatusCode;
use std::fmt::Debug;
use tracing::instrument;

#[instrument(
    name = "Registering New User",
    fields(email = new_user.email),
    skip(service, new_user))]
pub async fn create_user(
    service: Extension<SharedService>,
    Json(new_user): Json<NewUser>,
) -> Result<Json<User>, CreateUserError> {
    let mut service = service.write().await;
    let user = service.create(&new_user);

    match user {
        Ok(user) => Ok(Json(user)),
        Err(e) => {
            let e = match e {
                ServiceError::EmailTaken(_) => CreateUserError::DuplicateEmail(e),
                ServiceError::ValidationError(_) => CreateUserError::Validation(e),
                ServiceError::Internal(e) => CreateUserError::Internal(e),
                _otherwise => unreachable!(),
            };
            tracing::warn!("Error while registering new user: {:#?}", e);
            Err(e)
        }
    }
}

#[derive(thiserror::Error)]
pub enum CreateUserError {
    /// Duplicate email. The email was registered previously.
    #[error("Email Already In Use")]
    DuplicateEmail(#[source] ServiceError),

    /// Email might be gibberish.
    #[error("Validation error while processing data: {0}")]
    Validation(#[source] ServiceError),

    /// !Business & !Use-Case Errors
    #[error(transparent)]
    Internal(#[from] OpaqueError),
}

impl CreateUserError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::DuplicateEmail(_) => StatusCode::CONFLICT,
            Self::Validation(_) => StatusCode::BAD_REQUEST,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for CreateUserError {
    fn into_response(self) -> axum::response::Response {
        let (status, error) = (self.status_code(), self.to_string());
        (status, Json(ErrorBody { error })).into_response()
    }
}

impl Debug for CreateUserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error::get_error_cause(self, f)
    }
}
