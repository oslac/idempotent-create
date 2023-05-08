use self::error::CacheLayerError;
use self::handle::CacheHandle;
use crate::ikey::IKey;
use crate::user::User;
use crate::warehouse::CachedResponse;

use axum::body::boxed;
use axum::body::Body;
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
use tracing::instrument;

pub mod error;
pub mod handle;
pub mod manager;
pub mod msg;

/// Middleware for `POST /users`.
///
/// When `Idempotency-Key` header is provided, the `req` is further
/// processed; *otherwise* the layer short-circuits.
#[instrument(name = "Check Request for Cached Response", skip(cache, req, next))]
pub async fn process(
    Extension(cache): Extension<CacheHandle>,
    req: Request<Body>,
    next: Next<Body>,
) -> Result<Response, CacheLayerError> {
    let Ok(ref key) = IKey::from_headers(req.headers()) else {
        tracing::info!("Request without Idempotency Header");
        return Ok(next.run(req).await)
    };

    process_with_key(&cache, key, req, next).await
}

/// Processes a `req` with an [IKey].
///
/// When there is a cache hit for `key`, returns the cached response;
/// *otherwise* processes the uncached request.
#[instrument(
    name = "Processing Request with Idempotency Header",
    fields(key = key.0),
    skip(cache, key, req, next)
)]
async fn process_with_key(
    cache: &CacheHandle,
    key: &IKey,
    req: Request<Body>,
    next: Next<Body>,
) -> Result<Response, CacheLayerError> {
    if let Some(cached) = cache.get(key).await {
        tracing::warn!("Cache Hit: {key} -> {}", cached.status);
        return Ok((StatusCode::CREATED, Json(cached.user)).into_response());
    };

    tracing::warn!("Cache Miss: {}", key);
    process_uncached(cache, key, req, next).await
}

/// Processes an uncached request with an `Idempotency-Key` header.
#[instrument(
    name = "Processing Uncached Request with Idempotency Key",
    fields(key = key.0),
    skip(cache, key, req, layers)
)]
async fn process_uncached(
    cache: &CacheHandle,
    key: &IKey,
    req: Request<Body>,
    layers: Next<Body>,
) -> Result<Response, CacheLayerError> {
    // Run rest of the middleware layers, all the way down to the handler.
    let response = layers.run(req).await;
    // After the handler has run, only then upsert the cache
    let (head, body) = response.into_parts();
    let body = body::to_bytes(body).await.context("Failed to onvert body to bytes")?;
    match DeserSlice::<User>(&body) {
        Ok(new_user) => {
            tracing::info!("Uncached Request Proceessed: {:#?}", new_user);
            let res = CachedResponse { status: head.status, user: new_user };
            cache.set(key, &res).await.context("Cache Update Failed")?;
            tracing::warn!("Cache Miss Updated: {key} -> {}", res.status);
            Ok((res.status, Json(res.user)).into_response())
        }
        Err(error_from_layer) => {
            // Layer did not return `User`, but something-else.
            // Return that something-else without touching things.
            // This can be cached too.
            tracing::warn!("Deserialization Failure: {:#?}", error_from_layer);
            Ok(Response::from_parts(head, boxed(Body::from(body))))
        }
    }
}
