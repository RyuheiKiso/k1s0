#![allow(clippy::unwrap_used)]
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tokio::sync::RwLock;
use uuid::Uuid;

use k1s0_event_monitor_server::domain::entity::event_record::EventRecord;
use k1s0_event_monitor_server::domain::entity::flow_definition::{
    FlowDefinition, FlowSlo, FlowStep,
};
use k1s0_event_monitor_server::domain::entity::flow_instance::{FlowInstance, FlowInstanceStatus};
use k1s0_event_monitor_server::domain::repository::{
    EventRecordRepository, FlowDefinitionRepository, FlowInstanceRepository,
};
use k1s0_event_monitor_server::infrastructure::dlq_client::{
    DlqManagerClient, ReplayPreviewResponse, ReplayRequest, ReplayResponse,
};

use k1s0_event_monitor_server::usecase::create_flow::{
    CreateFlowError, CreateFlowInput, CreateFlowUseCase,
};
use k1s0_event_monitor_server::usecase::delete_flow::{DeleteFlowError, DeleteFlowUseCase};
use k1s0_event_monitor_server::usecase::execute_replay::{
    ExecuteReplayError, ExecuteReplayInput, ExecuteReplayUseCase,
};
use k1s0_event_monitor_server::usecase::get_flow::{GetFlowError, GetFlowUseCase};
use k1s0_event_monitor_server::usecase::get_flow_instance::{
    GetFlowInstanceError, GetFlowInstanceUseCase,
};
use k1s0_event_monitor_server::usecase::get_flow_instances::{
    GetFlowInstancesError, GetFlowInstancesInput, GetFlowInstancesUseCase,
};
use k1s0_event_monitor_server::usecase::get_flow_kpi::{GetFlowKpiError, GetFlowKpiUseCase};
use k1s0_event_monitor_server::usecase::get_kpi_summary::{
    GetKpiSummaryError, GetKpiSummaryUseCase,
};
use k1s0_event_monitor_server::usecase::get_slo_burn_rate::{
    GetSloBurnRateError, GetSloBurnRateUseCase,
};
use k1s0_event_monitor_server::usecase::get_slo_status::{GetSloStatusError, GetSloStatusUseCase};
use k1s0_event_monitor_server::usecase::list_events::{
    ListEventsError, ListEventsInput, ListEventsUseCase,
};
use k1s0_event_monitor_server::usecase::list_flows::{
    ListFlowsError, ListFlowsInput, ListFlowsUseCase,
};
use k1s0_event_monitor_server::usecase::preview_replay::{
    PreviewReplayError, PreviewReplayInput, PreviewReplayUseCase,
};
use k1s0_event_monitor_server::usecase::trace_by_correlation::{
    TraceByCorrelationError, TraceByCorrelationUseCase,
};
use k1s0_event_monitor_server::usecase::update_flow::{
    UpdateFlowError, UpdateFlowInput, UpdateFlowUseCase,
};

// ============================================================================
// Test Stub: In-Memory EventRecordRepository
// ============================================================================

struct StubEventRecordRepository {
    records: RwLock<Vec<EventRecord>>,
    should_fail: bool,
}

impl StubEventRecordRepository {
    fn new() -> Self {
        Self {
            records: RwLock::new(Vec::new()),
            should_fail: false,
        }
    }

    fn with_error() -> Self {
        Self {
            records: RwLock::new(Vec::new()),
            should_fail: true,
        }
    }

    fn with_records(records: Vec<EventRecord>) -> Self {
        Self {
            records: RwLock::new(records),
            should_fail: false,
        }
    }
}

#[async_trait]
impl EventRecordRepository for StubEventRecordRepository {
    async fn create(&self, record: &EventRecord) -> anyhow::Result<()> {
        if self.should_fail {
            return Err(anyhow::anyhow!("storage backend unavailable"));
        }
        self.records.write().await.push(record.clone());
        Ok(())
    }

    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<EventRecord>> {
        if self.should_fail {
            return Err(anyhow::anyhow!("storage backend unavailable"));
        }
        let records = self.records.read().await;
        Ok(records.iter().find(|r| &r.id == id).cloned())
    }

    async fn find_all_paginated(
        &self,
        page: u32,
        page_size: u32,
        domain: Option<String>,
        event_type: Option<String>,
        source: Option<String>,
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
        status: Option<String>,
    ) -> anyhow::Result<(Vec<EventRecord>, u64)> {
        if self.should_fail {
            return Err(anyhow::anyhow!("storage backend unavailable"));
        }
        let records = self.records.read().await;
        let filtered: Vec<EventRecord> = records
            .iter()
            .filter(|r| domain.as_ref().is_none_or(|d| &r.domain == d))
            .filter(|r| event_type.as_ref().is_none_or(|et| &r.event_type == et))
            .filter(|r| source.as_ref().is_none_or(|s| &r.source == s))
            .filter(|r| from.is_none_or(|f| r.timestamp >= f))
            .filter(|r| to.is_none_or(|t| r.timestamp <= t))
            .filter(|r| status.as_ref().is_none_or(|s| &r.status == s))
            .cloned()
            .collect();

        let total = filtered.len() as u64;
        let start = ((page - 1) * page_size) as usize;
        let result: Vec<EventRecord> = filtered
            .into_iter()
            .skip(start)
            .take(page_size as usize)
            .collect();
        Ok((result, total))
    }

    async fn find_by_correlation_id(
        &self,
        correlation_id: String,
    ) -> anyhow::Result<Vec<EventRecord>> {
        if self.should_fail {
            return Err(anyhow::anyhow!("storage backend unavailable"));
        }
        let records = self.records.read().await;
        Ok(records
            .iter()
            .filter(|r| r.correlation_id == correlation_id)
            .cloned()
            .collect())
    }
}

// ============================================================================
// Test Stub: In-Memory FlowDefinitionRepository
// ============================================================================

struct StubFlowDefinitionRepository {
    flows: RwLock<Vec<FlowDefinition>>,
    should_fail: bool,
}

impl StubFlowDefinitionRepository {
    fn new() -> Self {
        Self {
            flows: RwLock::new(Vec::new()),
            should_fail: false,
        }
    }

    fn with_error() -> Self {
        Self {
            flows: RwLock::new(Vec::new()),
            should_fail: true,
        }
    }

    fn with_flows(flows: Vec<FlowDefinition>) -> Self {
        Self {
            flows: RwLock::new(flows),
            should_fail: false,
        }
    }
}

#[async_trait]
impl FlowDefinitionRepository for StubFlowDefinitionRepository {
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<FlowDefinition>> {
        if self.should_fail {
            return Err(anyhow::anyhow!("storage backend unavailable"));
        }
        let flows = self.flows.read().await;
        Ok(flows.iter().find(|f| &f.id == id).cloned())
    }

    async fn find_all(&self) -> anyhow::Result<Vec<FlowDefinition>> {
        if self.should_fail {
            return Err(anyhow::anyhow!("storage backend unavailable"));
        }
        Ok(self.flows.read().await.clone())
    }

    async fn find_all_paginated(
        &self,
        page: u32,
        page_size: u32,
        domain: Option<String>,
    ) -> anyhow::Result<(Vec<FlowDefinition>, u64)> {
        if self.should_fail {
            return Err(anyhow::anyhow!("storage backend unavailable"));
        }
        let flows = self.flows.read().await;
        let filtered: Vec<FlowDefinition> = flows
            .iter()
            .filter(|f| domain.as_ref().is_none_or(|d| &f.domain == d))
            .cloned()
            .collect();

        let total = filtered.len() as u64;
        let start = ((page - 1) * page_size) as usize;
        let result: Vec<FlowDefinition> = filtered
            .into_iter()
            .skip(start)
            .take(page_size as usize)
            .collect();
        Ok((result, total))
    }

    async fn find_by_domain_and_event_type(
        &self,
        domain: String,
        event_type: String,
    ) -> anyhow::Result<Vec<FlowDefinition>> {
        if self.should_fail {
            return Err(anyhow::anyhow!("storage backend unavailable"));
        }
        let flows = self.flows.read().await;
        Ok(flows
            .iter()
            .filter(|f| f.domain == domain && f.steps.iter().any(|s| s.event_type == event_type))
            .cloned()
            .collect())
    }

    async fn create(&self, flow: &FlowDefinition) -> anyhow::Result<()> {
        if self.should_fail {
            return Err(anyhow::anyhow!("storage backend unavailable"));
        }
        self.flows.write().await.push(flow.clone());
        Ok(())
    }

    async fn update(&self, flow: &FlowDefinition) -> anyhow::Result<()> {
        if self.should_fail {
            return Err(anyhow::anyhow!("storage backend unavailable"));
        }
        let mut flows = self.flows.write().await;
        if let Some(existing) = flows.iter_mut().find(|f| f.id == flow.id) {
            *existing = flow.clone();
            Ok(())
        } else {
            Err(anyhow::anyhow!("flow not found"))
        }
    }

    async fn delete(&self, id: &Uuid) -> anyhow::Result<bool> {
        if self.should_fail {
            return Err(anyhow::anyhow!("storage backend unavailable"));
        }
        let mut flows = self.flows.write().await;
        let before = flows.len();
        flows.retain(|f| &f.id != id);
        Ok(flows.len() < before)
    }

    async fn exists_by_name(&self, name: String) -> anyhow::Result<bool> {
        if self.should_fail {
            return Err(anyhow::anyhow!("storage backend unavailable"));
        }
        let flows = self.flows.read().await;
        Ok(flows.iter().any(|f| f.name == name))
    }
}

// ============================================================================
// Test Stub: In-Memory FlowInstanceRepository
// ============================================================================

struct StubFlowInstanceRepository {
    instances: RwLock<Vec<FlowInstance>>,
    should_fail: bool,
}

impl StubFlowInstanceRepository {
    fn new() -> Self {
        Self {
            instances: RwLock::new(Vec::new()),
            should_fail: false,
        }
    }

    fn with_error() -> Self {
        Self {
            instances: RwLock::new(Vec::new()),
            should_fail: true,
        }
    }

    fn with_instances(instances: Vec<FlowInstance>) -> Self {
        Self {
            instances: RwLock::new(instances),
            should_fail: false,
        }
    }
}

#[async_trait]
impl FlowInstanceRepository for StubFlowInstanceRepository {
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<FlowInstance>> {
        if self.should_fail {
            return Err(anyhow::anyhow!("storage backend unavailable"));
        }
        let instances = self.instances.read().await;
        Ok(instances.iter().find(|i| &i.id == id).cloned())
    }

    async fn find_by_correlation_id(
        &self,
        correlation_id: String,
    ) -> anyhow::Result<Option<FlowInstance>> {
        if self.should_fail {
            return Err(anyhow::anyhow!("storage backend unavailable"));
        }
        let instances = self.instances.read().await;
        Ok(instances
            .iter()
            .find(|i| i.correlation_id == correlation_id)
            .cloned())
    }

    async fn find_by_flow_id_paginated(
        &self,
        flow_id: &Uuid,
        page: u32,
        page_size: u32,
    ) -> anyhow::Result<(Vec<FlowInstance>, u64)> {
        if self.should_fail {
            return Err(anyhow::anyhow!("storage backend unavailable"));
        }
        let instances = self.instances.read().await;
        let filtered: Vec<FlowInstance> = instances
            .iter()
            .filter(|i| &i.flow_id == flow_id)
            .cloned()
            .collect();

        let total = filtered.len() as u64;
        let start = ((page - 1) * page_size) as usize;
        let result: Vec<FlowInstance> = filtered
            .into_iter()
            .skip(start)
            .take(page_size as usize)
            .collect();
        Ok((result, total))
    }

    async fn find_in_progress(&self) -> anyhow::Result<Vec<FlowInstance>> {
        if self.should_fail {
            return Err(anyhow::anyhow!("storage backend unavailable"));
        }
        let instances = self.instances.read().await;
        Ok(instances
            .iter()
            .filter(|i| i.status == FlowInstanceStatus::InProgress)
            .cloned()
            .collect())
    }

    async fn create(&self, instance: &FlowInstance) -> anyhow::Result<()> {
        if self.should_fail {
            return Err(anyhow::anyhow!("storage backend unavailable"));
        }
        self.instances.write().await.push(instance.clone());
        Ok(())
    }

    async fn update(&self, instance: &FlowInstance) -> anyhow::Result<()> {
        if self.should_fail {
            return Err(anyhow::anyhow!("storage backend unavailable"));
        }
        let mut instances = self.instances.write().await;
        if let Some(existing) = instances.iter_mut().find(|i| i.id == instance.id) {
            *existing = instance.clone();
            Ok(())
        } else {
            Err(anyhow::anyhow!("instance not found"))
        }
    }
}

// ============================================================================
// Test Stub: In-Memory DlqManagerClient
// ============================================================================

struct StubDlqClient {
    should_fail: bool,
}

impl StubDlqClient {
    fn new() -> Self {
        Self { should_fail: false }
    }

    fn with_error() -> Self {
        Self { should_fail: true }
    }
}

#[async_trait]
impl DlqManagerClient for StubDlqClient {
    async fn preview_replay(&self, _req: &ReplayRequest) -> anyhow::Result<ReplayPreviewResponse> {
        if self.should_fail {
            return Err(anyhow::anyhow!("DLQ service unavailable"));
        }
        Ok(ReplayPreviewResponse {
            total_events_to_replay: 3,
            affected_services: vec!["task-server".to_string()],
            dlq_messages_found: 2,
            estimated_duration_seconds: 10,
        })
    }

    async fn execute_replay(&self, _req: &ReplayRequest) -> anyhow::Result<ReplayResponse> {
        if self.should_fail {
            return Err(anyhow::anyhow!("DLQ service unavailable"));
        }
        Ok(ReplayResponse {
            replay_id: "replay-001".to_string(),
            status: "completed".to_string(),
            total_events: 3,
            replayed_events: 3,
        })
    }
}

// ============================================================================
// Helpers
// ============================================================================

fn make_flow_step(event_type: &str, source: &str) -> FlowStep {
    FlowStep {
        event_type: event_type.to_string(),
        source: source.to_string(),
        source_filter: Some(source.to_string()),
        timeout_seconds: 30,
        description: format!("{} step", event_type),
    }
}

fn make_slo() -> FlowSlo {
    FlowSlo {
        target_completion_seconds: 120,
        target_success_rate: 0.99,
        alert_on_violation: true,
    }
}

fn make_flow(name: &str, domain: &str) -> FlowDefinition {
    // テスト用フロー定義: tenant_id は "system" を使用する
    FlowDefinition::new(
        "system".to_string(),
        name.to_string(),
        format!("{} flow description", name),
        domain.to_string(),
        vec![
            make_flow_step("TaskCreated", "task-server"),
            make_flow_step("ActivityCreated", "activity-server"),
        ],
        make_slo(),
    )
}

fn make_flow_with_id(id: Uuid, name: &str, domain: &str) -> FlowDefinition {
    let now = Utc::now();
    // テスト用フロー定義: tenant_id は "system" を使用する
    FlowDefinition {
        id,
        tenant_id: "system".to_string(),
        name: name.to_string(),
        description: format!("{} flow description", name),
        domain: domain.to_string(),
        steps: vec![
            make_flow_step("TaskCreated", "task-server"),
            make_flow_step("ActivityCreated", "activity-server"),
        ],
        slo: make_slo(),
        enabled: true,
        created_at: now,
        updated_at: now,
    }
}

fn make_event_record(
    correlation_id: &str,
    event_type: &str,
    source: &str,
    domain: &str,
) -> EventRecord {
    // テスト用イベント記録: tenant_id は "system" を使用する
    EventRecord::new(
        "system".to_string(),
        correlation_id.to_string(),
        event_type.to_string(),
        source.to_string(),
        domain.to_string(),
        format!("trace-{}", correlation_id),
        Utc::now(),
    )
}

fn make_event_record_with_flow(
    correlation_id: &str,
    event_type: &str,
    source: &str,
    domain: &str,
    flow_id: Uuid,
    step_index: i32,
) -> EventRecord {
    let mut record = make_event_record(correlation_id, event_type, source, domain);
    record.flow_id = Some(flow_id);
    record.flow_step_index = Some(step_index);
    record
}

fn make_instance(flow_id: Uuid, correlation_id: &str, status: FlowInstanceStatus) -> FlowInstance {
    // テスト用フローインスタンス: tenant_id は "system" を使用する
    FlowInstance {
        id: Uuid::new_v4(),
        tenant_id: "system".to_string(),
        flow_id,
        correlation_id: correlation_id.to_string(),
        status,
        current_step_index: 0,
        started_at: Utc::now(),
        completed_at: None,
        duration_ms: None,
    }
}

fn make_instance_with_duration(
    flow_id: Uuid,
    status: FlowInstanceStatus,
    duration_ms: i64,
) -> FlowInstance {
    // テスト用フローインスタンス（継続時間付き）: tenant_id は "system" を使用する
    FlowInstance {
        id: Uuid::new_v4(),
        tenant_id: "system".to_string(),
        flow_id,
        correlation_id: format!("corr-{}", Uuid::new_v4()),
        status,
        current_step_index: 0,
        started_at: Utc::now(),
        completed_at: Some(Utc::now()),
        duration_ms: Some(duration_ms),
    }
}

fn make_instance_with_id(
    id: Uuid,
    flow_id: Uuid,
    correlation_id: &str,
    status: FlowInstanceStatus,
) -> FlowInstance {
    // テスト用フローインスタンス（ID指定）: tenant_id は "system" を使用する
    FlowInstance {
        id,
        tenant_id: "system".to_string(),
        flow_id,
        correlation_id: correlation_id.to_string(),
        status,
        current_step_index: 0,
        started_at: Utc::now(),
        completed_at: None,
        duration_ms: None,
    }
}

// ============================================================================
// CreateFlowUseCase Tests
// ============================================================================

#[tokio::test]
async fn test_create_flow_success() {
    let repo = Arc::new(StubFlowDefinitionRepository::new());

    let uc = CreateFlowUseCase::new(repo.clone());

    // テスト用入力: tenant_id は "system" を使用する
    let input = CreateFlowInput {
        tenant_id: "system".to_string(),
        name: "task_flow".to_string(),
        description: "Task assignment flow".to_string(),
        domain: "service.task".to_string(),
        steps: vec![make_flow_step("TaskCreated", "task-server")],
        slo: make_slo(),
    };

    let result = uc.execute(&input).await;
    assert!(result.is_ok());

    let flow = result.unwrap();
    assert_eq!(flow.name, "task_flow");
    assert_eq!(flow.domain, "service.task");
    assert!(flow.enabled);

    // Verify flow was persisted
    let flows = repo.flows.read().await;
    assert_eq!(flows.len(), 1);
    assert_eq!(flows[0].name, "task_flow");
}

#[tokio::test]
async fn test_create_flow_already_exists() {
    let existing = make_flow("task_flow", "service.task");
    let repo = Arc::new(StubFlowDefinitionRepository::with_flows(vec![existing]));

    let uc = CreateFlowUseCase::new(repo);

    // テスト用入力: tenant_id は "system" を使用する
    let input = CreateFlowInput {
        tenant_id: "system".to_string(),
        name: "task_flow".to_string(),
        description: "duplicate".to_string(),
        domain: "service.task".to_string(),
        steps: vec![make_flow_step("TaskCreated", "task-server")],
        slo: make_slo(),
    };

    let result = uc.execute(&input).await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        CreateFlowError::AlreadyExists(_)
    ));
}

#[tokio::test]
async fn test_create_flow_validation_empty_name() {
    let repo = Arc::new(StubFlowDefinitionRepository::new());

    let uc = CreateFlowUseCase::new(repo);

    // テスト用入力（名前が空のバリデーションエラーケース）
    let input = CreateFlowInput {
        tenant_id: "system".to_string(),
        name: "".to_string(),
        description: "test".to_string(),
        domain: "service.task".to_string(),
        steps: vec![make_flow_step("TaskCreated", "task-server")],
        slo: make_slo(),
    };

    let result = uc.execute(&input).await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        CreateFlowError::Validation(_)
    ));
}

#[tokio::test]
async fn test_create_flow_validation_empty_steps() {
    let repo = Arc::new(StubFlowDefinitionRepository::new());

    let uc = CreateFlowUseCase::new(repo);

    // テスト用入力（ステップが空のバリデーションエラーケース）
    let input = CreateFlowInput {
        tenant_id: "system".to_string(),
        name: "task_flow".to_string(),
        description: "test".to_string(),
        domain: "service.task".to_string(),
        steps: vec![],
        slo: make_slo(),
    };

    let result = uc.execute(&input).await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        CreateFlowError::Validation(_)
    ));
}

#[tokio::test]
async fn test_create_flow_internal_error() {
    let repo = Arc::new(StubFlowDefinitionRepository::with_error());

    let uc = CreateFlowUseCase::new(repo);

    // テスト用入力（内部エラーケース）
    let input = CreateFlowInput {
        tenant_id: "system".to_string(),
        name: "task_flow".to_string(),
        description: "test".to_string(),
        domain: "service.task".to_string(),
        steps: vec![make_flow_step("TaskCreated", "task-server")],
        slo: make_slo(),
    };

    let result = uc.execute(&input).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        CreateFlowError::Internal(msg) => {
            assert!(msg.contains("unavailable"));
        }
        e => panic!("expected Internal, got: {:?}", e),
    }
}

// ============================================================================
// GetFlowUseCase Tests
// ============================================================================

#[tokio::test]
async fn test_get_flow_success() {
    let flow_id = Uuid::new_v4();
    let flow = make_flow_with_id(flow_id, "task_flow", "service.task");
    let repo = Arc::new(StubFlowDefinitionRepository::with_flows(vec![flow]));

    let uc = GetFlowUseCase::new(repo);

    let result = uc.execute(&flow_id).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().name, "task_flow");
}

#[tokio::test]
async fn test_get_flow_not_found() {
    let repo = Arc::new(StubFlowDefinitionRepository::new());

    let uc = GetFlowUseCase::new(repo);

    let result = uc.execute(&Uuid::new_v4()).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), GetFlowError::NotFound(_)));
}

#[tokio::test]
async fn test_get_flow_internal_error() {
    let repo = Arc::new(StubFlowDefinitionRepository::with_error());

    let uc = GetFlowUseCase::new(repo);

    let result = uc.execute(&Uuid::new_v4()).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        GetFlowError::Internal(msg) => {
            assert!(msg.contains("unavailable"));
        }
        e => panic!("expected Internal, got: {:?}", e),
    }
}

// ============================================================================
// UpdateFlowUseCase Tests
// ============================================================================

#[tokio::test]
async fn test_update_flow_success_description() {
    let flow_id = Uuid::new_v4();
    let flow = make_flow_with_id(flow_id, "task_flow", "service.task");
    let repo = Arc::new(StubFlowDefinitionRepository::with_flows(vec![flow]));

    let uc = UpdateFlowUseCase::new(repo.clone());

    let input = UpdateFlowInput {
        id: flow_id,
        description: Some("Updated description".to_string()),
        steps: None,
        slo: None,
        enabled: None,
    };

    let result = uc.execute(&input).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().description, "Updated description");

    // Verify persisted
    let flows = repo.flows.read().await;
    assert_eq!(flows[0].description, "Updated description");
}

#[tokio::test]
async fn test_update_flow_success_disable() {
    let flow_id = Uuid::new_v4();
    let flow = make_flow_with_id(flow_id, "task_flow", "service.task");
    let repo = Arc::new(StubFlowDefinitionRepository::with_flows(vec![flow]));

    let uc = UpdateFlowUseCase::new(repo);

    let input = UpdateFlowInput {
        id: flow_id,
        description: None,
        steps: None,
        slo: None,
        enabled: Some(false),
    };

    let result = uc.execute(&input).await;
    assert!(result.is_ok());
    assert!(!result.unwrap().enabled);
}

#[tokio::test]
async fn test_update_flow_success_steps_and_slo() {
    let flow_id = Uuid::new_v4();
    let flow = make_flow_with_id(flow_id, "task_flow", "service.task");
    let repo = Arc::new(StubFlowDefinitionRepository::with_flows(vec![flow]));

    let uc = UpdateFlowUseCase::new(repo);

    let new_steps = vec![
        make_flow_step("TaskCreated", "task-server"),
        make_flow_step("ActivityCreated", "activity-server"),
        make_flow_step("BoardColumnIncremented", "board-server"),
    ];
    let new_slo = FlowSlo {
        target_completion_seconds: 300,
        target_success_rate: 0.95,
        alert_on_violation: false,
    };

    let input = UpdateFlowInput {
        id: flow_id,
        description: None,
        steps: Some(new_steps),
        slo: Some(new_slo),
        enabled: None,
    };

    let result = uc.execute(&input).await;
    assert!(result.is_ok());

    let updated = result.unwrap();
    assert_eq!(updated.steps.len(), 3);
    assert_eq!(updated.slo.target_completion_seconds, 300);
    assert!((updated.slo.target_success_rate - 0.95).abs() < f64::EPSILON);
}

#[tokio::test]
async fn test_update_flow_not_found() {
    let repo = Arc::new(StubFlowDefinitionRepository::new());

    let uc = UpdateFlowUseCase::new(repo);

    let input = UpdateFlowInput {
        id: Uuid::new_v4(),
        description: Some("test".to_string()),
        steps: None,
        slo: None,
        enabled: None,
    };

    let result = uc.execute(&input).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), UpdateFlowError::NotFound(_)));
}

#[tokio::test]
async fn test_update_flow_internal_error() {
    let repo = Arc::new(StubFlowDefinitionRepository::with_error());

    let uc = UpdateFlowUseCase::new(repo);

    let input = UpdateFlowInput {
        id: Uuid::new_v4(),
        description: Some("test".to_string()),
        steps: None,
        slo: None,
        enabled: None,
    };

    let result = uc.execute(&input).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        UpdateFlowError::Internal(msg) => {
            assert!(msg.contains("unavailable"));
        }
        e => panic!("expected Internal, got: {:?}", e),
    }
}

// ============================================================================
// DeleteFlowUseCase Tests
// ============================================================================

#[tokio::test]
async fn test_delete_flow_success() {
    let flow_id = Uuid::new_v4();
    let flow = make_flow_with_id(flow_id, "task_flow", "service.task");
    let repo = Arc::new(StubFlowDefinitionRepository::with_flows(vec![flow]));

    let uc = DeleteFlowUseCase::new(repo.clone());

    let result = uc.execute(&flow_id).await;
    assert!(result.is_ok());

    // Verify deleted
    let flows = repo.flows.read().await;
    assert!(flows.is_empty());
}

#[tokio::test]
async fn test_delete_flow_not_found() {
    let repo = Arc::new(StubFlowDefinitionRepository::new());

    let uc = DeleteFlowUseCase::new(repo);

    let result = uc.execute(&Uuid::new_v4()).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), DeleteFlowError::NotFound(_)));
}

#[tokio::test]
async fn test_delete_flow_internal_error() {
    let repo = Arc::new(StubFlowDefinitionRepository::with_error());

    let uc = DeleteFlowUseCase::new(repo);

    let result = uc.execute(&Uuid::new_v4()).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        DeleteFlowError::Internal(msg) => {
            assert!(msg.contains("unavailable"));
        }
        e => panic!("expected Internal, got: {:?}", e),
    }
}

#[tokio::test]
async fn test_delete_flow_preserves_other_flows() {
    let flow_id1 = Uuid::new_v4();
    let flow_id2 = Uuid::new_v4();
    let flows = vec![
        make_flow_with_id(flow_id1, "task_flow", "service.task"),
        make_flow_with_id(flow_id2, "activity_flow", "service.activity"),
    ];
    let repo = Arc::new(StubFlowDefinitionRepository::with_flows(flows));

    let uc = DeleteFlowUseCase::new(repo.clone());

    let result = uc.execute(&flow_id1).await;
    assert!(result.is_ok());

    let remaining = repo.flows.read().await;
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].name, "activity_flow");
}

// ============================================================================
// ListFlowsUseCase Tests
// ============================================================================

#[tokio::test]
async fn test_list_flows_success() {
    let flows = vec![
        make_flow("task_flow", "service.task"),
        make_flow("activity_flow", "service.activity"),
    ];
    let repo = Arc::new(StubFlowDefinitionRepository::with_flows(flows));

    let uc = ListFlowsUseCase::new(repo);

    let input = ListFlowsInput {
        page: 1,
        page_size: 50,
        domain: None,
    };

    let result = uc.execute(&input).await;
    assert!(result.is_ok());

    let output = result.unwrap();
    assert_eq!(output.flows.len(), 2);
    assert_eq!(output.total_count, 2);
    assert!(!output.has_next);
}

#[tokio::test]
async fn test_list_flows_with_domain_filter() {
    let flows = vec![
        make_flow("task_flow", "service.task"),
        make_flow("activity_flow", "service.activity"),
        make_flow("board_flow", "service.task"),
    ];
    let repo = Arc::new(StubFlowDefinitionRepository::with_flows(flows));

    let uc = ListFlowsUseCase::new(repo);

    let input = ListFlowsInput {
        page: 1,
        page_size: 50,
        domain: Some("service.task".to_string()),
    };

    let result = uc.execute(&input).await;
    assert!(result.is_ok());

    let output = result.unwrap();
    assert_eq!(output.flows.len(), 2);
    assert!(output.flows.iter().all(|f| f.domain == "service.task"));
}

#[tokio::test]
async fn test_list_flows_pagination() {
    let flows: Vec<FlowDefinition> = (1..=5)
        .map(|i| make_flow(&format!("flow_{}", i), "service.task"))
        .collect();
    let repo = Arc::new(StubFlowDefinitionRepository::with_flows(flows));

    let uc = ListFlowsUseCase::new(repo);

    let input = ListFlowsInput {
        page: 1,
        page_size: 2,
        domain: None,
    };

    let result = uc.execute(&input).await;
    assert!(result.is_ok());

    let output = result.unwrap();
    assert_eq!(output.flows.len(), 2);
    assert!(output.has_next);
    assert_eq!(output.total_count, 5);
}

#[tokio::test]
async fn test_list_flows_empty() {
    let repo = Arc::new(StubFlowDefinitionRepository::new());

    let uc = ListFlowsUseCase::new(repo);

    let input = ListFlowsInput {
        page: 1,
        page_size: 50,
        domain: None,
    };

    let result = uc.execute(&input).await;
    assert!(result.is_ok());
    assert!(result.unwrap().flows.is_empty());
}

#[tokio::test]
async fn test_list_flows_internal_error() {
    let repo = Arc::new(StubFlowDefinitionRepository::with_error());

    let uc = ListFlowsUseCase::new(repo);

    let input = ListFlowsInput {
        page: 1,
        page_size: 50,
        domain: None,
    };

    let result = uc.execute(&input).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        ListFlowsError::Internal(msg) => {
            assert!(msg.contains("unavailable"));
        }
    }
}

// ============================================================================
// GetFlowInstanceUseCase Tests
// ============================================================================

#[tokio::test]
async fn test_get_flow_instance_success() {
    let instance_id = Uuid::new_v4();
    let flow_id = Uuid::new_v4();
    let instance = make_instance_with_id(
        instance_id,
        flow_id,
        "corr-001",
        FlowInstanceStatus::InProgress,
    );
    let repo = Arc::new(StubFlowInstanceRepository::with_instances(vec![instance]));

    let uc = GetFlowInstanceUseCase::new(repo);

    let result = uc.execute(&instance_id).await;
    assert!(result.is_ok());

    let inst = result.unwrap();
    assert_eq!(inst.id, instance_id);
    assert_eq!(inst.correlation_id, "corr-001");
}

#[tokio::test]
async fn test_get_flow_instance_not_found() {
    let repo = Arc::new(StubFlowInstanceRepository::new());

    let uc = GetFlowInstanceUseCase::new(repo);

    let result = uc.execute(&Uuid::new_v4()).await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        GetFlowInstanceError::NotFound(_)
    ));
}

#[tokio::test]
async fn test_get_flow_instance_internal_error() {
    let repo = Arc::new(StubFlowInstanceRepository::with_error());

    let uc = GetFlowInstanceUseCase::new(repo);

    let result = uc.execute(&Uuid::new_v4()).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        GetFlowInstanceError::Internal(msg) => {
            assert!(msg.contains("unavailable"));
        }
        e => panic!("expected Internal, got: {:?}", e),
    }
}

// ============================================================================
// GetFlowInstancesUseCase Tests
// ============================================================================

#[tokio::test]
async fn test_get_flow_instances_success() {
    let flow_id = Uuid::new_v4();
    let instances = vec![
        make_instance(flow_id, "corr-001", FlowInstanceStatus::Completed),
        make_instance(flow_id, "corr-002", FlowInstanceStatus::InProgress),
        make_instance(Uuid::new_v4(), "corr-003", FlowInstanceStatus::Completed), // different flow
    ];
    let repo = Arc::new(StubFlowInstanceRepository::with_instances(instances));

    let uc = GetFlowInstancesUseCase::new(repo);

    let input = GetFlowInstancesInput {
        flow_id,
        page: 1,
        page_size: 50,
    };

    let result = uc.execute(&input).await;
    assert!(result.is_ok());

    let output = result.unwrap();
    assert_eq!(output.instances.len(), 2);
    assert_eq!(output.total_count, 2);
    assert!(output.instances.iter().all(|i| i.flow_id == flow_id));
}

#[tokio::test]
async fn test_get_flow_instances_pagination() {
    let flow_id = Uuid::new_v4();
    let instances: Vec<FlowInstance> = (1..=5)
        .map(|i| {
            make_instance(
                flow_id,
                &format!("corr-{:03}", i),
                FlowInstanceStatus::Completed,
            )
        })
        .collect();
    let repo = Arc::new(StubFlowInstanceRepository::with_instances(instances));

    let uc = GetFlowInstancesUseCase::new(repo);

    let input = GetFlowInstancesInput {
        flow_id,
        page: 1,
        page_size: 2,
    };

    let result = uc.execute(&input).await;
    assert!(result.is_ok());

    let output = result.unwrap();
    assert_eq!(output.instances.len(), 2);
    assert!(output.has_next);
}

#[tokio::test]
async fn test_get_flow_instances_internal_error() {
    let repo = Arc::new(StubFlowInstanceRepository::with_error());

    let uc = GetFlowInstancesUseCase::new(repo);

    let input = GetFlowInstancesInput {
        flow_id: Uuid::new_v4(),
        page: 1,
        page_size: 50,
    };

    let result = uc.execute(&input).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        GetFlowInstancesError::Internal(msg) => {
            assert!(msg.contains("unavailable"));
        }
    }
}

// ============================================================================
// ListEventsUseCase Tests
// ============================================================================

#[tokio::test]
async fn test_list_events_success() {
    let records = vec![
        make_event_record("corr-001", "TaskCreated", "task-server", "service.task"),
        make_event_record(
            "corr-002",
            "ActivityCreated",
            "activity-server",
            "service.activity",
        ),
    ];
    let repo = Arc::new(StubEventRecordRepository::with_records(records));

    let uc = ListEventsUseCase::new(repo);

    let input = ListEventsInput {
        page: 1,
        page_size: 50,
        domain: None,
        event_type: None,
        source: None,
        from: None,
        to: None,
        status: None,
    };

    let result = uc.execute(&input).await;
    assert!(result.is_ok());

    let output = result.unwrap();
    assert_eq!(output.events.len(), 2);
    assert_eq!(output.total_count, 2);
}

#[tokio::test]
async fn test_list_events_with_domain_filter() {
    let records = vec![
        make_event_record("corr-001", "TaskCreated", "task-server", "service.task"),
        make_event_record(
            "corr-002",
            "ActivityCreated",
            "activity-server",
            "service.activity",
        ),
        make_event_record(
            "corr-003",
            "TaskManagementUpdated",
            "task-server",
            "service.task",
        ),
    ];
    let repo = Arc::new(StubEventRecordRepository::with_records(records));

    let uc = ListEventsUseCase::new(repo);

    let input = ListEventsInput {
        page: 1,
        page_size: 50,
        domain: Some("service.task".to_string()),
        event_type: None,
        source: None,
        from: None,
        to: None,
        status: None,
    };

    let result = uc.execute(&input).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().events.len(), 2);
}

#[tokio::test]
async fn test_list_events_with_event_type_filter() {
    let records = vec![
        make_event_record("corr-001", "TaskCreated", "task-server", "service.task"),
        make_event_record("corr-002", "TaskCreated", "task-server", "service.task"),
        make_event_record(
            "corr-003",
            "TaskManagementUpdated",
            "task-server",
            "service.task",
        ),
    ];
    let repo = Arc::new(StubEventRecordRepository::with_records(records));

    let uc = ListEventsUseCase::new(repo);

    let input = ListEventsInput {
        page: 1,
        page_size: 50,
        domain: None,
        event_type: Some("TaskCreated".to_string()),
        source: None,
        from: None,
        to: None,
        status: None,
    };

    let result = uc.execute(&input).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().events.len(), 2);
}

#[tokio::test]
async fn test_list_events_empty() {
    let repo = Arc::new(StubEventRecordRepository::new());

    let uc = ListEventsUseCase::new(repo);

    let input = ListEventsInput {
        page: 1,
        page_size: 50,
        domain: None,
        event_type: None,
        source: None,
        from: None,
        to: None,
        status: None,
    };

    let result = uc.execute(&input).await;
    assert!(result.is_ok());
    assert!(result.unwrap().events.is_empty());
}

#[tokio::test]
async fn test_list_events_internal_error() {
    let repo = Arc::new(StubEventRecordRepository::with_error());

    let uc = ListEventsUseCase::new(repo);

    let input = ListEventsInput {
        page: 1,
        page_size: 50,
        domain: None,
        event_type: None,
        source: None,
        from: None,
        to: None,
        status: None,
    };

    let result = uc.execute(&input).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        ListEventsError::Internal(msg) => {
            assert!(msg.contains("unavailable"));
        }
    }
}

// ============================================================================
// TraceByCorrelationUseCase Tests
// ============================================================================

#[tokio::test]
async fn test_trace_by_correlation_success_without_flow() {
    let records = vec![
        make_event_record("corr-001", "TaskCreated", "task-server", "service.task"),
        make_event_record(
            "corr-001",
            "ActivityCreated",
            "activity-server",
            "service.activity",
        ),
    ];
    let event_repo = Arc::new(StubEventRecordRepository::with_records(records));
    let flow_def_repo = Arc::new(StubFlowDefinitionRepository::new());
    let flow_inst_repo = Arc::new(StubFlowInstanceRepository::new());

    let uc = TraceByCorrelationUseCase::new(event_repo, flow_def_repo, flow_inst_repo);

    let result = uc.execute("corr-001").await;
    assert!(result.is_ok());

    let output = result.unwrap();
    assert_eq!(output.correlation_id, "corr-001");
    assert_eq!(output.events.len(), 2);
    assert!(output.flow_instance.is_none());
    assert!(output.flow_name.is_none());
    assert!(output.pending_steps.is_empty());
}

#[tokio::test]
async fn test_trace_by_correlation_success_with_flow() {
    let flow_id = Uuid::new_v4();
    let flow = make_flow_with_id(flow_id, "task_flow", "service.task");

    let records = vec![make_event_record(
        "corr-001",
        "TaskCreated",
        "task-server",
        "service.task",
    )];
    let event_repo = Arc::new(StubEventRecordRepository::with_records(records));
    let flow_def_repo = Arc::new(StubFlowDefinitionRepository::with_flows(vec![flow]));
    let instance = make_instance(flow_id, "corr-001", FlowInstanceStatus::Completed);
    let flow_inst_repo = Arc::new(StubFlowInstanceRepository::with_instances(vec![instance]));

    let uc = TraceByCorrelationUseCase::new(event_repo, flow_def_repo, flow_inst_repo);

    let result = uc.execute("corr-001").await;
    assert!(result.is_ok());

    let output = result.unwrap();
    assert_eq!(output.correlation_id, "corr-001");
    assert!(output.flow_instance.is_some());
    assert_eq!(output.flow_name.as_deref(), Some("task_flow"));
}

#[tokio::test]
async fn test_trace_by_correlation_not_found() {
    let event_repo = Arc::new(StubEventRecordRepository::new());
    let flow_def_repo = Arc::new(StubFlowDefinitionRepository::new());
    let flow_inst_repo = Arc::new(StubFlowInstanceRepository::new());

    let uc = TraceByCorrelationUseCase::new(event_repo, flow_def_repo, flow_inst_repo);

    let result = uc.execute("nonexistent").await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        TraceByCorrelationError::NotFound(_)
    ));
}

#[tokio::test]
async fn test_trace_by_correlation_internal_error() {
    let event_repo = Arc::new(StubEventRecordRepository::with_error());
    let flow_def_repo = Arc::new(StubFlowDefinitionRepository::new());
    let flow_inst_repo = Arc::new(StubFlowInstanceRepository::new());

    let uc = TraceByCorrelationUseCase::new(event_repo, flow_def_repo, flow_inst_repo);

    let result = uc.execute("corr-001").await;
    assert!(result.is_err());
    match result.unwrap_err() {
        TraceByCorrelationError::Internal(msg) => {
            assert!(msg.contains("unavailable"));
        }
        e => panic!("expected Internal, got: {:?}", e),
    }
}

// ============================================================================
// GetFlowKpiUseCase Tests
// ============================================================================

#[tokio::test]
async fn test_get_flow_kpi_success() {
    let flow_id = Uuid::new_v4();
    let flow = make_flow_with_id(flow_id, "task_flow", "service.task");
    let flow_def_repo = Arc::new(StubFlowDefinitionRepository::with_flows(vec![flow]));

    let instances = vec![
        make_instance_with_duration(flow_id, FlowInstanceStatus::Completed, 1000),
        make_instance_with_duration(flow_id, FlowInstanceStatus::Completed, 2000),
        make_instance_with_duration(flow_id, FlowInstanceStatus::Failed, 500),
    ];
    let flow_inst_repo = Arc::new(StubFlowInstanceRepository::with_instances(instances));

    let uc = GetFlowKpiUseCase::new(flow_def_repo, flow_inst_repo);

    let result = uc.execute(&flow_id, "24h").await;
    assert!(result.is_ok());

    let output = result.unwrap();
    assert_eq!(output.flow_id, flow_id);
    assert_eq!(output.flow_name, "task_flow");
    assert_eq!(output.period, "24h");
    assert_eq!(output.kpi.total_started, 3);
    assert_eq!(output.kpi.total_completed, 2);
    assert_eq!(output.kpi.total_failed, 1);
}

#[tokio::test]
async fn test_get_flow_kpi_not_found() {
    let flow_def_repo = Arc::new(StubFlowDefinitionRepository::new());
    let flow_inst_repo = Arc::new(StubFlowInstanceRepository::new());

    let uc = GetFlowKpiUseCase::new(flow_def_repo, flow_inst_repo);

    let result = uc.execute(&Uuid::new_v4(), "24h").await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), GetFlowKpiError::NotFound(_)));
}

#[tokio::test]
async fn test_get_flow_kpi_internal_error() {
    let flow_def_repo = Arc::new(StubFlowDefinitionRepository::with_error());
    let flow_inst_repo = Arc::new(StubFlowInstanceRepository::new());

    let uc = GetFlowKpiUseCase::new(flow_def_repo, flow_inst_repo);

    let result = uc.execute(&Uuid::new_v4(), "24h").await;
    assert!(result.is_err());
    match result.unwrap_err() {
        GetFlowKpiError::Internal(msg) => {
            assert!(msg.contains("unavailable"));
        }
        e => panic!("expected Internal, got: {:?}", e),
    }
}

#[tokio::test]
async fn test_get_flow_kpi_empty_instances() {
    let flow_id = Uuid::new_v4();
    let flow = make_flow_with_id(flow_id, "task_flow", "service.task");
    let flow_def_repo = Arc::new(StubFlowDefinitionRepository::with_flows(vec![flow]));
    let flow_inst_repo = Arc::new(StubFlowInstanceRepository::new());

    let uc = GetFlowKpiUseCase::new(flow_def_repo, flow_inst_repo);

    let result = uc.execute(&flow_id, "24h").await;
    assert!(result.is_ok());

    let output = result.unwrap();
    assert_eq!(output.kpi.total_started, 0);
    assert_eq!(output.kpi.completion_rate, 0.0);
    assert!(!output.slo_status.is_violated);
}

// ============================================================================
// GetKpiSummaryUseCase Tests
// ============================================================================

#[tokio::test]
async fn test_get_kpi_summary_success() {
    let flow_id1 = Uuid::new_v4();
    let flow_id2 = Uuid::new_v4();
    let flows = vec![
        make_flow_with_id(flow_id1, "task_flow", "service.task"),
        make_flow_with_id(flow_id2, "activity_flow", "service.activity"),
    ];
    let flow_def_repo = Arc::new(StubFlowDefinitionRepository::with_flows(flows));

    let instances = vec![
        make_instance_with_duration(flow_id1, FlowInstanceStatus::Completed, 1000),
        make_instance_with_duration(flow_id1, FlowInstanceStatus::Completed, 2000),
        make_instance_with_duration(flow_id2, FlowInstanceStatus::Completed, 500),
        make_instance_with_duration(flow_id2, FlowInstanceStatus::Failed, 300),
    ];
    let flow_inst_repo = Arc::new(StubFlowInstanceRepository::with_instances(instances));

    let uc = GetKpiSummaryUseCase::new(flow_def_repo, flow_inst_repo);

    let result = uc.execute("24h").await;
    assert!(result.is_ok());

    let output = result.unwrap();
    assert_eq!(output.period, "24h");
    assert_eq!(output.total_flows, 2);
    assert_eq!(output.flows.len(), 2);
    assert!(output.overall_completion_rate > 0.0);
}

#[tokio::test]
async fn test_get_kpi_summary_empty() {
    let flow_def_repo = Arc::new(StubFlowDefinitionRepository::new());
    let flow_inst_repo = Arc::new(StubFlowInstanceRepository::new());

    let uc = GetKpiSummaryUseCase::new(flow_def_repo, flow_inst_repo);

    let result = uc.execute("24h").await;
    assert!(result.is_ok());

    let output = result.unwrap();
    assert_eq!(output.total_flows, 0);
    assert!(output.flows.is_empty());
    assert_eq!(output.overall_completion_rate, 0.0);
}

#[tokio::test]
async fn test_get_kpi_summary_internal_error() {
    let flow_def_repo = Arc::new(StubFlowDefinitionRepository::with_error());
    let flow_inst_repo = Arc::new(StubFlowInstanceRepository::new());

    let uc = GetKpiSummaryUseCase::new(flow_def_repo, flow_inst_repo);

    let result = uc.execute("24h").await;
    assert!(result.is_err());
    match result.unwrap_err() {
        GetKpiSummaryError::Internal(msg) => {
            assert!(msg.contains("unavailable"));
        }
    }
}

#[tokio::test]
async fn test_get_kpi_summary_slo_violations() {
    let flow_id = Uuid::new_v4();
    let flow = make_flow_with_id(flow_id, "task_flow", "service.task");
    let flow_def_repo = Arc::new(StubFlowDefinitionRepository::with_flows(vec![flow]));

    // 90% success rate violates 99% SLO target
    let mut instances: Vec<FlowInstance> = (0..90)
        .map(|_| make_instance_with_duration(flow_id, FlowInstanceStatus::Completed, 1000))
        .collect();
    instances.extend(
        (0..10).map(|_| make_instance_with_duration(flow_id, FlowInstanceStatus::Failed, 500)),
    );
    let flow_inst_repo = Arc::new(StubFlowInstanceRepository::with_instances(instances));

    let uc = GetKpiSummaryUseCase::new(flow_def_repo, flow_inst_repo);

    let result = uc.execute("24h").await;
    assert!(result.is_ok());

    let output = result.unwrap();
    assert_eq!(output.flows_with_slo_violation, 1);
}

// ============================================================================
// GetSloStatusUseCase Tests
// ============================================================================

#[tokio::test]
async fn test_get_slo_status_success() {
    let flow_id = Uuid::new_v4();
    let flow = make_flow_with_id(flow_id, "task_flow", "service.task");
    let flow_def_repo = Arc::new(StubFlowDefinitionRepository::with_flows(vec![flow]));

    let instances = vec![
        make_instance(flow_id, "corr-001", FlowInstanceStatus::Completed),
        make_instance(flow_id, "corr-002", FlowInstanceStatus::Completed),
    ];
    let flow_inst_repo = Arc::new(StubFlowInstanceRepository::with_instances(instances));

    let uc = GetSloStatusUseCase::new(flow_def_repo, flow_inst_repo);

    let result = uc.execute().await;
    assert!(result.is_ok());

    let items = result.unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].flow_name, "task_flow");
    assert!(!items[0].is_violated);
}

#[tokio::test]
async fn test_get_slo_status_with_violation() {
    let flow_id = Uuid::new_v4();
    let flow = make_flow_with_id(flow_id, "task_flow", "service.task");
    let flow_def_repo = Arc::new(StubFlowDefinitionRepository::with_flows(vec![flow]));

    let mut instances: Vec<FlowInstance> = (0..80)
        .map(|_| make_instance(flow_id, "corr", FlowInstanceStatus::Completed))
        .collect();
    instances
        .extend((0..20).map(|_| make_instance(flow_id, "corr-fail", FlowInstanceStatus::Failed)));
    let flow_inst_repo = Arc::new(StubFlowInstanceRepository::with_instances(instances));

    let uc = GetSloStatusUseCase::new(flow_def_repo, flow_inst_repo);

    let result = uc.execute().await;
    assert!(result.is_ok());

    let items = result.unwrap();
    assert_eq!(items.len(), 1);
    assert!(items[0].is_violated);
    assert!(items[0].burn_rate > 1.0);
}

#[tokio::test]
async fn test_get_slo_status_internal_error() {
    let flow_def_repo = Arc::new(StubFlowDefinitionRepository::with_error());
    let flow_inst_repo = Arc::new(StubFlowInstanceRepository::new());

    let uc = GetSloStatusUseCase::new(flow_def_repo, flow_inst_repo);

    let result = uc.execute().await;
    assert!(result.is_err());
    match result.unwrap_err() {
        GetSloStatusError::Internal(msg) => {
            assert!(msg.contains("unavailable"));
        }
    }
}

// ============================================================================
// GetSloBurnRateUseCase Tests
// ============================================================================

#[tokio::test]
async fn test_get_slo_burn_rate_success() {
    let flow_id = Uuid::new_v4();
    let flow = make_flow_with_id(flow_id, "task_flow", "service.task");
    let flow_def_repo = Arc::new(StubFlowDefinitionRepository::with_flows(vec![flow]));

    let instances = vec![
        make_instance(flow_id, "corr-001", FlowInstanceStatus::Completed),
        make_instance(flow_id, "corr-002", FlowInstanceStatus::Completed),
    ];
    let flow_inst_repo = Arc::new(StubFlowInstanceRepository::with_instances(instances));

    let uc = GetSloBurnRateUseCase::new(flow_def_repo, flow_inst_repo);

    let result = uc.execute(&flow_id).await;
    assert!(result.is_ok());

    let output = result.unwrap();
    assert_eq!(output.flow_name, "task_flow");
    assert_eq!(output.windows.len(), 4); // 1h, 6h, 24h, 30d
    assert_eq!(output.alert_status, "ok");
    assert!(output.alert_fired_at.is_none());
}

#[tokio::test]
async fn test_get_slo_burn_rate_alert_firing() {
    let flow_id = Uuid::new_v4();
    let flow = make_flow_with_id(flow_id, "task_flow", "service.task");
    let flow_def_repo = Arc::new(StubFlowDefinitionRepository::with_flows(vec![flow]));

    // High failure rate triggers alerts
    let mut instances: Vec<FlowInstance> = (0..50)
        .map(|_| make_instance(flow_id, "corr", FlowInstanceStatus::Completed))
        .collect();
    instances
        .extend((0..50).map(|_| make_instance(flow_id, "corr-fail", FlowInstanceStatus::Failed)));
    let flow_inst_repo = Arc::new(StubFlowInstanceRepository::with_instances(instances));

    let uc = GetSloBurnRateUseCase::new(flow_def_repo, flow_inst_repo);

    let result = uc.execute(&flow_id).await;
    assert!(result.is_ok());

    let output = result.unwrap();
    assert_eq!(output.alert_status, "firing");
    assert!(output.alert_fired_at.is_some());
    assert!(output.windows.iter().any(|w| w.burn_rate > 1.0));
}

#[tokio::test]
async fn test_get_slo_burn_rate_flow_not_found() {
    let flow_def_repo = Arc::new(StubFlowDefinitionRepository::new());
    let flow_inst_repo = Arc::new(StubFlowInstanceRepository::new());

    let uc = GetSloBurnRateUseCase::new(flow_def_repo, flow_inst_repo);

    let result = uc.execute(&Uuid::new_v4()).await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        GetSloBurnRateError::NotFound(_)
    ));
}

#[tokio::test]
async fn test_get_slo_burn_rate_internal_error() {
    let flow_def_repo = Arc::new(StubFlowDefinitionRepository::with_error());
    let flow_inst_repo = Arc::new(StubFlowInstanceRepository::new());

    let uc = GetSloBurnRateUseCase::new(flow_def_repo, flow_inst_repo);

    let result = uc.execute(&Uuid::new_v4()).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        GetSloBurnRateError::Internal(msg) => {
            assert!(msg.contains("unavailable"));
        }
        e => panic!("expected Internal, got: {:?}", e),
    }
}

// ============================================================================
// ExecuteReplayUseCase Tests
// ============================================================================

#[tokio::test]
async fn test_execute_replay_success() {
    let dlq_client = Arc::new(StubDlqClient::new());

    let uc = ExecuteReplayUseCase::new(dlq_client);

    let input = ExecuteReplayInput {
        correlation_ids: vec!["corr-001".to_string()],
        from_step_index: 0,
        include_downstream: true,
        dry_run: false,
    };

    let result = uc.execute(&input).await;
    assert!(result.is_ok());

    let output = result.unwrap();
    assert_eq!(output.replay_id, "replay-001");
    assert_eq!(output.status, "completed");
    assert_eq!(output.total_events, 3);
    assert_eq!(output.replayed_events, 3);
}

#[tokio::test]
async fn test_execute_replay_dlq_failure() {
    let dlq_client = Arc::new(StubDlqClient::with_error());

    let uc = ExecuteReplayUseCase::new(dlq_client);

    let input = ExecuteReplayInput {
        correlation_ids: vec!["corr-001".to_string()],
        from_step_index: 0,
        include_downstream: true,
        dry_run: false,
    };

    let result = uc.execute(&input).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        ExecuteReplayError::Failed(msg) => {
            assert!(msg.contains("DLQ service unavailable"));
        }
        e => panic!("expected Failed, got: {:?}", e),
    }
}

#[tokio::test]
async fn test_execute_replay_multiple_correlation_ids() {
    let dlq_client = Arc::new(StubDlqClient::new());

    let uc = ExecuteReplayUseCase::new(dlq_client);

    let input = ExecuteReplayInput {
        correlation_ids: vec![
            "corr-001".to_string(),
            "corr-002".to_string(),
            "corr-003".to_string(),
        ],
        from_step_index: 1,
        include_downstream: false,
        dry_run: false,
    };

    let result = uc.execute(&input).await;
    assert!(result.is_ok());
}

// ============================================================================
// PreviewReplayUseCase Tests
// ============================================================================

#[tokio::test]
async fn test_preview_replay_success() {
    let flow_id = Uuid::new_v4();
    let records = vec![
        make_event_record_with_flow(
            "corr-001",
            "TaskCreated",
            "task-server",
            "service.task",
            flow_id,
            0,
        ),
        make_event_record_with_flow(
            "corr-001",
            "ActivityCreated",
            "activity-server",
            "service.activity",
            flow_id,
            1,
        ),
    ];
    let event_repo = Arc::new(StubEventRecordRepository::with_records(records));

    let flow = make_flow_with_id(flow_id, "task_flow", "service.task");
    let flow_def_repo = Arc::new(StubFlowDefinitionRepository::with_flows(vec![flow]));

    let dlq_client = Arc::new(StubDlqClient::new());

    let uc = PreviewReplayUseCase::new(event_repo, flow_def_repo, dlq_client);

    let input = PreviewReplayInput {
        correlation_ids: vec!["corr-001".to_string()],
        from_step_index: 0,
        include_downstream: true,
    };

    let result = uc.execute(&input).await;
    assert!(result.is_ok());

    let output = result.unwrap();
    assert_eq!(output.affected_flows.len(), 1);
    assert_eq!(output.affected_flows[0].flow_name, "task_flow");
    assert!(!output.affected_services.is_empty());
    assert_eq!(output.dlq_messages_found, 2);
}

#[tokio::test]
async fn test_preview_replay_no_events() {
    let event_repo = Arc::new(StubEventRecordRepository::new());
    let flow_def_repo = Arc::new(StubFlowDefinitionRepository::new());
    let dlq_client = Arc::new(StubDlqClient::new());

    let uc = PreviewReplayUseCase::new(event_repo, flow_def_repo, dlq_client);

    let input = PreviewReplayInput {
        correlation_ids: vec!["nonexistent".to_string()],
        from_step_index: 0,
        include_downstream: true,
    };

    let result = uc.execute(&input).await;
    assert!(result.is_ok());

    let output = result.unwrap();
    assert_eq!(output.total_events_to_replay, 0);
    assert!(output.affected_flows.is_empty());
}

#[tokio::test]
async fn test_preview_replay_internal_error() {
    let event_repo = Arc::new(StubEventRecordRepository::with_error());
    let flow_def_repo = Arc::new(StubFlowDefinitionRepository::new());
    let dlq_client = Arc::new(StubDlqClient::new());

    let uc = PreviewReplayUseCase::new(event_repo, flow_def_repo, dlq_client);

    let input = PreviewReplayInput {
        correlation_ids: vec!["corr-001".to_string()],
        from_step_index: 0,
        include_downstream: true,
    };

    let result = uc.execute(&input).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        PreviewReplayError::Internal(msg) => {
            assert!(msg.contains("unavailable"));
        }
        e => panic!("expected Internal, got: {:?}", e),
    }
}

#[tokio::test]
async fn test_preview_replay_dlq_failure() {
    let event_repo = Arc::new(StubEventRecordRepository::new());
    let flow_def_repo = Arc::new(StubFlowDefinitionRepository::new());
    let dlq_client = Arc::new(StubDlqClient::with_error());

    let uc = PreviewReplayUseCase::new(event_repo, flow_def_repo, dlq_client);

    let input = PreviewReplayInput {
        correlation_ids: vec!["corr-001".to_string()],
        from_step_index: 0,
        include_downstream: true,
    };

    // Even with no events, dlq preview is still called
    let result = uc.execute(&input).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        PreviewReplayError::Internal(msg) => {
            assert!(msg.contains("DLQ service unavailable"));
        }
        e => panic!("expected Internal, got: {:?}", e),
    }
}

// ============================================================================
// Cross-cutting / Integration-style Usecase Tests
// ============================================================================

#[tokio::test]
async fn test_create_then_get_flow_roundtrip() {
    let repo = Arc::new(StubFlowDefinitionRepository::new());

    let create_uc = CreateFlowUseCase::new(repo.clone());
    let get_uc = GetFlowUseCase::new(repo.clone());

    // テスト用入力（ラウンドトリップ検証）
    let input = CreateFlowInput {
        tenant_id: "system".to_string(),
        name: "task_flow".to_string(),
        description: "test flow".to_string(),
        domain: "service.task".to_string(),
        steps: vec![make_flow_step("TaskCreated", "task-server")],
        slo: make_slo(),
    };

    let created = create_uc.execute(&input).await.unwrap();
    let retrieved = get_uc.execute(&created.id).await.unwrap();

    assert_eq!(created.id, retrieved.id);
    assert_eq!(retrieved.name, "task_flow");
}

#[tokio::test]
async fn test_create_update_then_get_flow() {
    let repo = Arc::new(StubFlowDefinitionRepository::new());

    let create_uc = CreateFlowUseCase::new(repo.clone());
    let update_uc = UpdateFlowUseCase::new(repo.clone());
    let get_uc = GetFlowUseCase::new(repo.clone());

    // Create: テスト用入力（更新後取得検証）
    let input = CreateFlowInput {
        tenant_id: "system".to_string(),
        name: "task_flow".to_string(),
        description: "original".to_string(),
        domain: "service.task".to_string(),
        steps: vec![make_flow_step("TaskCreated", "task-server")],
        slo: make_slo(),
    };
    let created = create_uc.execute(&input).await.unwrap();

    // Update
    let update_input = UpdateFlowInput {
        id: created.id,
        description: Some("updated description".to_string()),
        steps: None,
        slo: None,
        enabled: Some(false),
    };
    update_uc.execute(&update_input).await.unwrap();

    // Get
    let retrieved = get_uc.execute(&created.id).await.unwrap();
    assert_eq!(retrieved.description, "updated description");
    assert!(!retrieved.enabled);
}

#[tokio::test]
async fn test_create_then_delete_then_get_fails() {
    let repo = Arc::new(StubFlowDefinitionRepository::new());

    let create_uc = CreateFlowUseCase::new(repo.clone());
    let delete_uc = DeleteFlowUseCase::new(repo.clone());
    let get_uc = GetFlowUseCase::new(repo.clone());

    // Create: テスト用入力（削除後取得失敗検証）
    let input = CreateFlowInput {
        tenant_id: "system".to_string(),
        name: "task_flow".to_string(),
        description: "test".to_string(),
        domain: "service.task".to_string(),
        steps: vec![make_flow_step("TaskCreated", "task-server")],
        slo: make_slo(),
    };
    let created = create_uc.execute(&input).await.unwrap();

    // Delete
    delete_uc.execute(&created.id).await.unwrap();

    // Get should fail
    let result = get_uc.execute(&created.id).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), GetFlowError::NotFound(_)));
}

#[tokio::test]
async fn test_kpi_reflects_instance_statuses() {
    let flow_id = Uuid::new_v4();
    let flow = make_flow_with_id(flow_id, "task_flow", "service.task");
    let flow_def_repo = Arc::new(StubFlowDefinitionRepository::with_flows(vec![flow]));

    // All completed -> 100% success rate, SLO not violated
    let instances: Vec<FlowInstance> = (0..100)
        .map(|_| make_instance_with_duration(flow_id, FlowInstanceStatus::Completed, 1000))
        .collect();
    let flow_inst_repo = Arc::new(StubFlowInstanceRepository::with_instances(instances));

    let uc = GetFlowKpiUseCase::new(flow_def_repo, flow_inst_repo);

    let result = uc.execute(&flow_id, "24h").await.unwrap();
    assert_eq!(result.kpi.total_started, 100);
    assert_eq!(result.kpi.total_completed, 100);
    assert!((result.kpi.completion_rate - 1.0).abs() < f64::EPSILON);
    assert!(!result.slo_status.is_violated);
}

#[tokio::test]
async fn test_multiple_flows_kpi_summary_isolation() {
    let flow_id1 = Uuid::new_v4();
    let flow_id2 = Uuid::new_v4();
    let flows = vec![
        make_flow_with_id(flow_id1, "task_flow", "service.task"),
        make_flow_with_id(flow_id2, "activity_flow", "service.activity"),
    ];
    let flow_def_repo = Arc::new(StubFlowDefinitionRepository::with_flows(flows));

    // flow1: 100% success, flow2: 50% success
    let mut instances = Vec::new();
    for _ in 0..10 {
        instances.push(make_instance_with_duration(
            flow_id1,
            FlowInstanceStatus::Completed,
            1000,
        ));
    }
    for _ in 0..5 {
        instances.push(make_instance_with_duration(
            flow_id2,
            FlowInstanceStatus::Completed,
            500,
        ));
    }
    for _ in 0..5 {
        instances.push(make_instance_with_duration(
            flow_id2,
            FlowInstanceStatus::Failed,
            300,
        ));
    }
    let flow_inst_repo = Arc::new(StubFlowInstanceRepository::with_instances(instances));

    let uc = GetKpiSummaryUseCase::new(flow_def_repo, flow_inst_repo);

    let result = uc.execute("24h").await.unwrap();
    assert_eq!(result.total_flows, 2);

    let task_flow = result
        .flows
        .iter()
        .find(|f| f.flow_name == "task_flow")
        .unwrap();
    assert!((task_flow.completion_rate - 1.0).abs() < f64::EPSILON);
    assert!(!task_flow.slo_violated);

    let activity_flow = result
        .flows
        .iter()
        .find(|f| f.flow_name == "activity_flow")
        .unwrap();
    assert!((activity_flow.completion_rate - 0.5).abs() < f64::EPSILON);
    assert!(activity_flow.slo_violated);
}
