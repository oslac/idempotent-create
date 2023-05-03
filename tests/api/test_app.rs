use lib::obs;
use lib::obs::get_sub;
use lib::server::UserApi;
use lib::user::NewUser;
use lib::warehouse::UserRepository;

use axum::Router;
use hyper::header;
use hyper::http::HeaderValue;
use hyper::Body;
use hyper::Method;
use hyper::Request;
use serde_json::json;
use std::net::Ipv4Addr;
use std::sync::LazyLock;
use tokio::net::TcpListener;

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
    pub address: String,
    pub port: u16,
    pub test_user: NewUser,
}

impl TestApp {
    pub async fn new(pool: UserRepository) -> Self {
        LazyLock::force(&TRACING);
        let socket = (Ipv4Addr::new(127, 0, 0, 1), 0);
        let listener = TcpListener::bind(socket).await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let address = format!("http://localhost:{}", port);
        let app = UserApi::new(listener, pool.clone());
        let test_user = NewUser::new("first@email".to_string());
        Self { pool, app, address, port, test_user }
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
    pub async fn run(self) -> Result<(), color_eyre::Report> {
        self.app.run().await
    }

    pub fn router(&self) -> Router {
        self.app.api.clone()
    }

    pub fn post_user(&self, new_user: &NewUser) -> Request<Body> {
        Request::builder()
            .method(Method::POST)
            .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
            .uri(format!("{}/users", self.address))
            .body(Body::from(serde_json::to_vec(&json!(new_user)).unwrap()))
            .unwrap()
    }

    pub fn get_users(&self) -> Request<Body> {
        Request::builder().uri(format!("{}/users", self.address)).body(Body::empty()).unwrap()
    }

    pub fn with_idempotency(req: Request<Body>, key: u64) -> Request<Body> {
        let mut req = req;
        req.headers_mut().insert("Idempotency-Key", HeaderValue::from(key));
        req
    }
}
