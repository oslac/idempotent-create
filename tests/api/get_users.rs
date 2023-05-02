use crate::test_app::TestApp;

use hyper::body::to_bytes as BodyToBytes;
use hyper::Body;
use hyper::Request;
use hyper::StatusCode;
use lib::warehouse::UserRepository;
use serde_json::Value;
use tower::ServiceExt;

#[tokio::test]
async fn empty_users() {
    // I. Arrange
    let app = TestApp::new(UserRepository::new());
    let req = Request::builder().uri("/users").body(Body::empty()).unwrap();

    // II. Act
    let response = app.router().oneshot(req).await.unwrap();

    // III. Assert
    let expected_status = StatusCode::OK;
    let actual_status = response.status();
    assert_eq!(expected_status, actual_status);

    let expected_content_type = "application/json";
    let actual_content_type = response.headers().get("Content-Type").unwrap();
    assert_eq!(expected_content_type, actual_content_type);

    let actual_body = BodyToBytes(response.into_body()).await.unwrap();
    let actual_body: Value = serde_json::from_slice(&actual_body).unwrap();
    insta::assert_json_snapshot!(&actual_body);
}

#[tokio::test]
async fn lists_all_users() {
    // 1. Arrange
    let data = TestApp::init_repo_data();
    let app = TestApp::new(data);
    let req = Request::builder().uri("/users").body(Body::empty()).unwrap();

    // II. Act
    let response = app.router().oneshot(req).await.unwrap();

    // III. Assert
    let expected_status = StatusCode::OK;
    let actual_status = response.status();
    assert_eq!(expected_status, actual_status);

    let expected_content_type = "application/json";
    let actual_content_type = response.headers().get("Content-Type").unwrap();
    assert_eq!(expected_content_type, actual_content_type);

    let actual_body = BodyToBytes(response.into_body()).await.unwrap();
    let actual_body: Value = serde_json::from_slice(&actual_body).unwrap();
    insta::assert_json_snapshot!(&actual_body);
}
