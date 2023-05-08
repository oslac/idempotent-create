use super::msg::Msg;
use crate::ikey::IKey;
use crate::warehouse::CacheError;
use crate::warehouse::CachedResponse;

use color_eyre::eyre::Context;
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot;
use tokio::sync::oneshot::channel;
use tracing::instrument;

/// A handler to *command* and *query* the
/// [CacheManager](crate::middleware::cache::manager::CacheManager).
#[derive(Debug, Clone)]
pub struct CacheHandle {
    mailman: Sender<Msg>,
}

impl CacheHandle {
    pub fn new(mailman: Sender<Msg>) -> Self {
        Self { mailman }
    }

    /// Given `key` exists in a cache, returns some [CachedResponse];
    /// *otherwise* returns `None`.
    #[instrument(name = "Get a Response", skip(self), fields(key=key.0))]
    pub async fn get(&self, key: &IKey) -> Option<CachedResponse> {
        let key = key.clone();
        let (ret, res) = channel();
        let msg = Msg::Get { key, ret };

        self.mailman.send(msg).await.context("Receiver Was Dropped").unwrap();
        let response = res.await.context("Failed to Receive Response").unwrap();

        tracing::info!("Response received");
        response
    }

    /// Maps `key` to `val` in [Cache](crate::warehouse::Cache);
    /// *otherwise* returns a [CacheError].
    #[instrument(name = "Update a Response", fields(key = key.0), skip(self, key, val))]
    pub async fn set(&self, key: &IKey, val: &CachedResponse) -> Result<(), CacheError> {
        let key = key.clone();
        let val = val.clone();
        let (ret, res) = oneshot::channel();
        let msg = Msg::Set { key, val, ret };

        self.mailman.send(msg).await.context("Receiver Was Dropped")?;
        let res = res.await.context("Failed to Receive Response")?;
        tracing::info!("Update Received");
        res
    }
}
