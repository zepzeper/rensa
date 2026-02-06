use crate::cache::CacheManager;
use crate::error::{RensaError, Result};
use serde::{Deserialize, Serialize};
use std::time::Duration;

const DEFAULT_RETRIES: u32 = 3;
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug, Clone)]
pub struct HttpClient {
    client: reqwest::Client,
    retries: u32,
    timeout: Duration,
    cache: Option<CacheManager>,
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpClient {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(DEFAULT_TIMEOUT)
            .build()
            .expect("Failed to build HTTP client");

        Self {
            client,
            retries: DEFAULT_RETRIES,
            timeout: DEFAULT_TIMEOUT,
            cache: None,
        }
    }

    pub fn with_cache(mut self, cache: CacheManager) -> Self {
        self.cache = Some(cache);
        self
    }

    pub fn with_retries(mut self, retries: u32) -> Self {
        self.retries = retries;
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    fn cache_key_from_url(&self, url: &str) -> String {
        let parts: Vec<&str> = url.split('/').collect();
        if let Some(last) = parts.last() {
            last.to_string()
        } else {
            url.to_string()
        }
    }

    async fn fetch<T>(&self, url: &str) -> Result<T>
    where
        T: for<'a> Deserialize<'a>,
    {
        for attempt in 0..=self.retries {
            match self.client.get(url).send().await {
                Ok(response) if response.status().is_success() => {
                    return Ok(response.json().await.map_err(|e| RensaError::RegistryError {
                        registry: url.to_string(),
                        source: e,
                    })?);
                }
                Ok(_response) => {
                    if attempt < self.retries {
                        let delay = Duration::from_secs(2u64.pow(attempt));
                        tokio::time::sleep(delay).await;
                    }
                }
                Err(_e) if attempt < self.retries => {
                    let delay = Duration::from_secs(2u64.pow(attempt));
                    tokio::time::sleep(delay).await;
                }
                Err(e) => {
                    return Err(RensaError::RegistryError {
                        registry: url.to_string(),
                        source: e,
                    });
                }
            }
        }
        unreachable!()
    }

    async fn fetch_post<T, B>(&self, url: &str, body: &B) -> Result<T>
    where
        T: for<'a> Deserialize<'a>,
        B: serde::Serialize,
    {
        for attempt in 0..=self.retries {
            match self.client.post(url).json(body).send().await {
                Ok(response) if response.status().is_success() => {
                    return Ok(response.json().await.map_err(|e| RensaError::RegistryError {
                        registry: url.to_string(),
                        source: e,
                    })?);
                }
                Ok(_response) => {
                    if attempt < self.retries {
                        let delay = Duration::from_secs(2u64.pow(attempt));
                        tokio::time::sleep(delay).await;
                    }
                }
                Err(_e) if attempt < self.retries => {
                    let delay = Duration::from_secs(2u64.pow(attempt));
                    tokio::time::sleep(delay).await;
                }
                Err(e) => {
                    return Err(RensaError::RegistryError {
                        registry: url.to_string(),
                        source: e,
                    });
                }
            }
        }
        unreachable!()
    }

    pub async fn get<T>(&self, url: &str) -> Result<T>
    where
        T: for<'a> Deserialize<'a> + Clone + Serialize,
    {
        if let Some(ref cache) = self.cache {
            let key = self.cache_key_from_url(url);

            if let Some(entry) = cache.get::<T>("api", &key).ok().flatten() {
                return Ok(entry.data().clone());
            }

            let result = self.fetch::<T>(url).await?;

            if let Err(e) = cache.set("api", &key, &result) {
                eprintln!("Warning: Failed to cache API response: {}", e);
            }

            Ok(result)
        } else {
            self.fetch::<T>(url).await
        }
    }

    pub async fn post<T, B>(&self, url: &str, body: &B) -> Result<T>
    where
        T: for<'a> Deserialize<'a> + Clone + Serialize,
        B: std::fmt::Debug + Serialize,
    {
        if let Some(ref cache) = self.cache {
            let key = format!("{}-{:?}", url, body);

            if let Some(entry) = cache.get::<T>("api", &key).ok().flatten() {
                return Ok(entry.data().clone());
            }

            let result = self.fetch_post::<T, B>(url, body).await?;

            if let Err(e) = cache.set("api", &key, &result) {
                eprintln!("Warning: Failed to cache API response: {}", e);
            }

            Ok(result)
        } else {
            self.fetch_post::<T, B>(url, body).await
        }
    }
}
