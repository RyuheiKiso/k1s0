# 09. tier1 全体配置と SDK 境界

本ファイルは tier1 を**ひとつの bounded context として束ねた視点**で、02 章〜05 章が分担して詳細化した 4 サブ領域（`contracts/` / `go/` / `rust/` / `infra/`）の相互関係と、tier1 の外縁（tier2 / tier3 への公開点）を方式として固定する。上流は概要設計 [DS-SW-COMP-120（tier1 のトップレベル構成）](../../04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/06_パッケージ構成_Rust_Go.md) および ADR-TIER1-003（内部言語不透明性）で、本章はそれを Phase 1a 着手時点の「公開 SDK 生成物とレジストリ配布境界」まで分解する。

## 本ファイルの位置付け

02 章〜05 章は tier1 の内部構造を 4 つの専門領域に切り分けて詳細化したが、tier1 を**ひとつの契約単位**として外部（tier2 / tier3）がどう利用するかは各章の関心外で、読み手は散在する記述から境界像を再構成する必要があった。本章はその欠落を埋める。

具体的には次の 4 点を確定する。(a) tier1 monorepo と tier2 / tier3 polyrepo の分離、(b) tier1 から tier2 / tier3 への唯一の公開点としての SDK レジストリ、(c) SDK 生成物の物理配置（どの言語の SDK が `src/tier1/` のどこで生成・ビルド・パッケージングされるか）、(d) SDK バージョニングと tier1 release の同期ルール。これらが曖昧なまま Phase 1b のパイロット tier2 サービス開発に入ると、「tier2 が tier1 の internal package を submodule で直接取り込む」「SDK が言語ごとに別 repo・別命名でバラつく」「SDK の release が tier1 release から遅延し tier2 の PR が永続 blocked になる」といった破綻が発生する。本章はその発散を Phase 1a の配置レベルで封じる役割を担う。

## 概要設計との役割分担

概要設計 [DS-SW-COMP-120](../../04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/06_パッケージ構成_Rust_Go.md) は `src/tier1/` の 4 サブディレクトリまでを方式として確定し、ADR-TIER1-003 は「tier2 / tier3 は tier1 の Protobuf IDL から生成された SDK のみを使う」と決定済みである。本章は (a) その SDK がどの言語で、どこで生成され、どこから配布されるかを実装視点で確定し、(b) tier1 repo と tier2 / tier3 repo の境界を設計 ID レベルで固定する。

## 設計 ID 一覧と採番方針

本ファイルで採番する設計 ID は `DS-IMPL-DIR-221` 〜 `DS-IMPL-DIR-240` の 20 件である。採番対象は tier1 の bounded context 境界と SDK 公開境界に限定し、tier1 内部の詳細は 02 章〜05 章の `DS-IMPL-DIR-021〜160` に委ねる。

## tier1 を束ねた視点でのレイヤ俯瞰

tier1 の 4 サブ領域は変更頻度が階差的に異なる。最上位の変更頻度を持つのは `contracts/` で、次に `go/` と `rust/` が続き、`infra/` が最も安定である。`contracts/` 変更は両実装（`go/` / `rust/`）に生成コード経由で波及するのに対し、`infra/` は container image の tag 更新のみで受け取るため、実装変更の大半は infra を触らずに完了する。この階差を understand したうえで読み分けることで、tier1 の配置意図が初めて統合的に見える。

```
src/tier1/（tier1 bounded context の物理境界）
├── contracts/                        # 変更頻度 ★★★★★（API 契約の真実）
│   ├── v1/*.proto                    # 11 API 定義（02 章 DS-IMPL-DIR-022）
│   ├── buf.yaml / buf.gen.yaml       # 生成設定（02 章 DS-IMPL-DIR-025〜026）
│   └── sdk-out/                      # 公開 SDK 生成物（本章 DS-IMPL-DIR-225）
│       ├── go/                       # Go 公開 SDK（Phase 1a スケルトン）
│       ├── csharp/                   # C# 公開 SDK（Phase 1b 導入）
│       ├── typescript/               # TypeScript 公開 SDK（Phase 2）
│       ├── rust/                     # Rust 公開 SDK（Phase 2 公開）
│       ├── java/                     # Java 公開 SDK（Phase 2）
│       └── python/                   # Python 公開 SDK（Phase 2）
├── go/                               # 変更頻度 ★★★★（Go 3 Pod 実装）
│   ├── cmd/<pod>/                    # main.go（03 章 DS-IMPL-DIR-045）
│   ├── internal/                     # 4 層（03 章 DS-IMPL-DIR-043）
│   ├── internal/proto/               # tier1 内部用 Protobuf 生成物（02 章）
│   └── pkg/k1s0sdk/                  # Go 公開 SDK の wrapper（本章 DS-IMPL-DIR-226）
├── rust/                             # 変更頻度 ★★★★（Rust 3 Pod 実装）
│   ├── crates/<pod>/                 # 3 bin crate + 共通 lib crate（04 章）
│   ├── crates/proto-gen/             # tier1 内部用 Protobuf 生成物（02 章）
│   └── crates/k1s0-sdk/              # Rust 公開 SDK の wrapper（本章 DS-IMPL-DIR-230）
└── infra/                            # 変更頻度 ★★（Helm / Kustomize / Dapr）
    └── ...（05 章 DS-IMPL-DIR-121〜160）
```

`contracts/sdk-out/` と `go/pkg/k1s0sdk/` / `rust/crates/k1s0-sdk/` は本章で新設する配置で、それ以外は 02〜05 章で既に確定した配置を再掲している。本章が追加するのは「tier1 の物理境界の内側で SDK をどこに置くか」の指定に集中する。

## DS-IMPL-DIR-221 tier1 を bounded context として定義する根拠

tier1 は 6 Pod（STATE / SECRET / WORKFLOW の Go facade 3 Pod、AUDIT / DECISION / PII の Rust custom 3 Pod）で構成され、外部には 11 API を**単一プラットフォーム**として提示する。この「単一プラットフォームに見せる」という約束（[04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/01_tier1全体コンポーネント俯瞰.md](../../04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/01_tier1全体コンポーネント俯瞰.md) DS-SW-COMP-004）を物理配置レベルで担保するのが、tier1 を **1 つの monorepo / 1 つの bounded context** として束ねる決定である。

bounded context として扱う実益は次の 3 点に集約される。第一に、`contracts/*.proto` を 1 点で管理することで 11 API の整合性（命名・エラー規約・メタデータ）を機械的に強制できる。第二に、Go 3 Pod と Rust 3 Pod が同一 repo にあるため、横断的な変更（Service Invoke 経由の PII 判定呼び出し追加など）を 1 PR で扱える。第三に、tier2 / tier3 が「tier1 のどれか 1 Pod」と会話するのではなく「tier1 全体」と会話することを、repo 境界によって視覚的にも強制できる。

**確定フェーズ**: Phase 0。**対応要件**: BR-PLATUSE-001（単一プラットフォーム）、NFR-C-NOP-001（2 名運用）、ADR-TIER1-001（Go/Rust ハイブリッド）、ADR-TIER1-003（内部言語不透明性）。**上流**: DS-SW-COMP-004、DS-SW-COMP-120。

## DS-IMPL-DIR-222 tier1 monorepo と tier2 / tier3 polyrepo の分離

tier1 は 1 つの monorepo（本 repo `k1s0`）に全サブ領域（contracts / go / rust / infra）を配置するが、tier2 ドメインサービスと tier3 エンドユーザアプリは **Backstage Software Template から生成される各自の独立 GitHub repo**で開発される（[DS-SW-DOC-001 ステップ 2](../../04_概要設計/20_ソフトウェア方式設計/05_利用者文書_暫定版/01_tier2_tier3開発者向けGoldenPath初版.md)）。tier1 の monorepo に tier2 / tier3 のコードを混ぜることは禁止し、逆に tier2 / tier3 の repo に tier1 の internal package を取り込むことも禁止する。

この polyrepo 方針を採る理由は 3 つある。(a) tier2 / tier3 の開発チームは 20〜30 名規模に拡大する予定（ADR-TIER1-003）で、1 つの monorepo に全サービスを入れると CI / レビュー負荷が爆発する。(b) tier2 / tier3 の CI / テスト要求は業務性質（金融系テストは厳しい、情報系テストは軽い等）で大きく異なり、CI 設定を service 別に独立させるほうが柔軟。(c) tier2 / tier3 の repo を Backstage Template で量産する運用は Golden Path 10 分ルールの前提であり、monorepo の PR フローでは 10 分以内の立ち上げが成立しない。

tier1 への寄与（例: 新 API 追加要望）は tier2 / tier3 側から Issue として提起し、tier1 チームが本 repo で PR 化する。tier2 / tier3 が tier1 に直接 PR を送る運用は禁止する（レビューオーナー分離のため）。

**確定フェーズ**: Phase 0。**対応要件**: DX-GP-001（Golden Path 10 分）、NFR-C-NOP-001、ADR-TIER1-003。**上流**: DS-SW-COMP-120、DS-SW-DOC-001。

## DS-IMPL-DIR-223 tier1 内部 4 サブ領域の変更頻度階差

tier1 の 4 サブ領域は変更頻度と変更主体が異なる。本設計 ID はその階差を明示し、PR レビュー負荷と CI 設計の前提として固定する。

- `contracts/`（最も変更頻度が高い、API オーナー主導）: 新 API 追加・既存 API の field 追加は `contracts/v1/*.proto` の変更で起きる。変更は `go/internal/proto/` と `rust/crates/proto-gen/` に生成コードとして波及し、さらに `sdk-out/` にも波及する。レビューは `@k1s0/api-leads` と `@k1s0/tier1-architects` のダブル必須。
- `go/` / `rust/`（次点、言語チーム主導）: ビジネスロジックの実装変更は `internal/` / `crates/<pod>/src/` で起きる。契約変更を伴わない修正は言語チーム単独でマージ可能。`contracts/` と同 PR で変更する場合は言語チーム + API リード の承認が必要。
- `infra/`（最も変更頻度が低い、infra / devex チーム主導）: Helm / Kustomize / Dapr Component の変更は実装リリースに追従するのみで、単独の infra-only PR は四半期に数回程度。

この階差は 01 章 DS-IMPL-DIR-018 の CODEOWNERS と、07 章 DS-IMPL-DIR-199 の詳細 CODEOWNERS で既にチーム単位で固定されている。本章はそれを「変更頻度」という視点で再構成し、CI リソース配分（PR 実行時間予算、並列数）の判断根拠に利用する。

**確定フェーズ**: Phase 0。**対応要件**: DX-CICD-\*、NFR-C-NOP-001。**上流**: DS-IMPL-DIR-018、DS-IMPL-DIR-199。

## DS-IMPL-DIR-224 tier1 → tier2 / tier3 への唯一の公開点（SDK レジストリ）

tier1 が tier2 / tier3 に公開する窓口は「Protobuf 契約から生成された SDK パッケージ、かつ内部レジストリ経由」の 1 点に限定する（ADR-TIER1-003）。本設計 ID はその「1 点」の物理実体を確定する。

公開は次の 2 段階に分ける。(1) `src/tier1/contracts/sdk-out/<lang>/` で生成された stub コードに、(2) `src/tier1/go/pkg/k1s0sdk/` / `src/tier1/rust/crates/k1s0-sdk/` 等の **wrapper 層** を被せて tenant_id / trace_id / user_id / Keycloak トークンの自動付与（[DS-SW-DOC-008](../../04_概要設計/20_ソフトウェア方式設計/05_利用者文書_暫定版/01_tier2_tier3開発者向けGoldenPath初版.md)）を施す。wrapper までセットにしたものが tier2 / tier3 向け SDK として内部レジストリに publish される。

**内部レジストリの選定**: Phase 1a では ghcr.io / GitHub Packages を使う。Phase 1b 以降は Nexus または Artifactory（ADR-TIER1-003 で placeholder 指定、本章では `nexus.k1s0.internal` を placeholder 名として統一使用）に移行する。移行手順は 08 章 DS-IMPL-DIR-213 と同様の sed 置換 + 監視期間を設ける。

公開 SDK 以外の経路（tier1 internal パッケージの直 import、tier1 gRPC エンドポイントの直叩き）は禁止する。ADR-TIER1-003 で決定済みだが、本章では物理配置として `contracts/sdk-out/` 以外を tier2 / tier3 に見せない運用を固定する。

**確定フェーズ**: Phase 1a（ghcr）、Phase 1b（Nexus 移行）。**対応要件**: ADR-TIER1-003、BR-PLATUSE-002（透過的動作）、NFR-H-INT-\*。**上流**: DS-SW-COMP-120、DS-SW-DOC-008。

## DS-IMPL-DIR-225 公開 SDK 生成物の配置（src/tier1/contracts/sdk-out/）

`src/tier1/contracts/sdk-out/` を公開 SDK の**生成物**配置として新設する。本ディレクトリの直下に言語別サブディレクトリ（`go/` / `csharp/` / `typescript/` / `rust/` / `java/` / `python/`）を置き、`buf generate --template buf.gen.pub.yaml` で 6 言語分の stub コードを一括生成する（`buf.gen.pub.yaml` は公開 SDK 用の生成設定で、tier1 internal 用の `buf.gen.yaml` とは別ファイル）。

ここで `internal/proto/` / `crates/proto-gen/` と `sdk-out/` を分離する理由は 2 点。(a) internal は tier1 の `gRPC server` 用（`*_grpc.pb.go` の Server 型を利用）、public は tier2 / tier3 の `gRPC client` 用（Client 型のみ）で、生成 option が異なる。(b) internal は「tier1 内部実装の詳細で公開しない」、public は「tier2 / tier3 に配布する」ためオーナーとライフサイクルが違う（public の破壊的変更は必ず ADR を要する）。

生成物は git 管理し、CI で `buf generate` 結果との差分を検出する。差分があれば PR を fail させ、`buf generate` の再実行を求める（02 章 DS-IMPL-DIR-026 と同じ運用）。

**確定フェーズ**: Phase 1a（go / rust のスケルトン）、Phase 1b（csharp 追加）、Phase 2（typescript / java / python 追加）。**対応要件**: ADR-TIER1-003、DX-GP-006。**上流**: DS-SW-COMP-122、DS-IMPL-DIR-026。

## DS-IMPL-DIR-226 Go 公開 SDK の配置と wrapper

Go 公開 SDK は 2 層構造で置く。生成 stub は `src/tier1/contracts/sdk-out/go/` に、tenant_id / trace_id 等の自動付与を行う wrapper は `src/tier1/go/pkg/k1s0sdk/` に配置する。配布は wrapper 層を entry point とし、wrapper が sdk-out/go/ を internal 依存として取り込む構成にする。

Go module path は `github.com/k1s0/k1s0-sdk-go`（tier1 repo 内に配置するが module としては独立）で、`go.mod` は `src/tier1/go/pkg/k1s0sdk/` に置く。tier1 本体の `go.mod`（`src/tier1/go/go.mod`）とは別 module にすることで、tier2 側が `go get github.com/k1s0/k1s0-sdk-go@v1.0.0` だけで取得でき、tier1 の internal 依存を引きずらない。

`src/tier1/go/pkg/k1s0sdk/` の内部構造は `client.go`（外向き API 11 種のラッパ関数）・`middleware.go`（Keycloak トークン・tenant_id・trace_id の 3 ヘッダ自動付与）・`retry.go`（期限切れ時 1 回 retry、DS-SW-DOC-008 準拠）・`examples/` の 4 要素とする。

**確定フェーズ**: Phase 1a（スケルトン + 3 API 分）、Phase 1b（11 API 完全版）。**対応要件**: DX-GP-006、BR-PLATUSE-002、ADR-TIER1-003。**上流**: DS-SW-COMP-124、DS-SW-DOC-003、DS-SW-DOC-008。

## DS-IMPL-DIR-227 C# 公開 SDK の配置（Phase 1b）

C# 公開 SDK は `src/tier1/contracts/sdk-out/csharp/` 配下に、stub と wrapper を 1 つの .NET ソリューションとして配置する（Go と異なり、.NET では stub と wrapper を別 project に分けると NuGet 配布時のパッケージ数が増えて管理コストが上がるため、1 project に統合する）。

ディレクトリ構造は `sdk-out/csharp/K1S0.Sdk/`（メインプロジェクト）と `sdk-out/csharp/K1S0.Sdk.Tests/`（単体テスト）の 2 プロジェクトで、親に `K1S0.Sdk.sln` を置く。`K1S0.Sdk.csproj` の `TargetFramework` は `net8.0`（Phase 1b 時点の LTS）、依存パッケージは `Grpc.Net.Client` / `Grpc.Tools` / `Google.Protobuf` を NuGet から取得する。

NuGet パッケージ名は `K1S0.Sdk`（大文字始まり、.NET 命名慣例）で、配布先は Phase 1b では Nexus placeholder `nexus.k1s0.internal/nuget/`。Phase 1a では C# SDK は生成のみで配布しない（tier2 が C# で書かれるのは Phase 1b からで、Phase 1a 時点では Go tier2 パイロットのみが想定される）。

**確定フェーズ**: Phase 1b。**対応要件**: DX-GP-006、ADR-TIER1-003。**上流**: DS-SW-DOC-003、DS-SW-COMP-122。

## DS-IMPL-DIR-228 TypeScript 公開 SDK の配置（Phase 2）

TypeScript 公開 SDK は `src/tier1/contracts/sdk-out/typescript/` 配下に、`package.json` + `tsconfig.json` + `src/` の構造で配置する。tier3 Web App（Next.js / React）および tier3 BFF（Node.js）の両方で使えるよう、`browser` / `node` のサブパス export（`./browser` と `./node`）を用意する。

npm パッケージ名は `@k1s0/sdk`（scoped package、k1s0 org）で、配布先は Phase 2 では Nexus の npm registry（`nexus.k1s0.internal/npm/`）。Phase 1a / 1b 期間中は TypeScript SDK は生成のみ行い、tier3 本格展開までは npm publish しない。

build は `tsc` で JavaScript + .d.ts を出力し、`dist/` を `.gitignore` で除外する。`src/` は TypeScript ソース（stub は `src/generated/` にプロトバッファ生成物、wrapper は `src/k1s0sdk/` にカスタム API）。

**確定フェーズ**: Phase 2。**対応要件**: DX-GP-006、ADR-TIER1-003。**上流**: DS-SW-DOC-003。

## DS-IMPL-DIR-229 Java / Python 公開 SDK の配置（Phase 2）

Java 公開 SDK は `src/tier1/contracts/sdk-out/java/` に Maven プロジェクト（`pom.xml` + `src/main/java/`）を、Python 公開 SDK は `src/tier1/contracts/sdk-out/python/` に poetry プロジェクト（`pyproject.toml` + `k1s0_sdk/`）を配置する。両者とも Phase 2 の tier2 拡大（C# 以外の言語での新規 tier2 サービス追加時）で有効化する。Phase 1a / 1b 時点では `sdk-out/java/` / `sdk-out/python/` は存在せず、ディレクトリも作成しない。

命名は Java が `io.k1s0.sdk`（逆ドメイン慣例）、Python が `k1s0-sdk`（pip パッケージ名、PEP 8 準拠）。配布先はそれぞれ Nexus の Maven repository と PyPI mirror。

**確定フェーズ**: Phase 2。**対応要件**: DX-GP-006、ADR-TIER1-003。**上流**: DS-SW-DOC-003。

## DS-IMPL-DIR-230 Rust 公開 SDK の配置（内部 proto-gen と別 crate）

Rust 公開 SDK は `src/tier1/rust/crates/k1s0-sdk/` に新設する bin crate ではない lib crate として配置する。tier1 内部用の `src/tier1/rust/crates/proto-gen/`（tier1 自身の Rust 3 Pod が `use k1s0_proto_gen::...` で利用するもの）とは別 crate で、公開 SDK 用に `proto-gen` の stub を再 export し、wrapper 層（tenant_id / trace_id 自動付与）を追加する。

crate 名は `k1s0-sdk`、publish 先は Phase 2 時点で Nexus の crates.io ミラー（`nexus.k1s0.internal/cargo/`）。Rust tier2 は Phase 2 以降の想定であり、Phase 1a では `k1s0-sdk` crate は作らずに wrapper 層の設計メモ（`docs/90_knowledge/` 配下）のみ用意する。

Rust SDK を Phase 1a に含めない理由は、Dapr Rust SDK が Phase 1a 時点で stable に達していない（ADR-TIER1-001）ためで、tier1 内部で既に使っている Rust の直接 gRPC 呼び出し（`tonic`）パターンを tier2 に開放する前に、Dapr Rust SDK 成熟度を再評価する必要がある。

**確定フェーズ**: Phase 2（正式公開）、Phase 1a（設計メモのみ）。**対応要件**: ADR-TIER1-003、ADR-TIER1-001。**上流**: DS-SW-COMP-129〜139。

## DS-IMPL-DIR-231 SDK バージョニング方針

公開 SDK のバージョニングは**tier1 リリースと同期した SemVer + Phase ラベル**とする。形式は `<major>.<minor>.<patch>-phase<N>`（例: `1.0.0-phase1a`、`1.1.0-phase1b`）で、08 章 DS-IMPL-DIR-214 の Git タグ規則と整合させる。

SemVer の解釈は次の規則で固定する。MAJOR の上昇は `contracts/v1/*.proto` の後方互換性を破壊する変更（`buf breaking` で検出される）に対応し、tier2 / tier3 側のコード変更を要求する。MINOR の上昇は新規 API 追加や field 追加（後方互換）で、tier2 / tier3 は再コンパイルのみで済む。PATCH の上昇は wrapper 層のバグ修正で、API 契約自体は変わらない。

Phase ラベルは tier1 全体リリースの Phase（Phase 1a / Phase 1b / Phase 1c / Phase 2）と同期する。同じ SDK の MAJOR が複数 Phase にまたがることは許容する（例: `1.0.0-phase1a` → `1.2.0-phase1b` → `1.5.0-phase1c` → `2.0.0-phase2`）。

**確定フェーズ**: Phase 0（ルール）、各 Phase（適用）。**対応要件**: DX-CICD-\*、ADR-TIER1-003、NFR-SUP-\*。**上流**: DS-IMPL-DIR-214。

## DS-IMPL-DIR-232 SDK 配布レジストリ（Phase 1a: ghcr、Phase 1b: Nexus placeholder）

SDK の配布レジストリは Phase 別に次のとおり使い分ける。Phase 1a 〜 1b 初期では GitHub Packages（`ghcr.io/k1s0/` Maven registry、npm registry、NuGet registry）を使う。Phase 1b 中盤以降で Nexus（placeholder `nexus.k1s0.internal`）に移行し、Phase 1c までには全 SDK を Nexus に集約する。

Nexus への移行手順は 08 章 DS-IMPL-DIR-213（Harbor placeholder）と並行し、ADR を 1 本起票して tier1 repo と tier2 / tier3 repo の `pom.xml` / `package.json` / `go.sum` 等を一括で書き換える。Nexus FQDN が確定するまで `nexus.k1s0.internal` を placeholder として本章内で統一使用し、sed 一括置換で移行する。

**確定フェーズ**: Phase 1a（ghcr）、Phase 1b 〜 1c（Nexus 移行）。**対応要件**: NFR-H-INT-\*、DX-CICD-\*、ADR-TIER1-003。**上流**: DS-IMPL-DIR-213。

## DS-IMPL-DIR-233 SDK リリースと tier1 リリースの同期

SDK の publish は tier1 本体（6 Pod の container image）の publish と**同一 GitHub Actions 上で**同時に実行する。release ワークフロー（07 章 DS-IMPL-DIR-192）に以下 2 ジョブを追加する。

- `publish-sdk-go`: `src/tier1/go/pkg/k1s0sdk/` を `goreleaser` で build し、GitHub Release のアセットとして添付 + ghcr.io の Go module proxy に publish。
- `publish-sdk-csharp`（Phase 1b 以降）: `src/tier1/contracts/sdk-out/csharp/` を `dotnet pack` で NuGet 化し、ghcr.io / Nexus の NuGet registry に push。

これにより「tier1 release 時点で SDK が未公開」の状態を構造的に不可能にする。ただし、wrapper 層のバグ修正のみの PATCH リリース（`1.0.1-phase1a` 等）は SDK のみを release できる例外経路として用意する（release ワークフローに `sdk-only-release.yml` を追加、Phase 1b）。

**確定フェーズ**: Phase 1a（Go）、Phase 1b（C# 追加）、Phase 2（他言語）。**対応要件**: DX-CICD-\*、ADR-TIER1-003。**上流**: DS-IMPL-DIR-192。

## DS-IMPL-DIR-234 SDK 互換性保証期間

SDK の MAJOR バージョンは最短でも**直前 2 つ**を並行サポートする。具体的には、MAJOR N を release した時点で MAJOR N-1 と MAJOR N-2 のサポートを継続し、N+1 release 時点で N-2 のサポートを終了する（つまり各 MAJOR は「release 時点から 2 MAJOR 先の release まで」の約 2 Phase 分サポートされる）。

この保証期間は tier2 / tier3 の移行猶予として設計されたもので、tier1 の MAJOR 上昇が四半期に 1 回以下の頻度（DS-SW-COMP-138）であることを前提に、tier2 / tier3 が半年〜1 年で移行できるマージンを確保する。保証期間中の SDK バージョンは Nexus registry に残し、削除しない。

**確定フェーズ**: Phase 0（ルール）、各 MAJOR リリース（適用）。**対応要件**: ADR-TIER1-003、NFR-SUP-\*、DX-CICD-\*。**上流**: DS-SW-COMP-138。

## DS-IMPL-DIR-235 tier1 repo と tier2 / tier3 repo の命名規則

tier1 本体 repo は `k1s0`（本 repo、固定）。tier2 / tier3 の repo は Backstage Template 生成時に次の命名で作成する。

- tier2 ドメインサービス: `k1s0-tier2-<ドメイン>-<サービス名>`（例: `k1s0-tier2-order-management`、`k1s0-tier2-payroll-gateway`）
- tier3 エンドユーザアプリ: `k1s0-tier3-<アプリ名>`（例: `k1s0-tier3-employee-portal`、`k1s0-tier3-approval-web`）
- SDK 公開 repo（該当する場合のみ、通常は k1s0 monorepo 内に閉じる）: `k1s0-sdk-<language>`（将来、SDK を tier1 monorepo から外出しする必要が出た場合の予約名）

これらの命名は GitHub organization `k1s0` 配下で一意にし、Backstage Software Template の `values.yaml` で `repoName` を検証する。命名違反は Template 側で reject する。

**確定フェーズ**: Phase 1a（ルール）、Phase 1b 以降（適用）。**対応要件**: DX-GP-001、NFR-C-NOP-001。**上流**: DS-SW-DOC-002。

## DS-IMPL-DIR-236 生成コードの git 管理ポリシーの再確認

`contracts/sdk-out/<lang>/` 配下の生成コードは**全て git 管理する**（02 章 DS-IMPL-DIR-026 と同じ方針）。CI で `buf generate --template buf.gen.pub.yaml` を実行し、commit 済みの生成物と差分があれば PR を fail させる。

生成コードを git 管理する理由は、tier2 / tier3 の立ち上げ時に「tier1 repo を clone → `buf` を install → generate を走らせる」という手順を省き、tier1 repo の tag をチェックアウトするだけで完全な SDK が利用可能にするためである。Nexus に publish されている SDK は生成物のコピーであり、生成過程は tier1 repo に再現可能な形で残る。

`.gitignore` では `sdk-out/<lang>/dist/` `sdk-out/<lang>/target/` `sdk-out/<lang>/node_modules/` などのビルド成果物のみを除外し、ソース相当の生成物（`*.pb.go` / `*.cs` / `*.ts`）は追跡する。

**確定フェーズ**: Phase 1a。**対応要件**: DX-CICD-\*、NFR-C-NOP-001。**上流**: DS-IMPL-DIR-026、DS-SW-COMP-122。

## DS-IMPL-DIR-237 SDK サンプルの examples/ 配置

各言語 SDK には `examples/` ディレクトリを置き、tier1 公開 11 API の最小呼び出し例を 1 言語あたり 11 ファイル配置する（API 数 × 1 サンプル）。例えば Go 版は `src/tier1/go/pkg/k1s0sdk/examples/state_get.go` / `pubsub_publish.go` / `decision_evaluate.go` ... のように分割する。

サンプルは**Backstage Golden Path の初回体験**で tier2 / tier3 開発者が最初にコピペする対象になるため、各ファイルは単体で `go run` / `dotnet run` / `ts-node` で実行可能にする。サンプル内で認証情報などの secret が要る場合は環境変数経由で注入する形を統一し、ハードコードは禁止する。

**確定フェーズ**: Phase 1a（Go の 3 API 分）、Phase 1b（Go の 11 API + C# の主要 API）、Phase 2（他言語）。**対応要件**: DX-GP-006、ADR-TIER1-003。**上流**: DS-SW-DOC-003。

## DS-IMPL-DIR-238 SDK テストの配置

SDK の単体テスト + 結合テスト（tier1 モック + SDK）の配置は次で固定する。

- Go SDK のテスト: `src/tier1/go/pkg/k1s0sdk/*_test.go`（unit） + `src/tier1/go/pkg/k1s0sdk/integration_test/`（tier1 モックとの結合、build tag `integration`）
- C# SDK のテスト: `src/tier1/contracts/sdk-out/csharp/K1S0.Sdk.Tests/`（xUnit）
- TypeScript SDK のテスト: `src/tier1/contracts/sdk-out/typescript/src/**/*.test.ts`（Vitest）

SDK のテストでは [DS-SW-DOC-008](../../04_概要設計/20_ソフトウェア方式設計/05_利用者文書_暫定版/01_tier2_tier3開発者向けGoldenPath初版.md) の「透過的動作の保証」3 ケース（auth ヘッダ自動付与、tenant_id 自動付与、trace_id の親 span 伝播）を必ず検証する。これらはリファレンス実装の main マージ条件で、CI で継続実行する。

**確定フェーズ**: Phase 1a（Go）、Phase 1b（C#）、Phase 2（他言語）。**対応要件**: DX-TEST-\*、BR-PLATUSE-002、ADR-TIER1-003。**上流**: DS-SW-DOC-008、DS-IMPL-DIR-171。

## DS-IMPL-DIR-239 SDK 破壊的変更時の移行ガイド

SDK の MAJOR 上昇時は、`docs/90_knowledge/sdk-migration/<from-version>-to-<to-version>.md` に移行ガイドを置く。ガイドは「削除された API、置換された API、新規追加された必須オプション、tier2 / tier3 側の具体的な書き換え例」の 4 セクションを最低限含む。

移行ガイドは SDK の MAJOR release と同一 PR で書き、release の前提条件とする。ガイド不備のまま MAJOR リリースすることは禁止する（release ワークフローの pre-condition で `docs/90_knowledge/sdk-migration/` の更新を検査）。

**確定フェーズ**: Phase 0（ルール）、各 MAJOR リリース（適用）。**対応要件**: ADR-TIER1-003、NFR-SUP-\*、DX-LD-\*。**上流**: DS-SW-COMP-138、DS-IMPL-DIR-192。

## DS-IMPL-DIR-240 tier1 bounded context 変更時の ADR 起票条件

tier1 の bounded context 境界そのものを変える変更は ADR を要する。以下が該当。

1. `src/tier1/` 直下の 4 サブディレクトリ（`contracts` / `go` / `rust` / `infra`）の増減
2. 公開 SDK の対応言語増減（現状: go / csharp / typescript / rust / java / python の 6 言語から外れる変更）
3. 公開 SDK 配布レジストリの変更（現状: ghcr → Nexus の 2 経路から外れる変更）
4. tier2 / tier3 repo の命名規則変更（DS-IMPL-DIR-235）
5. SDK 互換性保証期間の短縮（DS-IMPL-DIR-234 の「直前 2 MAJOR」を縮める変更）

軽微な追加（既存 4 サブ領域内のサブディレクトリ追加、既存 SDK 言語の新規 API 追加）は ADR 不要で、CODEOWNERS レビューで通す。

**確定フェーズ**: Phase 0（ルール）、各 Phase（適用）。**対応要件**: NFR-C-NOP-001、DX-CICD-\*、ADR-TIER1-003。**上流**: DS-SW-COMP-138、DS-IMPL-DIR-018。

## 章末サマリ

### 設計 ID 一覧

| 設計 ID | 内容 | 確定フェーズ |
|---|---|---|
| DS-IMPL-DIR-221 | tier1 を bounded context として定義する根拠 | Phase 0 |
| DS-IMPL-DIR-222 | tier1 monorepo と tier2/3 polyrepo の分離 | Phase 0 |
| DS-IMPL-DIR-223 | tier1 内部 4 サブ領域の変更頻度階差 | Phase 0 |
| DS-IMPL-DIR-224 | tier1 → tier2/3 への唯一の公開点 | Phase 1a/1b |
| DS-IMPL-DIR-225 | 公開 SDK 生成物の配置（sdk-out/） | Phase 1a/1b/2 |
| DS-IMPL-DIR-226 | Go 公開 SDK の配置と wrapper | Phase 1a/1b |
| DS-IMPL-DIR-227 | C# 公開 SDK の配置 | Phase 1b |
| DS-IMPL-DIR-228 | TypeScript 公開 SDK の配置 | Phase 2 |
| DS-IMPL-DIR-229 | Java / Python 公開 SDK の配置 | Phase 2 |
| DS-IMPL-DIR-230 | Rust 公開 SDK の配置 | Phase 2（設計メモ Phase 1a） |
| DS-IMPL-DIR-231 | SDK バージョニング方針 | Phase 0 |
| DS-IMPL-DIR-232 | SDK 配布レジストリ | Phase 1a/1b/1c |
| DS-IMPL-DIR-233 | SDK リリースと tier1 リリースの同期 | Phase 1a/1b/2 |
| DS-IMPL-DIR-234 | SDK 互換性保証期間 | Phase 0 |
| DS-IMPL-DIR-235 | tier1 repo と tier2/3 repo の命名規則 | Phase 1a |
| DS-IMPL-DIR-236 | 生成コードの git 管理ポリシー再確認 | Phase 1a |
| DS-IMPL-DIR-237 | SDK サンプルの examples/ 配置 | Phase 1a/1b/2 |
| DS-IMPL-DIR-238 | SDK テストの配置 | Phase 1a/1b/2 |
| DS-IMPL-DIR-239 | SDK 破壊的変更時の移行ガイド | Phase 0 |
| DS-IMPL-DIR-240 | tier1 bounded context 変更時の ADR 起票条件 | Phase 0 |

### 対応要件一覧

- BR-PLATUSE-001（単一プラットフォーム）、BR-PLATUSE-002（透過的動作）
- DX-GP-001（Golden Path 10 分）、DX-GP-006（SDK 1 行呼び出し）、DX-CICD-\*（CI/CD）、DX-TEST-\*（テスト）、DX-LD-\*（ローカル開発）
- NFR-C-NOP-001（2 名運用）、NFR-H-INT-\*（完整性）、NFR-SUP-\*（サポート）
- ADR-TIER1-001（Go/Rust ハイブリッド）、ADR-TIER1-003（内部言語不透明性）

### 上流設計 ID

DS-SW-COMP-004（tier1 不可視性）、DS-SW-COMP-120（tier1 トップレベル）、DS-SW-COMP-122（生成コード配置）、DS-SW-COMP-138（変更手続）、DS-SW-DOC-001（Golden Path 全体手順）、DS-SW-DOC-002（Backstage Template 生成物）、DS-SW-DOC-003（SDK 1 行呼び出し）、DS-SW-DOC-008（SDK 自動付与）、DS-IMPL-DIR-018（CODEOWNERS 母集団）、DS-IMPL-DIR-026（Go/Rust 内部 generate）、DS-IMPL-DIR-192（release.yml）、DS-IMPL-DIR-199（CODEOWNERS 詳細）、DS-IMPL-DIR-213（Harbor placeholder）、DS-IMPL-DIR-214（Git タグ規則）。
