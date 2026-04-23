# 07. partial clone / sparse index

本ファイルは partial clone と sparse index の導入理由と運用を規定する。cone mode だけでは解決できない「巨大リポジトリでの git 操作の遅さ」を解消する。

## 問題設定

Phase 1c 時点で k1s0 は以下を予測する。

- 全 tier 合計 50 万〜 100 万行
- 全コミット数 1 万超
- 全 blob（過去 version 含む）数百 GB

この規模では以下の操作が遅くなる:

- 初回 clone: 数十分
- `git status`: 数秒〜十数秒
- `git switch`: 数秒
- IDE 起動時の初回 indexing: 数分

解決策は 3 つ組み合わせる:

1. **cone mode sparse-checkout**: ワーキングツリーを絞る
2. **partial clone**: blob を遅延取得
3. **sparse index**: `.git/index` を圧縮

## partial clone

### 概要

`git clone --filter=<filter-spec>` で、指定 filter にマッチする object のみ clone する。k1s0 は `blob:none` を採用。

```bash
git clone --filter=blob:none --sparse https://github.com/k1s0/k1s0.git
```

- `--filter=blob:none`: blob（ファイル内容）は clone 時に取得しない、commit と tree のみ
- 必要な blob は on-demand で fetch（`git checkout` や `git show` 時）

### 効果

k1s0 リポジトリ想定値（Phase 1c）:

| 項目 | 通常 clone | partial clone |
|---|---|---|
| 初回 clone サイズ | 3 GB | 300 MB |
| 初回 clone 時間（100Mbps） | 4 分 | 25 秒 |
| ディスク使用量 | 3 GB | 300 MB 〜 1.5 GB（作業範囲による） |

### 注意点

- Offline 作業は未 fetch blob に触れると失敗する
- `git log --follow` のような history 巡回操作で不足 blob が fetch される（最初は遅い、以降キャッシュ）
- サーバ側（GitHub / Gitea / セルフホスト）が partial clone をサポートしている必要あり

## sparse index

### 概要

Git 2.37+ で導入された `.git/index` の圧縮形式。cone mode と組合せた場合に限り有効。

従来の index は全ファイルを entry として持つため、100 万ファイルのリポジトリでは index が数百 MB になり `git status` が遅い。sparse index では「cone 外ディレクトリは tree のまま」として持ち、index を数十分の 1 に圧縮。

### 有効化

```bash
git config core.sparseCheckoutCone true
git sparse-checkout init --cone --sparse-index
git sparse-checkout reapply
```

または既存の cone mode リポジトリで:

```bash
git update-index --split-index  # 古い split-index モード（不要、消す）
git config core.sparseCheckoutCone true
git sparse-checkout reapply
```

### 効果

Microsoft の dotnet/runtime ベンチマーク:

| 操作 | 通常 index | sparse index |
|---|---|---|
| git status | 1.5 s | 0.2 s |
| git checkout（branch 切替） | 3.5 s | 0.5 s |
| git add | 0.8 s | 0.1 s |

k1s0 でも同等の改善を期待。

### 注意点

- 一部の古い Git コマンド（`git stash apply` の古い実装など）で動作が不安定な報告あり → Git 2.42+ で解消
- Git 2.40 を最低バージョンとする（k1s0 は Git 2.42+ を推奨、Dev Container でも 2.42+ を pin）

## 両者の併用（推奨）

k1s0 は以下を標準とする。

```bash
# 初回 clone（推奨コマンド）
git clone \
    --filter=blob:none \
    --sparse \
    https://github.com/k1s0/k1s0.git

cd k1s0

# cone mode + sparse index 有効化
git config core.sparseCheckoutCone true
git sparse-checkout init --cone --sparse-index

# 役割適用
./tools/sparse/checkout-role.sh tier1-rust-dev
```

この 3 要素（partial clone + cone mode + sparse index）の組合せが 2026 年時点の monorepo 運用ベストプラクティス。

## リモート側の設定

GitHub は partial clone をデフォルトサポート。セルフホスト Git 運用（Gitea 等）の場合、サーバ設定で `uploadpack.allowFilter=true` が必要。

```
# /etc/gitconfig on Gitea server
[uploadpack]
    allowFilter = true
    allowAnySHA1InWant = true
```

## 運用時の注意

### 古い fetch の promote

partial clone で「blob を on-demand fetch」した結果、リポジトリ内に blob が蓄積する。定期的に `git gc --aggressive` を実行し、不要 blob を整理する。

```bash
# ops/runbooks/monthly/git-gc.md
git gc --aggressive --prune=now
```

### CI で partial clone は使わない

CI は基本毎回新規 clone。partial + sparse のセットアップコストよりも full clone の方が簡潔。CI キャッシュで `.git/` を保存する場合は partial 情報を保持するよう注意。

### submodule との併用

k1s0 は submodule 不採用のため、partial clone + submodule の複雑性は回避される。

## Phase 導入タイミング

| Phase | 設定 |
|---|---|
| Phase 0 | 未採用（リポジトリ容量が小さいため不要） |
| Phase 1a | オプション（開発者判断） |
| Phase 1b | 標準推奨（`bootstrap-developer.sh` が自動適用） |
| Phase 1c | 標準必須（開発者セットアップ文書で強制） |

## 対応 IMP-DIR ID

- IMP-DIR-SPARSE-132（partial clone / sparse index）

## 対応 ADR / 要件

- ADR-DIR-003
- DX-GP-\*
