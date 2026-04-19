# ARC-INF: インフラ要件

Kubernetes クラスタ、ノード構成、Istio Ambient Mesh、ネットワーク境界など、k1s0 プラットフォームの基盤インフラに関する要件を定義する。本ファイルの要件は全 tier に波及するため、変更時は `01_tier1.md` 以降との整合を必ず確認する。

---

## 前提

- [`../00_共通/03_constraint.md`](../00_共通/03_constraint.md) の技術スタック制約
- [`../../02_構想設計/adr/ADR-0001-istio-ambient-vs-sidecar.md`](../../02_構想設計/adr/ADR-0001-istio-ambient-vs-sidecar.md) Istio Ambient Mesh 採用
- [`../../02_構想設計/01_アーキテクチャ/`](../../02_構想設計/01_アーキテクチャ/) のアーキテクチャ設計

---

## 要件本体

> 本ファイルは骨格のみ。本文は後続タスクで記述する。
>
> 想定要件 (draft):
>
> - `ARC-INF-001` Kubernetes バージョン下限の指定
> - `ARC-INF-002` ノード構成（コントロールプレーン / ワーカー / ストレージ）
> - `ARC-INF-003` Istio Ambient Mesh の必須化
> - `ARC-INF-004` CNI の選定と MTU 要件
> - `ARC-INF-005` クラスタ DNS と社内 DNS の連携
> - `ARC-INF-006` マルチクラスタ / シングルクラスタ判断基準
