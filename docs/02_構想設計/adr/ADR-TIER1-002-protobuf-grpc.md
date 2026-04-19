# ADR-TIER1-002: tier1 内部通信に Protobuf gRPC を採用

- ステータス: Accepted
- 起票日: 2026-04-19
- 決定日: 2026-04-19
- 起票者: kiso ryuhei
- 関係者: システム基盤チーム / tier1 開発チーム / tier2 リードエンジニア

## コンテキスト

tier1 は Go と Rust のハイブリッド（ADR-TIER1-001）で実装され、内部の複数コンポーネント間（Decision Service / Workflow Service / Audit Service / Pii Service 等）で頻繁に RPC を行う。これらの内部通信の形式をどう設計するかで、言語境界のコスト・バージョン管理・可観測性の難易度が決定する。

選定の論点は以下の通り。

- **言語相互運用性**: Go ↔ Rust の型安全な通信が必須
- **スキーマ進化**: 後方互換性を保ったまま新フィールドを追加できること
- **性能**: tier1 全体で p99 < 500ms の予算を守る（内部 RPC は 5〜30ms / hop 程度に抑えたい）
- **ツールチェーン成熟度**: IDL から Go/Rust 双方のクライアント/サーバコードを自動生成可能
- **運用観測性**: gRPC interceptor でログ/メトリクス/トレースを一貫注入できること

JSON over HTTP/REST はシンプルだが、型安全性とスキーマ進化管理が手作業に寄り、Go/Rust の両方で中途半端な型検証になる。CBOR・MessagePack 等のバイナリ JSON も候補だが、IDL 文化が弱い。

## 決定

**tier1 内部通信は Protocol Buffers 3 + gRPC を採用する。**

- IDL は `.proto` ファイルで一元管理、`proto/k1s0/tier1/<api>/v<N>/` のパッケージ階層
- 生成は `buf generate` を CI/CD に組込み、Go (`protoc-gen-go-grpc`) と Rust (`tonic-build`) の両方でクライアント/サーバを自動生成
- スキーマ互換性は `buf breaking` で強制、MAJOR バージョン変更（破壊的変更）は OR-EOL-001 の非推奨ライフサイクル準拠
- インターセプタで OpenTelemetry Trace 自動伝搬、構造化ログ、SLI メトリクス発行を一元化
- tier2/tier3 が tier1 を呼ぶ外向き API も同じ IDL で提供（ADR-TIER1-003）

内部通信は常に mTLS、SPIFFE/SPIRE によるワークロード ID（ADR-SEC-003）でピアを識別する。

## 検討した選択肢

### 選択肢 A: Protobuf 3 + gRPC（採用）

- 概要: 業界標準の RPC 方式
- メリット:
  - Go/Rust 双方で成熟したツールチェーン（tonic は Rust gRPC の実質標準）
  - buf によるスキーマ管理が強固（lint、breaking check、format 統一）
  - OpenTelemetry/Prometheus の gRPC interceptor が豊富
  - Dapr Building Blocks も gRPC API を提供しており親和性が高い
- デメリット:
  - バイナリなので curl でのデバッグが困難（grpcurl 等のツール習熟が必要）
  - IDL 管理のオーバーヘッド（PR で .proto レビューが増える）

### 選択肢 B: JSON over HTTP/REST + OpenAPI

- 概要: RESTful API と OpenAPI 定義
- メリット: デバッグ容易（curl）、ブラウザ確認容易
- デメリット:
  - Go/Rust 双方で型生成の成熟度が gRPC に劣る
  - 内部 RPC のオーバーヘッドが大（JSON encode/decode コスト）
  - スキーマ進化の強制が弱く、運用で逸脱しやすい

### 選択肢 C: gRPC-Web / Connect RPC

- 概要: gRPC をブラウザ対応した派生
- メリット: フロントエンドからも利用可能
- デメリット:
  - 内部 RPC では不要なオーバーヘッド
  - tier1 内部通信は完全にサーバ間のため、ブラウザ対応は不要

### 選択肢 D: MessagePack RPC / CBOR RPC

- 概要: バイナリ JSON 系の軽量 RPC
- メリット: フォーマットが単純、学習曲線緩やか
- デメリット:
  - IDL 文化が弱く、スキーマ進化管理が手作業
  - Rust/Go の成熟ライブラリが gRPC に比べ脆弱

## 帰結

### ポジティブな帰結

- 言語境界（Go↔Rust）の型安全性確保
- buf の breaking check で後方互換性違反を PR で検出
- OpenTelemetry 自動計装で観測性統一
- 公開 API と内部 API が同じ IDL 体系で統一（学習コスト最小化）

### ネガティブな帰結

- IDL 変更時の buf review 工数（小規模変更でも PR が発生）
- デバッグツール（grpcurl）の習熟が運用チームに必要
- HTTP トレース解析ツール（Wireshark 等）では gRPC binary をそのまま追えない

## 実装タスク

- `buf.yaml` / `buf.gen.yaml` / `buf.lock` をモノレポ直下に配置
- CI で `buf lint` + `buf breaking` を必須チェック化
- Go 側生成: `protoc-gen-go-grpc` v1.5+
- Rust 側生成: `tonic-build` 0.12+
- gRPC Server/Client Interceptor を Go/Rust 双方で共通ロジック化（テナント伝搬、OTel、認可）

## 参考文献

- Protocol Buffers 仕様: protobuf.dev
- gRPC 仕様: grpc.io
- Buf: buf.build/docs
- Tonic (Rust gRPC): github.com/hyperium/tonic
