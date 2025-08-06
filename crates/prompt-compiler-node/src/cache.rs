use moka::future::Cache;
use std::time::Duration;
use crate::ChatCompletionResponse;

pub struct ResponseCache {
    cache: Cache<String, ChatCompletionResponse>,
}

impl ResponseCache {
    pub async fn new(max_capacity: u64) -> Self {
        let cache = Cache::builder()
            .max_capacity(max_capacity)
            .time_to_live(Duration::from_secs(3600)) // 1 hour TTL
            .build();

        Self { cache }
    }

    pub async fn get(&self, key: &str) -> Option<ChatCompletionResponse> {
        self.cache.get(key).await
    }

    pub async fn insert(&self, key: String, response: ChatCompletionResponse) {
        self.cache.insert(key, response).await;
    }

    pub async fn invalidate(&self, key: &str) {
        self.cache.invalidate(key).await;
    }

    pub async fn stats(&self) -> CacheStats {
        CacheStats {
            hit_count: self.cache.weighted_size(),
            entry_count: self.cache.entry_count(),
        }
    }
}

#[derive(Debug)]
pub struct CacheStats {
    pub hit_count: u64,
    pub entry_count: u64,
}
