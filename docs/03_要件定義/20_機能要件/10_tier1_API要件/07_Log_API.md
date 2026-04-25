# Log API

本書は、tier1 が公開する Log API の機能要件を定義する。tier2/tier3 の構造化ログ出力を統一し、Grafana Loki へ集約する。

## API 概要

tier2/tier3 の各言語（Go / C# / Rust / Python）で統一されたログフィールドを持つ構造化ログを出力する。W3C Trace Context の自動注入、tenant_id の必須化、PII マスキング連携により、ログ上の相関と監査対応を自動化する。

内部実装は各言語の OpenTelemetry Logs SDK をラップし、OpenTelemetry Collector 経由で Loki に配信する。tier2/tier3 は Logger 生成 → 呼び出しの 2 ステップで使える。

## 機能要件

### FR-T1-LOG-001: 構造化ログ出力

**業務根拠**: BR-PLATOPS-003（障害調査の平均時間短縮による MTTR 削減）。

**現状**: tier2 が言語別のログライブラリ（Go の zap、C# の Serilog、Rust の tracing、Python の structlog）を選ぶと、フィールド名、ログレベル名、タイムスタンプ形式がバラつき、Loki 検索で部門横断ができない。社内既存システムの障害調査実績では、ログ横断検索の準備（言語別フィールド名の正規化、タイムスタンプ形式の揃え込み）だけで平均 45 分を要しており、年間の障害対応件数 200 件 × 45 分 = 150 人時が純粋な前処理コストとして消費されている。

**要件達成後**: `k1s0.Log.Info(msg, fields)` 等の統一 API で構造化ログを出力する。フィールド名は tier1 が定めた標準（`timestamp` / `level` / `message` / `tenant_id` / `service` / `trace_id` / `span_id` / `user_id` / `extra`）に正規化される。全言語で同じ JSON 形式で出力される。前処理 150 人時/年が解消され、Loki の単一クエリで全言語横断検索が可能となり、MTTR は平均 15% 短縮見込み（業界ベンチマーク: 構造化ログ導入で MTTR 10〜20% 改善）。

**崩れた時**: ログ検索で言語別の差異を意識する羽目になり、障害調査の時間が数倍に膨らむ。Loki の全文検索で同じクエリが言語別に別ヒットする。重大障害発生時の初動遅延は SLA 違反リスクに直結し、NFR-A-CONT-001 の 99% 稼働率から逸脱する可能性が生じる。

**動作要件**:
- Go / C# / Rust / Python SDK で同一のフィールド名で JSON 出力
- ログレベルは `debug / info / warn / error / fatal` の 5 段階
- timestamp は RFC3339 ナノ秒精度
- 必須フィールド（timestamp、level、message、service、tenant_id、trace_id）は SDK が自動付与

**品質基準**:
- ログ出力レイテンシは NFR-B-PERF-006（tier2 業務処理への影響 5ms 以内）に従う
- 標準フィールド逸脱は CI の Loki Schema チェックで検出（`grep` による static analysis）

### FR-T1-LOG-002: W3C Trace Context 自動注入

**現状**: 分散トレースの trace_id / span_id をログに含めるには、tier2 開発者が context から抽出して手動で埋め込む必要がある。抽出漏れで Loki と Tempo の相関が切れる。

**要件達成後**: tier1 Log SDK が現在の OpenTelemetry Context から trace_id / span_id を自動抽出してログに含める。context がない場合は空値を出力し、警告を出さない（ノイズ回避）。

**崩れた時**: ログと分散トレースの相関が切れ、障害調査で「このログはどのリクエストのものか」を推測する羽目になる。

**受け入れ基準**:
- tier1 の他 API（Service Invoke、State、PubSub 等）を呼ぶと、その呼び出しから生成された span の trace_id / span_id がログに付与される
- HTTP / gRPC サーバ側では traceparent ヘッダから自動継承
- tier2 開発者が明示的に context を引き回さなくても動作

### FR-T1-LOG-003: tenant_id 必須フィールド強制

**現状**: ログに tenant_id を含めるルールを tier2 に徹底するのは難しく、書き忘れが頻発する。結果、監査時の「このログはどのテナントのものか」の特定にクエリコストがかかる。

**要件達成後**: SDK が JWT クレームから tenant_id を自動抽出し、全ログに必須付与する。JWT が無い context（バッチ処理等）では、Logger 初期化時に明示設定する。未設定のログは tier1 側で拒否され、`K1s0Error.TenantIdMissing` がログ出力側に通知される。

**崩れた時**: テナント越境ログの抽出コストが高止まりし、監査対応で数人日の工数がかかる。

**受け入れ基準**:
- tenant_id 未設定で Log 呼び出しすると警告が出力される（fatal でなく継続可能）
- Loki のラベルに tenant_id が含まれ、テナント別クエリが高速
- Kyverno ポリシーで tenant_id 無しのログパイプライン設定を拒否

### FR-T1-LOG-004: ログレベル動的変更

**現状**: ログレベルは起動時の環境変数で固定されることが多く、障害時に debug レベルを有効化するには Pod 再起動が必要。

**要件達成後**: Backstage プラグインまたは CLI で、稼働中の tier2 アプリのログレベルを動的変更する。反映は次回ログ呼び出しから。変更は Audit API に記録される。

**崩れた時**: 障害調査で debug レベル有効化に Pod 再起動を伴い、再現頻度の低いバグの追跡が困難になる。

**受け入れ基準**:
- ログレベル変更は REST API または CLI で実行可能
- 変更は即時反映（最大 30 秒以内）
- 変更者と変更時刻が Audit API に記録される
- 優先度 SHOULD（リリース時点 で評価）

## 入出力仕様

本 API の機械可読な契約骨格（Protobuf IDL）は [40_tier1_API契約IDL/07_Log_API.md](../40_tier1_API契約IDL/07_Log_API.md) に定義されている。SDK 生成・契約テストは IDL 側を正とする。以下は SDK 利用者向けの疑似インタフェースであり、IDL の `LogService` RPC と意味論的に対応する（同期書込は SDK 側で非同期バッファリングし、IDL の `LogBatch` メッセージにマッピングする）。

```
k1s0.Log.Info(message: string, fields?: map<string, any>) -> void
k1s0.Log.Warn(message: string, fields?: map<string, any>) -> void
k1s0.Log.Error(message: string, error: any, fields?: map<string, any>) -> void
k1s0.Log.Debug(message: string, fields?: map<string, any>) -> void
k1s0.Log.Fatal(message: string, error: any, fields?: map<string, any>) -> void  // プロセス終了

k1s0.Log.With(fields: map<string, any>) -> Logger  // scoped Logger
```

出力 JSON の例:
```json
{
  "timestamp": "2026-04-19T12:34:56.123456789Z",
  "level": "info",
  "message": "expense approved",
  "service": "expense-service",
  "tenant_id": "acme",
  "trace_id": "4bf92f3577b34da6a3ce929d0e0e4736",
  "span_id": "00f067aa0ba902b7",
  "user_id": "u-12345",
  "extra": { "expense_id": "e-67890", "amount": 50000 }
}
```

## 受け入れ基準（全要件共通）

- ログ出力は非ブロッキング（tier2 の業務処理を遅延させない）
- PII を含むフィールドは `pii:true` の attribute で自動マスキング（FR-T1-PII-001 連携）
- Log SDK 障害時も tier2 アプリはクラッシュしない（stderr フォールバック）

## 段階対応

- **リリース時点**: FR-T1-LOG-001、002、003（Go SDK）
- **リリース時点**: FR-T1-LOG-004、C# SDK 追加
- **リリース時点**: Python / Rust SDK 追加
- **採用後の運用拡大時**: ログレベルの Feature Flag 連動

## 関連非機能要件

- **NFR-B-PERF-006**: Log 出力が tier2 業務処理 p99 に 5ms 以上影響しない
- **NFR-C-NOP-001**: ログの Loki 集約と保管（7 日〜7 年の保持ポリシー）
- **NFR-E-ENC-002**: PII マスキングの強制
- **NFR-E-MON-001**: tenant_id 必須付与による監査可能性
