use crate::error::get_error_cause;
use crate::user::NewUser;
use crate::user::User;

use color_eyre::Report;
use std::collections::HashMap;
use std::fmt::Debug;

type UserId = u64;

#[derive(Clone, Debug, Default)]
pub struct UserRepository(HashMap<UserId, User>);

impl UserRepository {
    pub fn new() -> Self {
        Self(HashMap::default())
    }

    /// Create a new [User] from [NewUser].
    pub fn create(&mut self, new_user: &NewUser) -> Result<User, UserRepoError> {
        self.email_is_available(&new_user.email)?;
        tracing::info!("Email Is Free");
        let new_id = self.random_id();
        let new_user = User::new(new_id, new_user.email.to_owned());
        // This will always return ``None``:
        self.0.insert(new_user.id, new_user.clone());
        tracing::info!("New User Inserted to DB");
        Ok(new_user)
    }

    /// Get all users.
    pub fn list(&self) -> Result<Vec<User>, UserRepoError> {
        let users = self.0.values().cloned().collect();
        Ok(users)
    }

    /// Returns [User] with id ``id``; *otherwise* `NotFound`.
    pub fn get(&self, user_id: u64) -> Result<User, UserRepoError> {
        self.0.get(&user_id).cloned().ok_or(UserRepoError::UserNotFound(user_id))
    }

    fn random_id(&self) -> UserId {
        self.0.keys().max().map_or(1, |id| id + 1)
    }

    fn email_is_available(&self, new_email: &str) -> Result<(), UserRepoError> {
        match self.0.values().any(|user| user.email == new_email) {
            true => Err(UserRepoError::EmailTaken(new_email.to_string())),
            false => Ok(()),
        }
    }
}

#[derive(thiserror::Error)]
pub enum UserRepoError {
    #[error("User {0} Not Found")]
    UserNotFound(u64),
    #[error("Email {0} Is Taken")]
    EmailTaken(String),
    #[error(transparent)]
    Internal(#[from] Report),
}

impl Debug for UserRepoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        get_error_cause(self, f)
    }
}
