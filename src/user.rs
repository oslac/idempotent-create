use std::fmt::Display;

use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Deserialize)]
pub struct NewUser {
    pub email: String,
}

impl NewUser {
    pub fn new(email: String) -> Self {
        Self { email }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct User {
    pub id: u64,
    pub email: String,
}

impl User {
    pub fn new(id: u64, email: String) -> Self {
        Self { id, email }
    }
}

impl Display for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.id, self.email)
    }
}
