# 03. 設計 ID 体系（IMP-DIR）

本ファイルは実装フェーズのディレクトリ設計で採番する ID 体系 `IMP-DIR-<サブ>-<通番>` の命名規則と双方向トレース運用を定める。ID 体系は [../../../04_概要設計/00_設計方針/03_設計ID体系とトレーサビリティ.md](../../../04_概要設計/00_設計方針/03_設計ID体系とトレーサビリティ.md) の「実装 ID 体系」節で予約された `IMP-DIR-*` を具体化するものである。

## ID 体系が必要な理由

概要設計の設計 ID（`DS-SW-COMP-120` など）は論理レベルの配置を規定する。一方で実装フェーズでは、CODEOWNERS の path-pattern・cone 定義への追記・CI ワークフローの path-filter 設定・ArgoCD ApplicationSet の検索パスなど、物理配置に起因する複数資産の同時更新が発生する。これらを個別の章内参照のみで管理すると、ある章の配置変更が他章に波及した際に追跡が困難になる。

本ファイルで `IMP-DIR-*` を採番することにより、物理配置判断 1 件を 1 個の ID で識別し、双方向トレース（章 → 配置、配置 → 章）を維持する。ID は PR タイトル・コミットメッセージ・CHANGELOG にも前置することで、変更履歴の機械可読性を確保する。

## 書式

```
IMP-DIR-<サブ>-<通番>
```

- `IMP` : Implementation の略、固定接頭辞
- `DIR` : Directory Design、ディレクトリ設計カテゴリ
- `<サブ>` : サブ分類識別子（大文字英字 2〜7 文字）
- `<通番>` : 3 桁連番

## サブ分類と通番帯

サブ分類 8 種類を以下に予約する。通番帯は予想件数に応じて連続範囲を確保する。帯をまたいだ採番は禁止し、帯が枯渇した場合は本ファイルを改訂して帯を拡張する。

### ROOT（ルートレイアウト、001-020）

リポジトリルート直下のディレクトリ配置に関する IMP-DIR。

- `IMP-DIR-ROOT-001` : 責務境界のディレクトリ階層化原則
- `IMP-DIR-ROOT-002` : 依存方向の一方向化
- `IMP-DIR-ROOT-003` : 最大階層深度 5
- `IMP-DIR-ROOT-004` : 全主要ディレクトリに README.md
- `IMP-DIR-ROOT-005` : 生成物とソースの分離
- `IMP-DIR-ROOT-006` : 命名衝突回避
- `IMP-DIR-ROOT-007` : スパースチェックアウト cone 整合
- `IMP-DIR-ROOT-008` : ルート直下ファイル一覧（.gitattributes / CODEOWNERS / LICENSE 等）
- `IMP-DIR-ROOT-009` : src 配下の層別分割（contracts / tier1 / sdk / tier2 / tier3 / platform）
- `IMP-DIR-ROOT-010` : 横断ディレクトリ（infra / deploy / ops / tools / tests / examples / third_party）
- `IMP-DIR-ROOT-011` : 設定ファイル配置規約（.gitattributes / CODEOWNERS / LICENSE 等の運用規約）
- `IMP-DIR-ROOT-012` : 依存方向ルール（tier3 → tier2 → (sdk ← contracts) → tier1 → infra の単方向化）
- `IMP-DIR-ROOT-013` 〜 `IMP-DIR-ROOT-020` : Phase 1b 以降の予約

### T1（tier1 レイアウト、021-040）

tier1 層（`src/tier1/` + `src/contracts/` + `src/sdk/`）の配置。

- `IMP-DIR-T1-021` : tier1 全体配置（go / rust / contracts / sdk の関係）
- `IMP-DIR-T1-022` : src/contracts/ 配置（buf module 境界、tier1 / internal 分割）
- `IMP-DIR-T1-023` : src/tier1/go/ 配置（DS-SW-COMP-124 継承）
- `IMP-DIR-T1-024` : src/tier1/rust/ 配置（DS-SW-COMP-129 継承）
- `IMP-DIR-T1-025` : src/sdk/ 配置（4 言語独立）
- `IMP-DIR-T1-026` : Protobuf 生成コード配置（DS-SW-COMP-122 継承）
- `IMP-DIR-T1-027` 〜 `IMP-DIR-T1-040` : Phase 1b 以降の予約

### T2（tier2 レイアウト、041-055）

tier2 層（`src/tier2/`）の配置。

- `IMP-DIR-T2-041` : tier2 全体配置
- `IMP-DIR-T2-042` : tier2 dotnet solution 配置
- `IMP-DIR-T2-043` : tier2 go services 配置
- `IMP-DIR-T2-044` : tier2 サービス内部 Onion Architecture
- `IMP-DIR-T2-045` : tier2 テンプレート配置
- `IMP-DIR-T2-046` : tier2 依存管理（NuGet / go.mod 方針）
- `IMP-DIR-T2-047` 〜 `IMP-DIR-T2-055` : Phase 1b 以降の予約

### T3（tier3 レイアウト、056-070）

tier3 層（`src/tier3/`）の配置。

- `IMP-DIR-T3-056` : tier3 全体配置
- `IMP-DIR-T3-057` : tier3 web pnpm workspace 配置
- `IMP-DIR-T3-058` : tier3 maui native 配置
- `IMP-DIR-T3-059` : tier3 bff 配置
- `IMP-DIR-T3-060` : tier3 legacy-wrap 配置（ADR-MIG-001 連携）
- `IMP-DIR-T3-061` 〜 `IMP-DIR-T3-070` : Phase 1b 以降の予約

### INFRA（infra レイアウト、071-090）

`infra/` 配下の配置。

- `IMP-DIR-INFRA-071` : infra 全体配置
- `IMP-DIR-INFRA-072` : k8s ブートストラップ
- `IMP-DIR-INFRA-073` : サービスメッシュ配置（Istio Ambient）
- `IMP-DIR-INFRA-074` : Dapr Component 配置（旧 src/tier1/infra の移行）
- `IMP-DIR-INFRA-075` : データ層配置（CloudNativePG / Kafka / Valkey / MinIO）
- `IMP-DIR-INFRA-076` : セキュリティ層配置（Keycloak / OpenBao / SPIRE / cert-manager / Kyverno）
- `IMP-DIR-INFRA-077` : 観測性配置（LGTM + Pyroscope）
- `IMP-DIR-INFRA-078` : 環境別パッチ配置
- `IMP-DIR-INFRA-079` 〜 `IMP-DIR-INFRA-090` : Phase 1b 以降の予約

### OPS（deploy + ops、091-110）

`deploy/` + `ops/` 配下の配置。

- `IMP-DIR-OPS-091` : deploy 配置（GitOps）
- `IMP-DIR-OPS-092` : ArgoCD ApplicationSet 配置
- `IMP-DIR-OPS-093` : Helm charts 配置
- `IMP-DIR-OPS-094` : Kustomize overlays 配置
- `IMP-DIR-OPS-095` : ops 配置（Runbook / Chaos / DR）
- `IMP-DIR-OPS-096` : Backstage プラグイン配置
- `IMP-DIR-OPS-097` : OpenTofu 配置
- `IMP-DIR-OPS-098` 〜 `IMP-DIR-OPS-110` : Phase 1b 以降の予約

### COMM（共通資産、111-125）

`tools/` / `tests/` / `examples/` / `third_party/` 配下の配置。

- `IMP-DIR-COMM-111` : tools 配置
- `IMP-DIR-COMM-112` : tests 配置
- `IMP-DIR-COMM-113` : examples 配置
- `IMP-DIR-COMM-114` : third_party 配置
- `IMP-DIR-COMM-115` : devcontainer 配置
- `IMP-DIR-COMM-116` : codegen 配置
- `IMP-DIR-COMM-117` 〜 `IMP-DIR-COMM-125` : Phase 1b 以降の予約

### SPARSE（スパースチェックアウト運用、126-145）

スパースチェックアウト関連。

- `IMP-DIR-SPARSE-126` : cone mode 設計原則
- `IMP-DIR-SPARSE-127` : 役割別 cone 定義（10 ロール）
- `IMP-DIR-SPARSE-128` : 初期クローンとオンボーディング
- `IMP-DIR-SPARSE-129` : 役割切替運用
- `IMP-DIR-SPARSE-130` : CI 戦略と path_filter 統合
- `IMP-DIR-SPARSE-131` : 注意点と既知問題
- `IMP-DIR-SPARSE-132` : partial clone + sparse index
- `IMP-DIR-SPARSE-133` 〜 `IMP-DIR-SPARSE-145` : Phase 1b 以降の予約

## 通番採番ルール

通番は「カテゴリ + サブ」内で連続、欠番不可とする。ただし以下の例外を許容する。

- 帯の先頭（例: `ROOT-001`）から採番し、帯内で欠番を生じさせない
- 項目削除時は後続を詰めるが、Product Council 承認済みの ID は [../../../04_概要設計/00_設計方針/03_設計ID体系とトレーサビリティ.md](../../../04_概要設計/00_設計方針/03_設計ID体系とトレーサビリティ.md) の「廃番一覧」に準じて廃番扱いとする
- 帯の末尾で枯渇した場合、本ファイルを改訂して帯を拡張

## DS-SW-COMP / ADR との双方向トレース

各 IMP-DIR は対応する `DS-SW-COMP-*` / `ADR-*` を本文末尾「対応 DS-SW-COMP / ADR / 要件」節に明記する。以下の対応が典型。

- `IMP-DIR-T1-022`（src/contracts/ 配置） → `DS-SW-COMP-121`（改訂後）/ `ADR-DIR-001`
- `IMP-DIR-INFRA-074`（Dapr Component 配置） → `DS-SW-COMP-120`（改訂後）/ `ADR-DIR-002`
- `IMP-DIR-SPARSE-126` 〜 `132` → `ADR-DIR-003`

逆方向のトレースは [../../90_トレーサビリティ/01_IMP-DIR_ID一覧.md](../../90_トレーサビリティ/01_IMP-DIR_ID一覧.md) で一覧化する。

### 紐付け多重度ルール

IMP-DIR と DS-SW-COMP / ADR の紐付けは **1 : n** および **n : 1** を許容する。

- **1 IMP-DIR : n DS-SW-COMP**: 例えば `IMP-DIR-T1-026`（Protobuf 生成コード配置）は DS-SW-COMP-121 / 122 / 126 / 132 の 4 つに関連。生成コードの論理位置、物理配置、契約源、2 系統並行生成の各観点に跨るため
- **n IMP-DIR : 1 DS-SW-COMP**: 例えば DS-SW-COMP-120（旧 src/tier1/infra/ 廃止）は `IMP-DIR-INFRA-071` / `INFRA-074` / `OPS-091` / `OPS-095` の 4 つに分解。infra / deploy / ops の 3 階層分離の結果として複数物理配置に写像されるため
- **1 IMP-DIR : n ADR** も同様に許容。例えば `IMP-DIR-T1-022` は ADR-DIR-001（contracts 昇格）と ADR-TIER1-002（Protobuf 採用）の両方に関連

運用ルール:

- 対応する DS-SW-COMP / ADR は **すべて** 本文末尾に列挙する（省略禁止）
- 関連度（直接 / 間接）を明記する（[../../90_トレーサビリティ/03_ADR_との対応.md](../../90_トレーサビリティ/03_ADR_との対応.md) の「関連度」列準拠）
- 1 : n / n : 1 関係の逆引きは 90_トレーサビリティ/ 配下のマトリクス表で保証する

この多重度ルールにより、物理配置（IMP-DIR）と論理配置（DS-SW-COMP）の非 1:1 対応を前提とした設計変更追跡が可能になる。

## フェーズ属性

Phase 0 稟議承認時点で以下が確定する。

- `IMP-DIR-ROOT-001` 〜 `IMP-DIR-ROOT-012` の 12 件
- `IMP-DIR-T1-021` 〜 `IMP-DIR-T1-026` の 6 件
- `IMP-DIR-T2-041` 〜 `IMP-DIR-T2-046` の 6 件
- `IMP-DIR-T3-056` 〜 `IMP-DIR-T3-060` の 5 件
- `IMP-DIR-INFRA-071` 〜 `IMP-DIR-INFRA-078` の 8 件
- `IMP-DIR-OPS-091` 〜 `IMP-DIR-OPS-097` の 7 件
- `IMP-DIR-COMM-111` 〜 `IMP-DIR-COMM-116` の 6 件
- `IMP-DIR-SPARSE-126` 〜 `IMP-DIR-SPARSE-132` の 7 件

合計 57 件。残りの帯（各サブ分類の予約番号）は Phase 1b / Phase 1c / Phase 2 で追加採番する。

## 参照表記

`IMP-DIR-*` を本文中で参照する際は、ID のみ、または ID + 相対リンクを併記する。設計 ID の参照表記規約（[../../../04_概要設計/00_設計方針/03_設計ID体系とトレーサビリティ.md](../../../04_概要設計/00_設計方針/03_設計ID体系とトレーサビリティ.md) の「参照表記」節）と同一ルールを適用する。

## 対応 ADR

- ADR-DIR-001 / ADR-DIR-002 / ADR-DIR-003

## 対応概要設計

- [../../../04_概要設計/00_設計方針/03_設計ID体系とトレーサビリティ.md](../../../04_概要設計/00_設計方針/03_設計ID体系とトレーサビリティ.md) の「実装 ID 体系」節
