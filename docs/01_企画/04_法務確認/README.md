---
id: plan.legal_check
axis: overview
phase: plan
kind: index
status: draft
depends_on:
  - plan.background_purpose
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# 法務確認

## 一文方針
- 採用 OSS のライセンス階層は (a)〜(e) のいずれかに必ず帰属し、(d) AGPL / SSPL は不採用、商用利用 / 配布 / fork / sublicense / 自製拡張の各局面で license-checker / cargo-deny / dotnet-licenses で CI 物理 enforce する。GDPR / 各国データ局所性規制 / 業界規制（医薬品 GMP / ISO 9001 等）に対応する。

## OSS ライセンス階層

| 階層 | 例 | 採用条件 |
|---|---|---|
| (a) Permissive | Apache 2.0 / MIT / BSD-3 / ISC | default 採用 |
| (b) Weak Copyleft | MPL 2.0 / LGPL-2.1 | linkage_model（dynamic / static / process_boundary）で要件確認 |
| (c) Strong Copyleft（CE 付き） | GPL-2 + Classpath Exception（OpenJDK） | process_boundary（runtime 経由）のみ許容 |
| (d) AGPL / SSPL | AGPL-3 / SSPL-1.0 | **不採用**（Grafana / Tempo / Loki / k6 / Grafana OnCall / MongoDB / Elasticsearch / Redis 等） |
| (e) OS native 例外 | OS native API（DPAPI / Keychain / APNs / FCM） | linkage_model = OS-internal、用途限定（ops-edge escalation last-mile 等） |

## linkage_model 規律
本企画の非 Apache-2.0 採用 OSS の linkage_model:
- .NET 8 / Framework 4.6.2（MIT）: process_boundary（CLR runtime 経由）
- OpenJDK Temurin 21（GPLv2+CE）: process_boundary（JVM runtime 経由、Classpath Exception により application linking は許容）
- Node.js 20（MIT）: process_boundary（V8 runtime 経由）
- Tauri（MIT/Apache）: static
- rustc / Rust crate（MIT/Apache）: static
- signalfx/splunk-otel-dotnet fork（Apache 2.0）: dynamic（NuGet 動的 link）

linkage_model は license-checker / cargo-deny / dotnet-licenses で CI 検査。

## 採用しない OSS（ライセンス理由）
- AGPL-3 系: Grafana / Tempo / Loki / k6 / Grafana OnCall / Trufflehog（trufflehog は MIT 互換 fork で対応）
- SSPL-1.0 系: MongoDB / Elasticsearch
- BUSL-1.1 系: HashiCorp Vault → OpenBao（MPL 2.0 fork）/ Redis → Valkey（BSD-3 fork）
- Confluent Community License: Confluent Schema Registry v5+ → Apicurio / Karapace
- Elastic License v2: Elasticsearch → ClickHouse
- Salesforce / OutSystems / Mendix 等の商用 license

## GDPR 対応
- データポータビリティ: テナント data export 機構（[tier2 データライフサイクル](../../03_概要設計/03_tier2設計方針/20_データライフサイクル.md)）
- 物理削除（right to be forgotten）: KEK destroy で wrapped DEK を不可逆失効（crypto-shred、[KEK Shamir 分散](../../04_詳細設計/03_クロスカッティング適合仕様/02_KEK_shamir_distribution.md)）
- 同意管理: cookie consent banner + analytics opt-in（[tier3 セキュリティアーキテクチャ](../../03_概要設計/04_tier3設計方針/17_セキュリティアーキテクチャ.md)）
- データ局所性: cross-region preservation_class + tenant 単位 region 指定

## 各国規制対応
- 日本: 個人情報保護法（PII 5 class taxonomy + envelope encryption）
- EU: GDPR + データ局所性
- 米国: HIPAA（医療業界 pack v2 候補） / SOX（金融業界 pack v2 候補）
- 業界規制: 製造業 ISO 9001 / 医薬品 GMP / 食品 HACCP（業界 pack 単位で宣言）

## 商用 PenTest / 監査
- 1.0.0 ship 前に少なくとも 1 回の商用 PenTest 完了
- 自動 CVE 配信 + Renovate + 商用 PenTest の三重化必須
- audit hash chain 外部公証は RFC 3161 trusted timestamp + Sigstore transparency log

## 採用しない法務スタンス
- 商用 PenTest 結果のみで脆弱性管理を完了
- AGPL / SSPL 系 OSS の採用
- GDPR 物理削除を logical delete で代替
- LLM 単独 sign-off（必ず人間 reviewer 必須）

## 関連参照
- [背景と目的](../01_背景と目的/README.md)
- [OSS ライセンス規律](../../02_要件定義/04_技術選定/02_OSSライセンス規律.md)
- [脅威モデル適合仕様](../../04_詳細設計/01_適合仕様/15_脅威モデル適合仕様.md)
- [鍵管理適合仕様](../../04_詳細設計/01_適合仕様/05_鍵管理適合仕様.md)
- [tier2 データライフサイクル](../../03_概要設計/03_tier2設計方針/20_データライフサイクル.md)
