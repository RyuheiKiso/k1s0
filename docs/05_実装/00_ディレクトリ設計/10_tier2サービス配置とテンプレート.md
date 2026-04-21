# 10. tier2 サービス配置とテンプレート

本ファイルは tier2 ドメインサービスの**リポジトリ配置と内部構造**を方式として固定する。上流は概要設計 [DS-SW-DOC-001（Golden Path 全体手順）](../../04_概要設計/20_ソフトウェア方式設計/05_利用者文書_暫定版/01_tier2_tier3開発者向けGoldenPath初版.md) および [ADR-TIER1-003（内部言語不透明性）](../../02_構想設計/adr/ADR-TIER1-003-language-opacity.md) で、本ファイルは Backstage Software Template から生成される tier2 repo の骨格と、Template 自身のソース配置（k1s0 repo 内）を確定させる。

## 本ファイルの位置付け

tier2 ドメインサービスは [09 章 DS-IMPL-DIR-222](09_tier1全体配置とSDK境界.md) で確定したとおり、**k1s0 repo とは別の GitHub repo**（各サービス 1 repo の polyrepo）で開発される。新規 tier2 repo は Backstage Software Template から生成され、生成直後の骨格には CI / Dockerfile / SDK 呼び出し雛形 / TechDocs スケルトンが予め含まれている（[DS-SW-DOC-002](../../04_概要設計/20_ソフトウェア方式設計/05_利用者文書_暫定版/01_tier2_tier3開発者向けGoldenPath初版.md)）。本章は (a) Template 自身のソースが k1s0 repo 内のどこに置かれるか、(b) Template から生成された tier2 repo の内部構造、(c) tier2 repo が k1s0 repo の SDK をどう消費するか、の 3 点を実装視点で固定する。

tier2 ディレクトリ設計が曖昧なまま Phase 1b のパイロット開発に入ると、以下の破綻が起きる。(a) tier2 repo のディレクトリ構成が開発者任せになり、各 tier2 repo がバラバラ（ある repo は `src/`、別 repo は `cmd/`、さらに別 repo は `app/`）になる。Runbook が「tier2 ごとに別の手順」を持つ必要が生じ、JTC 情シス 2 名体制（NFR-C-NOP-001）の運用が破綻する。(b) SDK の import 先が repo ごとに異なり、SDK MAJOR 更新時に tier2 全台の改修が各 repo で個別発明される。本章はこの発散を **Template 生成物の骨格を規約化**することで構造的に封じる。

## 概要設計との役割分担

概要設計 [DS-SW-DOC-001](../../04_概要設計/20_ソフトウェア方式設計/05_利用者文書_暫定版/01_tier2_tier3開発者向けGoldenPath初版.md) は Golden Path 7 ステップと Backstage Template が生成するファイル群の**最小集合**（`.github/workflows/ci.yml` / CODEOWNERS / Dockerfile / .devcontainer / docs/）を宣言した。本章はその宣言を**具体ファイル名・ディレクトリ構造・SDK import path**まで翻訳する。概要設計に対して本章が追加する情報は (a) Template ソースの k1s0 repo 内配置、(b) Template Action 引数の定義、(c) 生成される tier2 repo の内部構造のフルツリー、(d) 生成物の「禁止改変」範囲の具体リストである。

## 設計 ID 一覧と採番方針

本ファイルで採番する設計 ID は `DS-IMPL-DIR-241` 〜 `DS-IMPL-DIR-260` の 20 件である。tier2 に範囲を限定し、tier3 は [11 章](11_tier3アプリ配置とテンプレート.md) で別採番する。

## 全体像: Template ソースと生成 repo の関係

本章が対象とする 2 つの領域を先に俯瞰する。一方は k1s0 repo 内の Template ソース、もう一方は Backstage 実行後に GitHub 上に出現する tier2 repo である。

```
k1s0/（tier1 monorepo、本 repo）
└── tools/backstage-templates/
    └── tier2/                           # tier2 マイクロサービス雛形（07 章 DS-IMPL-DIR-185）
        ├── go/                          # Go tier2 用 Template
        │   ├── template.yaml            # Backstage Software Template 定義
        │   ├── skeleton/                # 生成物の骨格（変数展開前）
        │   │   ├── .github/workflows/ci.yml
        │   │   ├── .github/CODEOWNERS
        │   │   ├── .devcontainer/
        │   │   ├── cmd/${{values.service_name}}/main.go
        │   │   ├── internal/
        │   │   ├── go.mod
        │   │   ├── Dockerfile
        │   │   ├── docs/
        │   │   └── README.md
        │   └── README.md                # Template 利用ガイド
        └── csharp/                      # C# tier2 用 Template（Phase 1b）
            ├── template.yaml
            ├── skeleton/
            │   └── ...（C# 用骨格、詳細は DS-IMPL-DIR-247）
            └── README.md

k1s0-tier2-<domain>-<service>/（Backstage 生成後の tier2 repo、GitHub org: k1s0）
├── .github/workflows/ci.yml            # Template から持ち込み（削除禁止）
├── .github/CODEOWNERS                  # tier1 基盤チーム + サービスチーム
├── .devcontainer/
├── cmd/<service_name>/main.go          # ユーザが TODO 箇所にロジック記述
├── internal/
│   ├── handler/
│   ├── service/
│   └── domain/
├── go.mod                              # k1s0-sdk-go に依存
├── Dockerfile                          # distroless、non-root
├── docs/                               # TechDocs（サービス概要・API・Runbook）
└── README.md
```

Template ソース（k1s0 repo 内）と生成 repo（別 repo）の 2 層構造を保つことで、Template の改訂は k1s0 repo の PR で一元管理でき、既存 tier2 repo への反映は Renovate 相当の仕組み（Backstage の `Software Template update` 機能）で行う。

## DS-IMPL-DIR-241 本章の位置付け（tier2 ディレクトリ設計の範囲）

本章が対象とする物理境界を明示する。本章は (a) k1s0 repo 内の `tools/backstage-templates/tier2/*/` ディレクトリ、および (b) Backstage Template から生成される tier2 repo の内部構造、の 2 つに限定される。(b) の tier2 repo は本 repo の外に存在するが、**生成直後の骨格は本章で規定**されるため、本章の合意がなければ tier2 新規開発は開始できない。

生成後に tier2 サービスチームが追加するドメインロジック（`internal/service/` 内部の具体実装など）は本章の対象外で、各 tier2 サービスチームの判断に委ねる。ただし骨格ディレクトリの**削除**と、Template が生成した設定ファイル（CI / CODEOWNERS / Dockerfile）の**改変**は本章で禁止する（DS-IMPL-DIR-258）。

**確定フェーズ**: Phase 0。**対応要件**: DX-GP-001、NFR-C-NOP-001。**上流**: DS-SW-DOC-001、DS-IMPL-DIR-222。

## DS-IMPL-DIR-242 tier2 repo は別 GitHub repo（polyrepo）

tier2 ドメインサービスは各サービス 1 つの独立 GitHub repo を持つ。repo は k1s0 GitHub organization 配下に配置し、命名は 09 章 DS-IMPL-DIR-235 で確定した `k1s0-tier2-<domain>-<service>` 形式に従う（例: `k1s0-tier2-order-management`）。

tier2 を k1s0 monorepo に含めない理由は 09 章 DS-IMPL-DIR-222 で詳述したが、本章の視点から 2 点を補強する。(a) tier2 repo の生成は Backstage Software Template の「Create Component」ボタン 1 クリックで完了する必要があり（Golden Path 10 分ルール）、monorepo 内で新規ディレクトリを作る運用では PR レビューを待つ必要があって不可能。(b) tier2 サービスのオーナーチームは各ドメインチーム（20〜30 名規模に拡大予定）で、tier1 基盤チーム（数名）と ACL が異なるため、同一 repo で CODEOWNERS を細かく分けるのは管理困難。

**確定フェーズ**: Phase 0（ルール）、Phase 1b 以降（適用）。**対応要件**: DX-GP-001、NFR-C-NOP-001、ADR-TIER1-003。**上流**: DS-SW-DOC-001、DS-IMPL-DIR-222、DS-IMPL-DIR-235。

## DS-IMPL-DIR-243 Backstage Template ソースの配置

tier2 用 Backstage Software Template のソース一式は、k1s0 repo 内の `tools/backstage-templates/tier2/` 配下に配置する。サブディレクトリは**言語別に分ける**（`tier2/go/` / `tier2/csharp/`）。言語別に分ける理由は (a) Template ごとに `template.yaml` の `parameters` セクションが異なる（Go は `go_module_path`、C# は `dotnet_target_framework` など）、(b) skeleton の拡張子やビルドファイル（`go.mod` vs `*.csproj`）が排他的、(c) Template 更新を言語別に独立管理したい、の 3 点である。

`tools/backstage-templates/` は 07 章 DS-IMPL-DIR-185（Backstage 系の配置）で `tier2/` と `tier3/` の 2 階層分割が確定済みで、本章はその `tier2/` 側を詳細化する。07 章では Phase 1c で `tools/backstage/` に TechDocs / Catalog 設定を置くと決定済みだが、Software Template は独立したサブディレクトリ `tools/backstage-templates/` に分ける（Backstage の TechDocs 設定と Software Template は異なる Backstage プラグインで管理され、変更頻度とオーナーも異なるため）。11 章で tier3 用 Template が `tools/backstage-templates/tier3/` 配下に配置される設計と対称にすることで、将来の新変種追加（`tier2/python/`、`tier3/desktop-tauri/` など）が機械的に拡張可能になる。

Template ソースの Phase 導入は次のとおり。Phase 1b では `tier2/go/` のみ配置し、C# 版は骨格のみ（`README.md` に Phase 1b 後半で正式化する旨を明記）。Phase 1b 後半で C# 版を完全実装。Phase 2 で Java / Python 用 Template（`tier2/java/`、`tier2/python/`）を追加する可能性があるが、追加には ADR を要する。

**確定フェーズ**: Phase 1a（Go）、Phase 1b（C# 完全版）、Phase 2（Java / Python 追加可）。**対応要件**: DX-GP-001、ADR-TIER1-003。**上流**: DS-SW-DOC-001、DS-SW-DOC-002。

## DS-IMPL-DIR-244 tier2 Template の構造

各 Template は以下 3 要素で構成する。(a) `template.yaml`（Backstage Software Template の宣言ファイル）、(b) `skeleton/`（変数展開前の生成物テンプレート）、(c) `docs/`（Template 利用者向けガイド）。

`template.yaml` の `parameters` セクションには Backstage ポータル上で tier2 開発者に入力を求めるフィールドを定義する。最小入力は次の 5 項目で、概要設計 [DS-SW-DOC-001 ステップ 1](../../04_概要設計/20_ソフトウェア方式設計/05_利用者文書_暫定版/01_tier2_tier3開発者向けGoldenPath初版.md) と同期させる。

- `service_name`: サービス名（kebab-case、30 文字以内、GitHub repo 名の一部に展開）
- `team_name`: 所属チーム名（CODEOWNERS に展開、`@k1s0/<team>` 形式）
- `contact_email`: 一次連絡先メール（TechDocs の `docs/runbook.md` に展開）
- `handles_pii`: PII 取扱有無（`boolean`、true の場合は `k1s0.Pii.*` API 呼び出し雛形を追加挿入）
- `tenant_id`: テナント ID（Kubernetes manifest の `K1S0_TENANT_ID` 環境変数に展開）

`skeleton/` は `${{values.service_name}}` 等の Backstage 変数記法で変数展開を記述する。ファイル名自体に変数を含める場合（例: `cmd/${{values.service_name}}/main.go`）は Backstage の `fetch:template` Action が処理する。

`docs/` は tier2 開発者が Template 選択時に Backstage ポータル上で参照する案内で、Template の適用条件（「Go で書きたい場合はこちらを選ぶ」など）・所要時間・生成後の次ステップを記述する。

**確定フェーズ**: Phase 1a（Go Template の完全版）、Phase 1b（C# Template 完全版）。**対応要件**: DX-GP-001、DX-GP-002。**上流**: DS-SW-DOC-001、DS-SW-DOC-002。

## DS-IMPL-DIR-245 生成される tier2 repo のルート構造

Backstage Template 実行後、GitHub 上に生成される tier2 repo のルートは以下の構造を持つ。Go / C# / 将来の Java / Python いずれの言語でもルート構造は共通（言語差は `src/` 配下と `go.mod` / `*.csproj` 等に閉じ込める）。

```
k1s0-tier2-<domain>-<service>/
├── .github/
│   ├── workflows/ci.yml             # 6 段 CI（build/unit test/integration test/SAST/SBOM/image scan）
│   ├── CODEOWNERS                   # サービスチーム + tier1 基盤チーム
│   └── PULL_REQUEST_TEMPLATE.md
├── .devcontainer/
│   ├── devcontainer.json            # VS Code Dev Container 定義
│   └── Dockerfile                   # 言語ランタイム + k1s0 CLI
├── .editorconfig
├── .gitignore
├── .gitattributes
├── <言語別ソース>                    # Go: cmd/ + internal/ + go.mod、C#: src/ + *.sln
├── Dockerfile                       # 配信用 image（distroless、non-root）
├── docs/                            # TechDocs
│   ├── index.md
│   ├── api.md
│   └── runbook.md
├── mkdocs.yml                       # TechDocs ビルド設定
├── README.md
└── LICENSE                          # Apache-2.0（固定、k1s0 プラットフォームの OSS ポリシーに従う）
```

`k8s/` ディレクトリは**生成しない**。tier2 の Kubernetes manifest（Deployment / Service / HPA / NetworkPolicy 等）は tier1 の Helm Chart が umbrella として供給し、tier2 開発者は一切触らない（[DS-SW-DOC-002](../../04_概要設計/20_ソフトウェア方式設計/05_利用者文書_暫定版/01_tier2_tier3開発者向けGoldenPath初版.md) で決定済み）。これは監査要件（NetworkPolicy・SecurityContext が全 tier2 で統一されていることの機械的保証）を担保する。

**確定フェーズ**: Phase 1b（初の tier2 パイロット生成時）。**対応要件**: DX-GP-004、DX-GP-005、NFR-E-AC-\*、NFR-G-PROT-\*。**上流**: DS-SW-DOC-002。

## DS-IMPL-DIR-246 tier2 repo の Go 言語スケルトン

Go 版 tier2 repo のソースディレクトリ構造は、tier1 Go 側の 4 層分割（[03 章 DS-IMPL-DIR-043](03_Goモジュール詳細構成.md)）と**同じパターン**を踏襲する。同じパターンにすることで、tier1 を読み書きする開発者と tier2 を読み書きする開発者のメンタルモデルが揃い、tier1 と tier2 の境界を跨いだ調査（障害時の一気通貫トレース追跡など）で迷わない。

```
cmd/<service_name>/
└── main.go                          # エントリポイント、k1s0 SDK 初期化

internal/
├── handler/                         # gRPC または HTTP ハンドラ（tier2 側 API 公開層）
│   └── <resource>_handler.go
├── service/                         # ドメインサービス（ビジネスロジック）
│   └── <resource>_service.go
├── domain/                          # ドメインモデル（値オブジェクト / エンティティ）
│   └── <aggregate>.go
└── adapter/                         # 外部連携（k1s0 SDK 呼び出しなど）
    └── k1s0_adapter.go

go.mod
```

`main.go` の骨格は次のようになる（Template が変数展開を含めて生成）。

```go
// ${{values.service_name}} サービスのエントリポイント
package main

import (
    "context"
    "github.com/k1s0/k1s0-sdk-go/k1s0sdk"
)

func main() {
    // k1s0 SDK 初期化（認証・テナント ID・trace_id の自動付与機構を内蔵）
    client := k1s0sdk.NewClient(context.Background())
    // TODO: write business logic here
    _ = client
}
```

`internal/adapter/k1s0_adapter.go` には k1s0 SDK 呼び出しのラッパを配置し、ビジネスロジック側（`internal/service/`）は SDK を直接呼ばずこのアダプタ経由で使う。これは依存方向を（service → adapter → k1s0 SDK）に固定し、SDK MAJOR 更新時にビジネスロジックが影響を受けない層分離を担保する。

**確定フェーズ**: Phase 1a（Template）、Phase 1b（初回パイロット生成）。**対応要件**: DX-GP-006、NFR-C-NOP-001、ADR-TIER1-003。**上流**: DS-SW-DOC-003、DS-IMPL-DIR-043。

## DS-IMPL-DIR-247 tier2 repo の C# 言語スケルトン（Phase 1b）

C# 版 tier2 repo のソース構造は、.NET 慣例に従う。Go と直接同じパターンを強制するのではなく、.NET 開発者が違和感なく書ける構造にする。ただし層分離の思想（handler / service / domain / adapter の役割）は Go 版と同じで、名称と配置が .NET 風である違いにとどめる。

```
src/
├── <ServiceName>/                   # メインプロジェクト
│   ├── <ServiceName>.csproj
│   ├── Program.cs                   # エントリポイント
│   ├── Handlers/                    # REST / gRPC コントローラ
│   ├── Services/                    # ドメインサービス
│   ├── Domain/                      # ドメインモデル
│   └── Adapters/                    # k1s0 SDK 呼び出しラッパ
└── <ServiceName>.Tests/             # 単体テスト（xUnit）
    └── <ServiceName>.Tests.csproj

<ServiceName>.sln
```

`Program.cs` では Minimal API + Dependency Injection 経由で `K1S0.Sdk` を登録する。NuGet 依存は `K1S0.Sdk`（09 章 DS-IMPL-DIR-227）、`Grpc.Net.Client`、`Microsoft.Extensions.Hosting` を最小セットで含める。

C# 版は Phase 1b で導入するため、Template 自身も Phase 1b で完全版化する。Phase 1a 時点では `tools/backstage-templates/tier2/csharp/` は README のみを配置し、「Phase 1b 公開予定」と明記する。

**確定フェーズ**: Phase 1b。**対応要件**: DX-GP-006、ADR-TIER1-003。**上流**: DS-SW-DOC-003、DS-IMPL-DIR-227。

## DS-IMPL-DIR-248 tier2 repo の CI 設定（GitHub Actions）

`.github/workflows/ci.yml` は 6 段固定の CI を実行する（[DS-SW-DOC-002](../../04_概要設計/20_ソフトウェア方式設計/05_利用者文書_暫定版/01_tier2_tier3開発者向けGoldenPath初版.md) の Template 生成物規定）。6 段は以下。

1. `build`: 言語ごとのビルド（Go は `go build ./...`、C# は `dotnet build`）
2. `unit-test`: 単体テスト（Go は `go test ./...`、C# は `dotnet test`）
3. `integration-test`: tier1 モック（k1s0 repo の `tools/mock-server/`、07 章 DS-IMPL-DIR-186）を Testcontainers で起動して結合テスト
4. `sast`: Semgrep による静的解析（PII 漏洩疑いのログ出力検出を含む）
5. `sbom`: syft による SBOM 生成、GitHub Actions のアセットに保存
6. `image-scan`: Dockerfile から image をビルドし trivy で脆弱性スキャン

6 段は**削除禁止**（DS-IMPL-DIR-258）。tier2 開発者がスピード優先で特定段を skip する改変は PR レビューで reject する。段の追加（7 段目として性能テストなど）は tier2 サービスチーム判断で許容する。

CI の OIDC 認証 / GitHub Packages への読み取り権限 / secrets（k1s0 レジストリのトークン）の注入は Template が .github/workflows/ci.yml に組み込み済みで、tier2 開発者が secrets を個別設定する必要はない。secrets の注入は GitHub Actions の Organization-level secrets を利用する。

**確定フェーズ**: Phase 1b。**対応要件**: DX-CICD-\*、NFR-H-INT-\*、NFR-E-AC-\*。**上流**: DS-SW-DOC-002、DS-IMPL-DIR-186。

## DS-IMPL-DIR-249 tier2 repo の CODEOWNERS 雛形

`.github/CODEOWNERS` は Template の変数展開で以下を出力する。

```
# k1s0-tier2-<domain>-<service> CODEOWNERS
# サービスのドメインチームが全体を所有、tier1 基盤チームは SDK 呼び出し部分をレビュー

*                                    @k1s0/${{values.team_name}}
/internal/adapter/k1s0_adapter.go    @k1s0/${{values.team_name}} @k1s0/tier1-architects
/src/**/Adapters/                    @k1s0/${{values.team_name}} @k1s0/tier1-architects
/.github/workflows/                  @k1s0/devex-team
/.github/CODEOWNERS                  @k1s0/${{values.team_name}} @k1s0/tier1-architects
/Dockerfile                          @k1s0/${{values.team_name}} @k1s0/security-team
```

tier2 サービスのドメインチーム（`@k1s0/${{values.team_name}}`）がデフォルトオーナーで、SDK 呼び出しアダプタ・CI 設定・Dockerfile は tier1 基盤側の各チームがセカンダリオーナーとして参加する。Template は `team_name` パラメータからこの展開を自動生成する。

tier1 基盤チームのチーム名は [01 章 DS-IMPL-DIR-018](01_リポジトリルート構成.md) で定義した canonical 8 チームに限定し、Template が不明なチーム名を受け取った場合は生成時に fail させる（`template.yaml` の `parameters` で `team_name` を `enum` 制約にする）。

**確定フェーズ**: Phase 1b。**対応要件**: NFR-C-NOP-001、DX-CICD-\*。**上流**: DS-IMPL-DIR-018、DS-SW-DOC-002。

## DS-IMPL-DIR-250 tier2 repo の Dockerfile と container image

`Dockerfile` は distroless ベース・non-root・読み取り専用ファイルシステムの 3 点を**削除禁止**の必須要素として含む。Go 版は以下の multi-stage 構成。

```dockerfile
# Go ビルド stage
FROM golang:1.22 AS builder
WORKDIR /src
COPY . .
RUN go build -ldflags="-s -w" -trimpath -o /out/${{values.service_name}} ./cmd/${{values.service_name}}

# 配信 stage
FROM gcr.io/distroless/static-debian12:nonroot
COPY --from=builder /out/${{values.service_name}} /bin/app
USER nonroot:nonroot
ENTRYPOINT ["/bin/app"]
```

C# 版は `.NET 8` SDK イメージでビルドし、`mcr.microsoft.com/dotnet/runtime-deps:8.0-jammy-chiseled` 配信ベース（distroless 相当、non-root）を使う。image tag 規則は [08 章 DS-IMPL-DIR-213](08_命名規約と配置ルール.md) の Pod と同じ形式（`<registry>/tier2/<service>:<semver>-phase<N>`）に従う。

Dockerfile の禁止改変リスト: (a) ベースイメージを distroless 以外に変更（`alpine` / `ubuntu` 禁止）、(b) USER 指定を root に変更、(c) `RUN` での apt install 等のパッケージ追加、(d) `readOnlyRootFilesystem: false` を要するような書き込み前提の構成。これらは Kyverno ClusterPolicy ([05 章 DS-IMPL-DIR-140](05_infra詳細構成.md)) で admission 段階でも再度検証する二重防御。

**確定フェーズ**: Phase 1b。**対応要件**: NFR-E-AC-\*、NFR-H-INT-\*、ADR-CICD-003（Kyverno）。**上流**: DS-SW-DOC-002、DS-IMPL-DIR-213、DS-IMPL-DIR-140。

## DS-IMPL-DIR-251 tier2 repo の SDK 消費方法

tier2 repo は k1s0 SDK を**パッケージレジストリ経由**で取得する（[09 章 DS-IMPL-DIR-224](09_tier1全体配置とSDK境界.md)）。Go 版の `go.mod` は次の形で k1s0-sdk-go に依存する。

```go.mod
module github.com/k1s0/k1s0-tier2-${{values.domain}}-${{values.service_name}}

go 1.22

require (
    github.com/k1s0/k1s0-sdk-go v1.0.0-phase1b
)
```

Phase 1a は ghcr.io の Go module proxy、Phase 1b 以降は placeholder `nexus.k1s0.internal/go/` からの取得に切り替える（[09 章 DS-IMPL-DIR-232](09_tier1全体配置とSDK境界.md)）。Go module proxy の切替は `.netrc` ファイル（GitHub Actions の secrets で注入）と `GOPRIVATE=github.com/k1s0/*` 環境変数で制御する。

C# 版は `<ServiceName>.csproj` で `K1S0.Sdk` を NuGet 依存として宣言し、`nuget.config` で tier1 の NuGet レジストリを参照する。Phase 1b では ghcr.io の NuGet registry を使い、Phase 1c で Nexus に移行する。

tier2 側では **SDK 以外の k1s0 internal package を import することを禁止**する。禁止の強制は (a) Go は `internal/` の Go compiler 強制により物理的に不可能、(b) C# は CI で `dotnet list package --include-transitive` の出力を検査し `K1S0.Internal.*` が含まれたら fail させる、の 2 経路で行う。

**確定フェーズ**: Phase 1b。**対応要件**: ADR-TIER1-003、DX-GP-006。**上流**: DS-IMPL-DIR-224、DS-IMPL-DIR-226、DS-IMPL-DIR-227。

## DS-IMPL-DIR-252 tier2 repo の Dapr 未露出原則

tier2 開発者は Dapr の存在を**一切意識しない**（[DS-SW-DOC-008](../../04_概要設計/20_ソフトウェア方式設計/05_利用者文書_暫定版/01_tier2_tier3開発者向けGoldenPath初版.md)）。tier2 repo の中には以下のいずれも配置されない。

- `dapr.yaml` / `dapr-components/` / Dapr sidecar 設定
- Dapr SDK 依存（`github.com/dapr/go-sdk` / `Dapr.Client` NuGet）
- Dapr 固有の annotation を含む Kubernetes manifest（そもそも k8s manifest が tier2 repo には存在しない、DS-IMPL-DIR-245）

tier2 から tier1 への呼び出しは全て k1s0 SDK 経由で、SDK 内部が gRPC で tier1 facade Pod を呼ぶ。facade Pod の先にいる Dapr sidecar や Building Block の存在は SDK / facade レイヤで完全に隠蔽される。

Dapr の存在を隠蔽する理由は 2 つ。(a) Dapr は tier1 基盤チームの選定物（[ADR-CICD-001 〜 003](../../02_構想設計/adr/)）であり、Dapr のバージョン更新・API 変更・replacement（Dapr → Knative など、将来的な選択肢）は tier1 側で完結させたい。tier2 が Dapr SDK を直接使うと、tier1 の選定変更が全 tier2 の改修を要求するため結合度が爆発する。(b) tier2 開発者の採用条件に Dapr 知識を含めるのは市場的に現実的でなく、プラットフォーム知識の最小化が Golden Path の核心（[ADR-TIER1-003](../../02_構想設計/adr/ADR-TIER1-003-language-opacity.md)）。

**確定フェーズ**: Phase 1a（ルール）、Phase 1b（適用）。**対応要件**: ADR-TIER1-003、BR-PLATUSE-002、NFR-C-NOP-001。**上流**: DS-SW-DOC-008、DS-IMPL-DIR-222。

## DS-IMPL-DIR-253 tier2 repo のテスト配置

tier2 repo のテストは 3 カテゴリで配置する。

- 単体テスト: Go は `internal/<layer>/*_test.go`（_test.go 隣接）、C# は `src/<ServiceName>.Tests/`。
- 結合テスト: Go は `internal/integrationtest/`（build tag `integration`）、C# は `src/<ServiceName>.IntegrationTests/`。tier1 モック（`tools/mock-server/`）を Testcontainers で起動して呼び出し経路を検証。
- 契約テスト: tier2 側は Pact の Consumer として振る舞い、契約ファイルを `tests/contract/` に生成。契約ファイルは k1s0 repo の `tests/contract/consumers/`（06 章 DS-IMPL-DIR-166）に pact JSON としてコミット（Phase 1b は手動 publish、Phase 1c 以降で Pact ブローカー連携）。

E2E テストは tier2 repo では行わず、k1s0 repo の `tests/e2e/scenarios/cross_pod/` で横断 E2E として実装する（tier2 → tier1 の呼び出し疎通確認）。これは tier2 repo 側で E2E 基盤を各 repo に持たせると運用コストが合計で膨張するため、E2E は tier1 側に集約する方針。

**確定フェーズ**: Phase 1b。**対応要件**: DX-TEST-\*、DX-CICD-\*、NFR-C-NOP-001。**上流**: DS-IMPL-DIR-058、DS-IMPL-DIR-166、DS-IMPL-DIR-186。

## DS-IMPL-DIR-254 tier2 repo の docs/（TechDocs）配置

`docs/` は Backstage TechDocs の source として配置し、最低限 3 ファイル（`index.md` / `api.md` / `runbook.md`）を Template で生成する。

- `index.md`: サービスの目的・責任範囲・関連サービス・オーナーチーム。
- `api.md`: tier2 サービスが公開する API（REST / gRPC）の一覧、リクエスト / レスポンス例、認証・認可、レート制限。
- `runbook.md`: 障害時のチェックリスト、典型的なエラーパターン、連絡先（one-pager）。

`mkdocs.yml` は TechDocs のビルド設定を規定し、Backstage がこの設定を読み取って TechDocs UI を生成する。これも Template が Phase 1b 時点の最新構成を出力する。

docs/ の 3 ファイルは**削除禁止**だが、内容は各 tier2 サービスチームが埋める。Template 生成時点では「TODO: このサービスの目的を記述する」などの埋め込みコメントが入っており、空のまま本番リリースすると Backstage Catalog の「ドキュメント未整備」アラートで検出される。

**確定フェーズ**: Phase 1b。**対応要件**: DX-DEVEX-\*、NFR-SUP-\*、NFR-C-NOP-001。**上流**: DS-SW-DOC-002、DS-SW-DOC-006。

## DS-IMPL-DIR-255 tier2 ローカル開発（k1s0 dev up）との統合

tier2 開発者がローカルで `k1s0 dev up`（07 章 DS-IMPL-DIR-184）を実行すると、以下が自動で起動する。

1. Testcontainers 上で tier1 mock-server（11 API のモック、`tools/mock-server/`）
2. Testcontainers 上で Dapr sidecar（daprd、tier2 からは見えない）
3. 自 tier2 container（Dockerfile でビルドしたもの、hot reload 対応）

tier2 repo の `.devcontainer/devcontainer.json` には `k1s0` CLI を起動時に install する設定が含まれ、開発者はこの CLI を通じて tier1 モックの起動・停止・リセットを操作する。これによりローカルで Kubernetes・Kafka・Valkey・Postgres を一切立てずに、コンテナ内に閉じた開発環境が完結する（[DS-SW-DOC-001 ステップ 3](../../04_概要設計/20_ソフトウェア方式設計/05_利用者文書_暫定版/01_tier2_tier3開発者向けGoldenPath初版.md)）。

**確定フェーズ**: Phase 1b（tier2 Pilot 時点で利用可能）。**対応要件**: DX-LD-\*、DX-GP-001。**上流**: DS-IMPL-DIR-184、DS-IMPL-DIR-186。

## DS-IMPL-DIR-256 tier2 repo の .devcontainer

`.devcontainer/devcontainer.json` は VS Code Dev Container の定義で、言語ランタイムのバージョン・必要な CLI・VS Code 拡張を固定する。

Go 版の最小例:

```json
{
  "name": "${{values.service_name}}",
  "build": { "dockerfile": "Dockerfile" },
  "features": {
    "ghcr.io/devcontainers/features/go:1": { "version": "1.22" }
  },
  "postCreateCommand": "k1s0 doctor",
  "customizations": {
    "vscode": { "extensions": ["golang.go"] }
  }
}
```

`postCreateCommand: k1s0 doctor` は 07 章 DS-IMPL-DIR-184 の CLI で、ローカル環境の前提（Docker / Go / C# ランタイム / k1s0 SDK バージョン）を自動チェックする。初回起動時に前提が満たされていなければ失敗し、開発者にインストール手順を示す。

**確定フェーズ**: Phase 1b。**対応要件**: DX-LD-\*、DX-DEVEX-\*。**上流**: DS-SW-DOC-002、DS-IMPL-DIR-184。

## DS-IMPL-DIR-257 tier2 repo の GitOps 配信（k1s0-gitops）

tier2 repo の CI は main ブランチマージ時に image build + SBOM 生成 + GitHub Release を作るが、**Kubernetes manifest を tier2 repo に持たない**（DS-IMPL-DIR-245）ため、デプロイは GitOps repo（`k1s0-gitops`）経由で行う。具体的には tier2 の image tag が更新された時点で、tier2 CI が `k1s0-gitops` repo に「`tier2/<service>/values.yaml` の `image.tag` を更新する PR」を自動作成する。

Argo CD はこの PR がマージされると自動で dev 環境へ sync し、Kubernetes 上に tier2 Pod を起動する。tier2 の Kubernetes Deployment / Service / HPA / NetworkPolicy / RBAC は tier1 の Helm Chart（`tier1-umbrella`）が umbrella として一括提供する。具体的には [05 章 DS-IMPL-DIR-124](05_infra詳細構成.md) で umbrella Chart の subchart として将来 `tier2-service` を追加することが予告されており、Phase 1b のパイロット時に subchart 1 本を追加する。

この GitOps 設計により、tier2 開発者は「main マージ + CI 通過 = Argo CD が自動配信」という単純なメンタルモデルで開発でき、Kubernetes manifest の書き方を覚える必要がない（Golden Path 10 分ルールの前提）。

**確定フェーズ**: Phase 1b。**対応要件**: DX-CICD-\*、DX-GP-001、NFR-C-NOP-001。**上流**: DS-SW-DOC-001、DS-IMPL-DIR-124、DS-IMPL-DIR-192。

## DS-IMPL-DIR-258 tier2 repo の生成後の禁止変更と Template Drift Detection

Template が生成した直後の骨格のうち、以下は**削除・改変禁止**とする。改変が必要な場合は基盤チーム（`@k1s0/tier1-architects`）への事前相談が必須。

- `.github/workflows/ci.yml`: 6 段 CI（DS-IMPL-DIR-248）から段を削除する改変は禁止。段の追加は OK。
- `.github/CODEOWNERS`: tier1 基盤チームをセカンダリオーナーから外す改変は禁止。
- `Dockerfile`: distroless ベース・non-root・readOnlyRootFilesystem の 3 要件を崩す改変は禁止（DS-IMPL-DIR-250）。
- `docs/`: 3 ファイル（`index.md` / `api.md` / `runbook.md`）の削除は禁止。内容の充実は歓迎。
- `internal/adapter/k1s0_adapter.go`（Go）/ `src/<ServiceName>/Adapters/`（C#）: k1s0 SDK 呼び出しのラッパ層で、削除すると SDK を直接使う実装になり SDK MAJOR 更新時の影響が分散する。削除禁止、構造変更も慎重に。

**Template Drift Detection の 3 層防御**: 禁止変更は個別の tier2 サービスチームが「スピード優先」の判断で小さく崩すことが最も起きやすい drift の発生源である。Backstage Template を採用する運用で最大のリスクは、**生成直後は準拠しているが時間経過と共に各 tier2 repo が独自進化して Template 改版の恩恵を受けられなくなる**ことである。これを防ぐため、検出経路を以下の 3 層で重ね合わせる。

- **Layer A — 生成時の固定化（fingerprint 埋込）**: Backstage Template の最終ステップで、生成された tier2 repo の `catalog-info.yaml` に `metadata.annotations."k1s0.io/template-version"` と `metadata.annotations."k1s0.io/template-commit-sha"` を埋め込む。さらに、禁止改変対象の各ファイル（`ci.yml` / `CODEOWNERS` / `Dockerfile` / `docs/{index,api,runbook}.md` / `k1s0_adapter.go`）について生成時の SHA256 を `.k1s0/template-manifest.json` に記録し、tier2 repo に commit する。このマニフェスト自体も改変禁止対象に加える（マニフェスト削除 = 高リスクな drift 兆候）。
- **Layer B — admission 時の検査（k1s0-gitops 側）**: Kyverno ClusterPolicy で、tier2 image から起動される Pod の `distroless 以外の baseimage`・`runAsNonRoot != true`・`readOnlyRootFilesystem != true` を admission reject する。これにより Dockerfile の骨抜き改変が本番 apply まで到達する経路を物理的に塞ぐ（DS-IMPL-DIR-140 の Kyverno 7 ポリシーの 1 つとして発現）。
- **Layer C — 定期 drift detector（k1s0-template-drift-detector）**: `tools/template-drift-detector/` を Phase 1b で追加する。GitHub Actions の scheduled workflow として **k1s0 モノレポ側で日次実行**し、次を実施する。(1) Backstage Catalog から tier2 repo 一覧を取得、(2) 各 repo の `.k1s0/template-manifest.json` と現在の禁止改変対象ファイルの SHA256 を照合、(3) drift があれば Severity を判定（`high`: CI 段削除 / Dockerfile 要件崩し / adapter 削除、`medium`: CODEOWNERS から基盤チーム外し、`low`: docs 削除）、(4) `high` は対象 tier2 repo の main 保護設定と GitOps 配信を即座に保留（Backstage Catalog の `lifecycle` を `deprecated` に自動遷移）、(5) `medium` / `low` は対象 repo に「Template 準拠への復元 PR」を `k1s0-template-drift-bot` アカウントから自動作成、(6) 結果を週次で `@k1s0/tier1-architects` に報告。

**Template 改版への追従**: Template 自体が改版（例: CI 6 段→7 段の段追加、adapter ラッパのインターフェイス変更）された時、既存 tier2 repo を自動で追従させる経路が必要になる。Phase 1c で `k1s0-template-migrate-bot` を追加し、Template 改版 PR が main にマージされた時点で、Layer A の `template-version` が古い全 tier2 repo に対して自動でマイグレーション PR を作成する。マイグレーションは差分適用可能な範囲（ci.yml の段追加、Dockerfile の baseimage バージョン更新など）に限定し、破壊的変更（adapter インターフェイス変更）は必ず人手レビューを通す。Template 改版時は DS-IMPL-DIR-260 の ADR 起票に「マイグレーション戦略」節の記述を必須とする。

**確定フェーズ**: Phase 1b（Layer A / B / C の 3 層防御確立）、Phase 1c（Template 改版マイグレーション bot 追加）。**対応要件**: NFR-E-AC-\*、NFR-H-INT-\*、NFR-C-NOP-001、NFR-SUP-\*、ADR-CICD-003（Kyverno）。**上流**: DS-SW-DOC-002、DS-IMPL-DIR-140、DS-IMPL-DIR-244、DS-IMPL-DIR-259、DS-IMPL-DIR-260。

## DS-IMPL-DIR-259 tier2 repo 命名と Backstage Catalog への登録

tier2 repo の命名は [09 章 DS-IMPL-DIR-235](09_tier1全体配置とSDK境界.md) で `k1s0-tier2-<domain>-<service>` と確定済み。Backstage Software Template は生成時にこの命名規則を強制し、違反する名前は `template.yaml` の `parameters` バリデーションで reject する。

生成された tier2 repo は自動で Backstage Catalog に登録される（Template の最終ステップが `publish:github` + `catalog:register` Action を実行）。Catalog 登録により、Backstage ポータルから tier2 サービスの一覧・ownership・dependency グラフ・TechDocs がすべて参照可能になる。

Catalog 登録時のメタデータ（`catalog-info.yaml`）には次を記録する。

- `name`: service_name
- `owner`: team_name
- `type`: `service`
- `lifecycle`: `production` / `experimental` / `deprecated` のいずれか（Template 生成時のデフォルトは `experimental`）
- `system`: 「tier2」（システム分類）
- `providesApis`: tier2 サービスが公開する API 一覧
- `consumesApis`: tier1 のどの API を消費するか（tier1 公開 11 API のうちのサブセット）

`consumesApis` は Template 選択時の入力 `handles_pii` や追加入力（`uses_state_api` 等）から自動生成する。これにより Backstage 上で「tier1 State API を消費している tier2 サービス一覧」のような動的検索が可能になる（Phase 1c 以降の利用想定）。

**確定フェーズ**: Phase 1b。**対応要件**: DX-GP-001、NFR-C-NOP-001、NFR-SUP-\*。**上流**: DS-SW-DOC-001、DS-IMPL-DIR-235、DS-IMPL-DIR-185。

## DS-IMPL-DIR-260 tier2 Template 変更時の ADR 起票条件

`tools/backstage-templates/tier2/*/` 配下の Template 変更のうち、以下は ADR を要する。軽微な変更は ADR 不要で `@k1s0/devex-team` + `@k1s0/tier1-architects` のレビューで通す。

1. 新規 tier2 言語 Template の追加（現状: Go / C# 以外の言語追加）
2. skeleton の構造変更（4 層分割 handler / service / domain / adapter の変更）
3. 生成される CI の 6 段構成の変更（DS-IMPL-DIR-248 の段数変更）
4. 生成される Dockerfile のベースイメージ変更
5. 生成される CODEOWNERS の tier1 基盤チーム外し
6. 生成される docs/ の必須ファイル 3 本（index / api / runbook）の変更
7. SDK 依存先の変更（GitHub Packages → Nexus 移行は Phase 1c で別 ADR）

これら ADR は `docs/02_構想設計/adr/ADR-TIER2-<NNN>-<title>.md` 形式で起票する（[08 章 DS-IMPL-DIR-218](08_命名規約と配置ルール.md) のカテゴリ拡張、`TIER2` カテゴリを新規追加）。

**確定フェーズ**: Phase 0（ルール）、各変更時（適用）。**対応要件**: NFR-C-NOP-001、DX-CICD-\*、ADR-TIER1-003。**上流**: DS-SW-COMP-138、DS-IMPL-DIR-218。

## 章末サマリ

### 設計 ID 一覧

| 設計 ID | 内容 | 確定フェーズ |
|---|---|---|
| DS-IMPL-DIR-241 | 本章の位置付け（tier2 配置の範囲） | Phase 0 |
| DS-IMPL-DIR-242 | tier2 repo は別 GitHub repo（polyrepo） | Phase 0 |
| DS-IMPL-DIR-243 | Backstage Template ソースの配置 | Phase 1a/1b |
| DS-IMPL-DIR-244 | tier2 Template の構造 | Phase 1a/1b |
| DS-IMPL-DIR-245 | 生成される tier2 repo のルート構造 | Phase 1b |
| DS-IMPL-DIR-246 | tier2 repo の Go 言語スケルトン | Phase 1a/1b |
| DS-IMPL-DIR-247 | tier2 repo の C# 言語スケルトン | Phase 1b |
| DS-IMPL-DIR-248 | tier2 repo の CI 設定（GitHub Actions） | Phase 1b |
| DS-IMPL-DIR-249 | tier2 repo の CODEOWNERS 雛形 | Phase 1b |
| DS-IMPL-DIR-250 | tier2 repo の Dockerfile と container image | Phase 1b |
| DS-IMPL-DIR-251 | tier2 repo の SDK 消費方法 | Phase 1b |
| DS-IMPL-DIR-252 | tier2 repo の Dapr 未露出原則 | Phase 1a/1b |
| DS-IMPL-DIR-253 | tier2 repo のテスト配置 | Phase 1b |
| DS-IMPL-DIR-254 | tier2 repo の docs/（TechDocs）配置 | Phase 1b |
| DS-IMPL-DIR-255 | tier2 ローカル開発との統合 | Phase 1b |
| DS-IMPL-DIR-256 | tier2 repo の .devcontainer | Phase 1b |
| DS-IMPL-DIR-257 | tier2 repo の GitOps 配信 | Phase 1b |
| DS-IMPL-DIR-258 | tier2 repo 生成後の禁止変更と Template Drift Detection | Phase 1b/1c |
| DS-IMPL-DIR-259 | tier2 repo 命名と Backstage Catalog 登録 | Phase 1b |
| DS-IMPL-DIR-260 | tier2 Template 変更時の ADR 起票条件 | Phase 0 |

### 対応要件一覧

- BR-PLATUSE-001（単一プラットフォーム）、BR-PLATUSE-002（透過的動作）
- DX-GP-001（Golden Path 10 分）、DX-GP-002（Template 選択）、DX-GP-004（禁止改変）、DX-GP-005（監査要件）、DX-GP-006（SDK 1 行呼び出し）、DX-TEST-\*、DX-CICD-\*、DX-LD-\*、DX-DEVEX-\*
- NFR-C-NOP-001（2 名運用）、NFR-E-AC-\*、NFR-G-PROT-\*、NFR-H-INT-\*、NFR-SUP-\*
- ADR-TIER1-003（内部言語不透明性）、ADR-CICD-003（Kyverno）

### 上流設計 ID

DS-SW-DOC-001（Golden Path 全体手順）、DS-SW-DOC-002（Template 生成物）、DS-SW-DOC-003（SDK 1 行呼び出し）、DS-SW-DOC-006（参考リンク）、DS-SW-DOC-008（SDK 自動付与）、DS-SW-COMP-138（変更手続）、DS-IMPL-DIR-018（CODEOWNERS 母集団）、DS-IMPL-DIR-043（Go 4 層）、DS-IMPL-DIR-058（Go integrationtest）、DS-IMPL-DIR-124（Helm umbrella）、DS-IMPL-DIR-140（Kyverno policy）、DS-IMPL-DIR-166（contract テスト）、DS-IMPL-DIR-184（k1s0 dev up）、DS-IMPL-DIR-185（backstage/）、DS-IMPL-DIR-186（mock-server）、DS-IMPL-DIR-192（release.yml）、DS-IMPL-DIR-213（image tag）、DS-IMPL-DIR-218（ADR 命名）、DS-IMPL-DIR-222（polyrepo 方針）、DS-IMPL-DIR-224（SDK 配布）、DS-IMPL-DIR-226（Go SDK wrapper）、DS-IMPL-DIR-227（C# SDK）、DS-IMPL-DIR-232（SDK レジストリ）、DS-IMPL-DIR-235（tier2 repo 命名）。
