# ADR-DIR-001: Protobuf 契約ディレクトリを `src/contracts/` に昇格

- ステータス: Accepted
- 起票日: 2026-04-23
- 決定日: 2026-04-23
- 起票者: kiso ryuhei
- 関係者: システム基盤チーム / tier1 開発チーム / tier2 開発チーム / tier3 開発チーム / SDK 開発チーム / 契約レビュー担当

## コンテキスト

概要設計の DS-SW-COMP-120 / DS-SW-COMP-121（[../../04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/06_パッケージ構成_Rust_Go.md](../../04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/06_パッケージ構成_Rust_Go.md) 参照）は、Protobuf 契約ファイルを `src/tier1/contracts/` に配置する方針を確定している。これは tier1 内部の Go ↔ Rust 間 gRPC 通信と、tier1 公開 11 API の両方を同じ `.proto` 配下で管理する前提である。

しかし実装フェーズのディレクトリ設計を全体俯瞰した結果、以下の構造的な矛盾が浮上した。

- 契約（`.proto` ファイル）は tier1 所有物ではなく、**tier1・tier2・tier3・SDK の 4 層横断で参照される共有資産**である。tier2 / tier3 が独自のドメイン Protobuf を持つケース（例: tier2 の業務 API、tier3 BFF の Schema First 開発）では、`src/tier1/contracts/tier2/` のような tier1 配下へのぶら下げはパス意味論が破綻する。
- CODEOWNERS 設計で契約変更のレビュー担当を `契約レビュー担当` として tier1 チームから分離する方針を取る場合、`src/tier1/contracts/` のパスは CODEOWNERS の `path-pattern` と意味論が合わない（tier1 配下なのに所有権は tier1 ではない）。
- SDK（C# / Go / TypeScript / Rust）は tier1 実装と独立に発展する前提（本 ADR と同時起票の [ADR-DIR-002](ADR-DIR-002-infra-separation.md)）で、SDK が tier1 配下の contracts を参照するのは依存方向の逆転リスクを生む。
- スパースチェックアウト運用（本 ADR と同時起票の [ADR-DIR-003](ADR-DIR-003-sparse-checkout-cone-mode.md)）で、`tier2-dev` / `tier3-web-dev` / `sdk-dotnet-dev` 等の役割 cone が契約ファイルを引く際、`src/tier1/contracts/` を含めると tier1 の不要なサブツリーを巻き込む可能性があり、cone 定義が複雑化する。
- buf module 境界を `src/contracts/` 直下で切れば、契約変更 PR を契約レビュー担当が独立レビューでき、tier1 実装の変更とレビュー経路を分離できる。

本 ADR 起票時点で実装コードはまだ書かれていない（Phase 0 稟議承認待ち）ため、後方互換を気にせず最適な配置に昇格できる最後のタイミングである。実装が進んだ後に昇格する場合、Go の import path / Rust の `build.rs` パス / CI の path-filter / ArgoCD ApplicationSet の検索パスが全て影響を受けるため、改修コストが数十倍に膨らむ。

## 決定

**Protobuf 契約ディレクトリを `src/tier1/contracts/` から `src/contracts/` に昇格する。**

改訂後の構成は以下とする。

```
src/
├── contracts/                   # ★昇格先: 契約の単一の真実
│   ├── buf.yaml
│   ├── buf.gen.yaml
│   ├── buf.lock
│   ├── tier1/
│   │   └── v1/                  # tier1 公開 11 API
│   │       ├── state.proto
│   │       ├── pubsub.proto
│   │       ├── serviceinvoke.proto
│   │       ├── secrets.proto
│   │       ├── binding.proto
│   │       ├── workflow.proto
│   │       ├── log.proto
│   │       ├── telemetry.proto
│   │       ├── decision.proto
│   │       ├── audit.proto
│   │       └── feature.proto
│   └── internal/
│       └── v1/                  # tier1 内部 gRPC（facade → rust）
│           ├── common.proto
│           ├── errors.proto
│           └── pii.proto
├── tier1/
│   ├── go/                      # DS-SW-COMP-124 継承、proto import 先のみ変更
│   └── rust/                    # DS-SW-COMP-129 継承、proto-gen の参照先のみ変更
├── sdk/                         # ★新設: src/contracts/tier1/v1/ を参照
│   ├── dotnet/
│   ├── go/
│   ├── typescript/
│   └── rust/
├── tier2/
└── tier3/
```

buf module 境界は `src/contracts/` 直下に置く。CODEOWNERS は `src/contracts/` に対して `契約レビュー担当` を割り当て、tier1 / tier2 / tier3 / SDK の各チームは契約変更 PR に対してレビュー権限を持つが承認権限は契約レビュー担当に委ねる。

## 検討した選択肢

### 選択肢 A: `src/contracts/` に昇格（採用）

- 概要: 契約ディレクトリをルート `src/` 直下の独立ディレクトリに昇格
- メリット:
  - 契約は tier1 / tier2 / tier3 / SDK の 4 層横断資産であることをディレクトリ階層で明示できる
  - CODEOWNERS で契約レビュー担当を明確に分離できる
  - buf module 境界が `src/contracts/` で自然に切れる
  - スパースチェックアウト cone 定義が `src/contracts/` と `src/tier1/` を独立に選択できる
  - SDK が tier1 実装に依存する見かけ上の逆転を排除
  - 将来 tier2 / tier3 が独自 Protobuf を持つ際に `src/contracts/tier2/v1/` として統一配置可能
- デメリット:
  - 概要設計 DS-SW-COMP-120 / 121 の改訂が必要
  - 既存の検討成果物（[../../04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/06_パッケージ構成_Rust_Go.md](../../04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/06_パッケージ構成_Rust_Go.md)）との齟齬を明示的に管理する必要がある

### 選択肢 B: `src/tier1/contracts/` を維持

- 概要: 既存 DS-SW-COMP-121 を変更せず、tier1 配下に contracts を置く
- メリット:
  - 概要設計の改訂が不要
  - Phase 1a 時点では tier1 公開 11 API のみなので、tier1 所有物とみなしても破綻しない
- デメリット:
  - Phase 1b 以降で tier2 / tier3 が独自 Protobuf を持った際に、`src/tier1/contracts/tier2/` という矛盾した階層が生まれる
  - CODEOWNERS で契約レビュー担当と tier1 チームの責任分界が不明瞭になる
  - SDK が tier1 配下の契約を import する形になり、依存方向の見かけ上の逆転が発生
  - スパースチェックアウト cone で tier2 / tier3 / SDK 開発者が tier1 のサブツリーを巻き込む

### 選択肢 C: 独立リポジトリ化（`k1s0-contracts` 別 repo）

- 概要: 契約を別 Git リポジトリに分離し、各 tier から BSR 経由で参照
- メリット:
  - 契約レビュー経路が完全に独立
  - 契約の独立バージョニングが容易
- デメリット:
  - モノレポ方針（[../../../CLAUDE.md](../../../CLAUDE.md) 参照）から外れる
  - 契約変更と実装変更の atomic commit が不可能になり、breaking change 管理が複雑化
  - 2 名運用（NFR-C-NOP-001）で別 repo の CI/CD を維持する負荷が過大
  - Phase 0 稟議承認前に別 repo を用意する意思決定コストが重い

### 選択肢 D: `contracts/` をリポジトリルート直下（`src/` 外）に配置

- 概要: `src/` の外に `contracts/` を置き、コード（`src/`）と契約（`contracts/`）を完全分離
- メリット:
  - 契約がソースコードとは別の意味を持つことを最上位で明示できる
  - OpenAPI Initiative 系の事例（Spring Cloud Contract 等）に近い
- デメリット:
  - ソースコードの一次ディレクトリ `src/` という既定方針（[../../../CLAUDE.md](../../../CLAUDE.md) 参照）から外れる
  - `infra/` `deploy/` `ops/` `tools/` 等のルート直下ディレクトリと並列になり、ルート階層が膨張
  - 契約も実装もコード生成対象であり、`src/` 配下にまとめる方が buf generate の出力配置と自然に一致

## 帰結

### ポジティブな帰結

- 契約・tier1 実装・SDK・tier2 / tier3 の 4 層が依存方向の見かけ上も論理上も一方向化する
- CODEOWNERS による契約レビューの責任分界が path-pattern と 1:1 対応する
- buf module 境界と buf.yaml の配置が自然な位置に収まる
- スパースチェックアウトで契約・tier1・SDK をそれぞれ独立に選択でき、役割 cone の粒度を最適化できる
- 将来 tier2 / tier3 が独自 Protobuf を持つ際の拡張経路が確保される

### ネガティブな帰結

- DS-SW-COMP-120 / DS-SW-COMP-121 / DS-SW-COMP-122 / DS-SW-COMP-132 の改訂作業が発生する
- tier1 Go の `internal/proto/v1/` 生成先は維持するが、生成元の `.proto` パスが `src/tier1/contracts/v1/*.proto` から `src/contracts/tier1/v1/*.proto` に変わるため、buf.gen.yaml の `input:` 指定を更新する必要がある
- tier1 Rust の `proto-gen` crate の `build.rs` の `.proto` 検索パスが変わる

### 移行・対応事項

- [../../04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/06_パッケージ構成_Rust_Go.md](../../04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/06_パッケージ構成_Rust_Go.md) の DS-SW-COMP-120 / 121 / 122 / 132 を改訂し、`src/tier1/contracts/` への言及を `src/contracts/` に置換する
- 改訂履歴に「ADR-DIR-001 起票により昇格」と明記する
- 実装フェーズのディレクトリ設計書（`docs/05_実装/00_ディレクトリ設計/20_tier1レイアウト/02_contracts配置.md`）で本 ADR を参照する
- CODEOWNERS サンプル（`docs/05_実装/00_ディレクトリ設計/00_設計方針/06_CODEOWNERSマトリクス設計.md`）で `src/contracts/` を契約レビュー担当に割り当てる
- buf.yaml / buf.gen.yaml の `input:` / `output:` パスを新構造に合わせる雛形を実装ドキュメントに収録
- IMP-DIR-T1-\*（contracts 配置）と IMP-DIR-ROOT-\*（依存方向ルール）との双方向トレースを [../../../docs/05_実装/00_ディレクトリ設計/90_トレーサビリティ/02_DS-SW-COMP_121-135_との対応.md](../../../docs/05_実装/00_ディレクトリ設計/90_トレーサビリティ/02_DS-SW-COMP_121-135_との対応.md) で管理

## 参考資料

- [ADR-TIER1-002: Protobuf / gRPC 採用](ADR-TIER1-002-protobuf-grpc.md)
- [ADR-TIER1-003: tier2/tier3 からの内部言語不可視](ADR-TIER1-003-language-opacity.md)
- [ADR-DIR-002: infra ディレクトリ分離](ADR-DIR-002-infra-separation.md)
- [ADR-DIR-003: スパースチェックアウト cone mode 採用](ADR-DIR-003-sparse-checkout-cone-mode.md)
- [DS-SW-COMP-120 / 121 / 122 / 132](../../04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/06_パッケージ構成_Rust_Go.md)
- [CLAUDE.md](../../../CLAUDE.md)
- Google Piper monorepo 論文: Potvin et al. "Why Google Stores Billions of Lines of Code in a Single Repository", CACM 2016
- Meta Buck2 build system: [buck2.build](https://buck2.build)
- AWS SDK for Rust monorepo 構成: [awslabs/aws-sdk-rust](https://github.com/awslabs/aws-sdk-rust)
