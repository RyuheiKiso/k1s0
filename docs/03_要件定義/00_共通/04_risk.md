# COM-RSK: リスク台帳

k1s0 要件定義上で認識する**リスク** を台帳化し、発生確率・影響度・発動トリガ・対処方針を管理する。前回の技術査読で指摘された High / Medium リスクは本台帳で要件と紐付ける。

リスクは要件と異なり「達成対象」ではなく「監視対象」。リスクに対する対処は、予防策（発生確率低減）と軽減策（影響度低減）の 2 系統で設計し、それぞれを要件 ID に紐付ける。

---

## 前提

- [`02_assumption.md`](./02_assumption.md) 前提条件
- 前回の技術査読結果（High / Medium リスク）

---

## 要件本体

### COM-RSK-001: tier1 p99 < 500ms 未達リスク

- 優先度: MUST（SLO 合意の根幹）
- Phase: Phase 1a
- 関連: `QUA-PRF-001`, 査読結果 High

企画書で宣言した p99 < 500ms は机上積算のみであり、実装前の確度が低い。Dapr サイドカー経由のオーバーヘッド、ハッシュチェーン監査の同期処理が想定以上のレイテンシ寄与をする可能性がある。

未達時、SLA 違反と顧客補償が発生する。最悪のシナリオでは、Phase 2 本番運用開始前に SLO の再設定が必要となり、事業計画の見直しに波及する。

**予防策**

- MVP-0 段階で mock load test（k6 / JMeter）を実施し p99 500ms 達成を検証
- ハッシュチェーン監査の非同期化フォールバック設計を MVP-1a 完了条件に追加

**軽減策**

- p99 未達時のエラーバジェット運用と SLA 補償条項を契約に反映

**検知指標**

- 毎日 p99 計測を Grafana ダッシュボードで監視
- 月次違反率 0.1% 超過でインシデント扱い

---

### COM-RSK-002: Dapr Workflow / Temporal 自動振り分けの複雑化リスク

- 優先度: MUST
- Phase: Phase 1b
- 関連: `ARC-EVT-003`, 査読結果 High

tier1 レベルの振り分けロジックは境界ケース（5 分 Saga + 人的承認）で複雑化する可能性が高い。運用工数 0.5 人月/年の前提が下振れする確度 70% 程度。

**予防策**

- MVP-1a で 20+ の実ワークフローパターンをシミュレーション
- 振り分けルールを Rego（OPA）で記述し、複雑化に耐える設計

**軽減策**

- 振り分け監視ダッシュボードで判断件数を月次レビュー

---

### COM-RSK-003: MVP-0 期間過小見積もりリスク

- 優先度: MUST
- Phase: Phase 1a
- 関連: `OPS-OPR-*`, 査読結果 High

企画書の「3〜4 週間」は集中稼働でも困難。JTC corporate proxy / 社内 DNS の工数が未反映。稟議説得力と実現性のトレードオフが顕在化する。

**予防策**

- 稟議時点で期間を「3〜4 週間」から「8〜10 週間」に修正
- MVP-0 完了条件を「実ネットワーク・corporate proxy 下の動作確認」に厳格化

**軽減策**

- 遅延時のエスカレーションパスを事前定義

---

### COM-RSK-004: Rust 人材確保失敗リスク

- 優先度: SHOULD
- Phase: Phase 1c
- 関連: `COM-ASM-004`, `BIZ-VEN-003`, 査読結果 Medium

「6 ヶ月育成で成功率 70%」は出典不明の推定値。SES フォールバック時のコスト増加が TCO に未算入。

**予防策**

- Phase 1c 完了時に「Rust チーム編成ゲート」を KPI に追加
- Phase 1b 段階で早期にパイロット Rust タスクを実施し育成効果を測定

**軽減策**

- SES コスト上限を契約で固定
- 自作領域の段階的縮小（必要に応じて PostgreSQL WORM+RLS 代替へ移行）

---

### COM-RSK-005: AGPL Phase 分割毀損リスク

- 優先度: SHOULD
- Phase: Phase 1a
- 関連: `QUA-OBS-005`, 査読結果 Medium

Phase 1a の Prometheus 素 UI / kubectl logs 運用は、パイロット業務のバッドニュース検知遅延につながる懸念。

**予防策**

- Phase 1a 段階から Grafana Cloud（SaaS / AGPL 非適用）で代替運用する選択肢を保持

**軽減策**

- Phase 1b 早期に Grafana Community Edition への移行を計画

---

### COM-RSK-006: OSS 依存の EOL / ライセンス変更リスク

- 優先度: MUST
- Phase: Phase 1b
- 関連: `OPS-EOL-*`, `BIZ-VEN-002`

Dapr / Istio / ZEN Engine / Keycloak など依存 OSS の EOL、ライセンス変更（例: Elastic → SSPL のような事例）で k1s0 自体の構成変更が必要になる可能性。

**予防策**

- SBOM ベースのライセンス変更監視を CI に組み込む
- 主要依存のアップストリームに貢献し早期情報を入手

**軽減策**

- 代替候補を事前に特定（例: Keycloak → Authentik など）

---

### COM-RSK-007: 監査ログ改ざん検知の実装誤りリスク

- 優先度: MUST
- Phase: Phase 1a
- 関連: `SEC-AUD-*`

ハッシュチェーンの実装誤りは「改ざん検知が機能しない」状態を生み、監査応答で重大な信用失墜を招く。

**予防策**

- ハッシュチェーン実装の外部セキュリティレビュー
- プロパティベーステストで不変条件を検証

**軽減策**

- 定期的なチェーン整合性バッチ検証

---

## 章末サマリ

### リスク分布

| 優先度 | 件数 | 代表 ID |
|---|---|---|
| MUST | 5 | COM-RSK-001, 002, 003, 006, 007 |
| SHOULD | 2 | COM-RSK-004, 005 |

### ID 一覧

| ID | タイトル | 優先度 | Phase | 査読分類 |
|---|---|---|---|---|
| COM-RSK-001 | p99 500ms 未達 | MUST | 1a | High |
| COM-RSK-002 | Workflow 振り分け複雑化 | MUST | 1b | High |
| COM-RSK-003 | MVP-0 過小見積 | MUST | 1a | High |
| COM-RSK-004 | Rust 人材確保失敗 | SHOULD | 1c | Medium |
| COM-RSK-005 | AGPL Phase 毀損 | SHOULD | 1a | Medium |
| COM-RSK-006 | OSS EOL / ライセンス変更 | MUST | 1b | — |
| COM-RSK-007 | 監査ログ改ざん検知誤実装 | MUST | 1a | — |
