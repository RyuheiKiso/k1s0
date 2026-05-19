// tier2 CQRS pgvector 類似検索クエリ（設計方針 15 / ベクトル検索読み取りモデル）
// pgvector 拡張を使用したテナント分離付きセマンティック検索を提供する

// anyhow: Result 型に使用する
use anyhow::Result;
// serde: クエリ結果のシリアライズに使用する
use serde::{Deserialize, Serialize};
// uuid: テナント識別子型に使用する
use uuid::Uuid;
// sqlx: PostgreSQL 非同期クライアント（PgPool を使用してコネクションプールを管理する）
use sqlx::PgPool;

// ベクトル検索結果の構造体（検索ヒット 1 件分のデータを保持する）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorSearchResult {
    // 検索ヒットしたドキュメントの識別子
    pub document_id: Uuid,
    // テナント識別子（RLS で自動フィルタリング済み）
    pub tenant_id: Uuid,
    // コサイン類似度スコア（0.0〜1.0 / 高いほど類似）
    pub similarity_score: f32,
    // ドキュメントメタデータ（JSON 形式）
    pub metadata: serde_json::Value,
}

// sqlx::FromRow 手動実装用の中間行型（query_as でマッピングするために使用する）
#[derive(sqlx::FromRow)]
struct VectorRow {
    // ドキュメント識別子
    document_id: Uuid,
    // テナント識別子
    tenant_id: Uuid,
    // コサイン類似度スコア（1 - cosine_distance として計算される）
    similarity_score: f32,
    // ドキュメントメタデータ（jsonb 型）
    metadata: serde_json::Value,
}

// PgVectorQuery: pgvector を使用したテナント分離付き埋め込みベクトル検索
// Repository 抽象を経由して生 SQL 文字列受付 API を回避する
pub struct PgVectorQuery {
    // クエリ実行に使用する PostgreSQL 接続プール（スレッドセーフ・再利用可能）
    pool: PgPool,
}

impl PgVectorQuery {
    // PgVectorQuery を生成する（sqlx::PgPool を受け取る）
    pub fn new(pool: PgPool) -> Self {
        // PgPool を保持するインスタンスを生成する
        Self { pool }
    }

    // テナント分離付き埋め込みベクトル類似検索を実行する
    // tenant_id: 検索対象テナント（WHERE 句テナント述語 + RLS FORCE で二重保護する）
    // query_vec: 検索クエリの埋め込みベクトル（f32 のスライスを text にフォーマットして渡す）
    // limit: 返却する最大結果件数（LIMIT 句に使用する）
    // sqlx prepare 実行後は query! マクロに置き換えること
    pub async fn embedding_search(
        &self,
        tenant_id: Uuid,
        query_vec: Vec<f32>,
        limit: u32,
    ) -> Result<Vec<VectorSearchResult>> {
        // f32 スライスを pgvector の text リテラル形式 "[x1,x2,...]" に変換する
        let vec_str = format!(
            "[{}]",
            query_vec
                .iter()
                // 各要素を文字列に変換してカンマ区切りにする
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(",")
        );
        // pgvector コサイン距離演算子 <=> を使用して類似度順に documents テーブルを検索する
        // $1: ベクトルリテラル（::vector にキャストして pgvector 型に変換する）
        // $2: テナント UUID（WHERE 句テナント述語 + RLS FORCE の二重保護）
        // $3: LIMIT 件数
        // sqlx prepare 実行後は query! マクロに置き換えること
        let rows = sqlx::query_as::<_, VectorRow>(
            r#"
            SELECT
                document_id,
                tenant_id,
                (1.0 - (embedding <=> $1::vector))::real AS similarity_score,
                metadata
            FROM documents
            WHERE tenant_id = $2
            ORDER BY embedding <=> $1::vector
            LIMIT $3
            "#,
        )
        // ベクトルリテラル文字列をバインドする（pgvector の ::vector キャストで型変換される）
        .bind(&vec_str)
        // テナント UUID をバインドする（RLS FORCE の WHERE 述語として使用する）
        .bind(tenant_id)
        // LIMIT 件数を i64 にキャストしてバインドする（PostgreSQL LIMIT は bigint を期待する）
        .bind(limit as i64)
        // 保持している PgPool を参照して非同期クエリを実行する
        .fetch_all(&self.pool)
        .await?;
        // VectorRow を VectorSearchResult に変換して返す
        let results = rows
            .into_iter()
            .map(|row| VectorSearchResult {
                // document_id をそのまま移動する
                document_id: row.document_id,
                // tenant_id をそのまま移動する
                tenant_id: row.tenant_id,
                // similarity_score をそのまま移動する
                similarity_score: row.similarity_score,
                // metadata をそのまま移動する
                metadata: row.metadata,
            })
            .collect();
        // 変換済みの結果ベクターを返す
        Ok(results)
    }

    // インデックスヒント付き ANN 検索のための IVFFlat プローブ数を設定する
    // nprobes が大きいほど精度が上がるがクエリ速度が下がる（トレードオフ）
    pub fn recommended_nprobes(index_lists: u32) -> u32 {
        // IVFFlat の推奨プローブ数: sqrt(lists) が一般的な経験則
        (index_lists as f64).sqrt().ceil() as u32
    }
}
