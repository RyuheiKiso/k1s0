# tools/catalog-check

IMP-TRACE-CAT-020〜029 の実装。`catalog-info.yaml` のスキーマ検証と Off-Path 検出を CI で行う 5 スクリプト群。

| スクリプト | IMP-ID | 責務 |
|---|---|---|
| `check-required-fields.sh` | CAT-020 | apiVersion/kind/metadata/spec の必須属性スキーマ検証 |
| `check-template-version.sh` | CAT-021 | `k1s0.io/template-version` annotation の存在と SemVer 形式検証 |
| `check-lifecycle.sh` | CAT-022 | `spec.lifecycle` が experimental/production/deprecated のいずれかか |
| `check-owner-system.sh` | CAT-023/024 | `spec.owner` Group 実在検証（前段）+ `spec.system` 実在検証（後段） |
| `scan-offpath.sh` | CAT-026/029 | catalog-info.yaml を持たない Off-Path component の検出（月次 cron 共用） |
