use crate::routing::Route;
use dashmap::DashMap;
use reqwest::header::HeaderMap;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration, Instant};
use tracing::{debug, warn};

#[derive(Debug)]
struct Bucket {

    queue: Mutex<()>,
    
    state: Mutex<BucketState>,
}

#[derive(Debug, Clone, Default)]
struct BucketState {
    remaining: Option<i64>,
    limit: Option<i64>,
    reset_at: Option<Instant>,
}

impl Bucket {
    fn new() -> Self {
        Self {
            queue: Mutex::new(()),
            state: Mutex::new(BucketState::default()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RateLimiter {
    buckets: Arc<DashMap<String, Arc<Bucket>>>,
    global_lock: Arc<Mutex<()>>,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            buckets: Arc::new(DashMap::new()),
            global_lock: Arc::new(Mutex::new(())),
        }
    }

    pub async fn await_bucket(&self, route: &Route<'_>) {
        // 1. Check global lock (wait if locked)
        {
            let _g = self.global_lock.lock().await;
        }

        let key = route.bucket_key();
        let bucket = self.buckets.entry(key).or_insert_with(|| Arc::new(Bucket::new())).clone();

        // 2. Lock the bucket queue
        let _guard = bucket.queue.lock().await;

        // 3. Check state
        let mut state = bucket.state.lock().await;
        
        if let Some(reset_at) = state.reset_at {
            let now = Instant::now();
            if now < reset_at {
                if let Some(remaining) = state.remaining {
                     if remaining <= 0 {
                         let diff = reset_at - now;
                         debug!("Bucket exhausted for {}, sleeping {:?}", route.path, diff);
                         sleep(diff).await;
                     }
                }
            }
        }
        
        // Optimistically decrement? 
        // We don't really know if we will succeed until we get headers. 
        // But for strict adherence, we should decrement.
        if let Some(ref mut rem) = state.remaining {
            if *rem > 0 {
                *rem -= 1;
            }
        }
        
        // Release state lock, keep queue lock?
        // No, we must release queue lock so the request can proceed. 
        // But then another request can enter.
        // The standard way: we consumed our "slot".
    }

    pub async fn update(&self, route: &Route<'_>, headers: &HeaderMap) {
        // Parse headers
        // X-RateLimit-Limit
        // X-RateLimit-Remaining
        // X-RateLimit-Reset-After (seconds)
        // X-RateLimit-Reset (epoch seconds)
        
        let remaining = headers.get("x-ratelimit-remaining")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<i64>().ok());
            
        let limit = headers.get("x-ratelimit-limit")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<i64>().ok());
            
        let reset_after = headers.get("x-ratelimit-reset-after")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<f64>().ok());

        if remaining.is_none() {
            return;
        }

        let key = route.bucket_key();
        if let Some(bucket) = self.buckets.get(&key) {
            let mut state = bucket.state.lock().await;
            state.remaining = remaining;
            state.limit = limit;
            
            if let Some(secs) = reset_after {
                state.reset_at = Some(Instant::now() + Duration::from_secs_f64(secs));
            }
        }
    }

    pub async fn handle_global_limit(&self, retry_after: f64) {
        warn!("Global rate limit hit! Sleeping for {}s", retry_after);
        let guard = self.global_lock.lock().await;
        sleep(Duration::from_secs_f64(retry_after)).await;
        drop(guard);
    }
}
