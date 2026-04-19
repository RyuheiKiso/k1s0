# ARC-T1: tier1 要件

tier1 ファサードのスコープ、Dapr 隠蔽ルール、Go と Rust のハイブリッド境界、内部通信（Protobuf gRPC 必須）に関する要件を定義する。

---

## 前提

- [`../00_共通/03_constraint.md`](../00_共通/03_constraint.md) 技術スタック制約
- [`../../02_構想設計/02_tier1設計/`](../../02_構想設計/02_tier1設計/) tier1 設計

---

## 要件本体

> 本ファイルは骨格のみ。本文は後続タスクで記述する。
>
> 想定要件 (draft):
>
> - `ARC-T1-001` tier1 の責務スコープ（業務ロジックを持たない）
> - `ARC-T1-002` Dapr 隠蔽の原則（tier2/3 から daprd を直接呼ばない）
> - `ARC-T1-003` Go ファサードと Rust 自作領域の境界判定ツリー
> - `ARC-T1-004` tier1 内部通信は Protobuf gRPC 必須
> - `ARC-T1-005` 公開 API バッジ（[Go] / [Rust]）
> - `ARC-T1-006` ワークフロー振り分け（Dapr Workflow / Temporal）
