//! gRPCサービス実装

use std::pin::Pin;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use tokio_stream::Stream;
use tonic::{Request, Response, Status};

use crate::application::ConfigService;
use crate::domain::SettingRepository;
use crate::infrastructure::cache::SettingCache;

// Generated code from proto
pub mod config_v1 {
    tonic::include_proto!("k1s0.config.v1");
}

use config_v1::config_service_server::ConfigService as ConfigServiceTrait;
use config_v1::{
    GetSettingRequest, GetSettingResponse, ListSettingsRequest, ListSettingsResponse,
    Setting as ProtoSetting, WatchSettingsRequest, WatchSettingsResponse,
};

/// gRPCサービス実装
pub struct GrpcConfigService<R, C>
where
    R: SettingRepository + 'static,
    C: SettingCache + 'static,
{
    service: Arc<ConfigService<R, C>>,
}

impl<R, C> GrpcConfigService<R, C>
where
    R: SettingRepository + 'static,
    C: SettingCache + 'static,
{
    /// 新しいgRPCサービスを作成
    pub fn new(service: Arc<ConfigService<R, C>>) -> Self {
        Self { service }
    }
}

fn system_time_to_rfc3339(time: SystemTime) -> String {
    let duration = time.duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = duration.as_secs();
    let datetime = chrono::DateTime::from_timestamp(secs as i64, 0)
        .unwrap_or_else(|| chrono::DateTime::from_timestamp(0, 0).unwrap());
    datetime.format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

fn setting_to_proto(setting: crate::domain::Setting) -> ProtoSetting {
    ProtoSetting {
        setting_id: setting.id,
        service_name: setting.service_name,
        env: setting.env,
        setting_key: setting.key,
        value_type: setting.value_type.as_str().to_string(),
        setting_value: setting.value,
        description: setting.description.unwrap_or_default(),
        status: if setting.is_active { 1 } else { 0 },
        created_at: system_time_to_rfc3339(setting.created_at),
        updated_at: system_time_to_rfc3339(setting.updated_at),
    }
}

#[tonic::async_trait]
impl<R, C> ConfigServiceTrait for GrpcConfigService<R, C>
where
    R: SettingRepository + 'static,
    C: SettingCache + 'static,
{
    async fn get_setting(
        &self,
        request: Request<GetSettingRequest>,
    ) -> Result<Response<GetSettingResponse>, Status> {
        let req = request.into_inner();

        let env = if req.env.is_empty() {
            None
        } else {
            Some(req.env.as_str())
        };

        let setting = self
            .service
            .get_setting(&req.service_name, &req.setting_key, env)
            .await
            .map_err(|e| Status::new(e.to_grpc_code().into(), e.to_string()))?;

        Ok(Response::new(GetSettingResponse {
            setting: Some(setting_to_proto(setting)),
        }))
    }

    async fn list_settings(
        &self,
        request: Request<ListSettingsRequest>,
    ) -> Result<Response<ListSettingsResponse>, Status> {
        let req = request.into_inner();

        let mut query = crate::domain::SettingQuery::new();
        if !req.service_name.is_empty() {
            query = query.with_service_name(&req.service_name);
        }
        if !req.key_prefix.is_empty() {
            query = query.with_key_prefix(&req.key_prefix);
        }
        if !req.env.is_empty() {
            query = query.with_env(&req.env);
        }
        if req.page_size > 0 {
            query = query.with_page_size(req.page_size as u32);
        }
        if !req.page_token.is_empty() {
            query = query.with_page_token(&req.page_token);
        }

        let result = self
            .service
            .list_settings(&query)
            .await
            .map_err(|e| Status::new(e.to_grpc_code().into(), e.to_string()))?;

        Ok(Response::new(ListSettingsResponse {
            settings: result.settings.into_iter().map(setting_to_proto).collect(),
            next_page_token: result.next_page_token.unwrap_or_default(),
        }))
    }

    type WatchSettingsStream =
        Pin<Box<dyn Stream<Item = Result<WatchSettingsResponse, Status>> + Send>>;

    async fn watch_settings(
        &self,
        _request: Request<WatchSettingsRequest>,
    ) -> Result<Response<Self::WatchSettingsStream>, Status> {
        // WatchSettings is not implemented yet
        // Return an error indicating this feature is not available
        Err(Status::unimplemented(
            "WatchSettings is not implemented yet",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_time_to_rfc3339() {
        let time = SystemTime::UNIX_EPOCH;
        let result = system_time_to_rfc3339(time);
        assert_eq!(result, "1970-01-01T00:00:00Z");
    }
}
