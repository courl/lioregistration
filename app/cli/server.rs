use crate::jobs::worker::WorkerSettings;
use crate::settings::Settings;
use crate::state::AppState;
use crate::{jobs, routes};
use axum::http::{Method, header};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

fn spawn_worker(state: &AppState, settings: &Settings) {
    let settings = WorkerSettings {
        admin_email: settings.admin_email.clone(),
        smtp: settings.smtp.clone(),
    };
    let ctx = state.email_attachment_ctx.clone();
    let db = state.db.clone();
    tokio::spawn(async move {
        jobs::worker::Worker::create(&settings, ctx, db).run().await;
    });
}

pub async fn run_server(state: AppState, settings: &Settings) -> anyhow::Result<()> {
    spawn_worker(&state, settings);

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

    let addr: SocketAddr = format!("{}:{}", settings.host, settings.port).parse()?;
    tracing::info!("Listening on {addr}");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
