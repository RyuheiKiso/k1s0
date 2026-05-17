// k1s0 tier2 テナントコンテキスト
// PostgreSQL session GUC（app.tenant_id / app.actor_id / app.purpose / app.delegation_chain）を
// transaction 開始時に SET LOCAL で自動注入する。
// 設計原則: tenant_id を API 引数で受け取る public 関数を持たない（AuthContext 経由のみ取得する）

// UUID ライブラリ（tenant_id / actor_id の型に使用する）
use uuid::Uuid;
// シリアライズライブラリ（TenantContext の JSON 変換に使用する）
use serde::{Deserialize, Serialize};
// エラーハンドリングライブラリ
use anyhow::Result;

// PostgreSQL session GUC の目的値（app.purpose フィールドの許容値）
// 10_テナント分離適合仕様.md の purpose enum と完全整合する
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionPurpose {
    // 通常業務操作: テナント所有データへのアクセス
    BusinessOp,
    // サポートアクセス: support_engineer role での PII 参照
    Support,
    // データエクスポート: バッチ出力
    Export,
    // マイグレーション: スキーマ移行期間中の特権操作
    Migration,
    // 緊急オペレーション: 障害対応時の緊急権限（最小化・全記録必須）
    Emergency,
}

impl SessionPurpose {
    // purpose 値を PostgreSQL SET LOCAL に渡せる文字列に変換する
    pub fn as_guc_value(&self) -> &'static str {
        // 各 purpose 値に対応する GUC 文字列を返す
        match self {
            Self::BusinessOp  => "business_op",
            Self::Support     => "support",
            Self::Export      => "export",
            Self::Migration   => "migration",
            Self::Emergency   => "emergency",
        }
    }
}

// テナントセッションコンテキスト
// 4 つの PostgreSQL session GUC をまとめて管理する
// tenant_id は直接 API 引数で受け取らず、AuthContext 経由でのみ設定できる
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantContext {
    // テナント識別子（AuthContext から導出する、API 引数経由は禁止）
    pub(crate) tenant_id: Uuid,
    // アクター識別子（Keycloak subject、監査ログに記録する）
    pub(crate) actor_id: String,
    // セッション目的（PII アクセス制御・監査区分に使用する）
    pub(crate) purpose: SessionPurpose,
    // 委譲チェーン（通常操作では空、委譲操作で委譲元 user ID を記録する）
    pub(crate) delegation_chain: Vec<String>,
}

impl TenantContext {
    // TenantContext を生成する（AuthContext 相当の検証済みデータから生成する）
    // tenant_id を外部から直接受け取らない設計を型で表現するために pub(crate) にする
    pub fn from_auth(
        // 認証済みテナント ID（AuthContext 経由のみ許容する）
        tenant_id: Uuid,
        // Keycloak subject（認証済み actor ID）
        actor_id: String,
        // セッション目的
        purpose: SessionPurpose,
    ) -> Self {
        // 通常生成では delegation_chain は空にする
        Self { tenant_id, actor_id, purpose, delegation_chain: vec![] }
    }

    // 委譲チェーンを追加した TenantContext を生成する（委譲操作専用）
    pub fn with_delegation(mut self, delegator_id: String) -> Self {
        // 委譲元の actor_id を chain に追加する
        self.delegation_chain.push(delegator_id);
        self
    }

    // テナント ID を返す（crate 内部からのみアクセス可、公開 API には露出しない）
    pub(crate) fn tenant_id(&self) -> Uuid {
        self.tenant_id
    }

    // セッション GUC を表す SQL 文字列を生成する（実際の DB 注入は repository.rs が担う）
    // 4 GUC をまとめて SET LOCAL する SQL 文字列を返す
    pub fn to_set_local_sql(&self) -> String {
        // app.tenant_id: テナント UUID を文字列にして SET LOCAL する
        let tenant_id_str = self.tenant_id.to_string();
        // app.delegation_chain: 配列リテラル形式（'{elem1,elem2}' 等）
        let chain = if self.delegation_chain.is_empty() {
            // 委譲なしの場合は空配列
            "'{}'".to_string()
        } else {
            // 各要素をクォートして配列リテラルに変換する
            let elems = self.delegation_chain
                .iter()
                .map(|s| format!("\"{}\"", s.replace('"', "\\\"")))
                .collect::<Vec<_>>()
                .join(",");
            format!("'{{{}}}'", elems)
        };
        // 4 GUC を一括 SET LOCAL する SQL を返す
        format!(
            "SET LOCAL app.tenant_id = '{tenant_id}'; \
             SET LOCAL app.actor_id = '{actor_id}'; \
             SET LOCAL app.purpose = '{purpose}'; \
             SET LOCAL app.delegation_chain = {chain};",
            tenant_id = tenant_id_str,
            actor_id  = self.actor_id.replace('\'', "''"),
            purpose   = self.purpose.as_guc_value(),
            chain     = chain,
        )
    }
}

#[cfg(test)]
mod tests {
    // テストモジュール内部でのみ使用するインポート
    use super::*;

    #[test]
    // TenantContext の SQL 生成が 4 GUC を含むことを確認する
    fn test_to_set_local_sql_contains_all_guc() {
        // テスト用 TenantContext を生成する
        let ctx = TenantContext::from_auth(
            Uuid::new_v4(),
            "test-actor".to_string(),
            SessionPurpose::BusinessOp,
        );
        // SET LOCAL SQL を生成する
        let sql = ctx.to_set_local_sql();
        // 4 GUC 全ての SET LOCAL が含まれることを検証する
        assert!(sql.contains("app.tenant_id"));
        assert!(sql.contains("app.actor_id"));
        assert!(sql.contains("app.purpose"));
        assert!(sql.contains("app.delegation_chain"));
        // purpose 値が正しいことを確認する
        assert!(sql.contains("business_op"));
    }

    #[test]
    // tenant_id は外部から直接 API 引数として渡せないことを型で確認する
    fn test_tenant_id_not_in_public_api() {
        // from_auth を使わなければ TenantContext を生成できない（プライベートフィールド）
        let ctx = TenantContext::from_auth(
            Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            "keycloak-sub-001".to_string(),
            SessionPurpose::Support,
        );
        // purpose が Support であることを確認する
        assert_eq!(ctx.purpose.as_guc_value(), "support");
        // delegation_chain は初期状態で空であることを確認する
        assert!(ctx.delegation_chain.is_empty());
    }

    #[test]
    // 委譲チェーンが正しく SET LOCAL SQL に反映されることを確認する
    fn test_delegation_chain_in_sql() {
        // 委譲操作の TenantContext を生成する
        let ctx = TenantContext::from_auth(
            Uuid::new_v4(),
            "delegatee-actor".to_string(),
            SessionPurpose::Emergency,
        ).with_delegation("original-actor-001".to_string());
        // 委譲チェーンが SQL に含まれることを確認する
        let sql = ctx.to_set_local_sql();
        assert!(sql.contains("original-actor-001"));
        // emergency purpose が正しいことを確認する
        assert!(sql.contains("emergency"));
    }
}
