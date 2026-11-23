//! Router-level smoke tests: real routing, extractors, auth cookie and status mapping.

use crate::entity::admin_user::UserRole;
use crate::entity::email_template::TemplateType;
use crate::entity::round;
use crate::entity::round_participation::ParticipationState;
use crate::routes;
use crate::utils::test_utils::tests::{
    TempDir, TestApp, create_credential, create_edition, create_email_template,
    create_participation, create_registration, create_round, create_school, create_student,
    create_task, date, setup_test,
};
use axum::Router;
use axum::body::Bytes;
use axum::body::{Body, to_bytes};
use axum::extract::ConnectInfo;
use axum::http::{HeaderMap, Method, Request, StatusCode, header};
use sea_orm::EntityTrait;
use serde_json::{Value, json};
use std::net::SocketAddr;
use tower::ServiceExt;

struct Client {
    router: Router,
    cookie: Option<String>,
}

/// Runs one request through `router`, with the `ConnectInfo` the rate limiter needs.
async fn call(router: &Router, mut request: Request<Body>) -> (StatusCode, HeaderMap, Bytes) {
    request
        .extensions_mut()
        .insert(ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 1))));
    let response = router.clone().oneshot(request).await.unwrap();
    let (parts, body) = response.into_parts();
    let bytes = to_bytes(body, usize::MAX).await.unwrap();
    (parts.status, parts.headers, bytes)
}

impl Client {
    fn new(app: &TestApp) -> Self {
        Self::with_router(routes::build(app.state.clone()))
    }

    fn with_router(router: Router) -> Self {
        Self {
            router,
            cookie: None,
        }
    }

    async fn send_raw(
        &self,
        method: Method,
        uri: &str,
        body: Option<Value>,
    ) -> (StatusCode, Bytes) {
        let mut request = Request::builder()
            .method(method)
            .uri(uri)
            .header(header::CONTENT_TYPE, "application/json");
        if let Some(cookie) = &self.cookie {
            request = request.header(header::COOKIE, cookie);
        }
        let request = request
            .body(body.map_or_else(Body::empty, |b| Body::from(b.to_string())))
            .unwrap();
        let (status, _, bytes) = call(&self.router, request).await;
        (status, bytes)
    }

    async fn send(&self, method: Method, uri: &str, body: Option<Value>) -> (StatusCode, Value) {
        let (status, bytes) = self.send_raw(method, uri, body).await;
        (
            status,
            serde_json::from_slice(&bytes).unwrap_or(Value::Null),
        )
    }

    async fn get_text(&self, uri: &str) -> (StatusCode, String) {
        let (status, bytes) = self.send_raw(Method::GET, uri, None).await;
        (status, String::from_utf8_lossy(&bytes).into_owned())
    }

    /// Logs in and returns the full `Set-Cookie` header, attributes included.
    async fn login_set_cookie(&self, email: &str, password: &str) -> String {
        let request = Request::builder()
            .method(Method::POST)
            .uri("/api/admin/login")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({ "email": email, "password": password }).to_string(),
            ))
            .unwrap();
        let (status, headers, _) = call(&self.router, request).await;
        assert_eq!(status, StatusCode::OK);
        headers[header::SET_COOKIE].to_str().unwrap().to_owned()
    }

    async fn login(mut self, email: &str, password: &str) -> Self {
        let set_cookie = self.login_set_cookie(email, password).await;
        self.cookie = Some(set_cookie.split(';').next().unwrap().to_owned());
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
    assert_eq!(status, StatusCode::FORBIDDEN);
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
    let (status, _) = admin
        .send(
            Method::PUT,
            &format!("/api/admin/editions/{edition_id}/activate"),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK);

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

#[tokio::test]
async fn inactive_edition_is_read_only() {
    let app = setup_test().await.with_admin("admin@lio.lu", "pw").await;
    let db = &app.state.db;
    let edition = create_edition(db, 2025, false, date(1), date(2)).await;
    let round = create_round(db, edition.id, "Round 1", date(3), date(4)).await;
    let task = create_task(db, round.id, "A", 1, 10).await;
    let school = create_school(db, "Lycée A").await;
    let student = create_student(db, "Ada", "Lovelace", date(2)).await;
    let registration = create_registration(db, &edition, &school, Some(student.id)).await;
    create_participation(db, &round, &student, ParticipationState::Accepted).await;
    let credential = create_credential(db, "u1", "p", &edition).await;
    let template = create_email_template(db, TemplateType::Round).await;
    let admin = Client::new(&app).login("admin@lio.lu", "pw").await;

    let (e, r, t, reg, s, c) = (
        edition.id,
        round.id,
        task.id,
        registration.id,
        student.id,
        credential.id,
    );
    let mutations = [
        (
            Method::POST,
            format!("/api/admin/editions/{e}/rounds"),
            Some(json!({ "name": "R2", "starts_at": "2026-03-05", "ends_at": "2026-03-06" })),
        ),
        (
            Method::POST,
            format!("/api/admin/editions/{e}/credentials"),
            Some(json!([{ "username": "u2", "password": "p" }])),
        ),
        (
            Method::PATCH,
            format!("/api/admin/rounds/{r}"),
            Some(json!({ "name": "Renamed" })),
        ),
        (Method::DELETE, format!("/api/admin/rounds/{r}"), None),
        (
            Method::PATCH,
            format!("/api/admin/rounds/{r}/participations/{s}"),
            Some(json!({ "points": [[t, 5.0]] })),
        ),
        (
            Method::POST,
            format!("/api/admin/rounds/{r}/scores"),
            Some(json!([])),
        ),
        (
            Method::POST,
            format!("/api/admin/rounds/{r}/add"),
            Some(json!([s])),
        ),
        (
            Method::POST,
            format!("/api/admin/rounds/{r}/tasks"),
            Some(json!({ "name": "B", "ordinal": 2, "max_points": 10 })),
        ),
        (
            Method::PATCH,
            format!("/api/admin/tasks/{t}"),
            Some(json!({ "name": "A2" })),
        ),
        (Method::DELETE, format!("/api/admin/tasks/{t}"), None),
        (
            Method::POST,
            format!("/api/admin/credentials/{c}/unassign"),
            None,
        ),
        (Method::DELETE, format!("/api/admin/credentials/{c}"), None),
        (
            Method::POST,
            format!("/api/admin/credentials/assign/{reg}"),
            None,
        ),
        (
            Method::POST,
            format!("/api/admin/registrations/{reg}/link"),
            None,
        ),
        (
            Method::POST,
            format!("/api/admin/registrations/{reg}/link/{s}"),
            None,
        ),
        (
            Method::DELETE,
            format!("/api/admin/registrations/{reg}/link"),
            None,
        ),
        (
            Method::DELETE,
            format!("/api/admin/registrations/{reg}/refuse"),
            None,
        ),
        (
            Method::POST,
            format!("/api/admin/emails/{}/send", template.id),
            Some(json!({ "round_id": r })),
        ),
    ];
    for (method, uri, body) in mutations {
        let (status, response) = admin.send(method.clone(), &uri, body).await;
        assert_eq!(status, StatusCode::CONFLICT, "{method} {uri}: {response}");
        assert!(
            response["error"].as_str().unwrap().contains("read-only"),
            "{method} {uri}: {response}"
        );
    }

    // Reads stay available.
    for uri in [
        format!("/api/admin/editions/{e}/rounds"),
        format!("/api/admin/rounds/{r}"),
        format!("/api/admin/registrations/{reg}"),
        format!("/api/admin/editions/{e}/credentials"),
    ] {
        let (status, _) = admin.send(Method::GET, &uri, None).await;
        assert_eq!(status, StatusCode::OK, "{uri}");
    }

    // Activating the edition makes it editable again.
    let (status, _) = admin
        .send(
            Method::PUT,
            &format!("/api/admin/editions/{e}/activate"),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let (status, body) = admin
        .send(
            Method::PATCH,
            &format!("/api/admin/rounds/{r}"),
            Some(json!({ "name": "Renamed" })),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
}

#[tokio::test]
async fn frontend_fallback_serves_spa_but_not_for_api() {
    let app = setup_test().await;
    let tmp = TempDir::new("lio-spa-test");
    let dir = &tmp.0;
    std::fs::create_dir_all(dir.join("assets")).unwrap();
    std::fs::write(dir.join("index.html"), "<html>spa</html>").unwrap();
    std::fs::write(dir.join("assets/app.js"), "console.log(1)").unwrap();
    let client = Client::with_router(routes::with_frontend(routes::build(app.state.clone()), dir));

    let spa = (StatusCode::OK, "<html>spa</html>".to_owned());
    assert_eq!(client.get_text("/").await, spa);
    // client-side route survives a reload
    assert_eq!(client.get_text("/admin/rounds/3").await, spa);
    assert_eq!(
        client.get_text("/assets/app.js").await,
        (StatusCode::OK, "console.log(1)".to_owned())
    );
    // a missing static file (stale bundle hash) is a 404, not the SPA
    let (status, body) = client.get_text("/assets/index-OLDHASH.js").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(!body.contains("spa"), "{body}");
    // the API keeps working and unknown API paths stay a JSON 404
    let (status, body) = client.get_text("/api/edition").await;
    assert_eq!(status, StatusCode::OK);
    assert!(!body.contains("spa"), "{body}");
    for uri in ["/api", "/api/", "/api/nope", "/api/admin/nope"] {
        let (status, body) = client.get_text(uri).await;
        assert_eq!(status, StatusCode::NOT_FOUND, "{uri}");
        assert!(!body.contains("spa"), "{uri}: {body}");
    }
}

#[tokio::test]
async fn login_cookie_samesite_follows_frontend_mode() {
    async fn set_cookie(cross_site: bool) -> String {
        let mut app = setup_test().await.with_admin("admin@lio.lu", "test").await;
        app.state.cross_site_frontend = cross_site;
        Client::new(&app)
            .login_set_cookie("admin@lio.lu", "test")
            .await
    }

    let same_site = set_cookie(false).await;
    assert!(same_site.contains("SameSite=Lax"), "{same_site}");
    assert!(!same_site.contains("Partitioned"), "{same_site}");
    let cross_site = set_cookie(true).await;
    assert!(cross_site.contains("SameSite=None"), "{cross_site}");
    assert!(cross_site.contains("Partitioned"), "{cross_site}");
    for cookie in [same_site, cross_site] {
        assert!(
            cookie.contains("HttpOnly") && cookie.contains("Secure"),
            "{cookie}"
        );
    }
}

#[tokio::test]
async fn round_email_custom_fields_and_preview() {
    let app = setup_test()
        .await
        .with_admin("admin@lio.lu", "pw")
        .await
        .with_user("obs@lio.lu", "pw", "Observer", UserRole::Observer)
        .await;
    let db = &app.state.db;
    let edition = create_edition(db, 2026, true, date(1), date(2)).await;
    let round = create_round(db, edition.id, "Semi-Finals", date(5), date(5)).await;
    let school = create_school(db, "LML - Lycée Michel Lucius").await;
    let mut students = Vec::new();
    for name in ["Ada", "Grace"] {
        let student = create_student(db, name, "Tester", date(2)).await;
        create_registration(db, &edition, &school, Some(student.id)).await;
        create_participation(db, &round, &student, ParticipationState::Accepted).await;
        students.push(student);
    }
    let template = create_email_template(db, TemplateType::Round).await;
    let admin = Client::new(&app).login("admin@lio.lu", "pw").await;
    let observer = Client::new(&app).login("obs@lio.lu", "pw").await;
    let base = format!("/api/admin/emails/{}", template.id);

    // Declaring fields: keys are validated.
    let update = |fields: Value| {
        json!({
            "name": "Round", "template_type": "round", "custom_fields": fields,
            "subject": "{{?headline}} – {{!round}}",
            "template": "<mjml><mj-body><mj-section><mj-column><mj-text>{{?content}} {{?place}} {{given_name}}</mj-text></mj-column></mj-section></mj-body></mjml>",
        })
    };
    let (status, _) = admin
        .send(
            Method::PUT,
            &base,
            Some(update(
                json!([{ "key": "Bad Key", "label": "x", "kind": "text" }]),
            )),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let fields = json!([
        { "key": "headline", "label": "Headline", "kind": "text", "default": "LIO - NEWS" },
        { "key": "content", "label": "Body", "kind": "multiline", "default": null },
        { "key": "place", "label": "Place", "kind": "text", "default": null },
        { "key": "day", "label": "Day", "kind": "date", "default": "2026-03-05" },
    ]);
    let (status, body) = admin
        .send(Method::PUT, &base, Some(update(fields.clone())))
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["custom_fields"], fields);
    assert_eq!(body["subject"], "{{?headline}} – {{!round}}");

    let (_, attachment) = admin
        .send(Method::POST, &format!("{base}/attachment"), None)
        .await;
    let (status, _) = admin
        .send(
            Method::PATCH,
            &format!("{base}/attachment/{}", attachment["id"]),
            Some(json!({ "name": "Letter", "template": "{{?place}} {{given_name}} #\"{{?day}}\".len()" })),
        )
        .await;
    assert_eq!(status, StatusCode::OK);

    // Read-only preview, also for observers: first participant by default, others on request.
    let preview = |student_id: Option<i32>, custom: Value| json!({ "round_id": round.id, "student_id": student_id, "custom": custom });
    let (status, body) = observer
        .send(
            Method::POST,
            &format!("{base}/round-preview"),
            Some(preview(
                None,
                json!({ "place": "<LGL>", "content": "See you **soon**" }),
            )),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["subject"], "LIO - NEWS – Semi-Finals");
    assert_eq!(body["student_id"], students[0].id);
    assert_eq!(body["recipients"].as_array().unwrap().len(), 2);
    let html = body["html"].as_str().unwrap();
    assert!(
        html.contains("See you <strong>soon</strong>")
            && html.contains("&lt;LGL&gt;")
            && html.contains("Ada"),
        "{html}"
    );
    assert!(body["attachments"][0]["data_base64"].is_string(), "{body}");

    let (status, body) = observer
        .send(
            Method::POST,
            &format!("{base}/round-preview"),
            Some(preview(
                Some(students[1].id),
                json!({ "place": "LGL", "content": "Hi" }),
            )),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["html"].as_str().unwrap().contains("Grace"));

    let outsider = create_student(db, "Eve", "Outsider", date(2)).await;
    let (status, _) = observer
        .send(
            Method::POST,
            &format!("{base}/round-preview"),
            Some(preview(
                Some(outsider.id),
                json!({ "place": "LGL", "content": "Hi" }),
            )),
        )
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // Sending enforces the declaration and stores the resolved values in the job.
    let send = |custom: Value| json!({ "round_id": round.id, "custom": custom });
    for bad in [
        json!({}),
        json!({ "place": "LGL" }),
        json!({ "place": "LGL", "content": "Hi", "day": "soon" }),
        json!({ "place": "LGL", "content": "Hi", "x": "1" }),
    ] {
        let (status, _) = admin
            .send(
                Method::POST,
                &format!("{base}/send"),
                Some(send(bad.clone())),
            )
            .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{bad}");
    }
    let (status, _) = observer
        .send(
            Method::POST,
            &format!("{base}/send"),
            Some(send(json!({ "place": "LGL", "content": "Hi" }))),
        )
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, _) = admin
        .send(
            Method::POST,
            &format!("{base}/send"),
            Some(send(json!({ "place": "LGL", "content": "Hi" }))),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let jobs = crate::entity::job::Entity::find().all(db).await.unwrap();
    assert_eq!(jobs.len(), 1);
    let payload: Value = serde_json::from_str(&jobs[0].payload).unwrap();
    let custom = &payload["global_context"]["custom"];
    assert_eq!(
        *custom,
        json!({
            "headline": { "kind": "text", "value": "LIO - NEWS" },
            "content": { "kind": "markdown", "value": "Hi" },
            "place": { "kind": "text", "value": "LGL" },
            "day": { "kind": "date", "value": "2026-03-05" },
        }),
        "{payload}"
    );
}

#[tokio::test]
async fn unscored_tasks_are_null_and_stay_unscored() {
    let app = setup_test().await.with_admin("admin@lio.lu", "pw").await;
    let db = &app.state.db;
    let edition = create_edition(db, 2026, true, date(1), date(2)).await;
    let round = create_round(db, edition.id, "Round 1", date(5), date(5)).await;
    let scored = create_task(db, round.id, "A", 1, 10).await;
    create_task(db, round.id, "B", 2, 10).await;
    let school = create_school(db, "S").await;
    let student = create_student(db, "Ada", "Tester", date(2)).await;
    create_registration(db, &edition, &school, Some(student.id)).await;
    create_participation(db, &round, &student, ParticipationState::Accepted).await;
    let admin = Client::new(&app).login("admin@lio.lu", "pw").await;

    let (status, _) = admin
        .send(
            Method::PATCH,
            &format!(
                "/api/admin/rounds/{}/participations/{}",
                round.id, student.id
            ),
            Some(json!({ "state": "noShow", "points": [[scored.id, 0.0]] })),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let (_, body) = admin
        .send(
            Method::GET,
            &format!("/api/admin/rounds/{}", round.id),
            None,
        )
        .await;
    assert_eq!(body["participants"][0]["points"], json!([0.0, null]));
}
