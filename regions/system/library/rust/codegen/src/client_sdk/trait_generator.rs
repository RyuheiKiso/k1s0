use super::ClientSdkConfig;

pub fn generate(config: &ClientSdkConfig) -> String {
    let mut methods = String::new();
    for m in &config.methods {
        methods.push_str(&format!(
            "    async fn {name}(&self, request: {req}) -> Result<{res}, ClientError>;\n",
            name = m.name,
            req = m.request_type,
            res = m.response_type,
        ));
    }

    format!(
        r#"use async_trait::async_trait;
use crate::error::ClientError;
use crate::types::*;

#[async_trait]
pub trait {service_name}Client: Send + Sync {{
{methods}}}
"#,
        service_name = config.service_name,
        methods = methods,
    )
}
