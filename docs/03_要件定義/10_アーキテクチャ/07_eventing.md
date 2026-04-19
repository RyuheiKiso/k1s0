# ARC-EVT: イベンティング要件

Kafka、Dapr Pub/Sub、Dapr Workflow、Temporal を組み合わせたイベント駆動アーキテクチャの要件を定義する。ワークフローの 2 基盤振り分けは High リスクとして監視対象。

---

## 前提

- [`01_tier1.md`](./01_tier1.md) tier1 要件
- [`../../01_企画/企画書.md`](../../01_企画/企画書.md) 13.5 章 ワークフロー振り分け

---

## 要件本体

> 本ファイルは骨格のみ。本文は後続タスクで記述する。
>
> 想定要件 (draft):
>
> - `ARC-EVT-001` Kafka をイベントバックボーンとして採用
> - `ARC-EVT-002` Pub/Sub 抽象は Dapr Pub/Sub を介する
> - `ARC-EVT-003` Dapr Workflow vs Temporal の振り分け基準
> - `ARC-EVT-004` イベントスキーマ管理（Schema Registry）
> - `ARC-EVT-005` At-least-once / 冪等性の扱い
