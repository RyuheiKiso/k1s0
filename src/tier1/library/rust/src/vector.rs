// vector.rs — k1s0 tier1 Library: ベクトル検索 L1+ facade trait
// pgvector 等の OSS 型を公開 API に露出しない（L1+ ラップ規約）。
// cosine 距離による k-NN 検索と upsert の 2 操作を提供する。
// 全 trait は Send + Sync を要求する（スレッド安全性の強制）。

// async_trait: async fn in trait を stable で使用するためのマクロ
use async_trait::async_trait;

// VectorSearch は pgvector を L1+ ラップするベクトル検索 facade trait。
// 公開 API シグネチャに OSS 型（pgvector::Vector 等）を一切含まない。
// embedding は f32 slice で表現する（次元数はテーブル定義で固定する）。
#[async_trait]
pub trait VectorSearch: Send + Sync {
    // upsert はベクトルを登録する（同一 id が存在する場合は上書きする）。
    // table はベクトルテーブル名（例: "document_embeddings" / "image_embeddings"）。
    // id は一意識別子（例: UUID v4 文字列）。
    // embedding は次元数固定の f32 slice（次元数はテーブル定義に依存する）。
    async fn upsert(&self, table: &str, id: &str, embedding: &[f32]) -> crate::Result<()>;

    // search_knn は最近傍 k 件の id を cosine 距離で検索して返す。
    // table はベクトルテーブル名（upsert と同一テーブルを指定すること）。
    // query は検索クエリの f32 slice（テーブルの次元数と一致すること）。
    // k は返す最大件数（1 以上を指定すること）。
    // 戻り値は cosine 距離の近い順に並んだ id の Vec（最大 k 件）。
    async fn search_knn(&self, table: &str, query: &[f32], k: usize) -> crate::Result<Vec<String>>;
}
