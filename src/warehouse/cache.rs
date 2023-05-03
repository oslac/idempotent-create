use crate::error::get_error_cause;
use crate::ikey::IKey;
use crate::user::User;

use axum::http::StatusCode;
use color_eyre::Report;
use std::collections::HashMap;
use std::fmt::Debug;
use std::fmt::Display;

/// A response cache, mapping client provided [IKey] to [CachedResponse].
#[derive(Clone, Debug, Default)]
pub struct Cache(HashMap<IKey, CachedResponse>);

#[derive(Clone, Debug)]
pub struct CachedResponse {
    pub status: StatusCode,
    pub user: User,
}

#[derive(thiserror::Error)]
pub enum CacheError {
    #[error("Cache for {0}")]
    CacheMiss(String),
    #[error(transparent)]
    Unexpected(#[from] Report),
}

impl Cache {
    pub fn new() -> Self {
        Self(HashMap::default())
    }

    /// Given a [IKey] and [CachedResponse], performs an upsert.
    pub async fn set(&mut self, key: &IKey, res: &CachedResponse) -> Result<(), CacheError> {
        let res = res.clone();
        let key = key.clone();
        self.0.insert(key, res);
        Ok(())
    }

    /// Given an [IKey], either returns a [CachedResponse] on a cache hit, or
    /// [CacheError] on miss.
    pub async fn get(&self, key: &IKey) -> Result<CachedResponse, CacheError> {
        let res = self.0.get(key).ok_or(CacheError::CacheMiss(key.to_string()))?;
        Ok(res.clone())
    }
}

impl Debug for CacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        get_error_cause(self, f)
    }
}

impl Display for CachedResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Cached ({}, {})", self.status, self.user.id)
    }
}
