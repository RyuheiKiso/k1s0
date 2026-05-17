# audit_chain: 監査イベントハッシュチェーン

## 概念

audit_chain は k1s0 システムの全監査イベントを暗号学的なハッシュチェーンで連結する機構です。各監査イベントは直前のイベントの SHA-256 ハッシュを含み、チェーンの任意の位置での改ざんを検知可能にします。

## チェーン構造

各 `audit_event` は以下のフィールドを持ちます。

- `event_id`: イベントの一意識別子（UUIDv7、単調増加する）
- `timestamp_hlc`: HLC（Hybrid Logical Clock）によるタイムスタンプ
- `event_type`: イベント種別（api_call / data_mutation / key_operation / policy_violation）
- `actor`: 操作を実行した主体（service account / user identity）
- `resource`: 操作対象リソースの識別子
- `payload_hash`: イベントペイロードの SHA-256 ハッシュ
- `prev_hash`: 直前のイベントの SHA-256 ハッシュ（チェーンの連結）
- `chain_hash`: この `audit_event` 全体の SHA-256 ハッシュ

## 実装について

audit_chain の実際の実装は data 軸（P9）で行います。

本ディレクトリは audit_chain の概念定義と、実装が完了するまでの placeholder として機能します。実装完了後は `src/data/audit_chain/` に配置されたコードがこの概念を物理実現します。

## hash chain divergence 検知と global freeze

hash chain の divergence（分岐）が検知された場合、システムは以下の手順で `global freeze` を実行します。

1. divergence を検知した node が `k1s0-slo-status` ConfigMap の `error_budget_freeze` を `true` に更新する
2. Kyverno `block-on-error-budget-freeze` ポリシーが全 workload の新規 deploy を reject する
3. security チームへの緊急アラートが Alertmanager 経由で発報される
4. forensic 調査が完了し、divergence の原因が特定されるまで freeze を継続する
5. security ロールが手動で `error_budget_freeze` を `false` に戻すことで freeze を解除する

global freeze は `k1s0.io/slo-exempt: "true"` annotation を持つ緊急修復 workload のみを例外とします。freeze 解除には security ロールの dual sign-off が必要です。

## 関連仕様

- `docs/04_詳細設計/01_適合仕様/` — audit chain 適合仕様
- `src/data/` — data 軸実装（P9 で実装）
- `src/security/openbao/transit_config.yaml` — audit キー（3650日保持）の定義
