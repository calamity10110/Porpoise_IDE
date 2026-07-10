use std::time::{Duration, Instant};

pub struct RateLimiter {
    capacity: u64,
    tokens: f64,
    refill_rate: f64, // tokens per second
    last_refill: Instant,
}

impl RateLimiter {
    pub fn new(capacity: u64, refill_per_second: u64) -> Self {
        Self {
            capacity,
            tokens: capacity as f64,
            refill_rate: refill_per_second as f64,
            last_refill: Instant::now(),
        }
    }

    pub fn try_consume(&mut self, count: u64) -> bool {
        self.refill();
        if self.tokens >= count as f64 {
            self.tokens -= count as f64;
            true
        } else {
            false
        }
    }

    pub async fn wait_and_consume(&mut self, count: u64) {
        loop {
            self.refill();
            if self.tokens >= count as f64 {
                self.tokens -= count as f64;
                return;
            }
            let wait = Duration::from_secs_f64((count as f64 - self.tokens) / self.refill_rate);
            tokio::time::sleep(wait).await;
        }
    }

    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.capacity as f64);
        self.last_refill = now;
    }
}

pub struct RateLimitConfig {
    pub requests_per_second: u64,
    pub burst_size: u64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_second: 10,
            burst_size: 30,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter_consume() {
        let mut rl = RateLimiter::new(10, 10);
        assert!(rl.try_consume(5));
        assert!(rl.try_consume(5));
        assert!(!rl.try_consume(1));
    }
}
