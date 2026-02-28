use std::sync::Arc;

use tracing::instrument;

use crate::domain::model::{
    decode_cursor, encode_cursor, PageInfo, Tenant, TenantConnection, TenantEdge,
};
use crate::infra::grpc::TenantGrpcClient;

pub struct TenantQueryResolver {
    client: Arc<TenantGrpcClient>,
}

impl TenantQueryResolver {
    pub fn new(client: Arc<TenantGrpcClient>) -> Self {
        Self { client }
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn get_tenant(&self, id: &str) -> anyhow::Result<Option<Tenant>> {
        self.client.get_tenant(id).await
    }

    /// Relay cursor-based pagination を gRPC の offset-based pagination に変換する
    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn list_tenants(
        &self,
        first: Option<i32>,
        after: Option<String>,
        _last: Option<i32>,
        _before: Option<String>,
    ) -> anyhow::Result<TenantConnection> {
        let page_size = first.unwrap_or(20);
        let offset = after
            .as_deref()
            .and_then(decode_cursor)
            .map(|o| o + 1)
            .unwrap_or(0);
        let page = if page_size > 0 {
            (offset as i32 / page_size) + 1
        } else {
            1
        };

        let raw = self.client.list_tenants(page, page_size).await?;

        // gRPC レスポンスを Relay Connection に変換
        let edges: Vec<TenantEdge> = raw
            .nodes
            .into_iter()
            .enumerate()
            .map(|(i, node)| TenantEdge {
                cursor: encode_cursor(offset + i),
                node,
            })
            .collect();

        let start_cursor = edges.first().map(|e| e.cursor.clone());
        let end_cursor = edges.last().map(|e| e.cursor.clone());

        Ok(TenantConnection {
            total_count: raw.total_count,
            page_info: PageInfo {
                has_next_page: raw.has_next,
                has_previous_page: offset > 0,
                start_cursor,
                end_cursor,
            },
            edges,
        })
    }
}
