// schema.rs — k1s0 tier1 Library backend: Schema Registry L2* trait
// backend 専用カテゴリ（frontend には提供しない）。
// L2*: 族内で共通の API + OSS 概念は Library 独自語彙に翻訳する。
// 公開 API に OSS 型（schema_registry_converter::SchemaRegistryClient 等）を露出しない。
// Confluent Schema Registry 互換の ID 体系を Library 独自型で隠蔽する。

// async_trait: async fn in trait を stable で使用するためのマクロ
use async_trait::async_trait;
// anyhow: エラーハンドリング（Result 型の統一）
use anyhow::Result;
// serde: スキーマメタデータのシリアライズに使用する
use serde::{Deserialize, Serialize};
// AuthContext: スキーマ操作に認証コンテキストを伝播する
use crate::core::auth::AuthContext;

// SchemaFormat はスキーマのフォーマットを表す Library 独自型。
// OSS の SchemaType（Avro / Protobuf / JSON Schema）を Library 独自語彙に変換する。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SchemaFormat {
    // Avro: Apache Avro スキーマフォーマット
    Avro,
    // Protobuf: Protocol Buffers スキーマフォーマット
    Protobuf,
    // JsonSchema: JSON Schema フォーマット
    JsonSchema,
}

// CompatibilityLevel はスキーマ互換性レベルを表す Library 独自型。
// Confluent Schema Registry の compatibility mode を Library 独自語彙で表現する。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompatibilityLevel {
    // None: 互換性チェックなし（全変更を許可する）
    None,
    // Backward: 後方互換性（新スキーマで旧メッセージを読める）
    Backward,
    // Forward: 前方互換性（旧スキーマで新メッセージを読める）
    Forward,
    // Full: 完全互換性（Backward + Forward）
    Full,
    // BackwardTransitive: 遡及後方互換性（全バージョンとの後方互換）
    BackwardTransitive,
}

// SchemaRecord はスキーマの登録済みレコードを表す Library 独自型。
// OSS の Schema オブジェクトを Library 独自型に変換して露出しない。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaRecord {
    // schema_id: スキーマの一意識別子（Schema Registry の ID に対応する Library 独自 wrapper）
    pub schema_id: String,
    // subject: スキーマが属するサブジェクト名（例: "payment-value"）
    pub subject: String,
    // version: スキーマのバージョン番号（1 始まり）
    pub version: u32,
    // format: スキーマのフォーマット
    pub format: SchemaFormat,
    // schema_definition: スキーマ定義文字列（Avro JSON / .proto テキスト / JSON Schema）
    pub schema_definition: String,
    // compatibility_level: 互換性レベル
    pub compatibility_level: CompatibilityLevel,
}

// SchemaLookupResult はスキーマ検索の結果を表す enum。
#[derive(Debug)]
pub enum SchemaLookupResult {
    // Found: スキーマが見つかった（SchemaRecord を伴う）
    Found(SchemaRecord),
    // NotFound: スキーマが見つからない（subject / version 未登録）
    NotFound,
    // IncompatibleChange: 互換性違反（登録しようとしたスキーマが互換性チェックを通らなかった）
    IncompatibleChange(String),
}

// SchemaRegistry は Schema Registry の L2* 抽象 trait。
// Confluent Schema Registry / Apicurio Registry 等を実装で切り替えられる。
// 公開 API に OSS 型を露出しない。
#[async_trait]
pub trait SchemaRegistry: Send + Sync {
    // register はスキーマを登録する（既存と一致する場合は既存の ID を返す）。
    // auth_ctx は登録権限の確認と audit ログに使用する。
    async fn register(
        &self,
        subject: &str,
        format: SchemaFormat,
        schema_definition: &str,
        auth_ctx: &AuthContext,
    ) -> Result<SchemaLookupResult>;

    // get_by_id は schema_id を受け取り、SchemaRecord を返す。
    // message デシリアライズ時のスキーマ解決に使用する。
    async fn get_by_id(&self, schema_id: &str, auth_ctx: &AuthContext)
        -> Result<SchemaLookupResult>;

    // get_by_subject_version は subject と version を受け取り、SchemaRecord を返す。
    // version=0 は最新バージョンを返す。
    async fn get_by_subject_version(
        &self,
        subject: &str,
        version: u32,
        auth_ctx: &AuthContext,
    ) -> Result<SchemaLookupResult>;

    // check_compatibility は新スキーマの互換性を事前検証する（登録は行わない）。
    // IncompatibleChange の場合は理由メッセージを返す。
    async fn check_compatibility(
        &self,
        subject: &str,
        format: SchemaFormat,
        schema_definition: &str,
        auth_ctx: &AuthContext,
    ) -> Result<SchemaLookupResult>;

    // set_compatibility は subject の互換性レベルを変更する。
    async fn set_compatibility(
        &self,
        subject: &str,
        level: CompatibilityLevel,
        auth_ctx: &AuthContext,
    ) -> Result<()>;
}
