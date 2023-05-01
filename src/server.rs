use crate::middleware::cache::handle::CacheHandle;
use crate::middleware::cache::manager::CacheManager;
use crate::middleware::cache::response_cache;
use crate::obs;
use crate::routes;
use crate::service::Service;
use crate::service::SharedService;
use crate::warehouse::UserRepository;
use crate::ServerResult;

use axum::handler::Handler;
use axum::middleware;
use axum::routing::get;
use axum::Extension;
use axum::Router;
use axum::Server;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::RwLock;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

pub struct UserApi {
    addr: SocketAddr,
    api: Router,
}

impl UserApi {
    pub fn new(addr: SocketAddr) -> Self {
        obs::init();
        let api = Self::configure();
        tracing::info!(".. API configured");
        Self { addr, api }
    }

    fn configure() -> Router {
        let tracing = TraceLayer::new_for_http();

        let (cache_handle, mut cache_manager) = {
            let (sender, receiver) = mpsc::channel(8);
            let cache_handle = CacheHandle::new(sender);
            let cache_manager = CacheManager::new(receiver);
            (cache_handle, cache_manager)
        };

        let db = UserRepository::default();
        let user_service: SharedService = Arc::new(RwLock::new(Service::new(db)));
        let service = ServiceBuilder::new()
            .layer(tracing)
            .layer(Extension(cache_handle))
            .layer(Extension(user_service));

        let post_user = routes::create_user.layer(middleware::from_fn(response_cache));
        let get_users = routes::get_users;
        let get_user = routes::get_user;

        tokio::spawn(async move { cache_manager.run().await });
        tracing::warn!("CacheManager Spawned");

        Router::new()
            .route("/users", get(get_users).post(post_user))
            .route("/users/:id", get(get_user))
            .layer(service)
    }

    pub async fn run(self) -> ServerResult<()> {
        use color_eyre::eyre::WrapErr;
        let server = Server::bind(&self.addr);
        tracing::info!(".. Serving API @ {}", self.addr);
        let api = self.api.into_make_service();
        server.serve(api).await.context("Server Creation Failed")
    }
}
