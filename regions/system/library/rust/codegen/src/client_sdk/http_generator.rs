use super::ClientSdkConfig;

pub fn generate(config: &ClientSdkConfig) -> String {
    let service = &config.service_name;

    let mut method_impls = String::new();
    for m in &config.methods {
        method_impls.push_str(&format!(
            r#"    async fn {name}(&self, request: {req}) -> Result<{res}, ClientError> {{
        let url = format!("{{}}/{endpoint}", self.base_url);
        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| ClientError::Transport(e.to_string()))?;
        let status = response.status().as_u16() as u32;
        if !response.status().is_success() {{
            let message = response.text().await.unwrap_or_default();
            return Err(ClientError::Request {{ status, message }});
        }}
        response
            .json::<{res}>()
            .await
            .map_err(|e| ClientError::Serialization(e.to_string()))
    }}

"#,
            name = m.name,
            req = m.request_type,
            res = m.response_type,
            endpoint = m.name,
        ));
    }

    format!(
        r#"use async_trait::async_trait;
use reqwest::Client;

use crate::client::{service}Client;
use crate::error::ClientError;
use crate::types::*;

pub struct Http{service}Client {{
    client: Client,
    base_url: String,
}}

impl Http{service}Client {{
    pub fn new(base_url: impl Into<String>) -> Self {{
        Self {{
            client: Client::new(),
            base_url: base_url.into(),
        }}
    }}
}}

#[async_trait]
impl {service}Client for Http{service}Client {{
{method_impls}}}
"#,
        service = service,
        method_impls = method_impls,
    )
}
