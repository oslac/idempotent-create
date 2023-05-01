use self::handle::CacheHandle;
use crate::ikey::IKey;
use crate::user::User;
use crate::warehouse::CachedResponse;

use axum::body::boxed;
use axum::body::Body;
use axum::body::BoxBody;
use axum::http::Request;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::Extension;
use axum::Json;
use color_eyre::eyre::Context;
use hyper::body;
use serde_json::from_slice as DeserSlice;

pub mod handle;
pub mod manager;
pub mod msg;

type ErrorRes = Response<BoxBody>;

/// Middleware for ``POST /users``.
///
/// If `Idempotency-Key` header was provided, the `req` is further processed;
/// *otherwise* this layer short-circuits.
#[tracing::instrument(name = "Checking for Cached Response", skip(cache, req, next))]
pub async fn response_cache(
    cache: Extension<CacheHandle>,
    req: Request<Body>,
    next: Next<Body>,
) -> Result<Response, ErrorRes> {
    let Ok(ref key) = IKey::from_headers(req.headers()) else {
        tracing::info!("Request without Key");
        return Ok(next.run(req).await)
    };

    tracing::info!("Request with Key {:#?}", &key);
    process_req(cache, key, req, next).await
}

/// Processes a `req` with a [IKey] in the header.
///
/// If there is a cache hit for the key, returns the cached response;
/// *otherwise* processes the uncached request.
async fn process_req(
    Extension(cache): Extension<CacheHandle>,
    key: &IKey,
    req: Request<Body>,
    next: Next<Body>,
) -> Result<Response, ErrorRes> {
    if let Some(cached) = cache.get(key).await {
        tracing::warn!("Cache Hit with ({}, {:#?})", key, cached);
        return Ok((StatusCode::CREATED, Json(cached.user)).into_response());
    };

    tracing::warn!("Cache Miss with {}", key);
    process_uncached(cache, key, req, next).await
}

/// Processes an uncached request that came with ``Idempotency-Key`` header
/// attached to it.
async fn process_uncached(
    cache: CacheHandle,
    key: &IKey,
    req: Request<Body>,
    create_user: Next<Body>,
) -> Result<Response, ErrorRes> {
    let res = create_user.run(req).await;
    let (head, body) = res.into_parts();
    let bytes = body::to_bytes(body).await.context("Failed to convert body to bytes").unwrap();
    let status = head.status;
    let user: User = DeserSlice(&bytes).map_err(|error| {
        tracing::warn!("Deserialization Failure: {:#?}", error);
        // We did not get back a User, but something-else.
        // Return that something-else without touching things:
        Response::from_parts(head, boxed(Body::from(bytes)))
    })?;

    tracing::info!("Uncached request processed");
    process_new_user(status, user, cache, key).await
}

async fn process_new_user(
    status: StatusCode,
    user: User,
    cache: CacheHandle,
    key: &IKey,
) -> Result<Response, ErrorRes> {
    let res = CachedResponse { status, user };
    cache.set(key, &res).await;
    tracing::info!("Updated Cache Miss for Key {} with {:#?}", &key, &res.user);
    Ok((res.status, Json(res.user)).into_response())
}
