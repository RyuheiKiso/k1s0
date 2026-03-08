use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entity::flow_instance::FlowInstance;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait FlowInstanceRepository: Send + Sync {
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<FlowInstance>>;
    async fn find_by_correlation_id(
        &self,
        correlation_id: String,
    ) -> anyhow::Result<Option<FlowInstance>>;
    async fn find_by_flow_id_paginated(
        &self,
        flow_id: &Uuid,
        page: u32,
        page_size: u32,
    ) -> anyhow::Result<(Vec<FlowInstance>, u64)>;
    #[allow(dead_code)]
    async fn find_in_progress(&self) -> anyhow::Result<Vec<FlowInstance>>;
    async fn create(&self, instance: &FlowInstance) -> anyhow::Result<()>;
    async fn update(&self, instance: &FlowInstance) -> anyhow::Result<()>;
}
