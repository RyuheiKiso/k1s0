// プロジェクトマスタイベントパブリッシャー trait。
// Kafka への発行を抽象化する。
use async_trait::async_trait;

/// プロジェクトタイプ変更イベント（Kafka へのシリアライズに Serialize が必要）
#[derive(Debug, Clone, serde::Serialize)]
pub struct ProjectTypeChangedEvent {
    pub project_type_id: String,
    pub code: String,
    pub change_type: String,
}

/// ステータス定義変更イベント（Kafka へのシリアライズに Serialize が必要）
#[derive(Debug, Clone, serde::Serialize)]
pub struct StatusDefinitionChangedEvent {
    pub status_definition_id: String,
    pub project_type_id: String,
    pub code: String,
    pub change_type: String,
    pub version_number: i32,
}

/// テナント拡張変更イベント（Kafka へのシリアライズに Serialize が必要）
#[derive(Debug, Clone, serde::Serialize)]
pub struct TenantExtensionChangedEvent {
    pub tenant_id: String,
    pub status_definition_id: String,
    pub change_type: String,
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ProjectMasterEventPublisher: Send + Sync {
    /// プロジェクトタイプ変更を発行する
    async fn publish_project_type_changed(
        &self,
        event: &ProjectTypeChangedEvent,
    ) -> anyhow::Result<()>;
    /// ステータス定義変更を発行する
    async fn publish_status_definition_changed(
        &self,
        event: &StatusDefinitionChangedEvent,
    ) -> anyhow::Result<()>;
    /// テナント拡張変更を発行する
    async fn publish_tenant_extension_changed(
        &self,
        event: &TenantExtensionChangedEvent,
    ) -> anyhow::Result<()>;
}

/// テスト・開発用の Noop パブリッシャー（実際には何も発行しない）
pub struct NoopProjectMasterEventPublisher;

#[async_trait]
impl ProjectMasterEventPublisher for NoopProjectMasterEventPublisher {
    async fn publish_project_type_changed(
        &self,
        _event: &ProjectTypeChangedEvent,
    ) -> anyhow::Result<()> {
        Ok(())
    }
    async fn publish_status_definition_changed(
        &self,
        _event: &StatusDefinitionChangedEvent,
    ) -> anyhow::Result<()> {
        Ok(())
    }
    async fn publish_tenant_extension_changed(
        &self,
        _event: &TenantExtensionChangedEvent,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}
