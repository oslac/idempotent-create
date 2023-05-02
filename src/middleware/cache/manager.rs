//! A task manager that handles access to [Cache].
use super::msg::Msg;
use crate::warehouse::Cache;

use tokio::sync::mpsc::Receiver;

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

    /// Processes messages from the channel until all senders are dropped
    #[tracing::instrument(name = "Running Cache")]
    pub async fn run(&mut self) {
        while let Some(mail) = self.mailbox.recv().await {
            tracing::info!("Mail {mail} Received");
            use Msg::*;
            match mail {
                Get { key, ret } => {
                    tracing::info!("Processing GET");
                    let cached_response = self.cache.get(&key).await.ok();
                    tracing::warn!("GET executed");
                    ret.send(cached_response);
                }

                Set { key, val, ret } => {
                    tracing::info!("Processing SET");
                    let res = self.cache.set(&key, &val).await;
                    tracing::warn!("SET executed");
                    ret.send(res);
                }
            }
        }
    }
}
