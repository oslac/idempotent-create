use crate::test_app::TestApp;
use lib::warehouse::UserRepository;

use hyper::body::to_bytes as BodyToBytes;
use hyper::StatusCode;
use serde_json::Value;
use tokio::spawn;
use tower::Service;
use tower::ServiceExt;

#[tokio::test]
async fn new_user() {
    // I. Arrange
    let app = TestApp::new(UserRepository::new()).await;
    let req = app.post_user(&app.test_user);

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
async fn duplicate_request_without_key_is_409() {
    // I. Arrange
    let mut app = TestApp::new(UserRepository::new()).await;
    let original = app.post_user(&app.test_user);
    let duplicate = app.post_user(&app.test_user);
    let router = ServiceExt::ready(&mut app.app.api).await.unwrap();

    // II. Act
    let original = router.call(original).await.unwrap();
    let duplicate = router.call(duplicate).await.unwrap();

    // III. Assert
    assert_eq!(StatusCode::OK, original.status());
    assert_eq!(StatusCode::CONFLICT, duplicate.status());
    assert_eq!("application/json", original.headers().get("Content-Type").unwrap());
    assert_eq!("application/json", duplicate.headers().get("Content-Type").unwrap());

    let original = BodyToBytes(original.into_body()).await.unwrap();
    let original: Value = serde_json::from_slice(&original).unwrap();
    insta::assert_json_snapshot!(&original);

    let duplicate: hyper::body::Bytes = BodyToBytes(duplicate.into_body()).await.unwrap();
    let duplicate: Value = serde_json::from_slice(&duplicate).unwrap();
    insta::assert_json_snapshot!(&duplicate);

    assert_ne!(original, duplicate);
}

#[tokio::test]
async fn duplicate_request_with_key_is_201() {
    // I. Arrange
    let app = TestApp::new(UserRepository::new()).await;
    let client = hyper::Client::new();
    let original = TestApp::with_idempotency(app.post_user(&app.test_user), 1);
    let duplicate = TestApp::with_idempotency(app.post_user(&app.test_user), 1);
    spawn(async move {
        app.run().await.unwrap();
    });

    // II. Act
    let original = client.request(original).await.unwrap();
    let duplicate = client.request(duplicate).await.unwrap();

    // III. Assert
    assert_eq!(StatusCode::OK, original.status());
    assert_eq!(StatusCode::CREATED, duplicate.status());

    assert_eq!("application/json", original.headers().get("Content-Type").unwrap());
    assert_eq!("application/json", duplicate.headers().get("Content-Type").unwrap());

    let original = BodyToBytes(original.into_body()).await.unwrap();
    let original: Value = serde_json::from_slice(&original).unwrap();
    insta::assert_json_snapshot!(&original);

    let duplicate = BodyToBytes(duplicate.into_body()).await.unwrap();
    let duplicate: Value = serde_json::from_slice(&duplicate).unwrap();
    insta::assert_json_snapshot!(&duplicate);

    assert_eq!(original, duplicate);
}
