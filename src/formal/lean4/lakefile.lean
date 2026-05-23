-- lakefile.lean — k1s0 formal Lean 4 プロジェクト定義
-- 19 軸の v1_property_axiom proof + Tier1 HLC 単調性 lemma を含む
-- Mathlib 不要の最小構成（omega / simp / decide のみ使用）

import Lake
open Lake DSL

-- k1s0-formal パッケージ定義
package «k1s0Formal» where
  -- デフォルト設定のみ（追加 PackageConfig 不要）

-- K1s0Formal ライブラリターゲットを定義する
-- 19 軸の v1_property_axiom proof module を全て登録する
@[default_target]
lean_lib «K1s0Formal» where
  -- 全 19 軸 property_axiom proof の root module リスト
  roots := #[
    -- Tier1 HLC 単調性 (formal_axiom_048 — 既存)
    `Tier1.HLC.Monotone,
    -- Tier1 transport シーケンシャル配信 (tier1_axiom_004)
    `Tier1.Transport.SequentialDelivery,
    -- Tier2 テナント分離非干渉 (tier2_axiom_009)
    `Tier2.TenantIsolation.NonInterference,
    -- Tier3 クライアント状態 pending queue 順序 (tier3_axiom_014)
    `Tier3.ClientState.PendingQueueOrder,
    -- Infra クラスタトポロジー到達可能性 (infra_axiom_019)
    `Infra.Cluster.TopologyReachability,
    -- Data 書き込み整合性 (data_axiom_023)
    `Data.Atomicity.WriteConsistency,
    -- Security アクセス制御 deny 優先 (security_axiom_028)
    `Security.AccessControl.DenyPrecedence,
    -- Ops アラートライフサイクル resolved 安定性 (ops_axiom_033)
    `Ops.AlertLifecycle.ResolvedStability,
    -- Client HLC 因果順序 (client_axiom_038)
    `Client.HLC.CausalOrder,
    -- Test カバレッジ単調性 (test_axiom_043)
    `Test.Coverage.Monotonicity,
    -- CrossCutting HTTP/2 ストリーム順序 (cross_http2_axiom_053)
    `CrossCutting.Http2.StreamOrdering,
    -- CrossCutting KEK Shamir しきい値 (cross_kek_axiom_058)
    `CrossCutting.Kek.ShamirThreshold,
    -- CrossCutting スキーマ後方互換性 (cross_schema_axiom_063)
    `CrossCutting.Schema.BackwardCompat,
    -- CrossCutting FSM 決定論 (cross_fsm_axiom_068)
    `CrossCutting.Fsm.Determinism,
    -- CrossCutting SLO エラーバジェット (cross_slo_axiom_073)
    `CrossCutting.Slo.ErrorBudget,
    -- CrossCutting BFF トークン失効 (cross_bff_axiom_078)
    `CrossCutting.Bff.TokenExpiry,
    -- CrossCutting PII 分離推移性 (cross_pii_axiom_083)
    `CrossCutting.Pii.IsolationTransitivity,
    -- CrossCutting Edge 同期冪等性 (cross_edge_axiom_088)
    `CrossCutting.Edge.SyncIdempotency,
    -- Meta 軸レジストリ上限不変条件 (meta_axiom_093)
    `Meta.AxisRegistry.CapInvariant
  ]
