// schema.rs — k1s0 tier1 Library: スキーマレジストリ L1+ facade trait
// Apicurio Registry 等の OSS 型を公開 API に露出しない（L1+ ラップ規約）。
// スキーマの登録・取得を artifact_id ベースで行う。
// 全 trait は Send + Sync を要求する（スレッド安全性の強制）。

// async_trait: async fn in trait を stable で使用するためのマクロ
use async_trait::async_trait;

// SchemaRegistry は Apicurio Registry を L1+ ラップするスキーマ管理 facade trait。
// 公開 API シグネチャに OSS 型（apicurio::client::RegistryClient 等）を一切含まない。
// スキーマは bytes として扱い、フォーマット（Avro / Protobuf / JSON Schema 等）は実装側が判定する。
#[async_trait]
pub trait SchemaRegistry: Send + Sync {
    // register_schema はスキーマを登録（または既存スキーマを取得）して artifact_id を返す。
    // subject はスキーマの主題（例: "k1s0.tier1.SessionCreatedEvent"）。
    // schema はスキーマ定義バイト列（Avro JSON / proto bytes 等）。
    // 戻り値は artifact_id（以降の get_schema 呼び出しで使用する）。
    async fn register_schema(&self, subject: &str, schema: Vec<u8>) -> crate::Result<String>;

    // get_schema は artifact_id でスキーマを取得する。
    // artifact_id は register_schema の戻り値として得た識別子。
    // スキーマが存在しない場合はエラーを返す（Option ではなく Result を使用する）。
    async fn get_schema(&self, artifact_id: &str) -> crate::Result<Vec<u8>>;
}
