# 10_tier1_API要件

本カテゴリは、k1s0 tier1 が tier2/tier3 に公開する 11 種類の API の個別要件を格納する。1 API 1 ファイルで構成し、各ファイルは「API 概要 → 機能要件 → 入出力仕様 → 受け入れ基準 → 関連非機能要件」の構造で記述する。

## ファイル構成

- [00_tier1_API共通規約.md](00_tier1_API共通規約.md): 11 API 横断の共通契約（認可・冪等性・バージョニング・エラー型・レート制限・監査）
- [01_Service_Invoke_API.md](01_Service_Invoke_API.md): サービス間同期呼び出し
- [02_State_API.md](02_State_API.md): 状態管理・キャッシュ
- [03_PubSub_API.md](03_PubSub_API.md): イベント配信
- [04_Secrets_API.md](04_Secrets_API.md): 秘密情報管理
- [05_Binding_API.md](05_Binding_API.md): 外部入出力バインディング
- [06_Workflow_API.md](06_Workflow_API.md): ワークフロー・Saga
- [07_Log_API.md](07_Log_API.md): 構造化ログ
- [08_Telemetry_API.md](08_Telemetry_API.md): 分散トレース・メトリクス
- [09_Decision_API.md](09_Decision_API.md): ビジネスルール決定表評価
- [10_Audit_Pii_API.md](10_Audit_Pii_API.md): 監査改ざん防止ログ、個人情報マスキング
- [11_Feature_API.md](11_Feature_API.md): Feature Flag

## 記述フォーマット

各 API ファイルは以下の章立てを共通化する。

- **API 概要**: 責務、tier2/tier3 からの利用者像、内部バックエンドの参照（詳細は構想設計を参照）
- **機能要件**: 要件 ID 単位で「現状 → 達成後 → 崩れた時」の散文記述
- **入出力仕様**: 主要メソッドのシグネチャ（言語非依存な擬似インタフェース）、エラー型
- **受け入れ基準**: 検収時の測定可能な判定項目
- **Phase 対応**: Phase 1a / 1b / 1c の段階的提供範囲
- **関連非機能要件**: 性能・可用性・セキュリティの該当 NFR-* ID

## tier2/tier3 から見える契約

全 API 横断の共通契約は [00_tier1_API共通規約.md](00_tier1_API共通規約.md) に集約する。本 README は索引としての役割に限定し、共通規約の詳細は本書内で繰り返さない。個別 API ファイル（01〜11）は共通規約の遵守を前提に、その API 固有の振る舞いのみを記述する。

## クライアント SDK の提供言語

Phase 1a は Go、Phase 1b で C# / Rust、Phase 1c で Python、Phase 2 以降で追加言語を検討する。tier3 の UI 層（MAUI / React）向けは HTTP/1.1 プロキシ経由で利用する。提供言語・版の詳細は各 API ファイルの「Phase 対応」節を参照。

## 共通受け入れ基準

すべての API は、API ファイル個別の受け入れ基準に加えて以下を満たす。

- tier2/tier3 から Dapr の import が不要（`dapr.io/*` アノテーションが tier2/tier3 のマニフェストに出現しない）
- tier2/tier3 のエラーハンドリングで Dapr 固有エラー文字列を参照しない（`K1s0Error` 統一型のみ）
- クライアント SDK は minor version を越えた後方互換破壊を Phase 1〜2 では発生させない
- API 呼び出しから 1 span が自動生成され、Grafana Tempo で traceId を引くと呼び出し全体が辿れる

## 変更管理

API の互換破壊変更は重大改訂として扱う。追加メソッド、オプショナル引数追加は中規模改訂。既存メソッドの signature 変更・削除は必ず DEPRECATED → 新 ID 採番の順で段階移行する。
