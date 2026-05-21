# tier1

設計方針: [`docs/03_概要設計/02_tier1設計方針/`](../../docs/03_概要設計/02_tier1設計方針/README.md)

## 配下ディレクトリ構成

| ディレクトリ | 役割 |
|---|---|
| `server/` | Server 系成果物（Gateway / Sidecar / BfL / ControlPlane）— Rust stable |
| `library/` | Library 成果物（Rust / C# / Go / TypeScript の 4 言語等価）— 17 機能カテゴリ |
| `schema/` | Proto + Buf workspace / Bidi schema / Migration schema / OSS lifecycle schema / Categories |
| `lint/` | tier1 独自 lint ツール（cargo-k1s0-lint / buf-plugin-k1s0）|
| `lock/` | build artifact lock.yaml 群（手書き禁止）|
| `operator/` | Go Kubernetes Operator（OSSInventory CRD / LifecycleSignal controller）|
| `tests/` | E2E / conformance テスト群（Migration / OSS lifecycle / Bidi）|

## 数値サマリ

| 項目 | 数値 |
|---|---|
| Transport adapter | 8 |
| Bidi conformance class | 5 |
| Bidi cell（applicable / total） | 29 / 40 |
| Library 言語 | 4（Rust / C# / Go / TypeScript）|
| 機能カテゴリ | 17 |
| Migration pair | 4 |
| Migration phase | 5 |
| OSS lifecycle drill | 21 |

## 関連 lock.yaml

| ファイル | 説明 |
|---|---|
| `lock/capabilities.lock.yaml` | Bidi 40 cell の green / not_applicable 状態 |
| `lock/dry_run.lock.yaml` | Migration pair × phase の 20 cell drill 記録 |
| `lock/oss_inventory.lock.yaml` | OSS lifecycle 21 drill の状態 |
| `lock/enforcement_points.lock.yaml` | Quota enforcement point の build artifact |
