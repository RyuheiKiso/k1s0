# Feature API

本書は、tier1 が公開する Feature API の機能要件を定義する。tier2/tier3 の Feature Flag による段階ロールアウト・A/B テスト・circuit breaker を、flagd（CNCF OpenFeature 参照実装）バックエンドで提供する。

## API 概要

新機能の段階的公開、障害時の即時フォールバック、機能単位の ON/OFF 制御を可能にする。従来の「全社一斉リリース → 障害で全員影響」を「10% ユーザで検証 → 問題なければ 50% → 100%」に置き換える。

内部実装は flagd の gRPC / HTTP エンドポイントを tier1 ファサードでラップする。flag 定義は Git 管理 + Argo CD デプロイ、または Backstage プラグインで情シス管理者が直接更新（Phase 2+）。

## 機能要件

### FR-T1-FEATURE-001: Feature Flag 評価（flagd）

**現状**: tier2 が環境変数や ConfigMap で機能 ON/OFF を管理すると、変更に Pod 再起動が必要。ユーザー属性による条件分岐（特定部門のみ新機能）の実装が個別バラつく。

**要件達成後**: `k1s0.Feature.IsEnabled("flag-name", context)` で flag 評価を行う。context には tenant_id、user_id、部門、ロール等を渡す。flagd が JDM 類似の評価ロジックでフラグ値を返す。評価結果はインメモリキャッシュで高速化（30 秒 TTL）。

**崩れた時**: 機能 ON/OFF 変更のリードタイムが長期化し、障害発生時の切り戻しが遅れる。tier2 ごとに異なる flag 実装で、情シス管理者が把握できなくなる。

**受け入れ基準**:
- boolean / string / int / JSON の 4 型のフラグをサポート
- 評価 p99 < 10ms（キャッシュヒット時）
- 評価失敗時はデフォルト値を返す（`K1s0Error` にしない、業務継続優先）
- フラグ定義は Git 管理、Backstage で一覧参照可能

### FR-T1-FEATURE-002: 段階ロールアウト（%）

**業務根拠**: BR-PLATOPS-005（新機能リリースの全社一斉障害リスクの構造的低減）。

**現状**: 新機能のロールアウト率（10% → 50% → 100%）を手動で変更すると、ロール管理が属人化する。ハッシュベースの割り当てを tier2 で個別実装すると、ユーザー識別子の違いで再現性が崩れる。社内既存システムでは過去 3 年間で「全社一斉リリース後に重大バグ発覚 → 全ユーザ影響 → 緊急ロールバック」が年 2〜3 件発生しており、1 件あたり平均 500 人時の対応工数（業務部門 300 人時 + 情シス 200 人時）と、影響ユーザ 1,000 名規模の業務停止時間が発生している。

**要件達成後**: flag 定義に `rollout: { percentage: 10 }` を指定すると、ユーザー ID のハッシュベースで 10% のユーザに有効化される。パーセントを 10 → 50 → 100 に変更するだけで段階拡大。同一 user_id は常に同じ結果を返す。全社一斉障害は構造的に発生しえなくなり、10% ユーザで異常検知した段階で切り戻しが可能。年 2〜3 件 × 500 人時 = 1,000〜1,500 人時/年 の緊急対応工数が大幅削減され、業務停止時間も 10% 規模に限定される。

**崩れた時**: ロールアウト率変更のたびに tier2 コード改修が発生し、変更リードタイムが長期化する。A/B テストの再現性が損なわれる。全社一斉リリースが継続し、重大バグ発覚時の影響がシステム全体に波及する。

**動作要件**:
- 0〜100% の整数で指定
- 同一 user_id は常に同じ結果（ハッシュ一貫性）
- tenant_id × user_id のスコープで分離

**品質基準**:
- 評価レイテンシは NFR-B-PERF-007（p99 < 10ms）に従う
- flagd 障害時は NFR-A-CONT-006 に従いデフォルト値でフォールバック

### FR-T1-FEATURE-003: circuit breaker ルール

**現状**: 新機能で想定外の障害が発生した場合、手動で flag を false にする必要がある。発見が遅れると障害範囲が拡大する。

**要件達成後**: flag 定義に circuit breaker ルール（例: エラー率 > 5% で自動 false 化）を指定する。Prometheus メトリクスを評価し、閾値超過で flagd が自動でフラグを false に切り替え。復旧は手動または一定時間後の自動リトライ。

**崩れた時**: 障害検知から切り戻しまでの時間が長期化し、影響ユーザー数が拡大する。SRE の負荷が増す。

**受け入れ基準**:
- Prometheus クエリを条件に指定可能
- 閾値超過から false 化まで 30 秒以内
- 自動切戻時の Audit 記録
- 優先度 SHOULD、Phase 1c で実装判定

### FR-T1-FEATURE-004: A/B テスト基盤

**現状**: A/B テスト（新旧 UI の効果比較）を tier3 で実施するには、配信制御とメトリクス集計を個別実装する。

**要件達成後**: flag 定義に A/B variant（`variant_a` 50% / `variant_b` 50%）を指定し、flag 評価で variant 名を返す。tier3 は variant に応じた UI を表示。効果測定は variant ラベル付き Prometheus メトリクスで集計。

**崩れた時**: A/B テスト実装がアプリごとにバラつき、統一的な効果測定ができない。

**受け入れ基準**:
- variant 数は最大 10
- variant 配分の合計は 100%
- variant 名を Telemetry の attribute に自動付与
- 優先度 COULD、Phase 2+

## 入出力仕様

本 API の機械可読な契約骨格（Protobuf IDL）は [40_tier1_API契約IDL/11_Feature_API.md](../40_tier1_API契約IDL/11_Feature_API.md) に定義されている。SDK 生成・契約テストは IDL 側を正とする。以下は SDK 利用者向けの疑似インタフェースであり、IDL の `FeatureService` RPC と意味論的に対応する。

```
k1s0.Feature.IsEnabled(
    flag_name: string,
    context: EvaluationContext
) -> bool

k1s0.Feature.GetString(flag_name: string, context: EvaluationContext) -> string
k1s0.Feature.GetInt(flag_name: string, context: EvaluationContext) -> int
k1s0.Feature.GetJSON(flag_name: string, context: EvaluationContext) -> JSON

EvaluationContext = {
    tenant_id: string,
    user_id: string,
    attributes: map<string, any>
}
```

## 受け入れ基準（全要件共通）

- 評価失敗時のデフォルト値フォールバック
- 評価結果のキャッシュ（30 秒 TTL）
- flag 定義の変更は Audit API に記録
- Backstage での一覧・編集 UI

## Phase 対応

- **Phase 1a**: 未提供
- **Phase 1b**: FR-T1-FEATURE-001、002（評価、段階ロールアウト、Go / C# SDK）
- **Phase 1c**: FR-T1-FEATURE-003（circuit breaker）
- **Phase 2+**: FR-T1-FEATURE-004（A/B テスト）、Backstage エディタ

## 関連非機能要件

- **NFR-B-PERF-007**: Feature 評価 p99 < 10ms
- **NFR-A-CONT-006**: flagd 障害時のデフォルト値フォールバック
- **NFR-E-MON-004**: flag 変更の Audit 記録
- **NFR-C-MGMT-002**: flag 定義の Git 管理
