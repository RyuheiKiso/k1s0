# Audit / Pii API

本書は、tier1 が公開する Audit API（監査改ざん防止ログ）と Pii API（個人情報マスキング）の機能要件を定義する。両 API は tier1 Rust 自作領域に属し、規制対応とセキュリティの中核を担う。

## API 概要

Audit API は、tier2/tier3 の業務操作とシステムの特権操作を、ハッシュチェーンで改ざん検知可能な監査ログとして記録する。長期保存（7 年）、改ざん検知クエリ、監査人向けのエクスポート機能を提供する。

Pii API は、個人情報（氏名、社員番号、メール、電話、マイナンバー等）のマスキング（表示用）と仮名化（統計用）を担う。個人情報保護法（2022 年改正版）の漏えい等報告義務、仮名加工情報・匿名加工情報の取扱いに準拠する。

両 API とも tier1 Rust 自作領域で実装し、Dapr に依存しない（Dapr にはこの機能が無いため）。

## Audit 機能要件

### FR-T1-AUDIT-001: ハッシュチェーン監査ログ

**現状**: 監査ログを PostgreSQL INSERT で記録するだけでは、DBA 権限で後から改ざん可能。書き込み専用メディアや WORM ストレージを用意するのは高コスト。

**要件達成後**: 各監査イベントに「前イベントのハッシュ」を含めるハッシュチェーン方式で記録する。任意のイベントを書き換えるには、以降の全イベントを再計算する必要があり、検知が容易。Kafka の監査トピックに追記し、最終的に MinIO へアーカイブ（Object Lock で改ざん拒否）。

**崩れた時**: 監査証跡の信頼性が疑われ、監査部門から追加の証跡強化要求が発生する。侵害インシデント時の影響範囲特定で、証跡不正の疑惑が残る。

**受け入れ基準**:
- 各イベントに `previous_hash`、`event_hash`、`sequence_number` を含む
- ハッシュは SHA-256、base64 エンコード
- イベント順序は Kafka パーティション順で保証、パーティションキーは tenant_id
- 追記のみ、UPDATE / DELETE 不可

### FR-T1-AUDIT-002: 改ざん検知クエリ

**現状**: 監査ログの改ざん検知を手作業で行うと、全件再ハッシュに時間がかかり、監査時に即答できない。

**要件達成後**: `k1s0.Audit.VerifyChain(tenant_id, from_time, to_time)` で指定範囲のハッシュチェーンを検証する。改ざん検知時は改ざん位置（sequence_number）と不整合ハッシュを返す。定期的な自動検証ジョブ（日次）で継続的に整合性を監視する。

**崩れた時**: 改ざんの検知が遅れ、監査部門への説明責任を果たせない。インシデント対応の信頼性が損なわれる。

**受け入れ基準**:
- 1 テナント × 1 日分のチェーン検証が 5 分以内に完了
- 改ざん検知時にアラート発報
- 定期検証（日次）の結果を Grafana で可視化

### FR-T1-AUDIT-003: 長期保存（7 年）

**現状**: 監査ログの 7 年保存は、ストレージコスト・検索性・コンプライアンス要件のバランスで設計が難しい。

**要件達成後**: 直近 90 日は Loki でホット検索可能、90 日〜1 年は Kafka / PostgreSQL でウォーム保存、1 年〜7 年は MinIO + Object Lock でアーカイブ保存。各層の保存コストは段階的に下がる設計。検索は Loki が最速、古い期間はエクスポート経由。

**崩れた時**: 保存期間満了前のログ喪失で、監査指摘を受ける。ストレージコストが予算を超過する。

**受け入れ基準**:
- ログ書き込みから 5 分以内に Loki で検索可能
- 90 日経過後、自動的に Kafka → PostgreSQL に移行
- 1 年経過後、自動的に PostgreSQL → MinIO にエクスポート
- 7 年経過後、自動削除（削除操作も Audit ログに記録）

## Pii 機能要件

### FR-T1-PII-001: PII マスキング

**現状**: ログや分析データに個人情報が混入すると、漏えいリスクが増大する。tier2 開発者が個別にマスキング実装をすると、実装バラつきで漏れが発生する。

**要件達成後**: `k1s0.Pii.Mask(field_type, value)` でマスキング処理を提供する。field_type は `email`、`phone`、`name`、`address`、`mynumber` 等の型指定。例: `email "john@example.com"` → `"j***@e***.com"`。ログ出力側でも `pii:true` 属性のついたフィールドを自動マスキング。

**崩れた時**: ログ・分析データに個人情報が直接漏出し、個人情報保護法の漏えい等報告義務が発生する。JTC のブランド影響が大きい。

**受け入れ基準**:
- email、phone、name、address、mynumber の 5 型をサポート
- マスキング結果は元に戻せない（不可逆）
- tier2 SDK で Log API 呼び出し時に `pii:true` マーキングで自動マスキング

### FR-T1-PII-002: PII 仮名化（k 匿名性）

**現状**: 統計分析・BI 用途で個人情報を使う場合、仮名化（再識別不可能な別 ID への置換）が必要。k 匿名性（同じ属性値を持つレコードが k 件以上存在する保証）を満たす設計は専門知識が要る。

**要件達成後**: `k1s0.Pii.Pseudonymize(field_type, value, salt)` で決定論的な仮名化を行う。同じ salt で同じ入力は同じ仮名値、異なる salt では異なる仮名値。tier2 は分析用途で salt を使い、本番用途では別 salt を使うことで越境防止。

**崩れた時**: 分析データで個人再識別が可能となり、仮名加工情報の取扱い義務違反となる。

**受け入れ基準**:
- 決定論的（同一 salt・同一入力で同一出力）
- 仮名化アルゴリズムは HMAC-SHA256
- salt は OpenBao で管理、直接露出しない
- k 匿名性の集計検証ジョブ（Phase 2+ で提供）
- 優先度 SHOULD

## 入出力仕様

```
// Audit
k1s0.Audit.Record(event: AuditEvent) -> error?
k1s0.Audit.Query(filter: AuditQuery) -> (events: AuditEvent[], error?)
k1s0.Audit.VerifyChain(tenant_id: string, from: time, to: time) -> (result: VerifyResult, error?)
k1s0.Audit.Export(tenant_id: string, from: time, to: time, format: "csv" | "json") -> (stream, error?)

// Pii
k1s0.Pii.Mask(field_type: PiiType, value: string) -> string
k1s0.Pii.Pseudonymize(field_type: PiiType, value: string, salt: string) -> string
```

AuditEvent には `timestamp`、`tenant_id`、`user_id`、`action`、`resource`、`result`、`ip_address`、`user_agent`、`previous_hash`、`event_hash`、`sequence_number`、`attributes`（任意） が含まれる。

## 受け入れ基準（全要件共通）

- Audit 書き込みは tier2 業務処理に 50ms 以上影響しない
- Audit ログに PII は直接書かれない（Pii API 経由でマスキング済み）
- Audit ログの読み取りは監査担当者のみ（Keycloak Role ベース）

## Phase 対応

- **Phase 1a**: FR-T1-AUDIT-001（ハッシュチェーン記録、Go SDK）
- **Phase 1b**: FR-T1-AUDIT-002、FR-T1-PII-001（改ざん検知、マスキング、C# SDK）
- **Phase 1c**: FR-T1-AUDIT-003（7 年保存の層別アーカイブ）、FR-T1-PII-002
- **Phase 2+**: k 匿名性検証、差分プライバシー、J-SOX 特化ビュー

## 関連非機能要件

- **NFR-E-MON-001**: 全特権操作の Audit 記録
- **NFR-E-ENC-002**: PII 暗号化・マスキング
- **NFR-E-SIR-002**: 漏えい等報告義務（3 日速報 / 30 日確報）
- **NFR-C-NOP-003**: Audit 7 年保存
- **NFR-D-OBJ-001**: 既存監査基盤との共存（Severity 1 転送）
