use crate::error::get_error_cause;
use crate::error::ErrorBody;
use crate::error::OpaqueError;
use crate::service::SharedService;
use crate::user::User;

use axum::response::IntoResponse;
use axum::Extension;
use axum::Json;
use color_eyre::eyre::Context;
use hyper::StatusCode;
use std::fmt::Debug;
use tracing::instrument;

type Users = Json<Vec<User>>;

#[instrument(name = "Get All Users", skip(service))]
pub async fn get_users(service: Extension<SharedService>) -> Result<Users, ListUsersError> {
    tracing::info!("Fetching All Users");
    let service = service.read().await;
    let mut users = service.list().await.context("Failed to get all users")?;
    users.sort();
    tracing::info!("All Users Fetched");
    Ok(Json(users))
}

#[derive(thiserror::Error)]
pub enum ListUsersError {
    #[error(transparent)]
    Unexpected(#[from] OpaqueError),
}

impl IntoResponse for ListUsersError {
    fn into_response(self) -> axum::response::Response {
        let (status, error) = match &self {
            ListUsersError::Unexpected(error) => {
                (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
            }
        };

        (status, Json(ErrorBody { error })).into_response()
    }
}

impl Debug for ListUsersError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        get_error_cause(self, f)
    }
}
