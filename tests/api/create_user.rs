use hyper::header;
use hyper::http::HeaderValue;
use hyper::Body;
use hyper::Method;
use hyper::Request;
use hyper::StatusCode;
use lib::user::NewUser;
use lib::warehouse::UserRepository;
use serde_json::json;
use serde_json::Value;
use tokio::spawn;
use tower::ServiceExt;

use crate::test_app::TestApp;

#[tokio::test]
async fn new_user() {
    // I. Arrange
    let app = TestApp::new(UserRepository::new());
    let new_user = NewUser::new("first@email.com".to_string());
    let req = Request::builder()
        .method(Method::POST)
        .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
        .uri("/users")
        .body(Body::from(serde_json::to_vec(&json!(new_user)).unwrap()))
        .unwrap();

    // II. Act
    let response = app.router().oneshot(req).await.unwrap();

    // III. Assert
    let expected_status = StatusCode::OK;
    let actual_status = response.status();
    assert_eq!(expected_status, actual_status);

    let expected_content_type = "application/json";
    let actual_content_type = response.headers().get("Content-Type").unwrap();
    assert_eq!(expected_content_type, actual_content_type);

    let actual_body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let actual_body: Value = serde_json::from_slice(&actual_body).unwrap();
    insta::assert_json_snapshot!(&actual_body);
}

#[tokio::test]
async fn duplicate_request_without_key_is_409() {
    // I. Arrange
    let app = TestApp::new(UserRepository::new());
    let original = TestApp::create_new_user(NewUser::new("first@email.com".to_string()));
    let duplicate = TestApp::create_new_user(NewUser::new("first@email.com".to_string()));

    // II. Act
    let original = app.router().oneshot(original).await.unwrap();
    let duplicate = app.router().oneshot(duplicate).await.unwrap();

    // III. Assert
    assert_eq!(StatusCode::OK, original.status());
    assert_eq!(StatusCode::CONFLICT, duplicate.status());
    assert_eq!("application/json", original.headers().get("Content-Type").unwrap());
    assert_eq!("application/json", duplicate.headers().get("Content-Type").unwrap());

    let original = hyper::body::to_bytes(original.into_body()).await.unwrap();
    let original: Value = serde_json::from_slice(&original).unwrap();
    insta::assert_json_snapshot!(&original);

    let duplicate: hyper::body::Bytes = hyper::body::to_bytes(duplicate.into_body()).await.unwrap();
    let duplicate: Value = serde_json::from_slice(&duplicate).unwrap();
    insta::assert_json_snapshot!(&duplicate);
}

#[tokio::test]
async fn duplicate_request_with_key_is_201() {
    // I. Arrange
    let app = TestApp::new(UserRepository::new());
    let address = app.app.addr;
    let api = format!("http://localhost:{}/users", address.port());
    let new_user = NewUser::new("email@mail".to_string());
    let key = 1;
    let client = hyper::Client::new();

    // FIXME requires an end-to-end test for now, because of cache
    spawn(async move {
        app.run_server().await.unwrap();
    });

    // II. Act
    let original = idempotent_request(&api, &new_user, key);
    let duplicate = idempotent_request(&api, &new_user, key);

    let original = client.request(original).await.unwrap();
    let duplicate = client.request(duplicate).await.unwrap();

    // III.  Assert
    assert_eq!(StatusCode::OK, original.status());
    assert_eq!(StatusCode::CREATED, duplicate.status());

    assert_eq!("application/json", original.headers().get("Content-Type").unwrap());
    assert_eq!("application/json", duplicate.headers().get("Content-Type").unwrap());

    let original = hyper::body::to_bytes(original.into_body()).await.unwrap();
    let original: Value = serde_json::from_slice(&original).unwrap();
    insta::assert_json_snapshot!(&original);

    let duplicate = hyper::body::to_bytes(duplicate.into_body()).await.unwrap();
    let duplicate: Value = serde_json::from_slice(&duplicate).unwrap();
    insta::assert_json_snapshot!(&duplicate);

    assert_eq!(original, duplicate);
}

fn idempotent_request(api: &str, new_user: &NewUser, key: u64) -> Request<Body> {
    Request::builder()
        .method(Method::POST)
        .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
        .header("Idempotency-Key", HeaderValue::from(key))
        .uri(api)
        .body(Body::from(serde_json::to_vec(&json!(new_user)).unwrap()))
        .unwrap()
}
