# 02. test-fixtures versioning

本ファイルは ADR-TEST-010 で確定した「test-fixtures を SDK と同 module / 同 version で同梱する」規約を実装段階の運用契約として固定する。release cycle / version bump trigger / 利用者側の依存解決 / SDK API 変更時の同期手順を ID として採番する。

## 本ファイルの位置付け

ADR-TEST-010 で test-fixtures を SDK と同 module / 同 SemVer で release することを確定したが、release cycle の具体（patch bump trigger / 4 言語の同時 release 経路 / 利用者側の依存解決）が未確定だった。本ファイルでは SDK と test-fixtures の version 一致を物理的に強制する運用規約を固定し、version drift の発生経路を構造的に塞ぐ。

## version 一致の原則

SDK と test-fixtures は以下の原則で版を一致させる。

- **同 SemVer**: SDK v1.2.3 が release されたら test-fixtures v1.2.3 も同 release で出る
- **同 module**: 同 SemVer source（Go の go.mod / Rust の Cargo.toml workspace.package.version / .NET の Directory.Build.props / TS の lerna.json）から両者が version を読む
- **同 release tag**: GitHub release tag は SDK 全体に紐付き、test-fixtures は内包される
- **同 patch bump**: test-fixtures の bug fix だけでも SDK 本体を patch bump する（release 単位を割らない）

これにより利用者が SDK v1.2.3 を pin した時点で test-fixtures も同 v1.2.3 が自動的に取得される。version 解決の組み合わせ matrix を解く負担が利用者に発生しない。

## release cycle の具体

各言語の release cycle は SDK と test-fixtures が連動する以下の手順で進める。

### Go

`src/sdk/go/k1s0/` 全体が単一 module。release tag は `sdk-go/v1.2.3` 形式（k1s0 monorepo の subdirectory module 命名）。

```text
1. src/sdk/go/k1s0/ 配下で SDK code or test-fixtures code 変更
2. PR で変更内容 + version bump 提案を merge
3. release tag を切る（tools/release/cut.sh sdk-go/v1.2.3）
4. cut.sh が ADR-TEST-011 の owner full PASS sha256 検証を実行
5. tag 公開後、GitHub release で notes を生成
```

利用者は `go.mod` の `require github.com/k1s0/k1s0/src/sdk/go/k1s0 v1.2.3` で pin。test-fixtures は同 module 内 sub-package のため、SDK pin と同時に取得される。

### Rust

`src/sdk/rust/` workspace の `[workspace.package].version` が両者の source。release tag は `sdk-rust/v1.2.3`。

```text
1. src/sdk/rust/Cargo.toml の workspace.package.version を bump
2. workspace 全 crate（k1s0-sdk + k1s0-test-fixtures）が同 version を取得
3. release tag を切る → cut.sh で検証 → cargo publish（採用初期で crates.io 公開、リリース時点では git tag のみ）
```

利用者は `Cargo.toml` の `[dev-dependencies] k1s0-test-fixtures = "1.2.3"` で pin。`[features] test-fixtures = ["dep:k1s0-test-fixtures"]` を有効化することで test target でのみ pull される。

### .NET

`src/sdk/dotnet/Directory.Build.props` の `<Version>` が両者の source。release tag は `sdk-dotnet/v1.2.3`。

```text
1. Directory.Build.props の <Version> を bump
2. K1s0.Sdk.csproj + K1s0.Sdk.TestFixtures.csproj が同 version を継承
3. release tag → cut.sh 検証 → dotnet pack + nuget push（採用初期）
```

利用者は `<PackageReference Include="K1s0.Sdk" Version="1.2.3" />` + `<PackageReference Include="K1s0.Sdk.TestFixtures" Version="1.2.3" />` で pin。同 version の組み合わせを CI で機械検証する経路は採用初期で `K1s0.Sdk.TestFixtures.csproj` の `<PackageReference Include="K1s0.Sdk" Version="$(Version)" />` で強制する。

### TypeScript

`src/sdk/typescript/` monorepo の version source は採用初期で確定する候補が 2 つ:

- (a) `lerna.json` の version + lerna fixed mode
- (b) `package.json` workspace で各 package が `"version": "$(common)"` を読む npm scripts

リリース時点では (a) lerna fixed mode を採用し、`@k1s0/sdk` と `@k1s0/sdk-test-fixtures` を同 version で release する。release tag は `sdk-typescript/v1.2.3`。

利用者は `package.json` の `"dependencies": { "@k1s0/sdk": "1.2.3" }` + `"devDependencies": { "@k1s0/sdk-test-fixtures": "1.2.3" }`。`@k1s0/sdk-test-fixtures` の `peerDependencies` に `@k1s0/sdk: 1.2.3` を pin することで version 不整合を npm が検出する。

## SDK API 変更時の同期手順

SDK 本体の API（client method / mock builder の引数等）が変更された場合、test-fixtures も同 PR で更新する。手順:

1. SDK 本体の API 変更（PR 起票）
2. 同 PR 内で 4 言語の test-fixtures を全て更新
3. 4 言語の `tests/contract/cross-lang-fixtures/` を更新（採用初期で整備）
4. CI で 4 言語の test-fixtures が compile + golden test PASS を確認
5. PR merge 後、次の release cycle で同 SemVer bump

API 変更を test-fixtures に伝播させずに merge することは禁止する（CI で検出可能にする経路は採用初期で `tools/qualify/fixture-api-conformance.sh` を整備）。

## version drift 検出経路

version drift は以下 3 経路で検出する。

| 経路 | 検出タイミング | 失敗の効果 |
|---|---|---|
| 同 module 構造 | compile / build 時点 | SDK API 変更で test-fixtures が compile error |
| `tools/audit/run.sh` | 月次 audit | SDK と test-fixtures の version source が不一致なら fail |
| `tools/qualify/fixture-api-conformance.sh` | PR 毎（採用初期） | 4 言語 fixtures の export API が一致しない場合 fail |

リリース時点は (1) と (2) のみ整備、(3) は採用初期。

## 利用者側の依存解決の取り扱い

利用者が複数の k1s0 SDK module を組み合わせる場合（例: tier3 BFF が Go SDK + Web frontend が TS SDK）、各 SDK の version は **独立に pin** できる（`@k1s0/sdk` v1.2.3 + Go SDK v1.3.0 など）。test-fixtures もそれぞれの SDK version に追従する。

ただし `tests/contract/` の cross-lang fixture conformance test は同 version の組み合わせ前提なので、cross-lang test を書く場合は 4 言語 SDK を同 version に揃える必要がある。これは採用初期での運用課題として明示する。

## major version bump の扱い

破壊的変更（SemVer major bump）が必要な場合、SDK 本体の major bump と test-fixtures の major bump が同期する。これは ADR-TEST-010 の「version drift 発生ゼロ」原則の核心で、major bump で test-fixtures を割ることは認めない。

major bump 時は以下を必須化する:

- 移行 ADR の起票（破壊的変更の根拠）
- 4 言語の migration guide を `docs/04_概要設計/70_開発者体験方式設計/` 配下に新設
- 旧 major version は最低 6 ヶ月の security patch 提供（採用組織の移行猶予）

リリース時点では major bump の発生機会は想定しない（v1.x 系のみ）が、運用拡大時で必要になった場合の手順を本ファイルで予め固定する。

## IMP ID

| ID | 内容 | 配置 |
|---|---|---|
| IMP-CI-E2E-012 | SDK + test-fixtures の同 module / 同 version 運用規約 | 本ファイル |

## 対応 ADR / 関連設計

- ADR-TEST-010（test-fixtures 4 言語 SDK 同梱）— 本ファイルの起源
- ADR-TEST-011（release tag ゲート代替保証）— release tag 切る時の owner full PASS 検証経路
- `01_4言語対称API.md`（同章）— 4 言語の API 対称性
- `03_mock_builder段階展開.md`（同章）— mock builder の version bump trigger
- `tools/release/cut.sh` — release tag 切る本体
- `tools/qualify/fixture-api-conformance.sh`（採用初期で新設）— API 対称性の機械検証
