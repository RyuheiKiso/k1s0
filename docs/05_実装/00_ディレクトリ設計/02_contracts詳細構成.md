# 02. contracts 詳細構成

本ファイルは `src/tier1/contracts/` 配下の詳細ファイル配置を方式として固定化する。上流は概要設計 [DS-SW-COMP-121（contracts/ の内部構成）](../../04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/06_パッケージ構成_Rust_Go.md)・[DS-SW-COMP-122（Protobuf 生成コード配置）](../../04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/06_パッケージ構成_Rust_Go.md)・[DS-SW-COMP-123（buf 設定方針）](../../04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/06_パッケージ構成_Rust_Go.md)・[DS-SW-COMP-115（Protobuf 破壊的変更検査）](../../04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/05_モジュール依存関係.md)で、本ファイルはそれらを個別 `.proto` ファイル・`buf` 設定・生成出力先のレベルまで分解する。

## 本ファイルの位置付け

Protobuf 契約は tier1 の**単一の真実（single source of truth）**である。全 6 Pod が同じ `.proto` を入力として Go / Rust のコードを生成し、内部 gRPC の型・エラー・metadata の全てを `.proto` 経由で共有する。ここが曖昧だと、例えば「STATE Pod が新しい error code を追加したが DECISION Pod はまだ古い生成コードを使っている」という状況が発生し、runtime で `Unknown error code` が頻発する。本ファイルは `.proto` のファイル分割・命名・パッケージ階層・生成コード出力先を厳密に固定化することで、上記の乖離を構造的に排除する役割を担う。

`.proto` の配置は一度決めると変更コストが極めて高い。`package` 宣言の変更は全 Go / Rust コードの import 書き換えを要し、ファイル移動は `buf breaking` で後方互換性違反として検出される。よって Phase 0 〜 1a 境界で本ファイルが確定していることが、Phase 1a 以降の実装速度を決定的に支配する。

## 設計 ID 一覧と採番方針

本ファイルで採番する設計 ID は `DS-IMPL-DIR-021` 〜 `DS-IMPL-DIR-040` の 20 件である。

## contracts/ の全体構造

`src/tier1/contracts/` の完全な構造は以下のとおり。概要設計では `v1/` 直下に 11 個の service proto を並列配置するとしたが、本ファイルではそれを **共通型・エラー型・サービス別プロト**の 3 分類に整理する。

```
src/tier1/contracts/
├── README.md                          # contracts 概要と buf generate 手順
├── buf.yaml                           # buf 設定（lint / breaking）
├── buf.gen.yaml                       # buf generate 設定（Go / Rust 生成）
├── buf.lock                           # buf 依存 lock
├── buf.md                             # Buf Schema Registry 用文書（Phase 1c から）
└── v1/
    ├── common/                        # 共通型（全 API 横断）
    │   ├── common.proto               # 基本型（TenantID / TraceContext / Timestamp）
    │   ├── metadata.proto             # gRPC metadata キー定義
    │   └── pagination.proto           # 一覧 API のページング共通型
    ├── errors/                        # エラー型（全 API 横断）
    │   ├── errors.proto               # 共通エラー型（google.rpc.Status 拡張）
    │   └── error_codes.proto          # エラーコード enum 定義
    ├── service_invoke/
    │   └── v1.proto                   # COMP-T1-STATE 隣接、Service Invoke API
    ├── state/
    │   └── v1.proto                   # COMP-T1-STATE、State API
    ├── pubsub/
    │   └── v1.proto                   # COMP-T1-STATE 同 Pod、PubSub API
    ├── binding/
    │   └── v1.proto                   # COMP-T1-STATE 同 Pod、Binding API
    ├── secrets/
    │   └── v1.proto                   # COMP-T1-SECRET、Secrets API
    ├── workflow/
    │   └── v1.proto                   # COMP-T1-WORKFLOW、Workflow API
    ├── log/
    │   └── v1.proto                   # COMP-T1-AUDIT、Log API
    ├── telemetry/
    │   └── v1.proto                   # COMP-T1-AUDIT、Telemetry API
    ├── decision/
    │   └── v1.proto                   # COMP-T1-DECISION、Decision API
    ├── audit/
    │   └── v1.proto                   # COMP-T1-AUDIT、Audit API（監査ログ）
    ├── pii/
    │   └── v1.proto                   # COMP-T1-PII、PII API（マスキング）
    └── feature/
        └── v1.proto                   # Feature API（flag 評価）
```

概要設計との差分は 2 点。第 1 に、概要設計では `v1/state.proto` と平坦配置だったのを、本ファイルでは `v1/state/v1.proto` のサブディレクトリ配置に変更する。これにより Phase 2 で `v2` を追加する際に `v1/state/v2.proto` と並列に増やすだけで済み、ファイル移動を要さない。第 2 に、`common/` と `errors/` のサブディレクトリを明示的に採番し、全 API 横断で共有する型の置き場を固定する。

### DS-IMPL-DIR-021 contracts/ ルートの必須ファイル 5 種

`src/tier1/contracts/` 直下に置くファイルは `README.md` / `buf.yaml` / `buf.gen.yaml` / `buf.lock` / `buf.md` の 5 種類のみを許可する。それ以外（例: 個別 `.proto` ファイル）を root に置くことは禁止する。理由は `.proto` を root に置くと `v1/` 階層との責務が混ざり、「これは tier1 全体の contract か、それとも特定 service 限定か」が不明瞭になるためである。

**確定フェーズ**: Phase 1a。**対応要件**: DX-CICD-\*。**上流**: DS-SW-COMP-121。

### DS-IMPL-DIR-022 v1/ 配下の API 別サブディレクトリ化

`v1/` 直下には 11 個の API（`service_invoke/` / `state/` / `pubsub/` / `binding/` / `secrets/` / `workflow/` / `log/` / `telemetry/` / `decision/` / `audit/` / `pii/` / `feature/` の 12 個で、`Audit-Pii` は Audit と PII の 2 API のため 11 ではなく 12 個）ごとのサブディレクトリを配置する。各サブディレクトリ直下は `v1.proto` ファイル 1 個を置くことを原則とし、ファイル分割が必要な場合（300 行を超える / 論理的にまとまりが異なる）は `v1.proto` に加えて `v1_types.proto`（型定義）や `v1_errors.proto`（API 固有エラー）を同階層に追加する。サブディレクトリ名は snake_case で、`service_invoke` のようにアンダースコアで区切る（Protobuf の package 名との対応を取るため）。

**確定フェーズ**: Phase 1a。**対応要件**: DX-CICD-\*、NFR-C-NOP-002。**上流**: DS-SW-COMP-121、DS-SW-COMP-107。

### DS-IMPL-DIR-023 common/ と errors/ の共通型配置

`v1/common/` と `v1/errors/` は全 API 横断の共有型を置く。`common/common.proto` には `TenantID` / `UserID` / `TraceContext` / `Timestamp`（google.protobuf.Timestamp を wrap したもの）を、`common/metadata.proto` には gRPC metadata のキー名定数（`x-k1s0-tenant-id` など）を、`common/pagination.proto` には `PageRequest` / `PageResponse` を定義する。`errors/errors.proto` には `google.rpc.Status` を継承した k1s0 固有 Error 型を、`errors/error_codes.proto` には エラーコード enum（`OK = 0`、`INVALID_ARGUMENT = 3`、`PERMISSION_DENIED = 7`、`K1S0_TENANT_ISOLATION_VIOLATION = 100` のように google.rpc.Code を拡張）を定義する。共通型を個別 API の `.proto` に内包せず `common/` に切り出す理由は、例えば `State.Get` の返値と `Decision.Evaluate` の返値が同じ `TenantID` 型を使う必要があり、API 間で型定義が分散すると import 先で型衝突が発生するためである。

**確定フェーズ**: Phase 1a。**対応要件**: DX-CICD-\*、NFR-H-INT-\*。**上流**: DS-SW-COMP-107、DS-SW-COMP-108。

### DS-IMPL-DIR-024 Protobuf package 命名規約

各 `.proto` の `package` 宣言は `k1s0.<api_name>.v1` 形式とする。例えば `v1/state/v1.proto` は `package k1s0.state.v1;`、`v1/common/common.proto` は `package k1s0.common.v1;`、`v1/errors/errors.proto` は `package k1s0.errors.v1;` となる。`go_package` オプションは `github.com/k1s0/k1s0/src/tier1/go/internal/shared/proto/v1/<api>;<api>v1` 形式で、例えば State の場合は `option go_package = "github.com/k1s0/k1s0/src/tier1/go/internal/shared/proto/v1/state;statev1";`。Rust 側は `prost-build` が自動的に `k1s0.state.v1` から `k1s0::state::v1` モジュールを生成する。package 名と go_package は `buf lint` で STANDARD ルールにより強制検証する。

**確定フェーズ**: Phase 1a。**対応要件**: DX-CICD-\*、NFR-C-NOP-002。**上流**: DS-SW-COMP-123。

## buf 設定

### DS-IMPL-DIR-025 buf.yaml の内容

`buf.yaml` は `version: v2` で以下の設定を行う。

```yaml
version: v2
modules:
  - path: .
lint:
  use:
    - STANDARD
  except:
    - PACKAGE_VERSION_SUFFIX   # v1.proto を許可（v1/state/v1.proto 形式）
breaking:
  use:
    - FILE
deps:
  - buf.build/googleapis/googleapis
```

`lint` の `PACKAGE_VERSION_SUFFIX` を例外にするのは、ファイル名 `v1.proto` が STANDARD 規則では拒否されるためである（通常は `state_service.proto` のように内容を表す名前を要求する）。本章では `v1/<api>/v1.proto` の形で version を最下層ディレクトリ名に寄せることで Phase 2 の `v2.proto` 追加を容易にしているため、この例外を明示的に許可する。`breaking: use: [FILE]` は概要設計と同一で、`.proto` ファイル単位で後方互換性を検査する。

**確定フェーズ**: Phase 1a。**対応要件**: DX-CICD-\*、ADR-TIER1-002。**上流**: DS-SW-COMP-123、DS-SW-COMP-115。

### DS-IMPL-DIR-026 buf.gen.yaml の内容

`buf.gen.yaml` は Go と Rust の両言語コード生成を宣言する。

```yaml
version: v2
plugins:
  # Go 生成（03 章 DS-IMPL-DIR-042 で確定した shared/proto/ に出力）
  - remote: buf.build/protocolbuffers/go
    out: ../go/internal/shared/proto
    opt:
      - paths=source_relative
  - remote: buf.build/grpc/go
    out: ../go/internal/shared/proto
    opt:
      - paths=source_relative
  # Rust 生成（04 章 DS-IMPL-DIR-083 で確定した shared/proto-gen/ crate に出力。Phase 1a は build.rs 方式も併用し、CI で drift 検出）
  - remote: buf.build/community/neoeinstein-prost
    out: ../rust/crates/shared/proto-gen/src/generated
    opt:
      - no_include
  - remote: buf.build/community/neoeinstein-tonic
    out: ../rust/crates/shared/proto-gen/src/generated
    opt:
      - no_include
      - no_server   # Rust crate は client と server 両方使うが Dapr facade 呼び出し側として client を主用
```

生成先の相対パス（`../go/internal/shared/proto` / `../rust/crates/shared/proto-gen/src/generated`）は `contracts/` から見た相対で、`src/tier1/contracts/` から `src/tier1/go/internal/shared/proto/` / `src/tier1/rust/crates/shared/proto-gen/src/generated/` を指す。Phase 1a の初期段階では `../go/internal/proto` / `../rust/crates/proto-gen/...` を使っていたが、03 / 04 章の `shared/` 深化に合わせて本書で `shared/` 下に移した（このパス移行は `buf.gen.yaml` と `internal/shared/proto/` / `crates/shared/proto-gen/` 側のディレクトリ作成を同 PR で行い、旧パスは完全削除する）。生成コードは git 管理（DS-SW-COMP-122 準拠）し、CI で `buf generate` 結果と commit 内容の差分を検出する。差分があれば PR を fail させ、開発者に `buf generate` の再実行を求める。

**確定フェーズ**: Phase 1a。**対応要件**: DX-CICD-\*、ADR-TIER1-002。**上流**: DS-SW-COMP-122、DS-SW-COMP-123。

### DS-IMPL-DIR-027 buf.lock の扱い

`buf.lock` は `buf dep update` で生成し、BSR（Buf Schema Registry）の外部依存（主に `googleapis`）のバージョンを固定する。`buf.lock` は git 管理し、開発者が手動編集することは禁止する（必ず `buf dep update` 経由）。依存更新は Renovate（Phase 1b 以降）で自動 PR 化し、`buf breaking` で後方互換性を確認した上でマージする。

**確定フェーズ**: Phase 1a。**対応要件**: DX-CICD-\*、NFR-SUP-\*。**上流**: DS-SW-COMP-123、DS-SW-COMP-119。

### DS-IMPL-DIR-028 buf.md の Phase 1c 導入

`buf.md` は BSR に contracts を publish する際の module 概要ドキュメントである。Phase 1a〜1b では BSR publish を行わないため不要だが、Phase 1c で tier2 / tier3 が contracts を取得する経路として BSR を採用する場合に配置する。Phase 1c 時点で BSR publish しない判断となっても、contracts の「この module は何か」を簡潔に説明する文書として残す価値があるため、空ファイルではなく 50 行程度の解説文書を Phase 1a で書き始めておく（`buf.md` が Phase 1a から存在すれば、開発者が contracts を読む時の入口として機能する）。

**確定フェーズ**: Phase 1a（雛形）、Phase 1c（BSR publish 判断）。**対応要件**: DX-BS-\*、ADR-TIER1-002。**上流**: DS-SW-COMP-115。

## 生成コードの配置

### DS-IMPL-DIR-029 Go 側生成コードの配置先

Go 側生成コードは `src/tier1/go/internal/shared/proto/v1/<api>/` 配下に `<api>.pb.go` と `<api>_grpc.pb.go` の 2 ファイルを生成する。例えば State の場合は `src/tier1/go/internal/shared/proto/v1/state/state.pb.go` と `state_grpc.pb.go`。`paths=source_relative` オプションにより、`.proto` 側のディレクトリ構造（`v1/state/`）が生成コード側にも維持される。生成コードは全て `// Code generated by protoc-gen-go. DO NOT EDIT.` のヘッダを持ち、`_test.go` の生成は禁止（test は `internal/shared/proto/` 直下に `proto_test.go` を置かず、生成コードの振る舞いは各 Pod の `internal/pods/<pod>/handler/` でテストする）。

**確定フェーズ**: Phase 1a。**対応要件**: DX-CICD-\*。**上流**: DS-SW-COMP-122、DS-SW-COMP-107。

### DS-IMPL-DIR-030 Rust 側生成コードの配置先

Rust 側生成コードは `src/tier1/rust/crates/shared/proto-gen/src/generated/` 配下に `k1s0.state.v1.rs` / `k1s0.common.v1.rs` などの flat なファイルとして生成される（prost の仕様）。`proto-gen/src/lib.rs` で `pub mod state { include!("generated/k1s0.state.v1.rs"); }` のように手書きで include する。この手書き部分は Phase 1a 時点では約 15 行（11 API + common + errors）で、`.proto` の追加時に合わせて手動更新する。将来的に生成できるよう [tonic-build の `FileDescriptorSet` 機能](https://docs.rs/tonic-build/latest/tonic_build/) を Phase 1b で評価する。

**確定フェーズ**: Phase 1a。**対応要件**: DX-CICD-\*、NFR-C-NOP-002。**上流**: DS-SW-COMP-122、DS-SW-COMP-132。

### DS-IMPL-DIR-031 proto-gen crate の完全構造

Rust 側 `proto-gen` crate の完全な内部構造は以下のとおり。概要設計 DS-SW-COMP-132 で `build.rs` 方式と pre-generated 方式の両方を用意するとしたため、本ファイルでは両方式を共存させる配置を採用する。

```
src/tier1/rust/crates/shared/proto-gen/
├── Cargo.toml
├── build.rs                          # tonic-build 実行（pre-generated の検証用）
├── README.md                         # buf と build.rs の 2 経路の説明
└── src/
    ├── lib.rs                        # pub mod state / pub mod decision / ... の手動 include
    └── generated/                    # buf generate の出力（git 管理）
        ├── k1s0.common.v1.rs
        ├── k1s0.errors.v1.rs
        ├── k1s0.state.v1.rs
        ├── k1s0.pubsub.v1.rs
        ├── k1s0.binding.v1.rs
        ├── k1s0.service_invoke.v1.rs
        ├── k1s0.secrets.v1.rs
        ├── k1s0.workflow.v1.rs
        ├── k1s0.log.v1.rs
        ├── k1s0.telemetry.v1.rs
        ├── k1s0.decision.v1.rs
        ├── k1s0.audit.v1.rs
        ├── k1s0.pii.v1.rs
        └── k1s0.feature.v1.rs
```

`build.rs` は `tonic-build` を呼び出して `OUT_DIR` に生成し、CI が `diff $OUT_DIR $src/generated` で drift を検出する。drift があれば CI fail させ、`buf generate` の再実行を開発者に促す。Phase 1b で `build.rs` の信頼性が確認できたら pre-generated 方式を廃止し、`generated/` を `.gitignore` 対象に変える判断を行う（その場合 DS-SW-COMP-132 を改訂する）。

**確定フェーズ**: Phase 1a。**対応要件**: DX-CICD-\*、ADR-TIER1-002。**上流**: DS-SW-COMP-132。

## API 別 proto ファイルの構成原則

### DS-IMPL-DIR-032 各 v1.proto の骨格

各 API の `v1/<api>/v1.proto` は以下の 4 節構造を原則とする。

```proto
// v1/state/v1.proto
syntax = "proto3";

package k1s0.state.v1;

import "google/protobuf/timestamp.proto";
import "v1/common/common.proto";
import "v1/errors/errors.proto";

option go_package = "github.com/k1s0/k1s0/src/tier1/go/internal/shared/proto/v1/state;statev1";

// 1. Service 定義
service StateService {
  rpc Get(GetRequest) returns (GetResponse);
  rpc Set(SetRequest) returns (SetResponse);
  rpc Delete(DeleteRequest) returns (DeleteResponse);
}

// 2. Request/Response 型
message GetRequest { ... }
message GetResponse { ... }

// 3. ドメインモデル（リクエスト/レスポンスに埋め込む型）
message StateItem { ... }

// 4. API 固有 enum（共通 errors では表現できないもの）
enum ConsistencyLevel {
  CONSISTENCY_LEVEL_UNSPECIFIED = 0;
  CONSISTENCY_LEVEL_EVENTUAL = 1;
  CONSISTENCY_LEVEL_STRONG = 2;
}
```

順序を固定することで、複数 API の `.proto` を並行レビューする際の認知負荷を下げる。1 API の `.proto` が 300 行を超えた場合は、3 のドメインモデルを `v1_types.proto` に分割し、4 の enum を `v1_enums.proto` に分割する。

**確定フェーズ**: Phase 1a。**対応要件**: DX-CICD-\*、NFR-C-NOP-002。**上流**: DS-SW-COMP-107、DS-SW-COMP-121。

### DS-IMPL-DIR-033 API 間の proto 参照ルール

API 間で `.proto` を import する場合は 3 ルールを守る。

1. `common/*.proto` と `errors/*.proto` は全 API から import 可能
2. API 間の相互 import は禁止（例: `state/v1.proto` から `decision/v1.proto` の import は不可）
3. API 間で共有したい型が現れた場合は `common/` に移動して両 API から import する

ルール 2 の違反は `buf lint` のカスタムルール（Phase 1b で `buf lint` plugin を追加）で検出する。理由は API 間の proto 依存が発生すると、片方の `.proto` を変更しただけで両方の Pod の再ビルドが必要になり、デプロイ時の整合性管理が複雑化するためである。

**確定フェーズ**: Phase 1a（ルール）、Phase 1b（CI 検出）。**対応要件**: DX-CICD-\*、NFR-A-CONT-003。**上流**: DS-SW-COMP-100〜103。

### DS-IMPL-DIR-034 外部 Protobuf 依存の制限

`.proto` から import できる外部 Protobuf は以下の 3 カテゴリに限定する。

1. Google 標準: `google/protobuf/*.proto`（Timestamp / Duration / Empty / Any / Struct / FieldMask 等）
2. Google API: `google/rpc/*.proto`（Status / Code）、`google/api/*.proto`（Annotations、HTTP マッピング用。Phase 2 で使用検討）
3. k1s0 内部: `v1/common/*.proto` / `v1/errors/*.proto`

上記以外の外部 Protobuf 依存（例: `grpc-gateway` 用 `protoc-gen-openapiv2`、`cloudevents`）を追加する場合は ADR 起票を要する。Phase 1a 時点では 1 と 3 のみ使い、Phase 2 で gRPC-Gateway 対応時に 2 を解禁する予定。

**確定フェーズ**: Phase 1a。**対応要件**: DX-CICD-\*、NFR-SUP-\*。**上流**: DS-SW-COMP-123。

## 破壊的変更検査

### DS-IMPL-DIR-035 buf breaking のブランチ比較

`buf breaking` は main ブランチとの比較で実行する。PR パイプライン（`.github/workflows/pr-contracts.yml`）で `buf breaking --against '.git#branch=main,subdir=src/tier1/contracts'` を実行し、破壊的変更を検出したら PR を fail させる。`subdir=src/tier1/contracts` を指定することで contracts 配下のみを検査対象とし、他ディレクトリの変更による誤検出を防ぐ。main 以外への PR（例: release ブランチ）では base ブランチを `${{ github.base_ref }}` で動的に指定する。

**確定フェーズ**: Phase 1a。**対応要件**: DX-CICD-\*、ADR-TIER1-002。**上流**: DS-SW-COMP-115。

### DS-IMPL-DIR-036 破壊的変更が必要な場合の手続

破壊的変更が業務上必要（例: 型の semantics 変更）な場合は、以下の 4 段階で進める。

1. ADR 起票（構想設計 `docs/02_構想設計/adr/` 配下に `ADR-CONTRACT-NNN-*.md` として起票）
2. 旧版 `v1/` に deprecation アノテーション（`[deprecated = true]`）を追加し、並行期間 6 か月を開始
3. 新版 `v2/` を `v1/` と並列に追加（`v1/state/v2.proto`）し、両方を CI で検証
4. 並行期間終了後に `v1/` を削除（DS-SW-COMP-118 deprecation 手続に従う）

段階 2 〜 4 の間、`buf breaking` は `v1/` に対しては引き続き後方互換性を要求し、`v2/` は新規ファイルとして扱う（`buf.yaml` の `breaking: use: [FILE]` により）。`v2/` を追加する際は生成コードも `internal/proto/v2/` / `generated/k1s0.state.v2.rs` と並列に増え、Pod 側は両版を同時 serving する（[../../04_概要設計/20_ソフトウェア方式設計/03_内部インタフェース方式設計/01_内部gRPC契約方式.md](../../04_概要設計/20_ソフトウェア方式設計/03_内部インタフェース方式設計/01_内部gRPC契約方式.md) 参照）。

**確定フェーズ**: Phase 1a（手続）、Phase 1c 以降（実際の v2 導入）。**対応要件**: DX-CICD-\*、NFR-C-NOP-001。**上流**: DS-SW-COMP-115、DS-SW-COMP-118。

### DS-IMPL-DIR-037 buf lint のカスタムルール

`buf lint` は STANDARD ルールに加え、k1s0 独自のルールを `buf.plugin.yaml`（Phase 1b）で追加する。初期ルールは以下。

- `K1S0_SERVICE_NAME_SUFFIX`: service 名は `Service` サフィックス必須（例: `StateService`）
- `K1S0_RPC_REQUEST_RESPONSE_NAMING`: RPC の request/response 型は `<Rpc>Request` / `<Rpc>Response` 必須
- `K1S0_NO_CROSS_API_IMPORT`: API 間の `.proto` 相互 import を禁止（DS-IMPL-DIR-033 の実装）
- `K1S0_ERROR_CODE_RANGE`: エラーコード enum は k1s0 固有範囲（100 〜 999）のみ使用

Phase 1a では STANDARD のみで開始し、Phase 1b でカスタムルールを順次追加する。カスタムルール追加時は既存の全 `.proto` を通るようにしてから CI に組み込む。

**確定フェーズ**: Phase 1a（STANDARD）、Phase 1b（カスタム）。**対応要件**: DX-CICD-\*、NFR-C-NOP-002。**上流**: DS-SW-COMP-115、DS-SW-COMP-123。

## 運用

### DS-IMPL-DIR-038 .proto ファイル変更時の PR 必須チェック

`.proto` を含む PR は以下の 5 チェックを必須で pass する必要がある。

1. `buf lint`（STANDARD + k1s0 独自）
2. `buf breaking`（main との比較）
3. `buf generate` 結果と commit 内容の一致
4. 生成コードを利用する全 Pod のビルド pass（Go / Rust）
5. 生成コードを利用する全 Pod の unit test pass

上記 5 つが 1 つでも fail したら PR マージ禁止とする。合計所要時間目標は PR パイプラインで 3 分以内（DS-SW-COMP-135 の選択的ビルド方針に従い、contracts 変更時は Go / Rust 両方をビルドする）。

**確定フェーズ**: Phase 1a。**対応要件**: DX-CICD-\*、DX-MET-\*、ADR-TIER1-002。**上流**: DS-SW-COMP-115、DS-SW-COMP-135。

### DS-IMPL-DIR-039 contracts/ の CODEOWNERS

`src/tier1/contracts/` 配下の変更は `@k1s0/tier1-architects` と `@k1s0/api-leads` の両方の承認を必須とする。`api-leads` は tier1 API 契約の責任者（初期は tier1 architects のサブセット）で、業務観点で API 契約の妥当性を審査する。`tier1-architects` は技術観点で互換性・命名・依存を審査する。Phase 1a 時点では 2 チームが同一メンバーで実質重複だが、Phase 2 以降の組織成長を見越して 2 チーム分離する。

**確定フェーズ**: Phase 1a。**対応要件**: DX-CICD-\*、NFR-C-NOP-001。**上流**: DS-IMPL-DIR-018。

### DS-IMPL-DIR-040 contracts/ のバージョニング戦略

contracts 全体のバージョニング戦略は以下のとおり。

- `v1/` ディレクトリ配下の `.proto` 変更は、後方互換性を維持する限り minor/patch 変更とみなす
- 後方互換性を壊す変更は `v2/` を新設して並行運用（DS-IMPL-DIR-036 参照）
- `v1/` と `v2/` の並行期間は最短 6 か月、最長 18 か月（Phase 2 以降の 2 年サイクルを超えない）
- `v0/` は原則使わない（Phase 1a 開始時点で `v1/` から始める）

tier2 / tier3 が contracts を取得する経路は Phase 1a 〜 1b では `git submodule` 方式、Phase 1c 以降で BSR 方式を検討する（DS-IMPL-DIR-028 の `buf.md` 整備と連動）。

**確定フェーズ**: Phase 1a（v1 開始）、Phase 1c（BSR 検討）。**対応要件**: NFR-C-NOP-001、ADR-TIER1-002、DX-BS-\*。**上流**: DS-SW-COMP-115、DS-SW-COMP-118。

## 章末サマリ

### 設計 ID 一覧

| 設計 ID | 内容 | 確定フェーズ |
|---|---|---|
| DS-IMPL-DIR-021 | contracts/ ルートの必須ファイル 5 種 | Phase 1a |
| DS-IMPL-DIR-022 | v1/ 配下の API 別サブディレクトリ化 | Phase 1a |
| DS-IMPL-DIR-023 | common/ と errors/ の共通型配置 | Phase 1a |
| DS-IMPL-DIR-024 | Protobuf package 命名規約 | Phase 1a |
| DS-IMPL-DIR-025 | buf.yaml の内容 | Phase 1a |
| DS-IMPL-DIR-026 | buf.gen.yaml の内容 | Phase 1a |
| DS-IMPL-DIR-027 | buf.lock の扱い | Phase 1a |
| DS-IMPL-DIR-028 | buf.md の Phase 1c 導入 | Phase 1a/1c |
| DS-IMPL-DIR-029 | Go 側生成コードの配置先 | Phase 1a |
| DS-IMPL-DIR-030 | Rust 側生成コードの配置先 | Phase 1a |
| DS-IMPL-DIR-031 | proto-gen crate の完全構造 | Phase 1a |
| DS-IMPL-DIR-032 | 各 v1.proto の骨格 | Phase 1a |
| DS-IMPL-DIR-033 | API 間の proto 参照ルール | Phase 1a/1b |
| DS-IMPL-DIR-034 | 外部 Protobuf 依存の制限 | Phase 1a |
| DS-IMPL-DIR-035 | buf breaking のブランチ比較 | Phase 1a |
| DS-IMPL-DIR-036 | 破壊的変更が必要な場合の手続 | Phase 1a/1c |
| DS-IMPL-DIR-037 | buf lint のカスタムルール | Phase 1a/1b |
| DS-IMPL-DIR-038 | .proto ファイル変更時の PR 必須チェック | Phase 1a |
| DS-IMPL-DIR-039 | contracts/ の CODEOWNERS | Phase 1a |
| DS-IMPL-DIR-040 | contracts/ のバージョニング戦略 | Phase 1a/1c |

### 対応要件一覧

- NFR-A-CONT-003、NFR-C-NOP-001、NFR-C-NOP-002、NFR-H-INT-\*、NFR-SUP-\*
- DX-CICD-\*、DX-MET-\*、DX-BS-\*
- ADR-TIER1-002（Protobuf）

### 上流設計 ID

DS-SW-COMP-100〜103（依存方向）、DS-SW-COMP-107（k1s0-proto）、DS-SW-COMP-108（k1s0-common）、DS-SW-COMP-115（破壊的変更検査）、DS-SW-COMP-118（deprecation）、DS-SW-COMP-119（外部 OSS 更新）、DS-SW-COMP-121（contracts 内部構成）、DS-SW-COMP-122（生成コード配置）、DS-SW-COMP-123（buf 設定）、DS-SW-COMP-132（proto-gen crate）、DS-SW-COMP-135（選択的ビルド）。本章は DS-IMPL-DIR-018（CODEOWNERS）と双方向トレースする。
