use dashmap::DashMap;
use std::net::IpAddr;
use std::time::{Duration, Instant};

pub struct RateLimiter {
    buckets: DashMap<IpAddr, Vec<Instant>>,
    max_requests: usize,
    window: Duration,
}

impl RateLimiter {
    #[must_use]
    pub fn new(max_requests: usize, window: Duration) -> Self {
        Self {
            buckets: DashMap::new(),
            max_requests,
            window,
        }
    }

    #[must_use]
    pub fn check(&self, ip: IpAddr) -> bool {
        let now = Instant::now();
        let mut entry = self.buckets.entry(ip).or_default();
        entry.retain(|&t| now.duration_since(t) < self.window);
        if entry.len() >= self.max_requests {
            return false;
        }
        entry.push(now);
        true
    }

    /// Drops buckets whose requests have all left the window, so IPs that stopped
    /// sending requests don't accumulate forever.
    pub fn cleanup(&self) {
        let now = Instant::now();
        self.buckets.retain(|_, timestamps| {
            timestamps.retain(|&t| now.duration_since(t) < self.window);
            !timestamps.is_empty()
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ip(s: &str) -> IpAddr {
        s.parse().unwrap()
    }

    #[test]
    fn allows_requests_up_to_limit() {
        let rl = RateLimiter::new(3, Duration::from_secs(60));
        let ip = ip("10.0.0.1");
        assert!(rl.check(ip));
        assert!(rl.check(ip));
        assert!(rl.check(ip));
        assert!(!rl.check(ip), "4th request should be denied");
    }

    #[test]
    fn allows_single_request_with_limit_1() {
        let rl = RateLimiter::new(1, Duration::from_secs(60));
        let ip = ip("10.0.0.2");
        assert!(rl.check(ip));
        assert!(!rl.check(ip));
    }

    #[test]
    fn different_ips_have_independent_buckets() {
        let rl = RateLimiter::new(1, Duration::from_secs(60));
        let ip1 = ip("10.0.0.1");
        let ip2 = ip("10.0.0.2");
        assert!(rl.check(ip1));
        assert!(!rl.check(ip1));
        assert!(rl.check(ip2), "different IP should still be allowed");
    }

    #[test]
    fn expired_timestamps_are_evicted() {
        // Window of 0 nanoseconds — everything expires immediately
        let rl = RateLimiter::new(1, Duration::from_nanos(0));
        let ip = ip("10.0.0.3");
        // First check inserts, immediately expires; second check should also pass
        // because the old entry is gone by the time retain() runs
        assert!(rl.check(ip));
        assert!(
            rl.check(ip),
            "expired entry should be evicted, allowing next request"
        );
    }

    #[test]
    fn cleanup_removes_expired_buckets_only() {
        let expired = RateLimiter::new(5, Duration::from_nanos(0));
        assert!(expired.check(ip("10.0.0.5")));
        expired.cleanup();
        assert!(expired.buckets.is_empty());

        let live = RateLimiter::new(5, Duration::from_secs(60));
        assert!(live.check(ip("10.0.0.6")));
        live.cleanup();
        assert_eq!(live.buckets.len(), 1);
    }

    #[test]
    fn zero_max_requests_always_denies() {
        let rl = RateLimiter::new(0, Duration::from_secs(60));
        let ip = ip("10.0.0.4");
        assert!(!rl.check(ip));
    }
}
