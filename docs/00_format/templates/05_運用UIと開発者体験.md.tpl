---
id: detail.<axis>.ops_dx
axis: <axis>
phase: detail
kind: ops_dx
status: draft
depends_on: []
covered_by:
  defense_in_depth_layers: [D, E]
  proof_classes: []
---

# <axis> 運用 UI と開発者体験

## 一文方針
- <axis> の運用 UI は <dashboard 名>（<目的>）と <ChatOps bot 名>（<機能>）の N 経路で提供、<artifact> 編集は <IDE plugin / CLI / Tekton Pipeline preview> で支援、<実行経路> は <CI runner / 夜間 batch> の双方を提供する。

## Perses / Grafana dashboard 構成

### <dashboard 名 1>
- <表示要素>
- <drill-down 経路>
- <alert 連動>

### <dashboard 名 2>
- <表示要素>
- <drill-down 経路>

## ChatOps bot 機能
- `/<axis> <command 1> <args>`: <機能>
- `/<axis> <command 2> <args>`: <機能>
- `/<axis> break-glass <args>`: 短期 cosign signing token を OpenBao response wrapping で発行 + audit_event emit + 24 hour 内 retro review

## IDE plugin / CLI

### VS Code plugin
- <syntax highlight / LSP>
- <inline preview>
- <warning display>

### IntelliJ plugin
- <plugin 名>

### vim / emacs
- 各 tool の official syntax / LSP support のみ。商用 fork は採用しない。

## CI runner / 夜間 batch
- CI runner: <短時間で完走可能な工程>
- 夜間 batch: <長時間 / 重 workload>
- artifact: <生成物 + 保存先>

## developer onboarding
- Backstage Software Catalog entry: `<axis>-onboarding`
- Tilt local environment: <local mock 構成>
- dev container: <ベース image + 同梱 toolchain>

## audit / 不可逆性
- 全 operator action（break-glass / approval / override）は audit_event subject に `v1_<axis>_<action>` fact emit
- ChatOps bot は dual reviewer signoff を要する operation を `cosign signed approval` で物理 enforce
