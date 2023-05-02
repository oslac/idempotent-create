use color_eyre::Report;
use server::UserApi;
use std::net::SocketAddr;

mod error;
mod ikey;
mod middleware;
mod obs;
mod routes;
pub mod server;
mod service;
mod user;
mod warehouse;

pub type ServerResult<T = ()> = Result<T, Report>;

pub async fn try_main() -> ServerResult {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    UserApi::new(addr).run().await
}
