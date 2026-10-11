use crate::jobs::worker::WorkerSettings;
use crate::settings::{DEV_SECRET_KEY, Settings};
use crate::state::AppState;
use crate::{jobs, routes};
use axum::http::{HeaderValue, Method, header};
use std::net::SocketAddr;
use std::time::Duration;
use tokio::sync::watch;
use tokio::task::JoinHandle;
use tower::util::option_layer;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

fn spawn_worker(
    state: &AppState,
    settings: &Settings,
    shutdown: watch::Receiver<bool>,
) -> JoinHandle<()> {
    let settings = WorkerSettings {
        admin_email: settings.admin_email.clone(),
        smtp: settings.smtp.clone(),
    };
    let ctx = state.email_attachment_ctx.clone();
    let db = state.db.clone();
    tokio::spawn(async move {
        jobs::worker::Worker::create(&settings, ctx, db)
            .run(shutdown)
            .await;
    })
}

/// Resolves on Ctrl-C or SIGTERM. In a container the server is PID 1, which gets no default
/// signal handlers, so without this `docker stop` waits out its timeout and SIGKILLs us.
async fn shutdown_signal() {
    let ctrl_c = async {
        if let Err(e) = tokio::signal::ctrl_c().await {
            tracing::error!("Could not listen for Ctrl-C: {e}");
            std::future::pending::<()>().await;
        }
    };
    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(e) => {
                tracing::error!("Could not listen for SIGTERM: {e}");
                std::future::pending::<()>().await;
            }
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {}
        () = terminate => {}
    }
    tracing::info!("Shutting down");
}

fn spawn_rate_limit_cleanup(state: &AppState) {
    let limiters = [
        state.rate_limiter.clone(),
        state.strict_rate_limiter.clone(),
    ];
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_mins(1));
        loop {
            interval.tick().await;
            for limiter in &limiters {
                limiter.cleanup();
            }
        }
    });
}

/// CORS is only needed when the frontend is hosted on another origin than this binary.
fn cors_layer(origins: &[String]) -> anyhow::Result<Option<CorsLayer>> {
    if origins.is_empty() {
        return Ok(None);
    }
    let origins = origins
        .iter()
        .map(|origin| origin.parse())
        .collect::<Result<Vec<HeaderValue>, _>>()?;
    Ok(Some(
        CorsLayer::new()
            .allow_origin(origins)
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::PATCH,
                Method::DELETE,
            ])
            .allow_headers([header::CONTENT_TYPE, header::ACCEPT])
            .allow_credentials(true),
    ))
}

pub async fn run_server(state: AppState, settings: &Settings) -> anyhow::Result<()> {
    if settings.secret_key == DEV_SECRET_KEY {
        if cfg!(debug_assertions) {
            tracing::warn!("Using the development secret_key; set LIO__SECRET_KEY in production");
        } else {
            anyhow::bail!("secret_key is the development default; set LIO__SECRET_KEY");
        }
    }

    let cors = cors_layer(&settings.cors_origins)?;
    if let Some(dir) = &settings.frontend_dir {
        if !dir.join("index.html").is_file() {
            anyhow::bail!("frontend_dir {} has no index.html", dir.display());
        }
        tracing::info!("Serving frontend from {}", dir.display());
    }

    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let worker = spawn_worker(&state, settings, shutdown_rx);
    spawn_rate_limit_cleanup(&state);

    let api = routes::build(state);
    let app = match &settings.frontend_dir {
        Some(dir) => routes::with_frontend(api, dir),
        None => api,
    }
    .layer(TraceLayer::new_for_http())
    .layer(option_layer(cors))
    .into_make_service_with_connect_info::<SocketAddr>();

    let addr: SocketAddr = format!("{}:{}", settings.host, settings.port).parse()?;
    tracing::info!("Listening on {addr}");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            shutdown_signal().await;
            let _ = shutdown_tx.send(true);
        })
        .await?;

    // Let a job that is mid-send finish before the process exits.
    worker.await?;
    Ok(())
}
