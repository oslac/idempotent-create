use axum::Router;
use hyper::header;
use hyper::Body;
use hyper::Method;
use hyper::Request;
use lib::obs;
use lib::obs::get_sub;
use lib::server::UserApi;
use lib::user::NewUser;
use lib::warehouse::UserRepository;
use serde_json::json;
use std::net::SocketAddr;
use std::net::TcpListener;
use std::sync::LazyLock;

/// Tracing must be initialized only once because of Global Default Subscriber
/// binding stdout/sink.
///
/// Enable `TEST_LOG` env. variable to toggle tracing for test runs.
static TRACING: LazyLock<()> = LazyLock::new(|| {
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_sub();
        obs::init_with(subscriber);
    }
});

pub struct TestApp {
    pub pool: UserRepository,
    pub app: UserApi,
}

impl TestApp {
    pub fn new(pool: UserRepository) -> Self {
        LazyLock::force(&TRACING);
        let addr = SocketAddr::from(([127, 0, 0, 1], 0));
        let tcp_listener = TcpListener::bind(addr).unwrap();
        let app = UserApi::new(tcp_listener, pool.clone());
        Self { pool, app }
    }

    /// Initialize a repository with test data.
    pub fn init_repo_data() -> UserRepository {
        let users = [
            NewUser::new("first@email".to_string()),
            NewUser::new("second@email".to_string()),
            NewUser::new("third@email".to_string()),
            NewUser::new("fourth@email".to_string()),
            NewUser::new("fifth@email".to_string()),
        ];
        let mut user_repo = UserRepository::new();
        for new_user in users {
            user_repo.create(&new_user).unwrap();
        }
        user_repo
    }

    /// Runs the actual server if required (for e2e-testing)
    pub async fn run_server(self) -> Result<(), color_eyre::Report> {
        self.app.run().await
    }

    pub fn router(&self) -> Router {
        self.app.api.clone()
    }

    pub fn create_new_user(new_user: NewUser) -> Request<Body> {
        Request::builder()
            .method(Method::POST)
            .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
            .uri("/users")
            .body(Body::from(serde_json::to_vec(&json!(new_user)).unwrap()))
            .unwrap()
    }
}
