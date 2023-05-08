//! A task manager that handles access to [Cache].
use super::msg::Msg;
use crate::warehouse::Cache;

use tokio::sync::mpsc::Receiver;
use tracing::instrument;

/// Processes the received [Msg]s from the
/// [CacheHandle](crate::middleware::cache::handle::CacheHandle).
#[derive(Debug)]
pub struct CacheManager {
    mailbox: Receiver<Msg>,
    cache: Cache,
}

impl CacheManager {
    pub fn new(mailbox: Receiver<Msg>) -> Self {
        let cache = Cache::new();
        Self { mailbox, cache }
    }

    /// Processes messages from the channel until all senders are dropped.
    #[instrument(name = "Cache Manager", skip(self))]
    pub async fn run(&mut self) {
        while let Some(mail) = self.mailbox.recv().await {
            tracing::info!("Received Request: {}", mail);
            use Msg::*;
            match mail {
                Get { key, ret } => {
                    let cached_response = self.cache.get(&key).await.ok();
                    tracing::warn!("Response Fetched");
                    ret.send(cached_response).expect("Graceful Shutdown");
                }

                Set { key, val, ret } => {
                    let res = self.cache.set(&key, &val).await;
                    tracing::warn!("Response Updated");
                    ret.send(res).expect("Graceful Shutdown");
                }
            }
        }
    }
}
