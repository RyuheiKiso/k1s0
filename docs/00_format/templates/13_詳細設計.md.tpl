---
id: detail.<axis>.<slug>
axis: <axis>
phase: detail
kind: detail
status: draft
depends_on: []
covered_by:
  defense_in_depth_layers: [A, B, C, D, E]
  proof_classes: []
lock_artifacts: []
---

# <タイトル>

## 一文方針
- <この詳細設計ドキュメントの核心を 1 文で>。

## 設計の物理転写
- <この設計が物理 artifact として何に転写されるか>。
    - <例: lock.yaml / cosign signature / Kyverno ClusterPolicy / Tekton Pipeline / proof certificate>。

## artifact / lock.yaml 体系

### `<artifact 名>.lock.yaml`
- 生成元: <generator 名>
- 入力: <入力 artifact 列挙>
- 出力 schema: <field 列挙 / または link>
- enforce 経路:
    - 層 A: <schema check>
    - 層 B: <Conftest rule 列挙>
    - 層 D: <Kyverno admission rule>
    - 層 E: <cosign sign 経路 + Object Lock retention>

## CI / CD 経路
- Tekton Pipeline:
    - stage: <stage 名> → <task 列挙>
- Argo CD ApplicationSet:
    - sync wave: <wave 順>
- release_gate.lock.yaml:
    - cell: `<slug>` → 1.0.0 ship blocker

## audit / immutability
- audit_event subject: `v1_<axis>_<action>`
- WORM 経路: <Object Lock retention period>
- cryptographic chaining: <hash chain 経路>

## 例外 / break-glass
- break-glass 経路:
    - 発行: OpenBao response wrapping (TTL = N hour)
    - audit: `v1_<axis>_break_glass` fact emit
    - 事後: 24 hour 内 retro review 必須

## 関連参照
- [<要件定義側の根拠>](../../02_要件定義/<相対パス>.md)
- [<概要設計側の方針>](../../03_概要設計/<相対パス>.md)
- [<同フェーズ内の隣接 doc>](<相対パス>.md)
