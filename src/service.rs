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
use tracing::instrument;

pub type SharedService = Arc<RwLock<Service>>;

// FIXME fake class-like thing
#[derive(Clone, Debug)]
pub struct Service {
    pub db: UserRepository,
}

impl Service {
    pub fn new(db: UserRepository) -> Self {
        Self { db }
    }

    #[instrument(name = "Create New User", skip(self, new_user))]
    pub fn create(&mut self, new_user: &NewUser) -> Result<User, ServiceError> {
        Self::validate_email(&new_user.email)?;
        tracing::info!("User Email is Valid");
        let user = self.db.create(new_user).map_err(|e| {
            tracing::error!("{:#?}", e);
            ServiceError::EmailTaken(e)
        })?;
        tracing::info!("New User Created");
        Ok(user)
    }

    #[tracing::instrument(name = "Get User", skip(self))]
    pub async fn get(&self, id: &str) -> Result<User, ServiceError> {
        let id = id
            .parse::<u64>()
            .map_err(|e| ServiceError::ValidationError(format!("{e} is not a valid user id")))?;
        tracing::info!("User ID Validated Successfully");
        let user = self.db.get(id).map_err(ServiceError::UserNotFound)?;
        tracing::info!("User {} Fetched", user.id);
        Ok(user)
    }

    #[tracing::instrument(name = "Get All Users", skip(self))]
    pub async fn list(&self) -> Result<Vec<User>, ServiceError> {
        let users = self.db.list().context("Failed to Get All Users")?;
        tracing::info!("Users Fetched");
        Ok(users)
    }

    fn validate_email(email: &str) -> Result<(), ServiceError> {
        let invalid_email = email.is_empty() || email.len() < 5;
        if invalid_email {
            tracing::error!("Invalid Email {:#?}", email);
            return Err(ServiceError::ValidationError("Email Empty".to_string()));
        }

        Ok(())
    }
}

#[derive(thiserror::Error)]
pub enum ServiceError {
    #[error("Service: Database failure was encountered while trying to create new user.")]
    EmailTaken(#[source] UserRepoError),

    /// This user was not found.
    #[error("Service: User {0} Not Found")]
    UserNotFound(#[source] UserRepoError),

    /// The provided Email or UserID was somehow malformed.
    #[error("Service: Validation Failed with: {0}")]
    ValidationError(String),

    /// !Business !Use-Case Errors
    #[error(transparent)]
    Internal(#[from] OpaqueError),
}

impl Debug for ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        get_error_cause(self, f)
    }
}
