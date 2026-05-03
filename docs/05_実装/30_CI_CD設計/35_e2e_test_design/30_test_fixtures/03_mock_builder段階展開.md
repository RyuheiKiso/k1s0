# 03. mock builder 段階展開

本ファイルは ADR-TEST-010 で確定した mock builder の段階展開（リリース時点 3 service / 採用初期 +3 / 運用拡大時 +6 で 12 完備）を実装段階の正典として固定する。各 phase の対象 service / 4 言語実装の同期 / 段階展開の判定基準を ID として採番する。

## 本ファイルの位置付け

ADR-TEST-010 で 12 service × 4 言語 = 48 mock builder の即時完備は実装工数で破綻すると確定した。本ファイルでは段階展開の対象 service・優先順位の根拠・各 phase での 4 言語同期手順を実装段階の規約として固定する。リリース時点で 3 service 先行、採用初期で +3、運用拡大時で 12 完備の 3 段階。

## 12 service の優先順位

12 service（State / Audit / PubSub / Workflow / Decision / Pii / Feature / Telemetry / Log / Binding / Secret / Invoke）の段階展開順は以下の判断軸で決定する。

- **使用頻度**: 利用者の自アプリ test で頻出する service ほど優先
- **複雑性**: mock data の組み立てが難しい service（async 性質 / hash chain）ほど早期に提供
- **fixture 提供価値**: 利用者がゼロから書くと工数が大きい service ほど fixtures に含める

| Phase | 対象 service | 根拠 |
|---|---|---|
| リリース時点 | State / Audit / PubSub | (1) State は ADR-TIER1-001 で「最も使われる service」と定義済 (2) Audit は WORM hash chain の verify 検証が必要で fixture 提供価値が高い (3) PubSub は async 性質で test-fixtures なしでは利用者が組みにくい |
| 採用初期 | Workflow / Decision / Secret | (1) Workflow は Temporal sidecar 起動を含む大きな仕掛け、利用者の負担を fixture が吸収 (2) Decision は ZEN Engine の rule 評価で mock rule の組み立てが面倒 (3) Secret は OpenBao 連携が必要 |
| 運用拡大時 | Pii / Feature / Telemetry / Log / Binding / Invoke | 残 6 service、いずれも単独機能で mock builder 実装が比較的シンプル、利用頻度も中位 |

各 phase の遷移は **起案者の判断** で行う（cron 強制ではない）。リリース時点で着手した利用者からのフィードバックを採用初期に反映し、優先順を組み替える余地を残す。

## 各 service mock builder の API 形

### State

```go
// Go
state := fx.MockBuilder.State().
    WithTenant("tenant-a").
    WithKey("config/db").
    WithValue([]byte(`{"host":"localhost"}`)).
    WithTTL(3600).
    Build(t)
```

builder は最終的に Protobuf message（`StateEntry`）を生成し、各言語 SDK の wire format と整合する。

### Audit

```go
audit := fx.MockBuilder.Audit().
    WithTenant("tenant-a").
    WithEntries(10).                  // 10 件の entry を hash chain で連結
    WithSequence(start_seq=0).
    Build(t)
```

`WithEntries(N)` で N 件の entry が prev_id chain で連結された一連の `AuditEntry` を生成する。WORM hash chain の verify テストで使用。

### PubSub

```go
pubsub := fx.MockBuilder.PubSub().
    WithTopic("events").
    WithMessages(20).                 // 20 件の event message
    WithDelay(100*time.Millisecond).  // publish 間隔
    Build(t)
```

非同期 publish の sequence を生成。subscriber 側 test で順序保証 / 重複検出を検証可能にする。

### Workflow（採用初期）

```go
workflow := fx.MockBuilder.Workflow().
    WithType("order-fulfillment").
    WithSignals(3).                   // 3 件の signal を inject
    Build(t)
```

Temporal worker の signal / activity / timer を mock 化する。workflow definition の mock は別途 `examples/` の workflow code と連携する。

### Decision（採用初期）

```go
decision := fx.MockBuilder.Decision().
    WithRuleId("price-discount").
    WithJDM(jdmPath).                 // JDM JSON file path
    WithInputs(map[string]any{"qty": 100}).
    Build(t)
```

ZEN Engine の rule evaluation の mock。JDM JSON の読み込み + input 注入を fixtures が吸収。

### Secret（採用初期）

```go
secret := fx.MockBuilder.Secret().
    WithVaultKey("api-keys/payment").
    WithValue("test-secret-value").
    Build(t)
```

OpenBao（Vault）の KV store mock。test 用の vault store を fixtures が起動 / cleanup する。

### 残 6 service（運用拡大時）

Pii / Feature / Telemetry / Log / Binding / Invoke は同パタンの fluent builder で実装する。詳細 API 形は採用初期段階で利用者からのフィードバックを反映して確定する（リリース時点では skeleton のみ提供）。

## 4 言語同期手順

各 phase で新 mock builder を追加する場合、4 言語で **同 PR / 同 release** で実装する。これは ADR-TEST-010 の version drift 防止原則の継続。

```text
1. 設計合意: API 形（fluent chain の関数名 + 引数）を 4 言語対称に確定
2. PR 起票: 4 言語の test-fixtures に builder 追加
3. tests/contract/cross-lang-fixtures/ で wire format 整合を検証
4. CI で 4 言語の compile + integration test PASS
5. PR merge → 次の release で配布
```

1 言語のみ先行 merge することは禁止する。これは利用者が「Go では使えるが Rust では使えない」状態を経験する DX 破壊を防ぐためである。

## skeleton 段階の取り扱い

リリース時点で 3 service（State / Audit / PubSub）の builder を real 実装し、残 9 service の builder は skeleton として配置する。skeleton 形は以下:

```go
// Go: skeleton 例
func (b *MockBuilder) Workflow() *WorkflowMockBuilder {
    panic("ADR-TEST-010 PHASE: 採用初期で実装、リリース時点未対応")
}
```

skeleton 状態の builder を呼ぶと panic で test が fail する設計。「動くと思って書いたら static error」を採用初期に防ぐため、明示的に panic させる。

採用初期で順次 panic を real 実装に置換する。`tools/audit/run.sh` で 4 言語の skeleton 件数を集計し、AUDIT.md で「未実装 N service / 4 言語」を月次報告する。

## wire format 整合の機械検証

cross-language の wire 形式整合は `tests/contract/cross-lang-fixtures/` で機械検証する。テスト構造:

```text
1. Go test で MockBuilder.Audit().WithEntries(5).Build() → Protobuf bytes export
2. Rust test で同 builder で生成した Protobuf bytes と byte-level comparison
3. .NET / TypeScript も同様
4. 4 言語で同一 bytes が生成されることを assert
```

リリース時点では skeleton（Go のみ comparison、他 3 言語は skip）、採用初期で全 4 言語 comparison を有効化する。実装は採用初期での課題として明示。

## IMP ID

| ID | 内容 | 配置 |
|---|---|---|
| IMP-CI-E2E-013 | mock builder 段階展開規約（3 → 6 → 12 service × 4 言語） | 本ファイル |

## 対応 ADR / 関連設計

- ADR-TEST-010（test-fixtures 4 言語 SDK 同梱）— 本ファイルの起源
- ADR-TIER1-001（Go + Rust ハイブリッド）— 4 言語対称性の前提
- `01_4言語対称API.md`（同章）— mock builder の API 形
- `02_versioning.md`（同章）— 4 言語同 release 規約
- `tests/contract/cross-lang-fixtures/`（採用初期で新設）— wire format 整合検証
