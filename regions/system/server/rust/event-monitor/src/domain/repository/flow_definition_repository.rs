use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entity::flow_definition::FlowDefinition;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait FlowDefinitionRepository: Send + Sync {
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<FlowDefinition>>;
    async fn find_all(&self) -> anyhow::Result<Vec<FlowDefinition>>;
    async fn find_all_paginated(
        &self,
        page: u32,
        page_size: u32,
        domain: Option<String>,
    ) -> anyhow::Result<(Vec<FlowDefinition>, u64)>;
    #[allow(dead_code)]
    async fn find_by_domain_and_event_type(
        &self,
        domain: String,
        event_type: String,
    ) -> anyhow::Result<Vec<FlowDefinition>>;
    async fn create(&self, flow: &FlowDefinition) -> anyhow::Result<()>;
    async fn update(&self, flow: &FlowDefinition) -> anyhow::Result<()>;
    async fn delete(&self, id: &Uuid) -> anyhow::Result<bool>;
    async fn exists_by_name(&self, name: String) -> anyhow::Result<bool>;
}
