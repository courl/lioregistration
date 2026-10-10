use crate::error::{AppError, AppResult};
use axum::extract::ConnectInfo;
use axum::http::request::Parts;
use std::net::{IpAddr, SocketAddr};

const X_FORWARDED_FOR: &str = "x-forwarded-for";

/// Resolves the IP of the client that sent the request.
///
/// With `trust_proxy`, the rightmost entry of `X-Forwarded-For` is used: it is the one
/// appended by our own reverse proxy, whereas everything left of it is client-controlled.
/// Without it (or if the header is missing/unparseable), the TCP peer address is used.
pub fn client_ip(parts: &Parts, trust_proxy: bool) -> AppResult<IpAddr> {
    if trust_proxy && let Some(ip) = forwarded_ip(parts) {
        return Ok(ip);
    }

    parts
        .extensions
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ConnectInfo(addr)| addr.ip())
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("Missing connection info")))
}

fn forwarded_ip(parts: &Parts) -> Option<IpAddr> {
    parts
        .headers
        .get_all(X_FORWARDED_FOR)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(','))
        .next_back()
        .and_then(|entry| entry.trim().parse().ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Request;

    fn parts(forwarded_for: Option<&str>) -> Parts {
        let mut builder = Request::builder();
        if let Some(value) = forwarded_for {
            builder = builder.header(X_FORWARDED_FOR, value);
        }
        let (mut parts, ()) = builder.body(()).unwrap().into_parts();
        parts
            .extensions
            .insert(ConnectInfo(SocketAddr::from(([10, 0, 0, 1], 1234))));
        parts
    }

    fn ip(s: &str) -> IpAddr {
        s.parse().unwrap()
    }

    #[test]
    fn uses_peer_address_without_trust_proxy() {
        let parts = parts(Some("1.2.3.4"));
        assert_eq!(client_ip(&parts, false).unwrap(), ip("10.0.0.1"));
    }

    #[test]
    fn uses_forwarded_for_with_trust_proxy() {
        let parts = parts(Some("1.2.3.4"));
        assert_eq!(client_ip(&parts, true).unwrap(), ip("1.2.3.4"));
    }

    #[test]
    fn picks_rightmost_forwarded_entry() {
        let parts = parts(Some("6.6.6.6, 1.2.3.4"));
        assert_eq!(client_ip(&parts, true).unwrap(), ip("1.2.3.4"));
    }

    #[test]
    fn falls_back_to_peer_on_garbage_or_missing_header() {
        assert_eq!(
            client_ip(&parts(Some("nonsense")), true).unwrap(),
            ip("10.0.0.1")
        );
        assert_eq!(client_ip(&parts(None), true).unwrap(), ip("10.0.0.1"));
    }

    #[test]
    fn missing_connect_info_is_an_error() {
        let (parts, ()) = Request::builder().body(()).unwrap().into_parts();
        assert!(client_ip(&parts, false).is_err());
    }
}
