use async_graphql::SimpleObject;

/// CRIT-007 監査対応: proto `FeatureFlag` と完全整合させた GraphQL モデル
/// proto に存在しない `name/rollout_percentage/target_environments` を削除し、
/// proto の `variants/rules/created_at/updated_at` を追加する
#[derive(Debug, Clone, SimpleObject)]
pub struct FeatureFlag {
    /// proto FeatureFlag.id（UUID）
    pub id: String,
    /// proto `FeatureFlag.flag_key（feature_flags.flag_key` カラム）
    pub flag_key: String,
    /// proto FeatureFlag.description
    pub description: Option<String>,
    /// proto FeatureFlag.enabled
    pub enabled: bool,
    /// proto FeatureFlag.variants（バリアント一覧）
    pub variants: Vec<FlagVariant>,
    /// proto FeatureFlag.rules（評価ルール一覧）
    pub rules: Vec<FlagRule>,
    /// proto `FeatureFlag.created_at（RFC3339形式文字列`）
    pub created_at: String,
    /// proto `FeatureFlag.updated_at（RFC3339形式文字列`）
    pub updated_at: String,
}

/// proto `FlagVariant` と整合するバリアント型
#[derive(Debug, Clone, SimpleObject)]
pub struct FlagVariant {
    /// バリアント名（例: "on", "off", "beta"）
    pub name: String,
    /// バリアント値（例: "true", "false"）
    pub value: String,
    /// ウェイト（0〜100 の整数）
    pub weight: i32,
}

/// proto `FlagRule` と整合するルール型
#[derive(Debug, Clone, SimpleObject)]
pub struct FlagRule {
    /// 評価対象の属性名（例: "environment", "`user_id`"）
    pub attribute: String,
    /// 比較演算子（例: "EQ", "NE", "CONTAINS", "GT", "LT"）
    pub operator: String,
    /// 比較値
    pub value: String,
    /// マッチ時に適用するバリアント名
    pub variant: String,
}
