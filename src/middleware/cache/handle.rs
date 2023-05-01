use super::msg::Msg;
use crate::ikey::IKey;
use crate::warehouse::CacheError;
use crate::warehouse::CachedResponse;

use color_eyre::eyre::Context;
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot;
use tokio::sync::oneshot::channel;

/// Handler to *command* and *query* the [CacheManager].
/// This thing does some cloning.
#[derive(Debug, Clone)]
pub struct CacheHandle {
    sender: Sender<Msg>,
}

impl CacheHandle {
    pub fn new(sender: Sender<Msg>) -> Self {
        Self { sender }
    }

    /// Given [IKey] exists in the cache, returns a [CachedResponse];
    /// *otherwise* `None`.
    #[tracing::instrument(name = "Check Cache for Response")]
    pub async fn get(&self, key: &IKey) -> Option<CachedResponse> {
        let key = key.clone();
        let (ret, res) = channel();
        let msg = Msg::Get { key, ret };

        self.sender.send(msg).await.context("Receiver Was Dropped").expect("Graceful Shutdown");
        tracing::info!("Get Sent");

        let res = res.await.context("Failed to Receive Response").unwrap();
        tracing::info!("Get Response Recv");
        res
    }

    /// Maps `key` to `val` in [Cache]; *otherwise* returns a [CacheError].
    #[tracing::instrument]
    pub async fn set(&self, key: &IKey, val: &CachedResponse) -> Result<(), CacheError> {
        let key = key.clone();
        let val = val.clone();
        let (ret, res) = oneshot::channel();
        let msg = Msg::Set { key, val, ret };

        self.sender.send(msg).await.context("CW9K: Receiver Broken").expect("Graceful Shutdown");
        tracing::info!("Set Sent");

        let res = res.await.context("Failed to Receive Response").unwrap();
        tracing::info!("Set Response Recv");
        res
    }
}
