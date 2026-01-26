//! gRPCサービス実装

use std::pin::Pin;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::{Stream, StreamExt};
use tonic::{Request, Response, Status};

use crate::application::{ConfigService, SettingChangeEvent};
use crate::domain::SettingRepository;
use crate::infrastructure::cache::SettingCache;

// Generated code from proto
pub mod config_v1 {
    tonic::include_proto!("k1s0.config.v1");
}

use config_v1::config_service_server::ConfigService as ConfigServiceTrait;
use config_v1::{
    ChangeType, GetSettingRequest, GetSettingResponse, ListSettingsRequest, ListSettingsResponse,
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
        request: Request<WatchSettingsRequest>,
    ) -> Result<Response<Self::WatchSettingsStream>, Status> {
        let req = request.into_inner();
        let service_name_filter = if req.service_name.is_empty() {
            None
        } else {
            Some(req.service_name)
        };
        let key_prefix_filter = if req.key_prefix.is_empty() {
            None
        } else {
            Some(req.key_prefix)
        };

        // 変更通知チャネルを購読
        let receiver = self.service.subscribe();

        // BroadcastStreamに変換し、フィルタリングとマッピングを適用
        let stream = BroadcastStream::new(receiver)
            .filter_map(move |result| {
                let service_filter = service_name_filter.clone();
                let key_filter = key_prefix_filter.clone();

                async move {
                    match result {
                        Ok(event) => {
                            // イベントに応じてフィルタリング
                            let (setting, change_type) = match &event {
                                SettingChangeEvent::Updated(setting) => {
                                    // サービス名フィルタ
                                    if let Some(ref filter) = service_filter {
                                        if &setting.service_name != filter {
                                            return None;
                                        }
                                    }
                                    // キープレフィックスフィルタ
                                    if let Some(ref prefix) = key_filter {
                                        if !setting.key.starts_with(prefix) {
                                            return None;
                                        }
                                    }
                                    (Some(setting_to_proto(setting.clone())), ChangeType::Updated)
                                }
                                SettingChangeEvent::Deleted {
                                    service_name,
                                    env,
                                    key,
                                } => {
                                    // サービス名フィルタ
                                    if let Some(ref filter) = service_filter {
                                        if service_name != filter {
                                            return None;
                                        }
                                    }
                                    // キープレフィックスフィルタ
                                    if let Some(ref prefix) = key_filter {
                                        if !key.starts_with(prefix) {
                                            return None;
                                        }
                                    }
                                    // 削除イベント用の最小限のSetting
                                    let deleted_setting = ProtoSetting {
                                        setting_id: 0,
                                        service_name: service_name.clone(),
                                        env: env.clone(),
                                        setting_key: key.clone(),
                                        value_type: String::new(),
                                        setting_value: String::new(),
                                        description: String::new(),
                                        status: 0,
                                        created_at: String::new(),
                                        updated_at: String::new(),
                                    };
                                    (Some(deleted_setting), ChangeType::Deleted)
                                }
                            };

                            Some(Ok(WatchSettingsResponse {
                                setting,
                                change_type: change_type as i32,
                            }))
                        }
                        Err(_) => {
                            // ラグによるメッセージロストはスキップ
                            None
                        }
                    }
                }
            });

        Ok(Response::new(Box::pin(stream) as Self::WatchSettingsStream))
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
