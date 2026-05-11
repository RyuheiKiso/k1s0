---
id: req.tech.oss_license_discipline
axis: overview
phase: requirement
kind: requirement
status: draft
depends_on:
  - req.tech.tech_index
  - plan.legal_check
covered_by:
  defense_in_depth_layers: [B, E]
  proof_classes: []
---

# OSS ライセンス規律

## 一文方針
- 採用 OSS のライセンスは (a) Permissive / (b) Weak Copyleft / (c) Strong Copyleft + CE / (d) AGPL / SSPL 不採用 / (e) OS native 例外 の 5 階層で分類する。license-checker / cargo-deny / dotnet-licenses で CI 物理 enforce、linkage_model（dynamic / static / process_boundary）を artifact_inventory.lock.yaml に固定する。

## 5 階層
- **(a) Permissive**: Apache 2.0 / MIT / BSD-3 / ISC（default 採用）
- **(b) Weak Copyleft**: MPL 2.0 / LGPL-2.1（linkage_model で要件確認）
- **(c) Strong Copyleft + CE**: GPL-2 + Classpath Exception（OpenJDK、process_boundary のみ許容）
- **(d) AGPL / SSPL**: 不採用（Grafana / Tempo / Loki / k6 / Grafana OnCall / MongoDB / Elasticsearch / Redis 等）
- **(e) OS native 例外**: OS native API（DPAPI / Keychain / APNs / FCM）、用途限定

## linkage_model 規律
- 全採用 OSS の linkage_model（dynamic / static / process_boundary）を `artifact_inventory.lock.yaml` に固定
- license-checker / cargo-deny / dotnet-licenses で CI 検査

## 不採用 OSS（ライセンス理由）
- AGPL-3 系: Grafana / Tempo / Loki / k6 / Grafana OnCall
- SSPL-1.0 系: MongoDB / Elasticsearch
- BUSL-1.1 系: HashiCorp Vault → OpenBao / Redis → Valkey
- Confluent Community License: Confluent Schema Registry v5+ → Apicurio / Karapace
- Elastic License v2: Elasticsearch → ClickHouse

## 受入条件
- 全採用 OSS が (a)〜(e) のいずれかに分類済
- license-checker / cargo-deny / dotnet-licenses CI green
- AGPL / SSPL / BSL 系 OSS の採用ゼロ

## 関連参照
- [技術選定 index](README.md)
- [OSS 採用一覧](01_OSS採用一覧.md)
- [法務確認](../../01_企画/04_法務確認/README.md)
