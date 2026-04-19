# SEC-SEC: セキュリティ基本要件

ネットワーク境界、mTLS、Pod セキュリティ、脆弱性管理に関する全体方針を定義する。Istio Ambient Mesh による透過的 mTLS が基盤。

---

## 前提

- [`../../02_構想設計/adr/ADR-0001-istio-ambient-vs-sidecar.md`](../../02_構想設計/adr/ADR-0001-istio-ambient-vs-sidecar.md)
- [`../10_アーキテクチャ/00_infra.md`](../10_アーキテクチャ/00_infra.md)

---

## 要件本体

> 本ファイルは骨格のみ。本文は後続タスクで記述する。
>
> 想定要件 (draft):
>
> - `SEC-SEC-001` クラスタ内通信は mTLS 必須
> - `SEC-SEC-002` Pod セキュリティ標準（Restricted 基準）
> - `SEC-SEC-003` CVE スキャンを CI 必須ゲートにする
> - `SEC-SEC-004` ネットワークポリシーのデフォルト deny
> - `SEC-SEC-005` Secrets を平文 ConfigMap に書かない
