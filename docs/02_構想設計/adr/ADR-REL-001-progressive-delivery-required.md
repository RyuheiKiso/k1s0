# ADR-REL-001: Progressive Delivery を全リリースで必須化

- ステータス: Accepted
- 起票日: 2026-04-24
- 決定日: 2026-04-24
- 起票者: kiso ryuhei
- 関係者: システム基盤チーム / tier1 開発チーム / tier2 開発チーム / tier3 開発チーム / SRE / セキュリティチーム

## コンテキスト

[ADR-CICD-002](ADR-CICD-002-argo-rollouts.md) によって Progressive Delivery ツールに Argo Rollouts が採用されているが、「どのリリースを Progressive Delivery にするか」の適用境界は未決のまま、tier1 公開 API と ZEN Engine / Temporal のような高 RPN コンポーネントに限定する運用が暗黙に想定されていた。k1s0 リリース時点で本方針を明示化し、全リリースに対する必須化の可否を決める必要がある。

採用側の運用体制が拡大しても適用境界が曖昧化しない構造にするため、Progressive Delivery を「特定コンポーネントのみ」と絞る運用は採らない。理由は次の通り。

- **適用境界の運用負荷**: 「この API は公開だから PD、この API は内部だから Rolling」という判定を PR ごとに行うコストが採用側の運用拡大で直線的に増える。判定者の慣習に依存した境界は 10 年保守の途中で必ず曖昧になる
- **Rollout CRD の学習は一度で済む**: Rollout CRD を書ける開発者を全員に作っておけば、後から適用範囲を広げる際のコストは増えない。逆に「Deployment しか書けない開発者」を量産すると、緊急時の PD 適用に時間がかかる
- **緊急パッチの例外化は SRE 承認で律する**: Progressive Delivery を既定としつつ、emergency patch（CVE 即時修正等）は SRE 承認で Rolling 化を許容する例外運用にした方が、例外の根拠が残り監査対応しやすい
- **内部バッチ・内部ツールは別軸**: バッチは顧客トラフィックがないため Canary Weight による検証ができない。ここは PD の対象外と明示する方が実態に合う
- **DX-FM（flagd）との役割分担**: PD は「コードデプロイの段階的公開」、flagd は「機能フラグの段階的公開」と役割を分けた上で、flagd の運用ルールを未決のままにしてはならない。フラグ定義の改ざんはリリースパイプラインのバイパス経路となるため、cosign 署名済みファイルを Kyverno で検証する経路を強制する必要がある

SLO/SLI 連動の自動ロールバックは AnalysisTemplate で記述するが、テンプレートがコンポーネントごとに乱立すると運用知識が分散する。tier 横断の共通 AnalysisTemplate セット（error rate / latency p99 / CPU / 依存コンポーネントの down rate）を整備し、コンポーネント固有の指標は追加テンプレで差分記述する構造にする。

本 ADR は Progressive Delivery の適用を k1s0 リリース時点から全リリース必須化し、例外運用の承認経路・AnalysisTemplate の共通セット・flagd フラグ運用の署名強制までを定める。

## 決定

**Argo Rollouts による Progressive Delivery を k1s0 リリース時点から全リリースで必須化する。**

### 適用範囲

- tier1 公開 11 API（Dapr ファサード層・自作 Rust 領域とも）: 必須
- tier1 内部 gRPC（DS-SW-COMP-125 以降の internal v1）: 必須
- tier2 ドメインサービス（.NET / Go）: 必須
- tier3 Web / BFF / Native: 必須
- SDK 配布: 対象外（SDK はバイナリ配布物であり、クラスタデプロイの対象ではない）

### 例外運用

以下のカテゴリは PD 対象外とし、Kubernetes 標準 Deployment の RollingUpdate を許容する。

- **内部ツール**: Backstage、開発者用ダッシュボード、社内ヘルプデスク系。エンドユーザートラフィックがないため Canary Weight 判定が無意味
- **バッチ**: CronJob / Job / Argo Workflows による定期実行。顧客トラフィック無依存
- **emergency patch**: CVSS 9.0+ の即時修正、業務停止級インシデントの復旧。この 1 カテゴリのみ SRE 承認のもとで Progressive Delivery をスキップ可能

例外適用時は以下を必須化する。

- GitHub Issue（または採用側の変更管理システム）で `emergency-bypass` ラベル付きのチケットを起票
- SRE オンコール（Primary）の承認を `/approve-bypass` コメントで記録
- デプロイ後 24 時間以内に事後レビューを BC-GOV-005（変更諮問会議）で実施
- 全例外を四半期ごとに集計し、例外発動率が月次 5% を超えた場合は恒常的な PD 設計の見直しを発動

### AnalysisTemplate の共通セット

`deploy/rollouts/analysis/` 配下に以下の共通 AnalysisTemplate を配置する。コンポーネント固有の AnalysisTemplate は共通セットを継承 + 差分記述の形で書く。

- `at-common-error-rate.yaml`: HTTP 5xx / gRPC error の比率が過去 30 分ベースラインを 2σ 超過で fail
- `at-common-latency-p99.yaml`: レイテンシ p99 が SLO 値を超過で fail
- `at-common-cpu.yaml`: Pod CPU 使用率が 80% を 10 分継続で fail
- `at-common-dependency-down.yaml`: 依存コンポーネント（Postgres / Kafka / Valkey）の down 判定で即時 fail

判定に使う Prometheus / Mimir のクエリは `deploy/rollouts/analysis/` 配下に共通化し、tier1 側の Ingress ラベリング規約（DS-SW-COMP-140 系）に揃える。

### Canary Weight の既定

tier1 公開 API: 10% → 25% → 50% → 100% の 4 段階。各段階で AnalysisTemplate を通過してから次に進む。
tier2 / tier3: 25% → 100% の 2 段階。tier1 ほどの細粒度は運用負荷に見合わないため簡素化。

### flagd フラグ運用の署名強制

flagd は Release / Experiment / Ops / Permission の 4 フラグ種別を扱うが、フラグ定義ファイル（`flags/*.json`）の改ざんはリリースパイプラインをバイパスする攻撃経路となる。以下を強制する。

- フラグ定義ファイルは Git で管理し、`main` ブランチへのマージは契約レビュー担当承認必須
- CI で `cosign sign-blob --bundle` により定義ファイルに署名し、バンドルを成果物として Release に添付
- flagd の sidecar ブート時に Kyverno ImageVerify または init container で `cosign verify-blob` を実行し、署名検証失敗時は flagd 起動を拒否
- Permission フラグ（認可に関わる）は Release / Experiment / Ops より厳格に扱い、変更時は 複数名承認 + 法務部事前通告を必須

### Blue-Green 戦略の位置づけ

Blue-Green は既定採用しない（リソース 2 倍消費のため）。以下に限り Blue-Green を許容する。

- tier1 のうちステートフルな ZEN Engine ルールサーバ（DS-SW-COMP-127）: ルール差替え時のロールバック性を最大化
- データ移行を伴うデプロイ: DB スキーマ切替時の整合

それ以外は Canary を既定とする。

## 検討した選択肢

### 選択肢 A: 全リリース PD 必須、例外は SRE 承認制（採用）

- 概要: k1s0 リリース時点から全リリースで Progressive Delivery を必須化、emergency patch / 内部ツール / バッチのみ例外
- メリット:
  - 適用境界判定の運用負荷を恒常的に排除
  - 全開発者が Rollout CRD を標準ツールとして扱うスキルに揃う
  - 例外の根拠が GitHub Issue + SRE 承認で監査証跡化
  - DORA Four Keys の Change Failure Rate を Elite ラインに引き上げる土台がリリース時点から整う
  - 10 年保守の中で Rolling ↔ PD 境界が曖昧化する運用劣化が構造的に起こらない
- デメリット:
  - 全開発者に Rollout CRD 学習コストを課す（Backstage テンプレで緩和）
  - 初期 1〜2 ヶ月は PR レビュー時の AnalysisTemplate 設計議論が増える
  - バッチ / 内部ツールを例外に回す判定ルールの文書化と周知が必要

### 選択肢 B: tier1 公開 API のみ PD、内部は Rolling

- 概要: 公開 API の影響範囲の広さに応じて PD を絞る
- メリット:
  - 開発者の学習コストが tier1 チームに局在化
  - AnalysisTemplate 設計負荷が絞られる
- デメリット:
  - 公開/内部境界の判定が PR ごとに発生、採用側の運用拡大期に運用負荷が線形増加
  - 緊急時に内部 API へ PD を後付けする際の学習コストが発生
  - tier2 / tier3 の障害を tier1 と切り離して評価する合理性が乏しい

### 選択肢 C: PD は任意採用（推奨レベル）

- 概要: 各コンポーネントチームに PD 採否を委ねる
- メリット: チーム自律性
- デメリット:
  - チーム間で品質格差が生まれる
  - 横断 SRE オンコールが「このコンポーネントは PD？」を毎回確認する運用負荷
  - 10 年保守で確実に劣化、本 ADR を書く意義を喪失

### 選択肢 D: Blue-Green 固定

- 概要: 全リリースを Blue-Green で実施
- メリット: ロールバックがトラフィック切替で瞬時
- デメリット:
  - リソース 2 倍消費でオンプレ容量を圧迫、NFR-C-NOP-001 のコスト制約に矛盾
  - ステートフルコンポーネント（DB / Kafka）で Blue-Green は不成立
  - メトリクス判定による段階的検証が Canary より弱い

## 帰結

### ポジティブな帰結

- リリース時点から全コンポーネントが PD を前提とする文化に揃い、10 年保守での境界劣化リスクを構造的に排除
- AnalysisTemplate 共通セット化により運用知識が分散せず、SRE オンコールの認知負荷が最小
- flagd 定義ファイルの cosign 署名強制により、フラグ経由のパイプラインバイパスが構造的に遮断される
- 例外運用が GitHub Issue + SRE 承認で証跡化され、監査対応（J-SOX / NFR-H 系）が容易
- Change Failure Rate 改善による事業継続性の底上げ

### ネガティブな帰結

- 全開発者に Rollout CRD / AnalysisTemplate の学習コストを課す
- Backstage Software Template による雛形提供をリリース時点で整備する必要がある
- Canary Weight 昇格の判定時間がデプロイ全体の時間を延ばす（tier1 公開 API で 30〜60 分程度）
- AnalysisTemplate の基準値チューニングを運用で回す規律が必要、安易な緩和は品質劣化を招く

### 移行・対応事項

- `deploy/rollouts/analysis/` 配下に共通 AnalysisTemplate 4 本を配置
- `tools/codegen/scaffold/` の Backstage Software Template に Rollout 雛形を追加、Deployment 直書きを抑止
- Kyverno（[ADR-CICD-003](ADR-CICD-003-kyverno.md)）で「本番 namespace での Deployment kind 禁止、Rollout のみ許可」を enforce ポリシー化
- flagd 定義ファイル署名検証の init container / Kyverno ImageVerify を `deploy/apps/flagd/` で実装
- 例外運用の GitHub Issue テンプレと `emergency-bypass` ラベル、SRE 承認 bot を `tools/ci/` に実装
- Runbook `RB-OPS-004: PD 失敗時の手動 Rollback` を Runbook 目録（`docs/04_概要設計/55_運用ライフサイクル方式設計/09_Runbook目録方式.md`）に追加
- 例外発動率の四半期集計レポートを BC-GOV-005 の定例議題に組込み
- 開発者向けハンズオン資料を `docs/05_実装/30_CI_CD設計/` 配下に整備

## 参考資料

- [ADR-CICD-002: Progressive Delivery に Argo Rollouts を採用](ADR-CICD-002-argo-rollouts.md)
- [ADR-CICD-003: Kyverno 採用](ADR-CICD-003-kyverno.md)
- [ADR-FM-001: flagd と OpenFeature 採用](ADR-FM-001-flagd-openfeature.md)
- [ADR-RULE-001: ZEN Engine 採用](ADR-RULE-001-zen-engine.md)
- [ADR-RULE-002: Temporal 採用](ADR-RULE-002-temporal.md)
- [CLAUDE.md](../../../CLAUDE.md)
- Argo Rollouts 公式: [argoproj.github.io/rollouts](https://argoproj.github.io/rollouts)
- Progressive Delivery Principles (James Governor, RedMonk 2018)
- DORA "Accelerate State of DevOps Report" 2024（Change Failure Rate Elite 基準）
- Sigstore cosign sign-blob: [docs.sigstore.dev](https://docs.sigstore.dev)
- OpenFeature 仕様: [openfeature.dev](https://openfeature.dev)
- Google SRE Workbook "Canarying Releases" 章
