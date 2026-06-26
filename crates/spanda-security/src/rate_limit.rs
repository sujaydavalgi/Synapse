//! In-memory token-bucket rate limiting for Control Center APIs.
//!
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
struct Bucket {
    tokens: f64,
    last_refill: Instant,
}

/// Per-client API rate limiter (token bucket, in-memory).
#[derive(Debug)]
pub struct RateLimiter {
    enabled: bool,
    limit_per_minute: u32,
    buckets: Mutex<HashMap<String, Bucket>>,
}

impl RateLimiter {
    pub fn from_env() -> Self {
        // Build a limiter from SPANDA_API_RATE_LIMIT_PER_MINUTE (0 disables).
        let limit_per_minute = std::env::var("SPANDA_API_RATE_LIMIT_PER_MINUTE")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(0);
        Self {
            enabled: limit_per_minute > 0,
            limit_per_minute,
            buckets: Mutex::new(HashMap::new()),
        }
    }

    pub fn with_limit(limit_per_minute: u32) -> Self {
        Self {
            enabled: limit_per_minute > 0,
            limit_per_minute,
            buckets: Mutex::new(HashMap::new()),
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn limit_per_minute(&self) -> u32 {
        self.limit_per_minute
    }

    /// Allow the request or return suggested retry delay in seconds.
    pub fn check(&self, client_key: &str) -> Result<(), u64> {
        if !self.enabled {
            return Ok(());
        }
        let now = Instant::now();
        let rate_per_sec = self.limit_per_minute as f64 / 60.0;
        let capacity = self.limit_per_minute as f64;
        let mut guard = self
            .buckets
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let bucket = guard
            .entry(client_key.to_string())
            .or_insert_with(|| Bucket {
                tokens: capacity,
                last_refill: now,
            });
        let elapsed = now
            .checked_duration_since(bucket.last_refill)
            .unwrap_or(Duration::ZERO)
            .as_secs_f64();
        bucket.tokens = (bucket.tokens + elapsed * rate_per_sec).min(capacity);
        bucket.last_refill = now;
        if bucket.tokens >= 1.0 {
            bucket.tokens -= 1.0;
            Ok(())
        } else {
            let deficit = 1.0 - bucket.tokens;
            let retry_after = (deficit / rate_per_sec).ceil() as u64;
            Err(retry_after.max(1))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_limiter_always_allows() {
        let limiter = RateLimiter::with_limit(0);
        assert!(!limiter.is_enabled());
        assert!(limiter.check("client-a").is_ok());
    }

    #[test]
    fn limiter_blocks_burst_over_capacity() {
        let limiter = RateLimiter::with_limit(2);
        assert!(limiter.check("client-a").is_ok());
        assert!(limiter.check("client-a").is_ok());
        assert!(limiter.check("client-a").is_err());
    }
}
