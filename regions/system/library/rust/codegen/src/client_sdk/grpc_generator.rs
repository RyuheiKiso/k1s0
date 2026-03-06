use super::ClientSdkConfig;

pub fn generate(config: &ClientSdkConfig) -> String {
    let service = &config.service_name;

    let mut method_impls = String::new();
    for m in &config.methods {
        method_impls.push_str(&format!(
            r#"    async fn {name}(&self, request: {req}) -> Result<{res}, ClientError> {{
        let grpc_request = tonic::Request::new(request.into());
        let response = self
            .client
            .lock()
            .await
            .{name}(grpc_request)
            .await
            .map_err(|e| ClientError::Transport(e.to_string()))?;
        Ok(response.into_inner().into())
    }}

"#,
            name = m.name,
            req = m.request_type,
            res = m.response_type,
        ));
    }

    format!(
        r#"use async_trait::async_trait;
use tonic::transport::Channel;
use tokio::sync::Mutex;

use crate::client::{service}Client;
use crate::error::ClientError;
use crate::types::*;

pub struct Grpc{service}Client {{
    client: Mutex<Channel>,
}}

impl Grpc{service}Client {{
    pub fn new(channel: Channel) -> Self {{
        Self {{
            client: Mutex::new(channel),
        }}
    }}
}}

#[async_trait]
impl {service}Client for Grpc{service}Client {{
{method_impls}}}
"#,
        service = service,
        method_impls = method_impls,
    )
}
