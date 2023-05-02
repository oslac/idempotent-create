use crate::middleware::cache;
use crate::middleware::cache::handle::CacheHandle;
use crate::middleware::cache::manager::CacheManager;
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
use std::net::TcpListener;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::RwLock;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

/// In a real implementation, this would be a connection pool:
type ConnectionPool = UserRepository;

pub struct UserApi {
    pub addr: SocketAddr,
    pub api: Router,
    pub cache_manager: CacheManager,
}

impl UserApi {
    /// Initialize a new [UserApi].
    pub fn new(addr: TcpListener, pool: ConnectionPool) -> Self {
        tracing::debug!(".. Configuring the API");

        let (cache_handle, cache_manager) = {
            let (sender, receiver) = mpsc::channel(8);
            let cache_handle = CacheHandle::new(sender);
            let cache_manager = CacheManager::new(receiver);
            (cache_handle, cache_manager)
        };

        tracing::info!(".. the API was configured successfully");
        let api = Self::router(cache_handle, pool);
        let addr = addr.local_addr().expect("Port was Bound");
        Self { addr, api, cache_manager }
    }

    pub fn router(cache_handle: CacheHandle, pool: UserRepository) -> Router {
        let tracing = TraceLayer::new_for_http();
        let user_service: SharedService = Arc::new(RwLock::new(Service::new(pool)));
        let service = ServiceBuilder::new()
            .layer(tracing)
            .layer(Extension(cache_handle))
            .layer(Extension(user_service));

        let post_user = routes::create_user.layer(middleware::from_fn(cache::process));
        let get_users = routes::get_users;
        let get_user = routes::get_user;

        Router::new()
            .route("/users", get(get_users).post(post_user))
            .route("/users/:id", get(get_user))
            .layer(service)
    }

    pub async fn run(self) -> ServerResult<()> {
        use color_eyre::eyre::WrapErr;

        let mut mngr = self.cache_manager;
        tokio::spawn(async move { mngr.run().await });
        tracing::warn!("CacheManager Spawned");

        let server = Server::bind(&self.addr);
        tracing::info!(".. Serving API @ {}", self.addr);

        let api = self.api.into_make_service();
        server.serve(api).await.context("Server Creation Failed")
    }
}
