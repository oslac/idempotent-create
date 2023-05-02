use std::env;
use std::str::FromStr;
use tracing::subscriber::set_global_default;
use tracing::Level;
use tracing::Subscriber;
use tracing_error::ErrorLayer;
use tracing_subscriber::filter::Targets;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;

pub fn get_sub() -> impl Subscriber + Sync + Send {
    let filter = Targets::from_str(env::var("RUST_LOG").as_deref().unwrap_or("info"))
        .expect("RUST_LOG should be a valid tracing filter!");

    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .pretty()
        .finish()
        .with(ErrorLayer::default())
        .with(filter)
}

/// This must only be called once.
pub fn init_with(subscriber: impl Subscriber + Sync + Send) {
    set_global_default(subscriber).expect("Failed to Set Global Default Subscriber");
}
