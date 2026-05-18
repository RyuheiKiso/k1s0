// vector.rs — k1s0 tier1 Library backend: Vector Search L1+ trait
// backend 専用カテゴリ（frontend には提供しない）。
// L1+: OSS の全機能を表現 + tier1 横断要素（auth context 伝播 / retry / tracing）を強制。
// 公開 API に OSS 型（qdrant_client::Client / pgvector 等）を露出しない。
// 埋め込みベクトルの生成（LLM API）は本 trait の範囲外（呼び出し元が行う）。

// async_trait: async fn in trait を stable で使用するためのマクロ
use async_trait::async_trait;
// anyhow: エラーハンドリング（Result 型の統一）
use anyhow::Result;
// serde: ベクトルポイントのシリアライズに使用する
use serde::{Deserialize, Serialize};
// AuthContext: ベクトル操作に認証コンテキストを伝播する
use crate::core::auth::AuthContext;
// SpanContext: ベクトル操作に tracing コンテキストを伝播する
use crate::core::observability::SpanContext;
// ServicePolicy: retry / timeout / circuit breaker を強制する
use crate::core::policy::ServicePolicy;

// VectorPoint はベクトルインデックスの 1 ポイントを表す Library 独自型。
// OSS の PointStruct（Qdrant）/ Embedding（pgvector）を Library 独自型に変換して露出しない。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorPoint {
    // point_id: ポイントの一意識別子（UUID v7 形式）
    pub point_id: String,
    // vector: 埋め込みベクトル（次元数はコレクション設定に依存する）
    pub vector: Vec<f32>,
    // payload: ポイントに付随するメタデータ（JSON 形式）
    pub payload: serde_json::Value,
    // tenant_id: このポイントが属するテナントの識別子（マルチテナント分離）
    pub tenant_id: String,
}

// SimilarityMetric はベクトル類似度の計算方法を表す Library 独自型。
// Qdrant / pgvector の Distance 型を Library 独自語彙で表現する。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SimilarityMetric {
    // Cosine: コサイン類似度（正規化済みベクトルの内積）
    Cosine,
    // DotProduct: 内積（スケールを考慮した類似度）
    DotProduct,
    // Euclidean: ユークリッド距離（最近傍検索向け）
    Euclidean,
}

// CollectionConfig はベクトルコレクションの設定を表す struct。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionConfig {
    // collection_name: コレクション名（テナント ID が prefix として埋め込まれる）
    pub collection_name: String,
    // dimension: ベクトルの次元数（例: 1536 for text-embedding-ada-002）
    pub dimension: u32,
    // similarity_metric: 類似度計算方法
    pub similarity_metric: SimilarityMetric,
    // on_disk_payload: ペイロードをディスクに保存するかどうか（メモリ節約）
    pub on_disk_payload: bool,
}

// SearchRequest はベクトル検索リクエストを表す Library 独自型。
#[derive(Debug, Clone)]
pub struct SearchRequest {
    // query_vector: 検索クエリベクトル（次元数はコレクション設定に合わせる）
    pub query_vector: Vec<f32>,
    // top_k: 返す類似ポイント数
    pub top_k: u32,
    // score_threshold: スコア閾値（これ以上のスコアのポイントのみ返す; None は閾値なし）
    pub score_threshold: Option<f32>,
    // filter_payload_key: ペイロードフィルターキー（None はフィルターなし）
    pub filter_payload_key: Option<String>,
    // filter_payload_value: ペイロードフィルター値（filter_payload_key と合わせて使う）
    pub filter_payload_value: Option<serde_json::Value>,
}

// SearchResult はベクトル検索の 1 結果を表す Library 独自型。
#[derive(Debug, Clone)]
pub struct SearchResult {
    // point_id: 一致したポイントの識別子
    pub point_id: String,
    // score: 類似度スコア（1.0 が最大; Euclidean は距離の逆数）
    pub score: f32,
    // payload: ポイントに付随するメタデータ
    pub payload: serde_json::Value,
}

// VectorStore は Vector Search の L1+ 抽象 trait。
// Qdrant / pgvector / Weaviate 等を実装で切り替えられる。
// auth context 伝播 / retry / tracing は強制する。
#[async_trait]
pub trait VectorStore: Send + Sync {
    // create_collection はベクトルコレクションを作成する。
    // auth_ctx は作成権限の確認と audit ログに使用する。
    async fn create_collection(
        &self,
        config: CollectionConfig,
        auth_ctx: &AuthContext,
    ) -> Result<()>;

    // upsert_points は複数のベクトルポイントを挿入または更新する。
    // auth_ctx は tenant_id の境界を保証し、span_ctx は tracing に使用する。
    // policy は retry / timeout / circuit breaker を適用する。
    async fn upsert_points(
        &self,
        collection_name: &str,
        points: Vec<VectorPoint>,
        auth_ctx: &AuthContext,
        span_ctx: Option<&SpanContext>,
        policy: &ServicePolicy,
    ) -> Result<u64>;

    // search はクエリベクトルに最も近いポイントを返す。
    // auth_ctx は tenant_id でフィルタリングして他テナントのデータを返さないようにする。
    async fn search(
        &self,
        collection_name: &str,
        request: SearchRequest,
        auth_ctx: &AuthContext,
        span_ctx: Option<&SpanContext>,
        policy: &ServicePolicy,
    ) -> Result<Vec<SearchResult>>;

    // delete_points は point_id のリストを受け取り、ポイントを削除する。
    // auth_ctx は tenant_id の境界を保証する。
    async fn delete_points(
        &self,
        collection_name: &str,
        point_ids: Vec<String>,
        auth_ctx: &AuthContext,
    ) -> Result<u64>;

    // get_point は point_id を受け取り、ベクトルポイントを返す。
    // auth_ctx は tenant_id の境界を保証する。
    async fn get_point(
        &self,
        collection_name: &str,
        point_id: &str,
        auth_ctx: &AuthContext,
    ) -> Result<Option<VectorPoint>>;
}
