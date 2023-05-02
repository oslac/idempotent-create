use hyper::body::to_bytes as BodyToBytes;
use hyper::Body;
use hyper::Request;
use hyper::StatusCode;
use lib::server::UserApi;
use serde_json::json;
use serde_json::Value;
use std::net::SocketAddr;
use tower::ServiceExt;

#[tokio::test]
async fn get_users() {
    // I. Arrange
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let api = UserApi::new(addr).api;
    let req = Request::builder().uri("/users").body(Body::empty()).unwrap();

    // II. Act
    let response = api.oneshot(req).await.unwrap();

    // III. Assert
    let expected_status = StatusCode::OK;
    let actual_status = response.status();
    assert_eq!(expected_status, actual_status);

    let expected_content_type = "application/json";
    let actual_content_type = response.headers().get("Content-Type").unwrap();
    assert_eq!(expected_content_type, actual_content_type);

    let actual_body = BodyToBytes(response.into_body()).await.unwrap();
    let actual_body: Value = serde_json::from_slice(&actual_body).unwrap();
    let expected_body: Value = json!([]);
    assert_eq!(expected_body, actual_body);
}
