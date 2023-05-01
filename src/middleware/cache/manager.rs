//! A task manager that handles access to [Cache].
use super::msg::Msg;
use crate::warehouse::Cache;

use tokio::sync::mpsc::Receiver;

/// Processes the [Msg]s from the channel.
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

    /// Runs the [CacheManager], processing messages in the channel.
    #[tracing::instrument(name = "Running Cache")]
    pub async fn run(&mut self) {
        while let Some(mail) = self.mailbox.recv().await {
            tracing::info!("Got Mail {:#?}", &mail);
            use Msg::*;
            match mail {
                Get { key, ret } => {
                    tracing::info!("GET received");
                    let cached_response = self.cache.get(&key).await.ok();
                    tracing::info!("GET executed");
                    ret.send(cached_response);
                }

                Set { key, val, ret } => {
                    tracing::info!("SET received");
                    let res = self.cache.set(&key, &val).await;
                    tracing::info!("SET executed");
                    ret.send(res);
                }
            }
        }
    }
}
