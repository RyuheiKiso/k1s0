# ADR-DIR-003: Git Sparse Checkout cone mode + Partial Clone の標準運用採用

- ステータス: Accepted
- 起票日: 2026-04-23
- 決定日: 2026-04-23
- 起票者: kiso ryuhei
- 関係者: 全開発チーム / DevX 担当 / SRE 担当 / CI/CD 担当

## コンテキスト

k1s0 はモノレポ構成（[../../../CLAUDE.md](../../../CLAUDE.md) 参照）であり、tier1 / tier2 / tier3 / SDK / infra / deploy / ops / docs / tools / tests / examples の全資産を単一リポジトリで管理する。全資産を揃えた場合の想定規模は以下の通り（リリース時点推定）。

- tier1 Go + Rust: 約 5 万行
- tier2 .NET + Go: 約 8 万行
- tier3 Web + MAUI + BFF: 約 12 万行
- SDK 4 言語: 約 3 万行
- Protobuf 契約 + 生成コード: 約 2 万行
- infra / deploy / ops: 約 10 万行（YAML / Helm / Kustomize / OpenTofu）
- docs: 約 20 万行（設計書 + ADR + Runbook）

合計で 60 万行超の Git リポジトリとなる見込み。ファイル数ベースで数万ファイル、`.git/` を含む working tree サイズは数 GB に達する可能性がある。

この規模でフルチェックアウトを全開発者に要求すると以下の問題が顕在化する。

- **オンボーディング時の初期クローン時間が数十分オーダー**になり、新規参加者の立ち上げ遅延が発生（DX-MET 指標の First PR Time 悪化）
- **tier1-rust-dev 担当者は tier3 の React / MAUI コードを編集しない**にもかかわらず、working tree に全ファイルが存在するため IDE のインデックス作成負荷が増大
- **rust-analyzer / gopls / TypeScript Language Server が tier1-rust-dev の編集と無関係な tier3 / tier2 ファイルもインデックスしようとして**、IDE 応答性が劣化（CPU / RAM 使用量増大）
- **CI の path-filter トリガ**と**開発者のワーキングセット**が一致せず、「CI は path 単位で回すのに開発者は全部チェックアウトする」という非対称性が生じる
- **Windows 環境では長パス問題や大小文字衝突**が発現しやすい（例: `src/contracts/v1/` と `src/Contracts/V1/` の衝突）

Git 2.27 以降で導入された `cone mode` と Git 2.32 以降で成熟した `partial clone`（filter=blob:none）、Git 2.37 以降の `sparse index` を組み合わせることで、役割別のワーキングセット最小化と Git 操作高速化を同時に実現できる。本 ADR ではこれを k1s0 の標準運用として位置付ける。

世界トップ企業のモノレポ運用事例（Google Piper + CitC、Meta Sapling + Eden、Microsoft VFS for Git / Scalar、Twitter Source、Uber Go monorepo）はいずれも Virtual File System または sparse checkout による部分取得を前提としており、数百万行規模の単一リポジトリは「全員が全部を持つ」前提では維持できないことが実証されている。k1s0 は 60 万行規模で Google / Meta には及ばないが、採用側の小規模運用を前提とする 採用側組織固有の制約（NFR-C-NOP-001）下では、1 人あたりの IDE / Git 操作負荷を最小化する設計判断が決定的に重要となる。

## 決定

**Git Sparse Checkout cone mode + Partial Clone + Sparse Index を k1s0 の標準運用として推奨する。**

詳細方針は以下の通り。

### 役割別 cone 定義

`.sparse-checkout/roles/` ディレクトリに 10 種類の役割別 cone 定義ファイルを配置する。各ファイルは cone mode のディレクトリ列挙形式で記述する。

- `tier1-rust-dev.txt` : `src/contracts/` + `src/tier1/rust/` + `docs/` + ルート設定ファイル
- `tier1-go-dev.txt` : `src/contracts/` + `src/tier1/go/` + `docs/` + ルート設定ファイル
- `tier2-dev.txt` : `src/contracts/` + `src/tier2/` + `src/sdk/dotnet/` + `src/sdk/go/` + `docs/`
- `tier3-web-dev.txt` : `src/sdk/typescript/` + `src/tier3/web/` + `src/tier3/bff/` + `docs/`
- `tier3-native-dev.txt` : `src/sdk/dotnet/` + `src/tier3/native/` + `docs/`
- `platform-cli-dev.txt` : `src/platform/` + `src/contracts/` + `docs/`
- `sdk-dev.txt` : `src/contracts/` + `src/sdk/` + `tools/codegen/` + `tests/contract/` + `docs/`
- `infra-ops.txt` : `infra/` + `deploy/` + `ops/` + `tools/` + `docs/`
- `docs-writer.txt` : `docs/` + ルート README 系
- `full.txt` : 全部（スパースなし相当）

`sdk-dev` は採用初期の小規模運用では他役割との兼任を想定するが、SDK の「公開 Protobuf → 4 言語 stub 翻訳」という独立責務を明示するため独立ロールとして定義する。SDK の利用者数が増えた時点で専任化を再評価する。

### 初期クローン手順

リポジトリの初期取得は以下のいずれかで行う。

- **partial clone + cone mode** を標準とする

```bash
git clone --filter=blob:none --no-checkout https://github.com/k1s0/k1s0.git
cd k1s0
git sparse-checkout init --cone --sparse-index
git sparse-checkout set --stdin < .sparse-checkout/roles/tier1-rust-dev.txt
git checkout main
```

- **Dev Container 初回起動時の対話スクリプト** `tools/sparse/checkout-role.sh` を実装し、役割選択のプロンプトで cone 切替を自動化

### 役割切替運用

開発者の担当変更時は `tools/sparse/checkout-role.sh tier2-dev` のような CLI で cone 定義を切り替える。切替時は未コミット変更を stash することで working tree 整合を保つ。

### full cone の許容

`full.txt`（全ディレクトリ含む）を選ぶことは禁止しない。以下のケースでは full を推奨する。

- リポジトリ全体に影響する横断リファクタリング
- 新規コンポーネント追加時の影響範囲調査
- インシデント調査で全コードを grep する必要がある場合

### submodule の不採用

本 ADR では submodule は採用しない。モノレポの利点（atomic commit、breaking change の一貫性保証）を損なうため。第三者 OSS のフォーク管理は `third_party/` 配下に vendoring（git subtree なし、直接 copy + CHANGELOG）で対応する。

### Git LFS の判定

Git LFS は本 ADR の対象外とする。drawio / svg / png など figura 類は通常ファイルで管理し、将来的に 100 MiB を超えるバイナリが発生した時点で別 ADR（ADR-DIR-004）を起票して判定する。

### 生成コードの commit

Protobuf 生成コード（tier1 Go の `internal/proto/v1/` / tier1 Rust の `proto-gen` crate 出力）は commit する（DS-SW-COMP-122 準拠）。cone 定義では生成コードのパスも含めて取得対象とする。

### Windows 環境配慮

全パスは小文字で統一し、大小文字区別の衝突を回避する。Windows 環境の長パス制限（260 文字）対策として、最大ディレクトリ階層を 5 階層に制限する。

### 採用側の運用拡大時の必須化判定

採用側の開発者数が増えた時点で、sparse checkout の必須化可否を再評価する。再評価基準は「開発者数 10 人以上」または「CI 時間が平均 20 分超」のいずれかを満たした場合。

## 検討した選択肢

### 選択肢 A: cone mode + partial clone + sparse index 標準化（採用）

- 概要: 上記の通り
- メリット:
  - Git 公式機能で追加依存なし（2.37 以降の標準 Git で利用可能）
  - 開発者ワーキングセットが役割に応じて最小化、IDE 応答性向上
  - CI の path-filter と開発者ワーキングセットの粒度が一致
  - リリース時点から習慣化することで、リポジトリ成長時にスムーズに移行
  - 採用側の小規模運用で 1 人 1 人の負荷を最小化（NFR-C-NOP-001 の前提）
- デメリット:
  - 初回セットアップで開発者が cone 概念を学習する必要（学習コスト 30 分程度）
  - 役割切替時に stash が必要なケースがあり、注意書きが必要
  - 一部 IDE / CI で sparse index 未対応の機能がある（`git status -u` 等）

### 選択肢 B: Meta Sapling + EdenFS 採用

- 概要: Meta OSS の仮想ファイルシステムを使い、OnDemand 取得
- メリット: ワーキングセット管理が自動
- デメリット:
  - EdenFS は Linux のみ対応（Windows / Mac で制限）
  - 採用側の小規模運用で保守する OSS スタックを増やすリスク
  - 60 万行規模では過剰、Google / Meta レベルの規模で意味を発揮

### 選択肢 C: VFS for Git (Microsoft Scalar)

- 概要: MS の Git 仮想化
- メリット: Windows 環境との親和性
- デメリット:
  - Scalar は Microsoft Azure Repos 前提の最適化が多く、GitHub との相性が劣る
  - Linux / Mac サポートが薄い
  - k1s0 の Dev Container 前提（Linux コンテナ）と噛み合わない

### 選択肢 D: フルチェックアウト運用（sparse checkout を使わない）

- 概要: 全員が全ファイルを持つ
- メリット: 設定不要、Git 操作がシンプル
- デメリット:
  - 初期クローン時間の増大
  - IDE インデックス負荷の悪化
  - 60 万行規模で運用が持続不能になる閾値を採用側の運用拡大期にほぼ確実に超える

### 選択肢 E: multi-repo 化（モノレポ放棄）

- 概要: tier1 / tier2 / tier3 / infra を別リポジトリに分割
- メリット: 各 repo が小さく、sparse checkout 不要
- デメリット:
  - モノレポの atomic commit 利点を完全喪失
  - 契約変更時に N 個の PR を同時マージする運用負荷
  - k1s0 の既定方針（[../../../CLAUDE.md](../../../CLAUDE.md) 参照）から根本的に外れる

## 帰結

### ポジティブな帰結

- 役割別の最小ワーキングセットによる IDE 応答性の確保
- 開発者オンボーディング時間の短縮（初回クローンが数分で完了）
- CI の path-filter と開発者環境の対称性確保（開発者が「CI が失敗するが自分のローカルでは見えない」状態を最小化）
- リポジトリ規模が採用側の運用拡大時に拡大しても運用持続
- モノレポの atomic commit 利点を維持したまま、分散リポジトリ的な局所性を獲得
- 世界トップ企業のモノレポ運用パターンに準拠

### ネガティブな帰結

- 新規参加者の初回セットアップで cone 概念の説明が必要（オンボーディングドキュメント整備必須）
- 役割切替時の stash オペレーションが必要
- `git status -u` 等の一部コマンドが期待通り動かないケース（sparse index の既知制約）
- CODEOWNERS と cone 定義の整合を維持する保守コスト（役割追加時に両方更新）

### 移行・対応事項

- `.sparse-checkout/roles/` 配下に 10 種類の役割別 cone 定義を配置（別タスク）
- `tools/sparse/checkout-role.sh` シェルスクリプトを実装（別タスク）
- Dev Container 起動時にロール選択プロンプトを出す仕組みを `tools/devcontainer/` に実装
- `docs/05_実装/00_ディレクトリ設計/80_スパースチェックアウト運用/` 配下に cone 定義 9 本の全文・切替手順・既知問題を記載
- `CLAUDE.md` の「プロジェクト構造」節でスパースチェックアウトを標準運用として明記
- CI ワークフローに sparse index 対応の git コマンド呼び出しを明示（`git sparse-checkout list` 等）
- 採用側の運用拡大時に必須化可否を再評価する条件と再評価チェックリストを [../../../docs/05_実装/00_ディレクトリ設計/80_スパースチェックアウト運用/07_partial_clone_sparse_index.md](../../../docs/05_実装/00_ディレクトリ設計/80_スパースチェックアウト運用/07_partial_clone_sparse_index.md) に記載

## 参考資料

- [ADR-DIR-001: contracts 昇格](ADR-DIR-001-contracts-elevation.md)
- [ADR-DIR-002: infra 分離](ADR-DIR-002-infra-separation.md)
- [CLAUDE.md](../../../CLAUDE.md)
- Git Sparse Checkout 公式ドキュメント: [git-scm.com/docs/git-sparse-checkout](https://git-scm.com/docs/git-sparse-checkout)
- Git Partial Clone 仕様: [git-scm.com/docs/partial-clone](https://git-scm.com/docs/partial-clone)
- Git 2.37 Sparse Index Release Notes
- "Scaling Git's garbage collection", GitHub Engineering Blog
- Potvin et al. "Why Google Stores Billions of Lines of Code in a Single Repository", CACM 2016
- Meta Sapling: [sapling-scm.com](https://sapling-scm.com)
- Microsoft Scalar: [github.com/microsoft/scalar](https://github.com/microsoft/scalar)
- Uber Go monorepo 事例: "The Go Monorepo: Lessons from 10 Years of Scale"
