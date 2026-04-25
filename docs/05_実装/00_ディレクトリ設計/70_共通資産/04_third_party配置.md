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

## vendoring 要否の 3 軸判定基準

上記の 4 ケースは動機の例示だが、実運用で「どのレベルで vendoring するか」を判断する際に迷いが生じる。以下の 3 軸のいずれか 1 つでも満たしたら vendoring（`third_party/` 配下配置）とし、全て満たさなければ通常依存（パッケージマネージャ）で扱う。

### 軸 1: パッチ本数（修正の重み）

upstream に merge されていない独自修正が **3 件以上**、または **1 件でも 200 行超** のパッチがある場合は vendoring する。

- 3 件未満かつ各 50 行未満 → 個別に fork せず upstream PR + monkey-patch 運用
- 3 件以上 / 大規模 → `third_party/<name>-patch/` に配置し PATCHES.md で管理

根拠: パッチ本数が多いと upstream rebase のコストが runtime パッチ方式を上回る。quilt 形式で連続管理する方が保守容易になる。

### 軸 2: ライセンス互換性（法務リスク）

以下のいずれかに該当する OSS は、たとえパッチ 0 本でも vendoring し、LICENSES/ に原文保存 + NOTICE 自動生成に載せる。

- GPL / AGPL / LGPL 系（linking 方式で本体ライセンスに影響が波及する可能性）
- dual-license で BSL / SSPL / Commons Clause / ELv2 等の不透明ライセンスが含まれる
- ライセンス不明 / 不記載

根拠: 法務監査時に「何をいつ取り込んだか」の証跡（コミット + UPSTREAM.md）がないと ADR-GOV-001 に違反する。パッケージマネージャ経由だとロックファイルに version のみ残り、原文が残らない。

### 軸 3: セキュリティ重要度（影響範囲）

tier1 が直接依存する以下の領域の OSS は、patchless でも vendoring を推奨する。

- crypto primitives（ring / rustls / openssl-sys 等）
- 認証 / 認可（keycloak admin-client 等）
- PII 処理 / 監査ログ（ADR-AUDIT-\* 関連）
- Dapr Runtime / Protobuf stub 生成器

根拠: これらは 採用側組織の監査で「どの version の何をどう検証したか」の再現性が求められる。upstream に依存していると、上流が削除された時点で監査証跡が失われる。vendoring により commit hash 単位で再現性を保つ。

### 判定フロー

```
新規 OSS 採用時
  │
  ├─ 軸 1（パッチ本数）判定 ─┐
  ├─ 軸 2（ライセンス）判定 ─┤
  └─ 軸 3（重要度）判定 ────┤
                            ↓
            1 つでも Yes → third_party/ に vendoring
            全て No     → パッケージマネージャ経由
```

判定は Pull Request 内で UPSTREAM.md に「どの軸で vendoring 判定したか」を記録する。軸 2 / 3 で取り込んだものは、期限なしで保持（upstream release で状況が変わっても原本保持継続）。軸 1 のみで取り込んだものは upstream merge 後に `third_party/` から削除する。

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

Dapr Workflow API の ○○ 機能が upstream v1.15 で追加予定だが、リリース時点 の 2026-06 までに本家 release が見込めない。
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
