use crate::error::{AppError, AppResult};
use crate::state::AppState;
use crate::utils::client_ip::client_ip;
use axum::extract::{FromRef, FromRequestParts, Request, State};
use axum::http::request::Parts;
use axum::middleware::Next;
use axum::response::Response;

/// Middleware applied to every route: enforces the global per-IP limit.
pub async fn global_rate_limit(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> AppResult<Response> {
    let (parts, body) = request.into_parts();
    let ip = client_ip(&parts, state.trust_proxy)?;
    if !state.rate_limiter.check(ip) {
        tracing::warn!("Global rate limit exceeded for {ip}");
        return Err(AppError::TooManyRequests);
    }
    Ok(next.run(Request::from_parts(parts, body)).await)
}

/// Extractor for sensitive endpoints (login, register): adding `_rl: StrictRateLimit` to a
/// handler counts the request against the strict per-IP limit. Place it before body extractors.
pub struct StrictRateLimit;

impl<S> FromRequestParts<S> for StrictRateLimit
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> AppResult<Self> {
        let app_state = AppState::from_ref(state);
        let ip = client_ip(parts, app_state.trust_proxy)?;
        if !app_state.strict_rate_limiter.check(ip) {
            tracing::warn!("Strict rate limit exceeded for {ip}");
            return Err(AppError::TooManyRequests);
        }
        Ok(Self)
    }
}

#[cfg(test)]
mod tests {
    use crate::routes;
    use crate::utils::rate_limiter::RateLimiter;
    use crate::utils::test_utils::tests::setup_test;
    use axum::body::Body;
    use axum::extract::ConnectInfo;
    use axum::http::{Method, Request, StatusCode, header};
    use std::net::SocketAddr;
    use std::sync::Arc;
    use std::time::Duration;
    use tower::ServiceExt;

    fn request(method: Method, uri: &str, ip: [u8; 4], body: &str) -> Request<Body> {
        let mut request = Request::builder()
            .method(method)
            .uri(uri)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_owned()))
            .unwrap();
        request
            .extensions_mut()
            .insert(ConnectInfo(SocketAddr::from((ip, 1234))));
        request
    }

    #[tokio::test]
    async fn global_limit_applies_to_every_route_per_ip() {
        let mut app = setup_test().await;
        app.state.rate_limiter = Arc::new(RateLimiter::new(2, Duration::from_secs(60)));
        let router = routes::build(app.state);

        let get = |ip| request(Method::GET, "/api/edition", ip, "");
        assert_ne!(
            router
                .clone()
                .oneshot(get([1, 1, 1, 1]))
                .await
                .unwrap()
                .status(),
            StatusCode::TOO_MANY_REQUESTS
        );
        assert_ne!(
            router
                .clone()
                .oneshot(get([1, 1, 1, 1]))
                .await
                .unwrap()
                .status(),
            StatusCode::TOO_MANY_REQUESTS
        );
        assert_eq!(
            router
                .clone()
                .oneshot(get([1, 1, 1, 1]))
                .await
                .unwrap()
                .status(),
            StatusCode::TOO_MANY_REQUESTS
        );
        assert_ne!(
            router.oneshot(get([2, 2, 2, 2])).await.unwrap().status(),
            StatusCode::TOO_MANY_REQUESTS
        );
    }

    #[tokio::test]
    async fn strict_limit_applies_to_login() {
        let mut app = setup_test().await;
        app.state.strict_rate_limiter = Arc::new(RateLimiter::new(2, Duration::from_secs(60)));
        let router = routes::build(app.state);

        let login = || {
            request(
                Method::POST,
                "/api/admin/login",
                [1, 1, 1, 1],
                r#"{"email":"nobody@lio.lu","password":"wrong"}"#,
            )
        };
        assert_eq!(
            router.clone().oneshot(login()).await.unwrap().status(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            router.clone().oneshot(login()).await.unwrap().status(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            router.clone().oneshot(login()).await.unwrap().status(),
            StatusCode::TOO_MANY_REQUESTS
        );
        // the strict limit doesn't affect regular routes
        let get = request(Method::GET, "/api/edition", [1, 1, 1, 1], "");
        assert_ne!(
            router.oneshot(get).await.unwrap().status(),
            StatusCode::TOO_MANY_REQUESTS
        );
    }
}
