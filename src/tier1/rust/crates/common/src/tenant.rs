// 本ファイルは TenantContext からの tenant_id 抽出ヘルパ。
//
// 設計正典:
//   docs/03_要件定義/00_共通規約.md §「テナント分離」
//   NFR-E-AC-003: tenant_id 越境防止
//
// 役割:
//   3 Pod のすべての副作用 RPC は handler 先頭で TenantContext.tenant_id の
//   存在を検証する必要がある。検証失敗時は `tonic::Status::invalid_argument`
//   を返却し、Pod 自身が空 tenant の処理を行わないことを保証する。

// 公開 API の TenantContext 型。
use k1s0_sdk_proto::k1s0::tier1::common::v1::TenantContext;

/// `require_tenant_id` は `TenantContext.tenant_id` を必須として取り出す。
///
/// `rpc_label` はエラーメッセージに含める RPC 名（`"State.Get"` のような形式）。
/// 不正時は `tonic::Status::invalid_argument` を返す。
pub fn require_tenant_id(ctx: Option<&TenantContext>, rpc_label: &str) -> Result<String, tonic::Status> {
    let tid = ctx
        .map(|c| c.tenant_id.clone())
        .unwrap_or_default();
    if tid.is_empty() {
        return Err(tonic::Status::invalid_argument(format!(
            "tier1: {}: tenant_id required in TenantContext",
            rpc_label
        )));
    }
    Ok(tid)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_context_is_invalid_argument() {
        let r = require_tenant_id(None, "Audit.Record");
        assert!(r.is_err());
        assert_eq!(r.err().unwrap().code(), tonic::Code::InvalidArgument);
    }

    #[test]
    fn empty_tenant_id_is_invalid_argument() {
        let ctx = TenantContext {
            tenant_id: String::new(),
            ..Default::default()
        };
        let r = require_tenant_id(Some(&ctx), "Audit.Record");
        assert!(r.is_err());
        assert_eq!(r.err().unwrap().code(), tonic::Code::InvalidArgument);
    }

    #[test]
    fn returns_tenant_id_when_present() {
        let ctx = TenantContext {
            tenant_id: "T1".into(),
            ..Default::default()
        };
        let r = require_tenant_id(Some(&ctx), "Audit.Record").unwrap();
        assert_eq!(r, "T1");
    }
}
