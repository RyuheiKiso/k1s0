# ADR-TEST-010: e2e test-fixtures を 4 言語 SDK と同 module / 同 version で同梱し、kind 起動 + k1s0 install + mock builder を提供する

- ステータス: Proposed
- 起票日: 2026-05-03
- 決定日: -
- 起票者: kiso ryuhei
- 関係者: 起案者 / 採用検討組織 / 利用者（採用初期）

## コンテキスト

ADR-TEST-008 で e2e テストを `tests/e2e/owner/`（オーナー専用）と `tests/e2e/user/`（利用者向け）に物理分離した。利用者は自分の tier2 / tier3 アプリを k1s0 SDK 越しに動作確認するが、kind 起動 + k1s0 install + tier1 facade 起動 + tenant context 注入 + 認証 JWT 注入 + mock data 投入の一連を「ゼロから利用者が組む」と、k1s0 内部の起動順序や依存関係を学ばないと test が書けない。これは利用者 DX を破壊する。

加えて、k1s0 SDK は 4 言語（Go / Rust / .NET / TypeScript）で並立しており、各言語の test framework（Go test / cargo test / xUnit / Vitest）とイディオムは大きく異なる。利用者が「自分が使っている言語」で test を書けないと、SDK を採用するハードルが上がる。

両者を解決する経路として、k1s0 SDK と同梱する `test-fixtures` ライブラリを 4 言語で提供し、利用者が import するだけで kind 起動 + k1s0 install + SDK client init を 1 行で立てられる構造が必要である。問題は **versioning と同期コスト** で、3 つの実用的な選択肢の対立がある。

第一に **配置と versioning**。SDK と同 module / 同 version（A 案、本 ADR で採用）/ 別 module / 別 version cycle（B 案）/ owner suite 内に閉じて利用者非公開（C 案）/ 提供しない（D 案）。

第二に **mock builder の射程**。kind 起動 + install まで（最小、X 案）/ + SDK client init（中、Y 案）/ + 全 12 service の mock builder（フル、Z 案、本 ADR で採用）。

第三に **二重提供（tier3-web）の責務**。owner Go(chromedp) / user TS(Playwright) を ADR-TEST-008 で確定したが、TS test-fixtures の Playwright 統合がどこまで踏み込むかが未確定。

設計上の制約と前提:

- 4 言語 SDK は `src/sdk/{go,rust,dotnet,typescript}/` に配置済（ADR-DIR-* 系列）
- 各 SDK は独自の release cycle / version 管理を持つが、本 ADR で test-fixtures を SDK と同 module / 同 version とすることで一致化する
- Go: `src/sdk/go/k1s0/` の同 module に `test-fixtures/` package を追加
- Rust: `src/sdk/rust/` workspace member として `k1s0-test-fixtures` crate を追加
- .NET: `src/sdk/dotnet/Sdk.sln` に `K1s0.Sdk.TestFixtures.csproj` を追加
- TypeScript: `src/sdk/typescript/` monorepo の `@k1s0/sdk-test-fixtures` package として追加
- testcontainers / kind / Vitest / Playwright / xUnit / Go testing の各イディオムに同梱が整合する必要
- ADR-TEST-008 の二重提供（tier3-web）と整合: owner は Go(chromedp)、利用者は TS(Playwright) が前提
- 個人 OSS の運用工数で 4 言語 fixtures を起案者 1 人で維持できる射程

選定では以下を満たす必要がある:

- **利用者 DX**: 利用者が import 1 行で kind 起動 + k1s0 install + SDK client init が立つ
- **SDK との版整合**: SDK API 変更時に test-fixtures が壊れない / 壊れたら同 release で同時修正される
- **4 言語の対称性**: 4 言語の fixtures が同じ責務を覆い、利用者がどの言語でも同等の DX を得る
- **mock builder の現実性**: 全 12 service の mock builder を 4 言語で完備する実装工数が個人 OSS で吸収できる
- **owner / user 二分との整合**: ADR-TEST-008 で owner / user を物理分離した責務境界を破らない

## 決定

**`test-fixtures` を 4 言語の SDK と同 module / 同 version で同梱する。射程は (1) kind 起動 + k1s0 minimum stack install、(2) SDK client init、(3) 全 12 service の mock builder、(4) common wait / assertion helper、(5) tier3-web の Playwright fixture（TS のみ）の 5 領域。** SDK release cycle に test-fixtures を内包し、SDK の major / minor / patch version を test-fixtures が継承する。利用者は自分のアプリ repo で SDK を import する時、test ターゲットでのみ test-fixtures を opt-in で import する。

### 1. 配置と命名

各言語の SDK package / module 構造に従い、test-fixtures を同梱する。

| 言語 | 配置 | import 名 | 提供形態 |
|---|---|---|---|
| Go | `src/sdk/go/k1s0/test-fixtures/` | `github.com/k1s0/sdk-go/k1s0/test-fixtures` | 同 module 内 sub-package、go.mod は SDK と共有 |
| Rust | `src/sdk/rust/test-fixtures/` | `k1s0-test-fixtures` crate | workspace member、Cargo.toml で `[features]` に `test-fixtures` を切り、test ターゲットでのみ enable |
| .NET | `src/sdk/dotnet/K1s0.Sdk.TestFixtures/` | `K1s0.Sdk.TestFixtures` namespace | 別 .csproj、Sdk.sln 同梱、NuGet package として `K1s0.Sdk.TestFixtures` で公開 |
| TypeScript | `src/sdk/typescript/packages/test-fixtures/` | `@k1s0/sdk-test-fixtures` | monorepo package、SDK 本体 `@k1s0/sdk` の peerDependency |

各言語で同じ責務を覆う API を提供し、関数名 / 引数の対称性を保つ（後述 §3）。

### 2. versioning ルール

SDK と test-fixtures は **同 SemVer / 同 release tag**。SDK v1.2.3 は test-fixtures v1.2.3 と組で release される。release cycle は以下:

- SDK API 変更（major / minor）時、test-fixtures も同 release で追従更新が必須
- test-fixtures bug fix（patch）でも SDK 本体を patch bump して同 release で出す
- Go: `src/sdk/go/k1s0/` 全体の go.mod tag が両者の version 源
- Rust: `src/sdk/rust/Cargo.toml` workspace の `[workspace.package].version` が両者の version 源
- .NET: `src/sdk/dotnet/Directory.Build.props` の `<Version>` が両者の version 源
- TypeScript: `src/sdk/typescript/lerna.json` または `package.json` workspace version が両者の version 源

利用者は SDK を pin した version で test-fixtures も自動的に同 version を取得する（同梱のため）。SDK と test-fixtures の version drift を物理的に発生させない構造とする。

### 3. API 対称性（4 言語で共通の責務）

各言語 fixtures は 5 領域の API を対称形で提供する。関数名は言語イディオムに合わせて命名するが、責務と引数は対称化する。

#### 領域 1: kind 起動 + k1s0 minimum stack install

```go
// Go
fx := testfixtures.Setup(t, testfixtures.Options{
    KindNodes: 2,
    Stack:     testfixtures.MinimumStack,
})
defer fx.Teardown()
```

```rust
// Rust
let fx = testfixtures::setup(testfixtures::Options {
    kind_nodes: 2,
    stack: testfixtures::Stack::Minimum,
}).await?;
```

```csharp
// .NET (xUnit fixture)
public class K1s0TestFixture : IAsyncLifetime {
    public async Task InitializeAsync() {
        await Fixtures.SetupAsync(new Options { KindNodes = 2, Stack = Stack.Minimum });
    }
}
```

```typescript
// TypeScript (Vitest)
beforeAll(async () => {
    await fixtures.setup({ kindNodes: 2, stack: 'minimum' });
});
```

`Setup` は kind cluster 起動（既存なら再利用）+ Dapr install + tier1 facade install + Keycloak install を冪等に実行。`Teardown` は test 後の cleanup（cluster 削除 or namespace 削除を opt-in）。

#### 領域 2: SDK client init

各言語の SDK client（既存）を fixtures が wrap し、tenant context / auth JWT / endpoint を test 用に inject。

#### 領域 3: 全 12 service の mock builder

12 service（State / Audit / PubSub / Workflow / Decision / Pii / Feature / Telemetry / Log / Binding / Secret / Invoke）の各 RPC に対応する mock data builder を提供。

```go
// 例: Go
audit := fx.MockBuilder.Audit().
    WithTenant("test-tenant").
    WithEntries(10).
    Build(t)
```

builder は 4 言語で同じ chain pattern（fluent API）で提供し、生成される mock data は **言語間で同一の wire 形式**（Protobuf 経由）に揃える。これにより、Go fixtures で生成した mock data を Rust SDK で読み込む cross-language test も成立する。

#### 領域 4: common wait / assertion helper

readiness 確認 / Pod の Ready 待機 / metric 値の到達待機 / log 出現待機などの helper を 4 言語で提供。Go は `testify/require` 互換、Rust は `assert!` macro 互換、.NET は `Assert.Equal` 互換、TS は Vitest の `expect` 互換。

#### 領域 5: tier3-web の Playwright fixture（TypeScript のみ）

TypeScript fixtures は Playwright 統合を提供。`fixtures.browserContext()` で k1s0 認証済の Playwright `BrowserContext` を返し、利用者が tier3 web frontend を test する時に Keycloak ログイン handshake を fixtures が肩代わりする。owner suite の `tests/e2e/owner/tier3-web/`（Go + chromedp）は本 fixtures を使わず独立した経路（ADR-TEST-008 §7）。

### 4. mock builder の段階展開

12 service × 4 言語 = 48 builder の即時完備は実装工数が爆発するため、段階展開する。

| Phase | 範囲 |
|---|---|
| リリース時点 | State / Audit / PubSub の 3 service × 4 言語 = 12 builder（最頻 RPC のみ）|
| 採用初期 | Workflow / Decision / Secret 追加（6 service × 4 言語 = 24 builder） |
| 採用後の運用拡大時 | 残 6 service（Pii / Feature / Telemetry / Log / Binding / Invoke）追加で 12 service × 4 言語 = 48 完備 |

3 service 先行の選定理由は、(a) State は ADR-TIER1-001 で「最も使われる service」と定義済、(b) Audit は WORM hash chain の verify 検証が必要で fixture 提供価値が高い、(c) PubSub は async 性質で test-fixtures なしでは利用者が組みにくい、の 3 点。

### 5. 提供経路と使い方

利用者は自分のアプリ repo で SDK を import する時、test ターゲットでのみ test-fixtures を opt-in で import する。

```go
// Go: 利用者の go.mod
require (
    github.com/k1s0/sdk-go/k1s0 v1.2.3
)
// _test.go ファイルでのみ
import "github.com/k1s0/sdk-go/k1s0/test-fixtures"
```

```rust
// Rust: 利用者の Cargo.toml
[dependencies]
k1s0-sdk = "1.2.3"

[dev-dependencies]
k1s0-test-fixtures = "1.2.3"
```

```xml
<!-- .NET: 利用者の Project.csproj -->
<PackageReference Include="K1s0.Sdk" Version="1.2.3" />
<PackageReference Include="K1s0.Sdk.TestFixtures" Version="1.2.3" />
```

```json
// TypeScript: 利用者の package.json
{
  "dependencies": { "@k1s0/sdk": "1.2.3" },
  "devDependencies": { "@k1s0/sdk-test-fixtures": "1.2.3" }
}
```

利用者の README や Golden Path examples（`docs/05_実装/50_開発者体験設計/20_Golden_Path_examples/`）から本 fixtures の使い方を導線で示す。

## 検討した選択肢

### 選択肢 A: SDK と同 module / 同 version で同梱（採用）

- 概要: 4 言語の SDK 本体と test-fixtures を同 module / 同 release tag で公開、SDK release で test-fixtures が必ず追従
- メリット:
  - **version drift が物理的に発生しない**: SDK と fixtures の version は構造的に一致
  - 利用者が pin した SDK version で fixtures も自動的に同 version 取得、依存解決が単純
  - SDK API 変更時に fixtures が compile error で即検知され、修正漏れが起きない
  - 4 言語で同 release cycle が走るため、cross-language test の整合性が保てる
- デメリット:
  - test-fixtures bug fix だけでも SDK 本体を patch bump する必要、release 頻度が増える（mitigation: SDK の patch release は cheap、Renovate で利用者側の更新も自動化）
  - SDK の最小依存に testcontainers / kind orchestrator が含まれる懸念（mitigation: Go: `// +build test_fixtures`、Rust: `[features]` 切り、.NET: 別 csproj、TS: 別 package で test target のみで pull）

### 選択肢 B: 別 module / 別 release cycle

- 概要: test-fixtures を SDK とは別 module / 別 version で release（例: SDK v1.2.3 と fixtures v0.7.1 の組）
- メリット:
  - SDK の release 頻度を test-fixtures bug fix で増やさずに済む
  - SDK 本体の依存に testcontainers が紛れ込まない
- デメリット:
  - **version drift が発生する**: SDK v1.3.0 で API 変更があっても fixtures v0.7.1 は古い API を参照、利用者が組み合わせ matrix を解く必要
  - 4 言語 × 2 release cycle = 8 release cycle、起案者 1 人運用で破綻
  - cross-language test で「Go fixtures v0.7.1 と Rust fixtures v0.8.0」の整合が取れない

### 選択肢 C: owner suite 内に閉じて利用者非公開

- 概要: test-fixtures を `tests/e2e/owner/helpers/` のような owner 専用 package に閉じ、利用者には提供しない
- メリット:
  - 4 言語提供が不要、Go のみで起案者の運用工数最小
  - SDK 本体の依存に test 関連が一切混入しない
- デメリット:
  - **利用者 DX が破壊**: 利用者が k1s0 SDK で test を書く時、kind 起動 + k1s0 install + SDK client init を ゼロから実装する必要
  - ADR-TEST-008 の user suite が機能しない: `tests/e2e/user/` の test を書く時 fixtures がなく、smoke test すら書けない
  - 採用検討者が「k1s0 で自アプリの test を書く時の DX が悪い」と判定し、採用判断にネガティブ

### 選択肢 D: test-fixtures を提供しない

- 概要: test-fixtures を一切提供せず、利用者が自分で kind / testcontainers を組む
- メリット:
  - 起案者の実装工数ゼロ
  - SDK の依存が最小
- デメリット:
  - **ADR-TEST-008 の user suite が成立しない**: smoke test を書くにも k1s0 内部の起動順序を学ぶ必要、ADR-TEST-008 の user 側責務が空洞化
  - 利用者の自アプリ test を書く DX が破壊、SDK 採用ハードルが上昇
  - Golden Path examples で test を含めない選択肢になり、example の完成度が下がる

## 決定理由

選択肢 A（SDK と同 module / 同 version で同梱）+ Z（全 12 service の mock builder、段階展開）を採用する根拠は以下。

- **version drift の構造的回避**: SDK と fixtures の version 一致を ADR レベルで強制することで、利用者が組み合わせ matrix を解く必要がなくなる。選択肢 B は drift を運用で吸収する設計で、4 言語 × 2 release cycle = 8 release cycle が起案者 1 人運用で破綻
- **ADR-TEST-008 の user suite との整合**: ADR-TEST-008 で `tests/e2e/user/smoke/` を CI 機械検証する設計を確定したが、fixtures なしでは smoke test が書けず、user suite が空洞化する。選択肢 C / D は ADR-TEST-008 と矛盾
- **利用者 DX**: import 1 行で kind 起動 + k1s0 install + SDK client init が立つ DX は SDK 採用判断のクリティカルパス。選択肢 C / D はこの DX を提供しない
- **mock builder の段階展開**: リリース時点 3 service / 採用初期 +3 / 運用拡大時 +6 の段階展開で、起案者 1 人の実装工数を吸収。即時 48 完備は工数爆発、ゼロは ADR-TEST-008 の user suite と矛盾、段階展開のみが個人 OSS の運用工数で持続可能
- **4 言語対称性**: 4 言語で同じ API 責務（5 領域）を覆うことで、採用組織がどの言語でも同等の DX を得る。これは ADR-TIER1-001（4 言語ハイブリッド）の射程と整合
- **退路の確保**: 選択肢 A は将来 fixtures が SDK 本体から分離する余地を持つ（同 module 内 sub-package を独立 module に切り出すのは Go / Rust / .NET / TS 全てで可能）。最初から分離する選択肢 B より退路が広い

## 影響

### ポジティブな影響

- ADR-TEST-008 の user suite が fixtures を使って成立し、smoke test / examples test が書けるようになる
- 利用者が import 1 行で kind 起動 + k1s0 install + SDK client init を立てられ、SDK 採用ハードルが下がる
- SDK と fixtures の version 一致が構造的に強制され、drift が物理的に発生しない
- 4 言語で対称な API を提供し、採用組織がどの言語でも同等の DX を得る
- mock builder の段階展開で、リリース時点 3 service の最小成立形から運用拡大時 12 完備まで継続的に拡張できる
- cross-language test（Go fixtures で生成した mock を Rust SDK で読む）が成立し、4 言語 SDK の cross-product 検証経路が開く

### ネガティブな影響 / リスク

- test-fixtures bug fix で SDK 本体を patch bump する必要、SDK release 頻度が増加（mitigation: SDK の patch release は cheap、Renovate で利用者側更新を自動化）
- 4 言語 × 12 service = 48 mock builder の保守が起案者の継続工数（mitigation: 段階展開でリリース時点 3 service / 採用初期 +3 / 運用拡大時 +6 と分散、SRE 増員後に分担）
- SDK 本体の依存 graph に testcontainers / kind orchestrator が紛れ込むリスク（mitigation: Go build tag / Rust feature / .NET 別 csproj / TS 別 package で test target のみ pull）
- TypeScript の Playwright 統合は SDK の peer dependency として Playwright を要求、利用者が他の test framework を使いたい場合の制約（mitigation: peer dependency なので利用者が opt-in、Playwright を使わない場合は領域 5 だけ skip 可能）
- API 対称性の維持工数（4 言語で同じ責務名を保つ）が継続発生（mitigation: ADR-TIER1-001 既存の 4 言語対称性運用と同じ工数枠で吸収）
- mock builder で生成される mock data の Protobuf wire 形式整合を CI で機械検証する経路が必要（mitigation: 採用初期で `tests/contract/` 配下に cross-language fixture conformance test を追加）

### 移行・対応事項

- `src/sdk/go/k1s0/test-fixtures/` package を新設し、5 領域の skeleton API を Go で実装、go.mod は SDK と共有
- `src/sdk/rust/test-fixtures/` crate を新設し、`Cargo.toml` workspace member に追加、`[features]` に `test-fixtures` を切る
- `src/sdk/dotnet/K1s0.Sdk.TestFixtures/` プロジェクトを新設し、`Sdk.sln` に追加、`Directory.Build.props` で SDK と version 同期
- `src/sdk/typescript/packages/test-fixtures/` package を新設し、monorepo workspace に追加、`peerDependencies` に Playwright 追加（領域 5 用）
- 各言語 fixtures に Setup / Teardown / WithTenant / WithAuth / MockBuilder（State / Audit / PubSub の 3 service）/ wait / assertion helper の skeleton 実装
- `tests/e2e/user/smoke/` を fixtures で書き直し、ADR-TEST-008 で配置した skeleton を fixtures 経由の real 実装に置換
- 4 言語の release cycle を `tools/release/cut.sh` で統一し、SDK release tag が test-fixtures version も同時に bump する経路を整備
- `docs/05_実装/50_開発者体験設計/20_Golden_Path_examples/` の各 example に test-fixtures 使用例を追加
- `docs/05_実装/40_SDK設計/` に test-fixtures の API リファレンスを追記（4 言語別）
- `docs/03_要件定義/00_要件定義方針/08_ADR索引.md` に本 ADR を追加
- `docs/05_実装/30_CI_CD設計/90_対応IMP-CI索引/01_対応IMP-CI索引.md` に本 ADR の対応 IMP-CI を追記
- `tests/contract/` に cross-language fixture conformance test を追加（採用初期）
- 採用初期で Workflow / Decision / Secret の mock builder 追加、運用拡大時で残 6 service 追加

## 参考資料

- ADR-TEST-001（Test Pyramid + testcontainers）— L4 / L2 の責務分界と本 ADR の位置づけ
- ADR-TEST-008（e2e owner / user 二分構造、別 commit で起票）— 本 ADR が user suite を補完
- ADR-TEST-009（観測性 E2E 5 検証 owner only、別 commit で起票予定）— 観測性 helpers は owner 内に閉じる根拠（本 ADR 射程外）
- ADR-TEST-011（release tag ゲート代替保証、別 commit で起票予定）— SDK と test-fixtures の version 同期の release tag 経路
- ADR-TIER1-001（Go + Rust ハイブリッド）— 4 言語対称性の根拠
- ADR-DIR-* 系列（src/sdk 配置）— 4 言語 SDK の配置経路
- ADR-POL-002（local-stack を構成 SoT に統一）— fixtures Setup が `tools/local-stack/up.sh --role user-e2e` を呼ぶ整合
- 関連 ADR（採用検討中）: ADR-TEST-007（テスト属性タグ + 実行フェーズ分離）— `@user-fixtures` タグの導入余地
