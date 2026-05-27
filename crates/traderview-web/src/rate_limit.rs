//! Per-PAT token-bucket rate limiter.
//!
//! In-process state: `DashMap<TokenId, Bucket>`. Each bucket carries a
//! capacity (the PAT's `rate_limit_per_min`) and refills continuously at
//! capacity/60 tokens per second. Refill is computed lazily on each check
//! so we don't need a background tick task.
//!
//! When the bucket is empty, the auth layer returns `ApiError::RateLimited`
//! with `retry_after_secs` set to ceil(time-to-next-token).
//!
//! Single-process only — fine for the typical desktop / single-server
//! deployment. Multi-instance deployments would swap this for Redis with
//! the same `RateLimiter` trait shape.

use dashmap::DashMap;
use once_cell::sync::Lazy;
use std::time::Instant;
use uuid::Uuid;

#[derive(Debug, Clone, Copy)]
pub struct Bucket {
    pub capacity:    f64,    // max tokens
    pub tokens:      f64,    // current
    pub last_refill: Instant,
}

#[derive(Debug, Clone, Copy)]
pub struct RateLimitResult {
    pub limit:     u32,
    pub remaining: u32,
    /// Seconds until the bucket regains the next token (0 if not throttled).
    pub retry_after_secs: u64,
    /// Unix epoch seconds when the bucket next refills to full.
    pub reset_epoch:      i64,
    pub allowed:   bool,
}

static BUCKETS: Lazy<DashMap<Uuid, Bucket>> = Lazy::new(DashMap::new);

/// Check + decrement one token for `id` with the given per-minute capacity.
/// Returns the resulting (allowed, headers) state.
pub fn check_and_consume(id: Uuid, capacity_per_min: u32) -> RateLimitResult {
    let cap = capacity_per_min.max(1) as f64;
    let refill_per_sec = cap / 60.0;
    let now = Instant::now();

    let mut entry = BUCKETS.entry(id).or_insert(Bucket {
        capacity: cap, tokens: cap, last_refill: now,
    });
    // Keep capacity in sync if the user changed it.
    entry.capacity = cap;
    let elapsed = now.duration_since(entry.last_refill).as_secs_f64();
    entry.tokens = (entry.tokens + elapsed * refill_per_sec).min(cap);
    entry.last_refill = now;

    let allowed = entry.tokens >= 1.0;
    if allowed {
        entry.tokens -= 1.0;
    }
    let remaining = entry.tokens.floor().max(0.0) as u32;

    // Time-to-next-token if throttled.
    let retry_after_secs = if allowed { 0 } else {
        let need = 1.0 - entry.tokens;
        (need / refill_per_sec).ceil().max(1.0) as u64
    };
    // Time-to-full reset.
    let to_full = ((cap - entry.tokens) / refill_per_sec).ceil().max(0.0) as i64;
    let reset_epoch = chrono::Utc::now().timestamp() + to_full;

    RateLimitResult {
        limit: capacity_per_min, remaining, retry_after_secs, reset_epoch, allowed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn bucket_exhausts_at_capacity_then_throttles() {
        let id = Uuid::new_v4();
        let cap = 5;
        // First 5 requests succeed.
        for i in 0..cap {
            let r = check_and_consume(id, cap);
            assert!(r.allowed, "request {} unexpectedly throttled", i);
        }
        // 6th request is throttled with a positive retry_after.
        let r = check_and_consume(id, cap);
        assert!(!r.allowed, "expected throttle at request {}", cap + 1);
        assert!(r.retry_after_secs >= 1, "retry_after_secs must be >= 1, got {}", r.retry_after_secs);
        assert_eq!(r.limit, cap);
        assert_eq!(r.remaining, 0);
    }

    #[test]
    fn independent_buckets_dont_share_state() {
        // Two distinct token ids must not drain each other.
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let cap = 2;
        assert!(check_and_consume(a, cap).allowed);
        assert!(check_and_consume(a, cap).allowed);
        assert!(!check_and_consume(a, cap).allowed);
        // b is untouched: capacity 2 still available.
        assert!(check_and_consume(b, cap).allowed);
        assert!(check_and_consume(b, cap).allowed);
        assert!(!check_and_consume(b, cap).allowed);
    }

    #[test]
    fn capacity_change_takes_effect_immediately() {
        // Same token id, raise capacity → bucket cap rises on next check.
        let id = Uuid::new_v4();
        check_and_consume(id, 1);
        // Bucket now empty at cap=1.
        assert!(!check_and_consume(id, 1).allowed);
        // Raise to cap=5: existing tokens stay (0), but cap is now 5, refill
        // rate is 5/60s ≈ 0.083 tps. Still throttled on next instant.
        let r = check_and_consume(id, 5);
        assert_eq!(r.limit, 5);
    }
}
