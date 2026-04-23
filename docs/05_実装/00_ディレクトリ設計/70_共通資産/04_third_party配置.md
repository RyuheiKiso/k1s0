# 04. third_party 配置

本ファイルは `third_party/` 配下の配置を確定する。社内フォーク OSS / パッチ版を vendoring する場所。通常の依存は各言語のパッケージマネージャ（cargo / go mod / NuGet / npm）で管理し、第三者リポジトリから直接取得するのが原則。`third_party/` はあくまで以下の例外用途に限る。

## third_party/ を使うケース

- **セキュリティパッチ適用**: OSS に脆弱性パッチを当て、upstream merge 待ち
- **未リリース機能の先取り**: upstream にある機能が正式リリース前に必要
- **実質メンテ停止 OSS の引き取り**: 重要機能の延命
- **ライセンス互換性のためのリワーク**: 部分的にコード改変が必要

## 使わないケース

- 通常の upstream version 依存 → cargo / go mod / NuGet / npm で管理
- 単なる OSS 参考コード → 引用 URL を README に記述
- 自作コード → `src/` 配下に置く

## レイアウト

```
third_party/
├── README.md
├── LICENSES/                       # 各 OSS の元ライセンス保持
│   ├── opentofu-MPL-2.0.txt
│   ├── dapr-Apache-2.0.txt
│   └── ...
├── dapr-patch/
│   ├── UPSTREAM.md                 # fork 元コミット、理由、期限
│   ├── PATCHES.md                  # 適用パッチ一覧
│   ├── Cargo.toml                  # 各言語毎の build 定義
│   └── src/                        # 改変したソース
├── strimzi-kafka-patch/
│   ├── UPSTREAM.md
│   ├── PATCHES.md
│   └── ...
└── custom-otel-collector-exporter/ # 独自 OTel exporter
    ├── UPSTREAM.md
    ├── go.mod
    └── ...
```

## UPSTREAM.md の必須項目

各 fork には `UPSTREAM.md` を必置。

```markdown
# UPSTREAM

## fork 元

- Repository: https://github.com/dapr/dapr
- Commit: a1b2c3d4...
- Branch: v1.14.0

## fork 理由

Dapr Workflow API の ○○ 機能が upstream v1.15 で追加予定だが、Phase 1b の 2026-06 までに本家 release が見込めない。
部分的に v1.15 相当の実装をバックポートする。

## 適用パッチ

- `patches/0001-backport-workflow-api.patch`: workflow API バックポート
- `patches/0002-fix-placement-timeout.patch`: placement timeout 修正（upstream PR #1234 相当）

## 解消条件

upstream v1.15 release 後、本 fork を削除して通常依存に戻す。

## 解消期限

2026-09-30（これを過ぎると fork 維持コストが upstream 追従を上回るため、再評価）
```

## PATCHES.md の運用

パッチは `quilt` 形式で管理、upstream rebase の負担を最小化。

```markdown
# PATCHES

## 0001-backport-workflow-api.patch

- upstream PR: https://github.com/dapr/dapr/pull/1234
- 目的: workflow API を v1.14 系にバックポート
- merge 可能性: 高（upstream で正式 merge 予定）
- 責任者: @k1s0/tier1-rust

## 0002-fix-placement-timeout.patch

- upstream issue: https://github.com/dapr/dapr/issues/5678
- 目的: placement timeout を 5s → 30s に変更
- merge 可能性: 低（upstream は SLO 論争中）
- 責任者: @k1s0/sre-ops
```

## 定期レビュー

四半期毎（1/4/7/10 月）に fork の解消状況をレビューする。`ops/runbooks/monthly/third-party-review.md` を Runbook として規定し、各 fork の `UPSTREAM.md` の解消期限を監査する。期限超過 fork は ADR で「延長／廃棄／upstream 吸収」を判断する。

## LICENSES/ の管理

全 fork の元 OSS ライセンス原文を `LICENSES/` に保存。ルートの `NOTICE` ファイル（SBOM から生成）でも本 fork を列挙。NFR-H-COMP-\*（OSS ライセンス遵守）の要件を満たす。

## ビルド / CI

各 fork は独立 build 可能。`.github/workflows/ci-third-party.yml` で build & test を実施。

- Rust fork: `cargo build && cargo test`
- Go fork: `go build ./... && go test ./...`
- .NET fork: `dotnet build && dotnet test`

これらの成果物は Harbor の `harbor.k1s0.internal/third-party/<name>:<tag>` に push。

## CODEOWNERS

```
/third_party/                       @k1s0/arch-council @k1s0/security-team
```

fork の追加・削除は arch-council と security-team の承認を要する。

## 対応 IMP-DIR ID

- IMP-DIR-COMM-114（third_party 配置）

## 対応 ADR / DS-SW-COMP / 要件

- ADR-GOV-001（OSS ライセンス遵守ポリシー）
- NFR-H-COMP-\*
