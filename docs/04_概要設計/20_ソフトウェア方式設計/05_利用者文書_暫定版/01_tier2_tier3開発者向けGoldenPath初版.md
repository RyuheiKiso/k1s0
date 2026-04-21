# 01. tier2 / tier3 開発者向け Golden Path（暫定版）

本ファイルは IPA 共通フレーム 2013 の **7.1.2.4 利用者文書（暫定版）の作成** に対応する利用者文書のうち、tier2 / tier3 開発者が k1s0 上で新規サービス・新規アプリを立ち上げるための手順書（Golden Path）の初版である。稟議通過時点では「骨子と各段の入出力」を確定し、Phase 1a 以降に画面ショット・実装 JSON・コマンド例を追補することで正式版へ昇格させる。

## 本ファイルの位置付け

プラットフォーム製品において、開発者体験の均質化は死活問題である。tier2 / tier3 開発者が自チーム流のリポジトリ構成・CI 設計・SDK 呼び出し・Secret 取得手順をそれぞれ発明した場合、稟議で約束した「年次ライセンス費 2,000 万円削減」「稟議期間 3 か月 → 0」の前提である **標準化された開発経路** が崩壊する。結果、1 チームごとに k1s0 基盤の独自流儀が並立し、JTC 情シス 2 名体制での運用が成立しなくなる。

Golden Path は、この発散を構造的に防止する仕組みである。Backstage の Software Template 実行から本番稼働までの全手順を一本の道として固定化し、コピー＆ペーストによる再発明を禁止する。企画重要コミットの一つである **「Golden Path 10 分ルール: Backstage Template 実行から本番稼働まで 10 分以内」** を達成するために、本手順は各ステップの所要時間目安も併記する。

本ファイルは手順書としての機能と、Phase 1a〜Phase 2 にかけて段階的に埋めていく差分を両立させる構造を持つ。Phase 0 稟議時点では「段数・各段の入出力・時間目安」を確定し、Phase 1a デモで実際のコマンド系列を差し込み、Phase 1b 以降で本番運用に耐える補遺（認証・PII・障害時リカバリ）を拡充する。

## 対象読者と前提スキル

対象読者は JTC 情シス配下の tier2 ドメインサービス開発者と、tier3 エンドユーザアプリ開発者である。tier2 開発者は C# または Go の中級スキル（単体テスト記述・依存注入・非同期処理の基礎）を、tier3 開発者は加えて TypeScript / React / .NET MAUI のいずれかの中級スキルを前提とする。Kubernetes・Dapr・Kafka・Valkey の直接操作スキルは **一切前提としない**。これらはすべて tier1 が吸収する抽象の内部であり、本手順でも意識する必要はない。

読者に要求するのは「JTC 社内 AD アカウント」「Git の基礎操作」「Backstage ポータルへの SSO ログイン」「エディタ（VS Code 推奨）」のみである。ローカル環境の前提は [05_CLI利用初版.md](05_CLI利用初版.md) と整合させ、`k1s0 doctor` コマンドで事前確認できる形とする。

## Why Golden Path — コピペ禁止の設計思想

従来の JTC 開発現場では、新規サービス立ち上げ時に既存サービスのリポジトリをコピー＆ペーストし、不要部分を削って独自にカスタマイズする慣行が根強かった。この手法は短期的には立ち上げ速度が出るが、長期的には以下 3 点で破綻する。

第一に、コピー元サービスが暗黙に依存していた旧バージョンの SDK・旧 CI 設定・非推奨 API 呼び出しが新規サービスにも伝播する。SDK 破壊的変更時、全サービスが個別改修を必要とし、基盤チームの工数が爆発する。第二に、コピー時に「この設定は要らないだろう」と削った設定が、実は監査要件・セキュリティ要件の前提であった場合、運用開始後に監査指摘で慌てて再導入することになる。第三に、各サービスが微妙に異なる形で同じ機能を実装するため、Runbook が「サービスごとに別の手順」を持つ必要があり、バス係数 2 の運用が破綻する。

Golden Path はこれらを以下の設計で封じる。Backstage Software Template のみが正式な立ち上げ経路であり、コピー＆ペーストは PR レビューで差し戻す。Template は Renovate が自動更新するため SDK バージョン発散が起こらない。Template 生成物は監査要件・セキュリティ要件の最小セットを既に含み、削除は禁止される。結果として、全サービスが同じ骨格を共有し、Runbook は「全サービス共通の手順」のみで済む。

## 設計項目 DS-SW-DOC-001 全体手順（7 ステップ）

Golden Path は以下 7 ステップで構成する。各ステップの所要時間目安を併記し、合計 10 分以内で Hello World 疎通を達成することを合格基準とする。

**ステップ 1（0〜1 分）**: Backstage ポータルにログインし、`Create Component` メニューから tier2 Microservice テンプレート、または tier3 Web App テンプレートを選択する。テンプレート選択時に「サービス名」「所属チーム」「一次連絡先メール」「PII 取扱の有無」「テナント ID」を入力する。

**ステップ 2（1〜2 分）**: Backstage が自動生成するリポジトリをクローンする。生成物は GitHub リポジトリ（main ブランチ + PR テンプレート + CODEOWNERS + DevContainer 定義）、GitHub Actions CI 設定（build / test / scan / SBOM 生成）、k1s0 SDK 呼び出し雛形コード、k1s0 マニフェスト（Dapr Component・HPA・NetworkPolicy は tier1 が供給するため tier2 は触らない）、TechDocs の初期骨格である。

**ステップ 3（2〜4 分）**: 生成物を VS Code で開き、`k1s0 dev up` を実行する。内部では Tilt が起動し、tier1 API 全 11 種のモック（Testcontainers 上の軽量プロセス）・Dapr sidecar（daprd プロセス）・自コンテナをまとめて立ち上げる。ホスト側の Valkey / Kafka / PostgreSQL は使用せず、全てコンテナ内に閉じる。

**ステップ 4（4〜6 分）**: 生成コード内の `// TODO: write business logic here` 箇所にビジネスロジックを記述する。tier1 公開 11 API の呼び出しは 1 行 SDK 呼び出しで完結し、認証ヘッダ・tenant_id・trace_id は SDK が自動で付与する。tier2 / tier3 は `k1s0.State.Get(ctx, key)` のような呼び出しだけを書けばよい。

**ステップ 5（6〜7 分）**: `git push` で main ブランチへ PR を出す。CI が build / test / scan を回し、CODEOWNERS に定義された tier1 基盤チームが自動レビュー対象となる。approve されると Argo CD が dev 環境へ自動同期し、dev Kubernetes 上に Pod が起動する。

**ステップ 6（7〜9 分）**: Grafana ダッシュボードで自サービスの Log / Metric / Trace を確認する。tier1 が自動挿入した trace_id で、tier2 → tier1 → 内部ストア までの呼び出しが一気通貫で可視化される。

**ステップ 7（9〜10 分）**: Feature Flag（flagd）で段階公開を設定する。初期は全ユーザに対して無効、動作確認後に社内限定 1% → 10% → 100% の順で開放する。異常検知時は Flag を落として即時停止できる。

10 分以内の達成は Phase 1b 完了時点で CI 上で継続測定する。達成率が 3 期連続で 80% を下回った場合、Golden Path 設計の見直しトリガーとする。

## 設計項目 DS-SW-DOC-002 Backstage Template の生成物

テンプレートが生成するファイル群は、監査・セキュリティ・観測性の必須セットを事前に含む。tier2 / tier3 開発者は以下を削除・改変してはならない。改変が必要な場合は基盤チームへの相談が必須である。

- `.github/workflows/ci.yml`: build / unit test / integration test / SAST（Semgrep）/ SBOM 生成（syft）/ コンテナ脆弱性スキャン（Trivy）の 6 段固定。
- `.github/CODEOWNERS`: tier1 基盤チームを自動レビュアに設定。
- `Dockerfile`: distroless ベース、非 root ユーザ、読み取り専用ファイルシステム。
- `k8s/deployment.yaml` は生成しない。tier1 が Helm Chart で供給する。
- `.devcontainer/devcontainer.json`: VS Code Dev Container 定義、言語ランタイム固定。
- `docs/`: TechDocs 用 Markdown 骨格。サービス概要・API 契約・運用 Runbook の 3 ファイル必須。

生成物の最小集合を削除禁止とするのは、監査時に「最低限のセキュリティ・観測性が全サービスで担保されている」ことを機械的に証明するためである。削除を許すと監査対応コストが爆発する。

## 設計項目 DS-SW-DOC-003 SDK 1 行呼び出しサンプル（Go / C# / TypeScript）

tier1 公開 11 API はそれぞれ 1 行 SDK 呼び出しで完結する。本章では代表 3 API の呼び出し例を言語別に示す。Phase 0 時点では疑似コード、Phase 1a デモで実コードに差し替える。

Go 言語での State.Get と PubSub.Publish は以下の骨格で記述する。

```go
// State.Get: tenant_id と trace_id は SDK が自動付与
value, err := k1s0.State.Get(ctx, "order-123")

// PubSub.Publish: トピック名と payload のみで発行可能
err := k1s0.PubSub.Publish(ctx, "order.created", orderEvent)

// Decision: JDM ルール評価、ZEN Engine 内部利用を隠蔽
result, err := k1s0.Decision.Evaluate(ctx, "credit-limit", input)
```

C# 言語でも対称な API を提供する。tier2 の主要想定言語である C# については、async/await 前提の API 設計とする。

```csharp
// State.Get: C# では Task<Result<T>> を返す
var result = await k1s0.State.GetAsync<Order>("order-123");

// PubSub.Publish: ジェネリクスで型安全
await k1s0.PubSub.PublishAsync("order.created", orderEvent);

// Workflow.Start: Dapr Workflow / Temporal の差異を隠蔽
var handle = await k1s0.Workflow.StartAsync("order-saga", input);
```

TypeScript 言語は tier3 Web App 用で、browser / node 両対応の SDK を提供する。

```typescript
// State.Get: Promise を返す、型推論あり
const order = await k1s0.state.get<Order>("order-123");

// Telemetry: エラーイベント記録、PII マスキング自動
await k1s0.telemetry.recordError("payment-failed", { orderId });

// Feature: flagd 評価、フォールバック値必須
const enabled = await k1s0.feature.isEnabled("new-checkout", false);
```

SDK は全言語で **tenant_id / trace_id / user_id / correlation_id** を自動付与する。tier2 / tier3 開発者がこれらを意識する必要はなく、仮に手動で付与した場合は SDK 側で警告ログを出す設計とする。

## 設計項目 DS-SW-DOC-004 よくある落とし穴と対処

Phase 1a デモで既に顕在化した落とし穴を初版時点で記録し、Phase 1b で受け入れ試験する開発者に共有する。3 カテゴリに分類する。

**認証・認可の落とし穴**: tier1 SDK は Keycloak の OIDC トークンを内部キャッシュするため、ローカル開発時に `k1s0 login` を忘れると「認証エラー」ではなく「接続タイムアウト」になり原因特定が遅れる。対処は `k1s0 dev up` 起動時に SDK が未ログイン状態を検知し、ログイン誘導を表示する。

**tenant_id の落とし穴**: 複数テナントを持つ tier2 サービスで、呼び出し元の tenant_id を誤って別テナントのリソースに渡すと、tier1 が 403 を返す。SDK はデフォルトで現在のリクエストコンテキストから tenant_id を自動抽出するが、バッチジョブのようにリクエストコンテキストがない場合、開発者が明示的に `WithTenant()` オプションを付ける必要がある。

**PII 扱いの落とし穴**: ログ出力・エラーメッセージに PII（個人情報保護法 2022 年改正版での個人関連情報を含む）を含めることは禁止である。tier1 SDK の Log API は既知 PII フィールド（mail / phone / 住所）を自動マスキングするが、自由記述フィールドに紛れた PII は検出できない。対処として Backstage Template 生成物には PII マスキングのテストケースが含まれており、CI 時に Semgrep で PII 漏洩疑いのあるログ出力を検出する。

## 設計項目 DS-SW-DOC-005 時間測定と改善サイクル

Golden Path の 10 分ルールは継続測定の対象とする。Backstage Template 実行時刻・PR マージ時刻・Argo CD sync 完了時刻・Grafana 初回メトリクス観測時刻の 4 点を計測し、median / p90 / p99 を週次で集計する。

Phase 1b 時点の暫定目標値は、median 7 分以内・p90 10 分以内・p99 15 分以内である。p99 が 15 分を超える場合は Template の生成ロジックに問題があると判断し、基盤チームが原因調査する。

改善サイクルとして、開発者が Backstage 上で Golden Path 完走時に 5 段階評価と自由記述コメントを残せる仕組みを Phase 1c で実装する。評価が 4.0 を下回った場合、四半期ごとの DevEx レビューで重点改善項目として取り上げる。

## 設計項目 DS-SW-DOC-006 参考リンクと次段手順

読者が Golden Path 完走後に参照するドキュメントを以下に示す。本章はあくまで「最短経路」であり、深掘りが必要な場合は下記を参照する。

- Backstage 利用の詳細: [03_Backstageポータル利用初版.md](03_Backstageポータル利用初版.md)
- CLI コマンド一覧: [05_CLI利用初版.md](05_CLI利用初版.md)
- tier1 API 仕様: [../02_外部インタフェース方式設計/06_API別詳細方式/](../02_外部インタフェース方式設計/06_API別詳細方式/)
- ローカル開発環境の詳細: [../../70_開発者体験方式設計/02_ローカル開発環境方式.md](../../70_開発者体験方式設計/02_ローカル開発環境方式.md)
- テスト戦略: [../../70_開発者体験方式設計/05_テスト戦略方式.md](../../70_開発者体験方式設計/05_テスト戦略方式.md)
- 障害発生時の連絡先: Backstage の各サービスページ「Ownership」タブ

TechDocs 経由で検索可能な FAQ を Phase 1b 時点で 20 件以上用意する。FAQ は開発者が Golden Path 完走中に遭遇した問い合わせを基盤チームが記録し、週次で集約する。

## 設計項目 DS-SW-DOC-007 Phase 別完成度

本手順書は Phase 進行に応じて記述密度を上げる。各 Phase での完成度を明示することで、記述未完部分を「未完のため参照不可」と誤解されないようにする。

- **Phase 0（稟議時点）**: 骨子確定、各ステップの入出力と時間目安を記述。実コマンドは Phase 1a 以降。
- **Phase 1a（MVP-0）**: 実コマンド・実画面ショットを挿入。Template 生成物の実ファイル一覧を確定。
- **Phase 1b（MVP-1a）**: FAQ 20 件、よくある落とし穴の実例 10 件、SDK 呼び出しサンプルの言語別完全版を追加。
- **Phase 1c（MVP-1b）**: 監査対応・障害時リカバリ・PII 対応の詳細手順を追加。
- **Phase 2 以降**: Backstage のプラグイン拡張連動、マルチクラスタ対応手順を追加。

Phase 進行のたびに本章の改訂 PR を基盤チームがレビューし、Product Council が四半期に一度承認する。

## 対応要件一覧

本ファイルで採番した設計 ID（`DS-SW-DOC-001` 〜 `DS-SW-DOC-007`）と、充足する要件 ID を以下に列挙する。

- `DS-SW-DOC-001`（全体手順 7 ステップ）: `DX-GP-001` / `DX-GP-002` / `DX-GP-003` / `BR-PLATUSE-001` / `BR-PLATUSE-002`
- `DS-SW-DOC-002`（Backstage Template 生成物）: `DX-GP-004` / `DX-GP-005` / `FR-T1-SECRETS-003` / `FR-T1-AUDIT-002`
- `DS-SW-DOC-003`（SDK 1 行呼び出しサンプル）: `DX-GP-006` / `FR-T1-STATE-001` / `FR-T1-PUBSUB-001` / `FR-T1-DECISION-001`
- `DS-SW-DOC-004`（落とし穴と対処）: `DX-GP-007` / `BR-PLATUSE-003` / `NFR-E-AC-001`
- `DS-SW-DOC-005`（時間測定と改善サイクル）: `DX-GP-001` / `DX-MET-003`
- `DS-SW-DOC-006`（参考リンクと次段手順）: `DX-LD-001` / `BR-PLATUSE-004`
- `DS-SW-DOC-007`（Phase 別完成度）: 間接対応（Phase 進行管理）
