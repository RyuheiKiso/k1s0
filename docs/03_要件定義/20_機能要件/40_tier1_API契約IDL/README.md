# 40. tier1 API 契約 IDL

本ディレクトリは tier1 が公開する 11 API のインタフェース契約（Protobuf IDL）をスケルトン形式で定義する。各 API の要件詳細は [../10_tier1_API要件/](../10_tier1_API要件/) に記述されており、本ディレクトリはそれに対応する機械可読な契約骨格を API ごとに 1 ファイルずつ提供する。tier2/tier3 はこの IDL から生成されるクライアントライブラリ経由でのみ tier1 を利用し、内部実装言語（Go/Rust）には依存しない。

## 本ディレクトリの位置付け

要件記述だけではインタフェース契約は一意に定まらない。例えば「State API は取得・更新・削除を提供する」と書かれていても、エラーコードの体系、冪等性キー、ETag、トランザクション境界、バルク操作可否といった具体は IDL を見ないと確定しない。tier2/tier3 開発者は IDL からクライアントを生成してから開発に入るため、IDL が無いと並行開発が始められない。

本ディレクトリの IDL は「要件定義段階で確定すべき契約の最小限」であり、詳細設計で message フィールドの追加・RPC の分割統合が行われる。ただし以下は要件定義の合意事項として本ディレクトリで固定する。

- 全 RPC は gRPC over HTTP/2、mTLS 必須、ヘッダで `x-tenant-id` `x-correlation-id` を伝搬
- 全エラーは `google.rpc.Status` を使い、`details` に `ErrorDetail`（共通エラー体系）を載せる
- メッセージのフィールドタグは 1〜15 の空きを予約領域として将来拡張用に空ける

## ファイル構成

全 11 API を 1 API 1 ファイルで定義する。共通型（`TenantContext` / `ErrorDetail` / `K1s0ErrorCategory`）は 00_共通型定義.md に切り出し、各 API ファイルから import する運用とする。

| ファイル | 内容 |
|---|---|
| [00_共通型定義.md](00_共通型定義.md) | 全 API 共通の型定義（TenantContext / ErrorDetail / K1s0ErrorCategory） |
| [01_Service_Invoke_API.md](01_Service_Invoke_API.md) | サービス間 RPC 仲介 API |
| [02_State_API.md](02_State_API.md) | KV / Relational / Document 状態管理 API |
| [03_PubSub_API.md](03_PubSub_API.md) | Kafka 抽象 Publish/Subscribe API |
| [04_Secrets_API.md](04_Secrets_API.md) | OpenBao 秘密情報取得・ローテーション API |
| [05_Binding_API.md](05_Binding_API.md) | 外部 HTTP/SMTP/S3 バインディング API |
| [06_Workflow_API.md](06_Workflow_API.md) | Temporal 長時間ワークフロー API |
| [07_Log_API.md](07_Log_API.md) | 構造化ログ送信 API（OpenTelemetry Logs 準拠） |
| [08_Telemetry_API.md](08_Telemetry_API.md) | メトリクス・トレース送信 API |
| [09_Decision_API.md](09_Decision_API.md) | ZEN Engine JDM 評価 API |
| [10_Audit_Pii_API.md](10_Audit_Pii_API.md) | 監査イベント記録・PII 自動判定 API |
| [11_Feature_API.md](11_Feature_API.md) | Feature Flag 評価 API（flagd / OpenFeature 準拠） |

## 責任分界表: 要件定義 / 基本設計の IDL 変更ルール

本ディレクトリの IDL は骨格であり、詳細設計で細案化される。tier2/tier3 開発者は IDL を「契約」として生成コードに取り込むため、どの要素が誰の責任で、どの段階で固定され、どう変わりうるかを明示しない限り、並行開発で破壊的変更による手戻りが発生する。本節は IDL 各要素の所有権とライフサイクルを定義する。

| IDL 要素 | 所有者 | 要件定義での扱い | 基本設計での変更ルール | 後方互換性 | 変更手続き |
|---|---|---|---|---|---|
| `service` / RPC メソッド名・シグネチャ | tier1 テックリード | **固定**（本ディレクトリで確定） | 破壊的変更禁止（semver major 相当） | クライアント再生成で継続動作 | ADR 記録必須、Product Council 承認 |
| `rpc` の引数型 / 戻り型 | tier1 テックリード | **固定** | 破壊的変更禁止（型置換は別 RPC を新設） | 同上 | 同上 |
| `message` のフィールド追加 | tier1 API 担当 | スケルトン、詳細設計で追加 | 後方互換を保ち追加のみ可、`reserved` で削除範囲宣言 | proto3 のデフォルト値で後方互換 | PR レビューのみ |
| `message` のフィールド削除 | tier1 API 担当 | 非推奨化は IDL コメントで宣言 | 1 年間の非推奨期間を経て削除、`reserved` 化 | 1 年猶予で互換担保 | ADR 記録、OR-EOL-* 連動 |
| フィールドのタグ番号（1〜15） | tier1 テックリード | **予約領域**として確保 | 後方互換リリースまで未使用フィールドを占有 | タグ番号変更は破壊的 | ADR 記録必須 |
| `ErrorDetail.code` の値体系 | tier1 テックリード + セキュリティ | **固定**（`E-<CATEGORY>-<MODULE>-<NUMBER>`） | 新コード追加のみ、既存コード意味変更禁止 | 未知コードは `UNKNOWN` 扱い | PR レビューのみ |
| `TenantContext` スキーマ | tier1 横断 | **固定**（全 API 共通） | 追加のみ、既存フィールド削除禁止 | 同上 | ADR 記録必須 |
| gRPC ステータスコードマッピング | tier1 テックリード | 本ディレクトリで確定（UNAVAILABLE → 503 等） | 新規マッピング追加のみ | 既存マッピング変更は破壊的 | ADR 記録必須 |
| ストリーミング RPC の追加 | tier1 API 担当 | スケルトン（InvokeStream 等） | 追加のみ、既存 unary を stream に置換禁止 | unary → stream は破壊的 | ADR 記録必須 |
| `option` / `extension` | tier1 テックリード | 未使用 | 詳細設計で Dapr 互換 option を追加可 | gRPC の拡張性で互換担保 | PR レビューのみ |

### TenantContext の伝搬方式

`TenantContext` は全 API の `message` に埋め込む形式（各 Request の `context` フィールド）と、gRPC メタデータヘッダ（`x-tenant-id` / `x-correlation-id` / `x-subject`）の二重伝搬とする。メタデータヘッダは Istio Ambient L7 ポリシーと OpenTelemetry トレース伝播が参照できる形式として必須、message 埋め込みは RPC 内での参照容易性とテストでの明示性を担保する。tier1 ファサードの interceptor は両者の整合性を検証し、不一致時は `E-AUTH-CTX-MISMATCH` を返す。詳細設計で interceptor 実装方針を ADR に記録する。

### API 要件ファイルとの対応

本ディレクトリ内の各 API ファイルは、`../10_tier1_API要件/` 配下の要件ファイルと 1 対 1 で対応する。要件の散文記述（現状→達成後→崩れた時）と IDL（機械可読な契約骨格）は相補的であり、要件ファイルの「入出力仕様」セクションは対応する IDL ファイルへのリンクで参照する運用とする。

| API 番号 | 要件ファイル | IDL ファイル |
|---|---|---|
| 01 | [../10_tier1_API要件/01_Service_Invoke_API.md](../10_tier1_API要件/01_Service_Invoke_API.md) | [01_Service_Invoke_API.md](01_Service_Invoke_API.md) |
| 02 | [../10_tier1_API要件/02_State_API.md](../10_tier1_API要件/02_State_API.md) | [02_State_API.md](02_State_API.md) |
| 03 | [../10_tier1_API要件/03_PubSub_API.md](../10_tier1_API要件/03_PubSub_API.md) | [03_PubSub_API.md](03_PubSub_API.md) |
| 04 | [../10_tier1_API要件/04_Secrets_API.md](../10_tier1_API要件/04_Secrets_API.md) | [04_Secrets_API.md](04_Secrets_API.md) |
| 05 | [../10_tier1_API要件/05_Binding_API.md](../10_tier1_API要件/05_Binding_API.md) | [05_Binding_API.md](05_Binding_API.md) |
| 06 | [../10_tier1_API要件/06_Workflow_API.md](../10_tier1_API要件/06_Workflow_API.md) | [06_Workflow_API.md](06_Workflow_API.md) |
| 07 | [../10_tier1_API要件/07_Log_API.md](../10_tier1_API要件/07_Log_API.md) | [07_Log_API.md](07_Log_API.md) |
| 08 | [../10_tier1_API要件/08_Telemetry_API.md](../10_tier1_API要件/08_Telemetry_API.md) | [08_Telemetry_API.md](08_Telemetry_API.md) |
| 09 | [../10_tier1_API要件/09_Decision_API.md](../10_tier1_API要件/09_Decision_API.md) | [09_Decision_API.md](09_Decision_API.md) |
| 10 | [../10_tier1_API要件/10_Audit_Pii_API.md](../10_tier1_API要件/10_Audit_Pii_API.md) | [10_Audit_Pii_API.md](10_Audit_Pii_API.md) |
| 11 | [../10_tier1_API要件/11_Feature_API.md](../10_tier1_API要件/11_Feature_API.md) | [11_Feature_API.md](11_Feature_API.md) |

要件ファイル側の「入出力仕様」セクションに疑似インタフェースを残す場合、本ディレクトリの IDL との対応（例: 疑似 `options.timeout_seconds` は IDL の `InvokeRequest.timeout_ms`）を明記する。対応記述のない疑似インタフェースは、要件ファイル更新時に IDL 側を破壊的に変更してしまうリスクがあるため、レビューで指摘される。

## IDL バージョニングと配布

tier1 API の IDL は SemVer で管理する。MAJOR は破壊的変更（メッセージ削除、RPC 削除）、MINOR は追加（新 RPC、新フィールド tag）、PATCH はドキュメント修正のみ。破壊的変更は OR-EOL-001 の非推奨ライフサイクルに従い 12 か月前告知。

IDL ファイルは Git モノレポ内の `proto/k1s0/tier1/` 配下で管理し、Buf（buf.build）で lint/breaking check を CI で強制する。tier2/tier3 クライアントライブラリは buf generate で Rust/Go/C# から生成、Nexus/Artifactory に公開する。

## メンテナンス

IDL の変更は ADR-TIER1-002（内部通信 Protobuf gRPC）と連動して行う。要件変更時に本ディレクトリの IDL スケルトンが整合しない場合、PR で同時更新必須。四半期ごとに Product Council で IDL の網羅性と SemVer 適合をレビュー。
