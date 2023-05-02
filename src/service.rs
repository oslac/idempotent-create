use crate::error::get_error_cause;
use crate::error::OpaqueError;
use crate::user::NewUser;
use crate::user::User;
use crate::warehouse::UserRepoError;
use crate::warehouse::UserRepository;

use color_eyre::eyre::Context;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::RwLock;

pub type SharedService = Arc<RwLock<Service>>;

#[derive(Clone, Debug)]
pub struct Service {
    pub db: UserRepository,
}

impl Service {
    pub fn new(db: UserRepository) -> Self {
        Self { db }
    }

    #[tracing::instrument]
    pub async fn create(&mut self, new_user: &NewUser) -> Result<User, ServiceError> {
        Self::validate_email(&new_user.email)?;
        tracing::info!("Email Validated");
        let user = self.db.create(new_user).map_err(ServiceError::EmailTaken)?;
        tracing::info!("User Created");
        Ok(user)
    }

    #[tracing::instrument]
    pub async fn get(&self, id: &str) -> Result<User, ServiceError> {
        let id = id
            .parse::<u64>()
            .map_err(|e| ServiceError::ValidationError(format!("{e} is not a valid user id")))?;
        tracing::info!("ID Validated");
        let user = self.db.get(id).map_err(ServiceError::UserNotFound)?;
        tracing::info!("User {} Fetched", user.id);
        Ok(user)
    }

    #[tracing::instrument(skip(self))]
    pub async fn list(&self) -> Result<Vec<User>, ServiceError> {
        let users = self.db.list().context("Failed to Get All Users")?;
        tracing::info!("Users Fetched");
        Ok(users)
    }

    fn validate_email(email: &str) -> Result<(), ServiceError> {
        let invalid_email = email.is_empty() || email.len() < 5;
        if invalid_email {
            return Err(ServiceError::ValidationError("Email Empty".to_string()));
        }

        Ok(())
    }
}

#[derive(thiserror::Error)]
pub enum ServiceError {
    // User was not found:
    #[error("User {0} Not Found")]
    UserNotFound(#[source] UserRepoError),

    /// Email is taken:
    #[error("Email {0} Is In Use Already")]
    EmailTaken(#[source] UserRepoError),

    /// Something about the payload didn' fit into business rules:
    #[error("Validation Error: {0}")]
    ValidationError(String),

    // Internal Errors (everything else returned by lower layers that don't map to business
    // errors).
    #[error(transparent)]
    UnexpectedError(#[from] OpaqueError),
}

impl Debug for ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        get_error_cause(self, f)
    }
}
