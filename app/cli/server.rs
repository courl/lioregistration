use crate::jobs::worker::WorkerSettings;
use crate::state::AppState;
use crate::{jobs, routes};
use axum::http::{Method, header};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

fn spawn_worker(state: &AppState) {
    let settings = WorkerSettings {
        admin_email: "admin@lio.lu".into(), // todo: load this from config,
        sender_email: "noreply@lio.lu".into(),
        smtp_credentials: None,
        smtp_server: "".into(),
        smtp_host: 465,
        smtp_implicit_tls: true,
        assets_root: "./assets".into(),
    };
    let db = state.db.clone();
    tokio::spawn(async move {
        jobs::worker::Worker::create(&settings, db).run().await;
    });
}

pub async fn run_server(state: AppState, host: &str, port: u16) -> anyhow::Result<()> {
    spawn_worker(&state);

    let cors = CorsLayer::new()
        // fixme: set proper origins
        .allow_origin([
            "http://localhost:5173".parse()?,
            "http://127.0.0.1:5173".parse()?,
        ])
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT, // todo check which methods are used
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([header::CONTENT_TYPE, header::ACCEPT])
        .allow_credentials(true);
    // fixme: what protocol headers

    let app = routes::build(state)
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .into_make_service_with_connect_info::<SocketAddr>();

    let addr: SocketAddr = format!("{host}:{port}").parse()?;
    tracing::info!("Listening on {addr}");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
