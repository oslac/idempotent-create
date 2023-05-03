use color_eyre::Report;
use server::UserApi;
use std::net::Ipv4Addr;
use tokio::net::TcpListener;
use warehouse::UserRepository;

mod error;
mod ikey;
mod middleware;
pub mod obs;
mod routes;
pub mod server;
mod service;
pub mod user;
pub mod warehouse;

pub type ServerResult<T = ()> = Result<T, Report>;

pub async fn try_main() -> ServerResult {
    let sub = obs::get_sub();
    obs::init_with(sub);
    let socket = (Ipv4Addr::new(127, 0, 0, 1), 0);
    let listener = TcpListener::bind(socket).await.expect("Socket is Bound");
    let addr = listener.local_addr();
    tracing::info!("Bound: {:#?}", addr);
    let pool = UserRepository::new();
    UserApi::new(listener, pool).run().await
}
