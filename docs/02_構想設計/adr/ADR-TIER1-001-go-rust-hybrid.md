# ADR-TIER1-001: tier1 内部実装の Go + Rust ハイブリッド採用

- ステータス: Accepted
- 起票日: 2026-04-19
- 決定日: 2026-04-19
- 起票者: kiso ryuhei
- 関係者: システム基盤チーム / tier1 開発チーム / 運用チーム

## コンテキスト

tier1 は State/PubSub/Workflow/Decision など 11 API を Protobuf gRPC で公開するプラットフォーム層である。実装言語の選定は、以降の採用・運用・保守の生涯コストを決定する最重要判断の 1 つとなる。

制約条件は以下の通り。

- Dapr Building Blocks の成熟した Go SDK（`dapr/go-sdk`）を使いたい。State/PubSub/Secrets/Binding は Dapr に寄せることで実装量を大幅削減できる
- 一方で、ZEN Engine は Rust 製であり、JDM ルール評価のパフォーマンスと表現力を活かすには Rust ネイティブ利用が不可避
- 採用側組織の固有機能（監査ログ完整性、PII 自動検出、国内法令対応）は性能/堅牢性要件が厳しく、GC 言語よりシステム言語が向く場面がある
- 採用側の運用チームは小規模であることを前提とし、言語スタックを 2 つまでに絞ることが現実的（3 言語は運用負荷破綻）

## 決定

**tier1 内部実装は Go と Rust のハイブリッドとする。**

- **Dapr ファサード領域（State/PubSub/Secrets/Binding/Service Invoke）**: Go + `dapr/go-sdk` stable で実装
- **自作領域（Decision/Workflow/Audit/Pii/Feature/Log/Telemetry の一部）**: Rust Edition 2024 で実装
- **内部通信**: Protobuf gRPC（ADR-TIER1-002 で規定）
- **tier2/tier3 からの利用**: 内部言語は不可視、クライアント SDK と IDL のみ公開（ADR-TIER1-003 で規定）

Go と Rust の選択基準は「成熟 SDK が存在する API は Go、ネイティブ性能が必要な API は Rust」。境界は Protobuf gRPC で疎結合化するため、後年の実装言語変更（Go → Rust 逆行、あるいは Rust → Go）が段階的に可能となる。

## 検討した選択肢

### 選択肢 A: Go + Rust ハイブリッド（採用）

- 概要: 上記の通り、2 言語で役割分担
- メリット:
  - Dapr Go SDK の成熟を享受（State/PubSub で数千行の自作削減）
  - ZEN Engine の Rust ネイティブ性能を活用
  - 採用市場での Go と Rust の両方を人材プールに持てる（採用側組織での採用強化策）
  - 性能臨界領域は Rust、生産性優先領域は Go と明確に分離可能
- デメリット:
  - 2 言語のビルドシステム・CI/CD・依存管理を並行維持
  - 開発者は最低限両方の基礎を学ぶ必要（完全な専門分業にはしない）

### 選択肢 B: Go 一本化

- 概要: 全 tier1 を Go で書く
- メリット: 言語スタック単一化、運用容易、Dapr との親和性最大
- デメリット:
  - Rust 製 ZEN Engine 連携は FFI/外部プロセス呼出になり、評価レイテンシが増大
  - PII 検出/暗号化など性能臨界機能で GC 停止がリスク

### 選択肢 C: Rust 一本化

- 概要: 全 tier1 を Rust で書く
- メリット: 性能最適、メモリ安全、依存 OSS の多数が Rust 製
- デメリット:
  - Dapr の Rust SDK は experimental で stable 欠如（自作負荷が大幅増）
  - 採用・育成期間が長い（Go の 2〜3 倍）
  - リリース時点での合格基準に間に合わせるのが困難

### 選択肢 D: Go + Rust + C# ハイブリッド

- 概要: 既存 .NET 資産と合わせて 3 言語
- メリット: .NET 連携が最大限容易
- デメリット:
  - 3 言語の運用は採用側の小規模運用で破綻
  - .NET 資産は tier1 の内部実装ではなく外部連携（FR-EXT-DOTNET）で扱うため、内部言語として C# は不要

## 帰結

### ポジティブな帰結

- 実装速度と性能の両立が可能（Go で早く、Rust で堅く）
- Protobuf gRPC 境界で言語切替が将来可能（可逆性）
- Dapr コミュニティと Rust エコシステムの両方の恩恵を受けられる

### ネガティブな帰結

- CI/CD が 2 系統必要（Go 側: go test、Rust 側: cargo test）
- 言語間のエラーハンドリング流儀が異なる（Go の error return vs Rust の Result）、共通エラー体系（E-<CAT>-<MOD>-<NUM>）で橋渡しを設計必須
- 開発者に 2 言語の最低限理解を求める（新規参加者のオンボーディング 2〜4 週間増）

## 実装タスク

- Rust Edition 2024 のビルド基盤（Cargo workspace）を CI に組込み
- Go modules と Cargo の依存管理ポリシーを明文化
- tier1 内部の Protobuf IDL を一元配布（buf lint / breaking check）
- 共通エラー辞書を Go と Rust の両側で同期生成する仕組みを構築

## 参考文献

- Dapr Go SDK: github.com/dapr/go-sdk
- ZEN Engine: gorules.io
- Rust Edition 2024 Release Notes
- Go 1.22 Release Notes
