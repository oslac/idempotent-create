use std::env;
use std::str::FromStr;
use tracing::subscriber::set_global_default;
use tracing::Subscriber;
use tracing_error::ErrorLayer;
use tracing_subscriber::filter::Targets;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::Layer;
use tracing_subscriber::Registry;
use tracing_tree::HierarchicalLayer;

pub fn get_sub() -> impl Subscriber + Sync + Send {
    let filter = Targets::from_str(env::var("RUST_LOG").as_deref().unwrap_or("info"))
        .expect("RUST_LOG is a valid filter");
    let tree = HierarchicalLayer::new(2).with_bracketed_fields(true).with_wraparound(2);
    let error = ErrorLayer::default();
    let fmt = tracing_subscriber::fmt::layer().with_ansi(true).with_filter(filter);
    Registry::default().with(fmt).with(tree).with(error)
}

/// This must only be called once.
pub fn init_with(subscriber: impl Subscriber + Sync + Send) {
    set_global_default(subscriber).expect("Failed to Set Global Default Subscriber");
}
