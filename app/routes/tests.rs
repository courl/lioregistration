//! Router-level smoke tests: real routing, extractors, auth cookie and status mapping.

use crate::entity::admin_user::UserRole;
use crate::entity::round;
use crate::entity::round_participation::ParticipationState;
use crate::routes;
use crate::utils::test_utils::tests::{
    TestApp, create_participation, create_registration, create_school, create_student, date,
    setup_test,
};
use axum::Router;
use axum::body::{Body, to_bytes};
use axum::extract::ConnectInfo;
use axum::http::{Method, Request, StatusCode, header};
use sea_orm::EntityTrait;
use serde_json::{Value, json};
use std::net::SocketAddr;
use tower::ServiceExt;

struct Client {
    router: Router,
    cookie: Option<String>,
}

impl Client {
    fn new(app: &TestApp) -> Self {
        Self {
            router: routes::build(app.state.clone()),
            cookie: None,
        }
    }

    async fn send(&self, method: Method, uri: &str, body: Option<Value>) -> (StatusCode, Value) {
        let mut request = Request::builder()
            .method(method)
            .uri(uri)
            .header(header::CONTENT_TYPE, "application/json");
        if let Some(cookie) = &self.cookie {
            request = request.header(header::COOKIE, cookie);
        }
        let mut request = request
            .body(body.map_or_else(Body::empty, |b| Body::from(b.to_string())))
            .unwrap();
        request
            .extensions_mut()
            .insert(ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 1))));
        let response = self.router.clone().oneshot(request).await.unwrap();
        let status = response.status();
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        (
            status,
            serde_json::from_slice(&bytes).unwrap_or(Value::Null),
        )
    }

    async fn login(mut self, email: &str, password: &str) -> Self {
        let mut request = Request::builder()
            .method(Method::POST)
            .uri("/api/admin/login")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({ "email": email, "password": password }).to_string(),
            ))
            .unwrap();
        request
            .extensions_mut()
            .insert(ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 1))));
        let response = self.router.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let cookie = response.headers()[header::SET_COOKIE].to_str().unwrap();
        self.cookie = Some(cookie.split(';').next().unwrap().to_owned());
        self
    }
}

#[tokio::test]
async fn admin_routes_require_auth_and_role() {
    let app = setup_test()
        .await
        .with_user("obs@lio.lu", "pw", "Observer", UserRole::Observer)
        .await;

    let anonymous = Client::new(&app);
    let (status, _) = anonymous
        .send(Method::GET, "/api/admin/editions", None)
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    let observer = Client::new(&app).login("obs@lio.lu", "pw").await;
    let (status, _) = observer
        .send(Method::GET, "/api/admin/editions", None)
        .await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = observer
        .send(
            Method::POST,
            "/api/admin/editions",
            Some(json!({
                "name": "2026", "year": 2026,
                "registration_opens_at": "2026-01-01", "registration_closes_at": "2026-02-01"
            })),
        )
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn round_lifecycle_over_http() {
    let app = setup_test().await.with_admin("admin@lio.lu", "pw").await;
    let admin = Client::new(&app).login("admin@lio.lu", "pw").await;

    // Invalid period is rejected.
    let (status, _) = admin
        .send(
            Method::POST,
            "/api/admin/editions",
            Some(json!({
                "name": "bad", "year": 2025,
                "registration_opens_at": "2026-02-01", "registration_closes_at": "2026-01-01"
            })),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let (status, edition) = admin
        .send(
            Method::POST,
            "/api/admin/editions",
            Some(json!({
                "name": "2026", "year": 2026,
                "registration_opens_at": "2026-01-01", "registration_closes_at": "2026-02-01"
            })),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let edition_id = edition["id"].as_i64().unwrap();

    // Regression: creating a round (no tasks yet) used to answer 404.
    let (status, round) = admin
        .send(
            Method::POST,
            &format!("/api/admin/editions/{edition_id}/rounds"),
            Some(json!({ "name": "Round 1", "starts_at": "2026-03-01", "ends_at": "2026-03-02" })),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{round}");
    assert_eq!(round["task_count"], 0);
    let round_id = round["id"].as_i64().unwrap();

    let (status, task) = admin
        .send(
            Method::POST,
            &format!("/api/admin/rounds/{round_id}/tasks"),
            Some(json!({ "name": "A", "ordinal": 1, "max_points": 10 })),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let task_id = task["id"].as_i64().unwrap();

    // Seed a participant directly.
    let db = &app.state.db;
    let edition_model =
        crate::services::editions::get_edition(i32::try_from(edition_id).unwrap(), db)
            .await
            .unwrap();
    let round_model = round::Entity::find_by_id(i32::try_from(round_id).unwrap())
        .one(db)
        .await
        .unwrap()
        .unwrap();
    let school = create_school(db, "Lycée A").await;
    let student = create_student(db, "Ada", "Lovelace", date(2)).await;
    create_registration(db, &edition_model, &school, Some(student.id)).await;
    create_participation(db, &round_model, &student, ParticipationState::Accepted).await;

    // Regression: the participation PATCH route extracted its two path params wrongly.
    let participation_uri = format!("/api/admin/rounds/{round_id}/participations/{}", student.id);
    let (status, body) = admin
        .send(
            Method::PATCH,
            &participation_uri,
            Some(json!({ "points": [[task_id, 7.5]] })),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, _) = admin
        .send(
            Method::PATCH,
            &participation_uri,
            Some(json!({ "points": [[task_id, 11.0]] })),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let (status, detail) = admin
        .send(Method::GET, &format!("/api/admin/rounds/{round_id}"), None)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(detail["participants"][0]["total"], 7.5);

    // Scored task / non-empty round cannot be deleted.
    let (status, _) = admin
        .send(Method::DELETE, &format!("/api/admin/tasks/{task_id}"), None)
        .await;
    assert_eq!(status, StatusCode::CONFLICT);
    let (status, _) = admin
        .send(
            Method::DELETE,
            &format!("/api/admin/rounds/{round_id}"),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::CONFLICT);

    // Duplicate credential usernames are a client error, not a 500.
    let creds_uri = format!("/api/admin/editions/{edition_id}/credentials");
    let (status, _) = admin
        .send(
            Method::POST,
            &creds_uri,
            Some(json!([{ "username": "u1", "password": "p" }])),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = admin
        .send(
            Method::POST,
            &creds_uri,
            Some(json!([{ "username": "u1", "password": "p" }])),
        )
        .await;
    assert_eq!(status, StatusCode::CONFLICT);
    let (status, _) = admin
        .send(
            Method::POST,
            &creds_uri,
            Some(json!([{ "username": "u2", "password": "p" }, { "username": "u2", "password": "p" }])),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let (status, _) = admin
        .send(Method::DELETE, "/api/admin/credentials/9999", None)
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
