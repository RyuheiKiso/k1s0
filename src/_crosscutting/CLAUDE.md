# _crosscutting コーディングポリシー

全軸共通ルールは `src/CLAUDE.md` を参照。本ファイルは _crosscutting 固有の制約のみ記述する。

## 配置・構成

- **命名**: `NN_<slug>/`（NN = 01〜13、slug = `[a-z][a-z0-9_-]+`）— lint 強制
- **上限**: 13 spec（追加時は lint 修正 + ARCHITECTURE.md dual sign-off 要）
- **SoT**: `docs/04_詳細設計/03_クロスカッティング適合仕様/00_cluster_bundle_map.md`

## コーディング制約

### spec と cluster の対応

- 各 spec は **1 cluster** にのみ bind（multi-cluster bind 禁止）
- `cluster_bundle_map.md` に先行登録なしで spec 実装禁止
- cluster_bundle_map.md の更新は dual sign-off 必須

### spec の実装範囲

- spec の設計範囲外の実装を `_crosscutting/NN_*/` 配下に追加しない
- 「この spec に関係ありそう」という理由だけで実装を追加しない
- 範囲外の実装は対応する軸（tier1 / tier2 / security 等）に帰属させる

### Lean4 proof の配置

- Lean 4 proof の実体は `src/formal/lean4_mathlib/` が primary
- `_crosscutting/` 配下に lean4/ stub ディレクトリを置かない（境界規律表を参照）

### ops-edge との境界

- `_crosscutting/10_ops_edge_cluster/`: K8s 基盤 manifests のみ
- escalation trigger config は `src/ops/ops_edge_cluster/manifests/` が管理（_crosscutting に置かない）

### 各言語の依存

- 各 spec 配下の `Cargo.toml` / `go.mod` / `package.json` は spec の設計方針 doc に従う
- tier1 Cargo workspace に含まれる _crosscutting crate:
  - `07_tauri_companion_sidecar/sidecar/`（sidecar exe）
  - `09_audit_ingest_gap_monitor/monitor/`
- tier1 go.work に含まれる _crosscutting module:
  - `04_protoc_gen_go_fsm/plugin/`
  - `03_apicurio_gitops_sot/controller/`

## spec 別メモ

| spec | primary 配置 | 各軸側の配置 |
|---|---|---|
| c01 HTTP/2 | Envoy config + enforcement test | `src/tier1/transport/http2/` |
| c02 KEK Shamir | M-of-N ceremony script | `src/tier1/kek/`（proof は `src/formal/lean4_mathlib/`）|
| c03 Apicurio GitOps | Apicurio additive controller（Go）| `src/data/apicurio/` |
| c04 FSM | protoc-gen-k1s0-go-fsm plugin | `src/client/sdk/go/generated/fsm/` |
| c05 SLO | Envoy Local Rate Limit config | `src/tier1/slo/` / `src/ops/alert_catalog/` |
| c06 BFF auth | BFF gateway（cookie / renew）| `src/tier1/auth/` |
| c07 Tauri sidecar | sidecar exe（WebUSB / BT / Serial）| `src/tier3/companion/` |
| c08 PII cluster | PII cluster K8s manifests | `src/data/pii_cluster/` |
| c09 audit gap | audit hash chain gap 監視実装 | `src/security/audit_chain/` |
| c10 ops-edge | ops-edge K8s 基盤 manifests | `src/ops/ops_edge_cluster/`（escalation trigger）|
| c11 OTel | .NET Framework OTel extension | `src/client/sdk/netfx/` |
| c12 UA adapter | UA-aware fetch adapter（TS）| `src/client/sdk/browser_ts/` |
| c13 Connect | k1s0.Connect.NetCore 実装 | `src/client/sdk/dotnet8/` |

## 関連参照

- `docs/04_詳細設計/03_クロスカッティング適合仕様/00_cluster_bundle_map.md` — 8 cluster × 13 spec SoT
- `docs/04_詳細設計/03_クロスカッティング適合仕様/` — 各 spec の詳細設計
