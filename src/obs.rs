use std::env;
use std::str::FromStr;
use tracing::Level;
use tracing_error::ErrorLayer;
use tracing_subscriber::filter::Targets;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub fn init() {
    let filter = Targets::from_str(env::var("RUST_LOG").as_deref().unwrap_or("info"))
        .expect("RUST_LOG should be a valid tracing filter!");

    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .pretty()
        .finish()
        .with(ErrorLayer::default())
        .with(filter)
        .init()
}
