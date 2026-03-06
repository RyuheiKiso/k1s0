use super::ClientSdkConfig;

pub fn generate(config: &ClientSdkConfig) -> String {
    let service = &config.service_name;

    let mut mock_methods = String::new();
    for m in &config.methods {
        mock_methods.push_str(&format!(
            "        async fn {name}(&self, request: {req}) -> Result<{res}, ClientError>;\n",
            name = m.name,
            req = m.request_type,
            res = m.response_type,
        ));
    }

    format!(
        r#"use mockall::mock;
use async_trait::async_trait;

use crate::client::{service}Client;
use crate::error::ClientError;
use crate::types::*;

mock! {{
    pub {service}Client {{}}

    #[async_trait]
    impl {service}Client for {service}Client {{
{mock_methods}    }}
}}

pub type Mock{service}Client = Mock{service}Client;
"#,
        service = service,
        mock_methods = mock_methods,
    )
}
