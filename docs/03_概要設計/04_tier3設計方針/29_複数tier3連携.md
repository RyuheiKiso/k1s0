---
id: arch.tier3.multi_tier3_collaboration
axis: tier3
phase: architecture
kind: policy
status: draft
depends_on:
  - arch.tier3.tier3_index
  - arch.tier3.auth_multi_tenant
  - detail.cross_bff.bff_auth_edge
covered_by:
  defense_in_depth_layers: [A, B, C, D, E]
  proof_classes:
    - v1_temporal_safety_proof
---

# tier3 複数 tier3 連携

## 一文方針
- 複数 tier3 業務 UI が並走する場合、各 tier3 ごとに独立 BFF auth-edge（cookie scope `Domain=<tier3>.<tenant>.<root_domain>`）+ 統一 SSO（Keycloak）+ tier3 間 cookie 漏洩構造的遮断 + back-channel logout 受信器の単一化で連携する。tier3 間の deep link は cross-domain redirect、業務処理は tier2 集約。

## 複数 tier3 並走の前提
- 業務領域別に tier3 を分離（製造業 pack の発注 / 検査 / FA など）
- 各 tier3 は別 sub-domain（`app.<tier3>.<tenant>.<root_domain>`）
- BFF auth-edge は tier3 ごとに 1 つ（[BFF auth-edge](../../04_詳細設計/03_クロスカッティング適合仕様/06_BFF_auth_edge.md)）

## 統一 SSO
- Keycloak で全 tier3 統一 SSO
- 各 tier3 BFF が Keycloak から token を取得
- back-channel logout は 1 receiver（global tier3 logout broadcast）
- session state は各 BFF で独立保持

## tier3 間 cookie 漏洩構造的遮断
- cookie scope `Domain=<tier3>.<tenant>.<root_domain>`（別 tier3 sub-domain から構造的に到達不能）
- CI で `Set-Cookie Domain` のスコープ違反検出
- SameSite=Lax 固定（OIDC redirect 戻り cookie 落ち回避）

## tier3 間 deep link
- 別 tier3 への navigate は cross-domain redirect
- 共通 SSO で再認証なし
- 業務 context（tenant_id / actor_id）は cookie + session で継続

## 業務処理の tier2 集約
- tier3 間で同 aggregate を扱う場合、tier2 が単一の真
- atomic 三表書込で aggregate 整合性 enforce
- BusinessConflict subtype で並行編集衝突解決

## tier3 間 navigation hint
- breadcrumb / 上部 navigation で他 tier3 への jump
- 業務シナリオ別の cross-tier3 deep link（例: 発注画面 → 検査画面）

## 採用しない設計
- tier3 間 cookie 共有（必ず Domain scope で分離）
- tier3 間で aggregate を多重所有（必ず tier2 集約）
- back-channel logout の tier3 別受信器（必ず単一受信器 + broadcast）
- 異なる SSO の tier3 並走

## 関連参照
- [tier3 設計方針 index](README.md)
- [認証認可マルチテナント](07_認証認可マルチテナント.md)
- [BFF auth-edge](../../04_詳細設計/03_クロスカッティング適合仕様/06_BFF_auth_edge.md)
- [tier2 マルチテナント方針](../03_tier2設計方針/05_マルチテナント方針.md)
