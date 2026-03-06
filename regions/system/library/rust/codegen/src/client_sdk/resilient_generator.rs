use super::ClientSdkConfig;

pub fn generate(config: &ClientSdkConfig) -> String {
    let service = &config.service_name;

    let mut method_impls = String::new();
    for m in &config.methods {
        method_impls.push_str(&format!(
            r#"    async fn {name}(&self, request: {req}) -> Result<{res}, ClientError> {{
        self.execute(|client| {{
            let req = request.clone();
            async move {{ client.{name}(req).await }}
        }})
        .await
    }}

"#,
            name = m.name,
            req = m.request_type,
            res = m.response_type,
        ));
    }

    format!(
        r#"use async_trait::async_trait;
use std::sync::atomic::{{AtomicU32, Ordering}};
use std::time::Duration;
use std::future::Future;

use crate::client::{service}Client;
use crate::error::ClientError;
use crate::types::*;

pub struct Resilient{service}Client<T: {service}Client> {{
    inner: T,
    max_retries: u32,
    timeout: Duration,
    circuit_breaker_threshold: u32,
    failure_count: AtomicU32,
}}

impl<T: {service}Client> Resilient{service}Client<T> {{
    pub fn new(inner: T) -> Self {{
        Self {{
            inner,
            max_retries: 3,
            timeout: Duration::from_secs(5),
            circuit_breaker_threshold: 5,
            failure_count: AtomicU32::new(0),
        }}
    }}

    pub fn with_max_retries(mut self, max_retries: u32) -> Self {{
        self.max_retries = max_retries;
        self
    }}

    pub fn with_timeout(mut self, timeout: Duration) -> Self {{
        self.timeout = timeout;
        self
    }}

    pub fn with_circuit_breaker_threshold(mut self, threshold: u32) -> Self {{
        self.circuit_breaker_threshold = threshold;
        self
    }}

    async fn execute<F, Fut, R>(&self, f: F) -> Result<R, ClientError>
    where
        F: Fn(&T) -> Fut,
        Fut: Future<Output = Result<R, ClientError>>,
    {{
        if self.failure_count.load(Ordering::Relaxed) >= self.circuit_breaker_threshold {{
            return Err(ClientError::CircuitBreakerOpen);
        }}

        let mut last_error = None;
        for _ in 0..=self.max_retries {{
            match tokio::time::timeout(self.timeout, f(&self.inner)).await {{
                Ok(Ok(response)) => {{
                    self.failure_count.store(0, Ordering::Relaxed);
                    return Ok(response);
                }}
                Ok(Err(e)) => {{
                    self.failure_count.fetch_add(1, Ordering::Relaxed);
                    last_error = Some(e);
                }}
                Err(_) => {{
                    self.failure_count.fetch_add(1, Ordering::Relaxed);
                    last_error = Some(ClientError::Timeout(self.timeout));
                }}
            }}
        }}

        Err(last_error.unwrap())
    }}
}}

#[async_trait]
impl<T: {service}Client> {service}Client for Resilient{service}Client<T> {{
{method_impls}}}
"#,
        service = service,
        method_impls = method_impls,
    )
}
