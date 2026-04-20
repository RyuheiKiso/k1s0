# 10. Audit-Pii API 方式

本ファイルは IPA 共通フレーム 2013 の 7.1.2.2「外部及びコンポーネント間のインタフェース方式設計（外部側）」に対応し、tier1 が tier2 / tier3 へ公開する `k1s0.public.audit.v1.Audit` サービスの外部インタフェース詳細を固定化する。共通契約は [../01_tier1_11API方式概要.md](../01_tier1_11API方式概要.md) DS-SW-EIF-001〜016 を参照とし、本ファイルは Audit-Pii API 固有の責務・メソッド・ハッシュチェーン設計・WORM 永続化・PII マスキングを扱う。

## 本ファイルの位置付け

Audit-Pii API は tier1 の「統制抽象」そのものである。J-SOX（金融商品取引法第 24 条の 4 の 4、内部統制報告制度）は上場企業に対して財務報告の信頼性を担保する業務プロセス監査を求め、個人情報保護法（2022 年改正）は個人情報の取扱い履歴を求める。過去 JTC 事例では監査証跡を個別アプリで実装した結果、フォーマット不統一・保管期間の不揃い・改ざん検知欠如で監査指摘を受ける事案が多発した。

tier1 は Rust 実装 custom-audit pod で「ハッシュチェーン監査」「WORM（Write-Once-Read-Many）永続化」「PII 自動マスキング」の 3 機能を一体提供し、アプリ側コードは `Audit.RecordEvent` 呼び出し 1 行で統制要件を充足する構成にする。本ファイルは [ADR-AUDIT-001]（WORM 監査採用）を API レベルで実装する方式設計であり、J-SOX / 個人情報保護法 / GDPR（EU 子会社向け）の 3 規制を単一 API で同時充足する契約を固定する。

## サービス定義と公開メソッド

Audit-Pii API は Protobuf サービス `k1s0.public.audit.v1.Audit` として定義し、以下 4 メソッドを公開する。記録系（RecordEvent）と検証系（Query / VerifyChain）、管理系（Seal）の 3 分類で権限分離する。

**設計項目 DS-SW-EIF-400 Audit サービスのメソッド粒度**

4 メソッドは `RecordEvent`（単一イベント記録）/ `Query`（監査イベント検索）/ `Seal`（四半期 chain root の外部 TSA 署名）/ `VerifyChain`（chain 整合性検証）で構成する。`RecordEvent` は tier2 / tier3 から日常的に呼ばれ、書き込み専用権限（`audit.record`）を要する。`Query` / `VerifyChain` は監査担当者・内部監査部からのみ呼ばれ、読み取り専用権限（`audit.read` / `audit.verify`）を要する。`Seal` は年 4 回の締め処理でのみ呼ばれ、統制責任者権限（`audit.seal`）を要する。これらの権限分離は [../../../03_要件定義/30_非機能要件/](../../../03_要件定義/30_非機能要件/) NFR-E-SEC-005 の「監査証跡への書き込み / 読み取り / 改ざん検証の責任分離」を API メソッド単位で物理実装する。書き込みエンドポイントと読み取りエンドポイントを別 Pod にもデプロイし、誤った権限設定で書き込み者が過去ログを読めてしまう事故を防ぐ構成は [../../50_非機能方式設計/03_セキュリティ方式.md](../../50_非機能方式設計/) と連動する。

## ハッシュチェーン監査

改ざん防止の要は「過去イベントを書き換えると以降全イベントのハッシュが変わる」性質を使うハッシュチェーンである。tier1 は Postgres テーブルレベルでこの性質を物理化する。

**設計項目 DS-SW-EIF-401 ハッシュチェーンの数学的構造**

監査イベントテーブル `audit_events` は以下カラムを持つ: `sequence_no`（BIGSERIAL PRIMARY KEY、単調増加）/ `tenant_id`（UUID、RLS 対象）/ `recorded_at`（TIMESTAMPTZ、サーバ時刻）/ `payload_json`（JSONB、イベント本体、DS-SW-EIF-404 参照）/ `prev_hash`（BYTEA、前イベントの entry_hash）/ `entry_hash`（BYTEA、`SHA-256(sequence_no || recorded_at || payload_json || prev_hash)`）。最初のイベントの `prev_hash` は固定値（SHA-256 of "k1s0-audit-chain-genesis-v1"）とする。`entry_hash` は `BEFORE INSERT` トリガで計算し、アプリから投入値を受け入れない。`sequence_no` は tenant 横断の単一チェーンとし、マルチテナント環境でも改ざん検知は横断で機能する。この構造は Amazon QLDB や Ethereum のブロックチェーンと同等の改ざん耐性を Postgres 単体で提供する。

**設計項目 DS-SW-EIF-402 WORM（Write-Once-Read-Many）制約の物理実装**

`audit_events` テーブルは以下の物理制約で WORM を強制する: (1) `UPDATE` / `DELETE` を許可する Role を存在させず、アプリ接続ロールは `INSERT` / `SELECT` のみ、(2) `BEFORE UPDATE` / `BEFORE DELETE` トリガで `RAISE EXCEPTION 'IMMUTABLE_VIOLATION'`、(3) Temporal Tables（Postgres 拡張）は採用せず、バージョン履歴そのものを持たない append-only 構成、(4) `pg_dump` / `pg_restore` 時の改変検知は Seal 時に chain root を外部 TSA 署名することで間接的に検出する。(1)(2) の二重防御は運用ミスによる Role 昇格で削除権限が付与された場合のセーフティネットとして機能する。

## 冷データ退避と長期保存

J-SOX は 7 年保管、個人情報保護法は原則 3 年（業種により 5〜7 年）、企業年金は 10 年と保管期間が最大 10 年に及ぶ。Postgres 単体では 10 年分のホットストレージは費用対効果が悪く、冷データ退避が必須である。

**設計項目 DS-SW-EIF-403 MinIO S3 Object Lock への冷データ退避**

90 日超過したイベントは夜間バッチで MinIO S3 Object Lock の COMPLIANCE モード（S3 互換、削除防止ロック）バケットに退避する。退避単位は「tenant_id + 日付（UTC）」で 1 オブジェクトとし、JSON Lines gzip 圧縮形式で書き込む。オブジェクトには Object Lock retention 10 年を強制設定し、MinIO 管理者権限でも削除不能とする（COMPLIANCE モードの定義上、root 権限での override も不可）。退避完了後は Postgres 側のレコードを削除せず、`archived_at` カラムに退避日時を記録する形で両系に保持し、退避 30 日後の二重整合性確認バッチで整合後に Postgres 側を削除する。これは退避中の障害で監査証跡を喪失するリスクを排除するためである。

## イベントスキーマと必須フィールド

監査イベントのスキーマは規制対応の要であり、欠落フィールドは監査指摘に直結する。本 API は必須フィールドを Protobuf レベルで型強制する。

**設計項目 DS-SW-EIF-404 RecordEvent のイベントスキーマ**

`RecordEventRequest` は以下必須フィールドを持つ: `actor`（`Actor` メッセージ: `subject_id` / `subject_type` ∈ {human, service, system} / `session_id`）、`action`（string、`<domain>.<verb>` 形式、例 `order.create` / `payment.approve`）、`resource`（`Resource` メッセージ: `type` / `id` / `tenant_id`）、`outcome`（enum: SUCCESS / FAILURE / DENIED / PARTIAL）、`pii_classification`（enum: NONE / INDIRECT_PII / DIRECT_PII / SENSITIVE_PII）、`context`（map<string, string>、trace_id / ip / user_agent 等）、`occurred_at`（Timestamp、クライアント時刻）。`pii_classification` が NONE 以外の場合は PII マスキング処理を tier1 側で自動適用する（DS-SW-EIF-406）。`pii_classification` の欠落は `PII_CLASSIFICATION_REQUIRED` エラーとし、アプリ側に PII 含有の有無を明示させることで「うっかり PII 記録」を構造的に防ぐ。

**設計項目 DS-SW-EIF-405 イベントサイズ上限とエンクリプト分離**

1 イベントの上限サイズは 64KB（`payload_json` + `context` の合計）とする。この値は Postgres TOAST 閾値（2KB）を超える JSONB でも効率的に格納でき、かつ監査ログが肥大化して DB サイズ爆発を起こさない境界として設計した。64KB を超えるペイロード（ファイルアップロード履歴など）は `resource.id` に MinIO オブジェクト URL を記録し、本体は別途 S3 保管する間接参照の運用に誘導する。`pii_classification = DIRECT_PII / SENSITIVE_PII` のイベントは上限を 16KB に厳格化し、かつ `payload_json` を Vault Transit で暗号化（KMS キー: `audit-pii-<year>`）してから Postgres に格納する。年次キーローテーションにより侵害時の影響範囲を当年分に限定する。

## PII マスキング

PII マスキングは「確定的である」（同一入力は常に同一マスク出力）ことが分析と削除要求（GDPR Right to Erasure）の両立に必要である。tier1 は Rust 実装で確定的マスキングを提供する。

**設計項目 DS-SW-EIF-406 確定的 PII マスキングの実装**

Rust の custom-audit pod は以下 4 種の PII を正規表現 + チェックデジット検証で検出する: (1) マイナンバー（12 桁、検査用数字 11 で照合）、(2) クレジットカード番号（13-19 桁、Luhn check）、(3) メールアドレス（RFC 5322）、(4) 電話番号（E.164 と国内 10/11 桁）。検出した PII は HMAC-SHA-256（秘密鍵: Vault 内 `pii-mask-key-<year>`）でハッシュし、先頭 8 文字を `<type>_<hash>` 形式で置換する（例: `090-1234-5678` → `phone_a3b7c1d2`）。この方式は同一入力値が常に同一マスク出力になる確定性を持ち、マスク後データ同士の集計（同一クレカ番号の使用回数など）を可能にしつつ、元値への復元は鍵保管者以外不可能である。マスキング辞書（正規表現 + 鍵世代）はバージョン管理し、過去イベントは記録当時のマスキングバージョンで保持する `masking_version` カラムを追加する。

**設計項目 DS-SW-EIF-407 GDPR Right to Erasure とハッシュ化置換**

GDPR 第 17 条（忘れられる権利）により EU 個人から削除要求を受けた場合、監査証跡上の当該個人情報を削除する必要がある。ただし J-SOX は削除を禁じる。この相反を解決するため、tier1 は「元イベントを削除せず、PII フィールドを不可逆ハッシュ（SHA-256 + ランダム salt、salt は削除要求時点で破棄）で置換する」手法を採る。これにより J-SOX 向けに「イベントが発生した事実」は残しつつ、GDPR 向けに「特定個人が識別不能」な状態を実現する。置換操作は例外的に `UPDATE` を許可する専用ロール `gdpr_eraser`（四半期に 1 回のみ手動付与、完了後即撤廃）で実行し、操作自体も別テーブル `gdpr_erasure_log` に記録する。ハッシュチェーン整合性は置換時に新 `entry_hash` を再計算し、以降の chain を chained rehash で再構築する（Seal 前のデータのみ対象、Seal 後は別途法務協議）。

## Seal と外部タイムスタンプ

ハッシュチェーンは「tier1 内部で改ざん検知できる」状態を提供するが、tier1 管理者自身が chain 全体を書き換える攻撃には無力である。これを防ぐため、chain root を外部 TSA（Time Stamping Authority）に定期署名させる。

**設計項目 DS-SW-EIF-408 四半期 Seal 処理と TSA 署名**

`Seal` メソッドは四半期末（3/6/9/12 月末）に統制責任者が手動起動する。処理内容は (1) 直近の `sequence_no` 最大値 N を取得、(2) `1` から `N` までの全イベントの `entry_hash` 連結を SHA-256 し chain root `R_quarter` を生成、(3) `R_quarter` を RFC 3161 準拠の外部 TSA（具体的には SECOM トラストシステムズ TSA、または Sigstore Rekor transparency log）に送付しタイムスタンプトークン `TSA_token` を取得、(4) `seals` テーブルに `(quarter, N, R_quarter, TSA_token, sealed_at, sealed_by)` を記録。Seal 完了後、当該四半期イベントは tier1 管理者でも改ざん不能となる（改ざんすると chain root が変わり、TSA 署名と不一致で検知される）。Seal 失敗時は `SEAL_FAILED` を返し、TSA 障害・ネットワーク障害・権限不足などの詳細を `details[]` に含める。

**設計項目 DS-SW-EIF-409 VerifyChain の整合性検証**

`VerifyChain` は指定期間（最大 7 年分）のハッシュチェーン整合性を検証する。処理内容は (1) 開始 `sequence_no` から終了 `sequence_no` まで全レコード取得、(2) 各レコードの `entry_hash` を再計算し DB 格納値と照合、(3) `prev_hash` が前レコードの `entry_hash` と一致することを確認、(4) 該当期間をカバーする `seals` エントリの `R_quarter` と TSA 署名の整合を検証。7 年分のフル検証は 10 億件規模となるため、インクリメンタル検証モード（前回検証完了時点 + 差分のみ）を提供する。差分検証結果は `chain_verifications` テーブルに記録し、年次監査時に過去の検証履歴を提示可能にする。検証 SLO は 7 年分インクリメンタルで p99 2 時間、フル検証は p99 2 日（年次バッチで実施）とする。

## SLO と性能設計

Audit-Pii API は書き込み性能が監査のボトルネックになる一方、クエリ性能は監査担当者の作業時間に直結する。両者のバランスを設計で固定する。

**設計項目 DS-SW-EIF-410 RecordEvent / Query の SLO 目標**

`RecordEvent` p99 30ms（DS-SW-EIF-013 概要ファイル記載）の内訳は PII マスキング 5ms + ハッシュ計算 5ms + Postgres WAL commit 15ms + オーバーヘッド 5ms である。`Query` p99 2s の内訳は Postgres インデックスルックアップ 500ms + 結果フェッチ 1,000ms + シリアライズ 500ms で、最大 10,000 件までの検索を対象とする。10,000 件超過のクエリは BigQuery エクスポート経由（別途 [../../40_制御方式設計/](../../40_制御方式設計/) で規定）に誘導し、本 API では拒否する。書き込み性能の安全側設計として、`RecordEvent` は同期的に Postgres への commit 完了を待ってから OK を返す（非同期キューを挟まない）。これは監査要件上「イベント発生時点で記録完了」が求められるためで、性能と規制遵守のトレードオフで規制遵守を優先した結果である。

**設計項目 DS-SW-EIF-411 クエリの RLS とクエリ制限**

`Query` は tenant_id / actor / time-range の 3 条件のいずれかを必須とし、条件なしの全件走査を禁止する。tenant_id は Postgres RLS（Row Level Security）で強制され、JWT の tenant_id claim と一致する行のみ返却する。time-range は最大 90 日の範囲制限、actor 検索は完全一致のみ（部分一致禁止、インデックス効率のため）。結果セットは最大 10,000 件でページング（cursor 方式、offset 方式は Postgres 深い offset の性能問題から禁止）する。この制約は監査担当者の通常業務を満たしつつ、悪意あるクエリによる DB 過負荷を構造的に防ぐ。

## エラーコード

Audit-Pii API 固有のエラーは K1s0Error の `code` フィールドに以下 6 種を登録する。

**設計項目 DS-SW-EIF-412 Audit-Pii 固有エラーコード**

| コード | gRPC status | 発生条件 | 根拠 |
|--------|-------------|----------|------|
| `CHAIN_BROKEN` | DATA_LOSS | VerifyChain でハッシュ不整合検出 | 即座の運用対応が必要な最重大イベント |
| `SEAL_FAILED` | UNAVAILABLE | TSA 送信失敗 or chain root 計算失敗 | 四半期締めの再実行判断用 |
| `PII_CLASSIFICATION_REQUIRED` | INVALID_ARGUMENT | RecordEvent で pii_classification 欠落 | DS-SW-EIF-404 PII 明示必須 |
| `IMMUTABLE_VIOLATION` | PERMISSION_DENIED | UPDATE / DELETE 試行 | DS-SW-EIF-402 WORM 制約 |
| `EVENT_TOO_LARGE` | RESOURCE_EXHAUSTED | 1 イベント > 64KB（PII 含む場合 > 16KB） | DS-SW-EIF-405 サイズ上限 |
| `QUERY_CONDITION_REQUIRED` | INVALID_ARGUMENT | tenant_id / actor / time-range 全欠落 | DS-SW-EIF-411 全件走査禁止 |

## 保持期間と規制対応

**設計項目 DS-SW-EIF-413 保持期間ポリシー**

保持期間はイベント種別で分類する: (1) 一般業務イベント（注文 / 決済 / 承認）7 年、(2) 個人情報アクセス / 削除イベント 5 年、(3) 認証 / セッション 2 年、(4) システム障害 / 例外 3 年。期間は `audit_events.retention_policy` カラムで記録時に決定し、冷データ退避後も MinIO Object Lock の retention 値に反映する。期間超過後の削除は年次バッチで行うが、Seal 済み期間のイベントは削除せず `retention_expired = true` フラグで検索結果から除外する運用に留める。これは規制改正で遡及的に保管期間が伸長される可能性（個人情報保護法の過去改正事例あり）への備えである。

## フェーズ別提供範囲

**設計項目 DS-SW-EIF-414 フェーズ別提供範囲**

Phase 1a（MVP-0）: 提供なし。Phase 1b（MVP-1a）: `RecordEvent` / `Query` の 2 メソッド、ハッシュチェーン有効、WORM 有効、PII マスキング（マイナンバー / クレカ / メール / 電話の 4 種）有効、冷データ退避は未実装（全量 Postgres 保持）。Phase 1c（MVP-1b）: `Seal` / `VerifyChain` 追加、MinIO S3 Object Lock 冷データ退避有効、TSA 連携（SECOM TSA）有効。Phase 2: GDPR Right to Erasure フロー実装、BigQuery エクスポート経由の長期分析、マスキング辞書の追加（住所 / 氏名の ML 検出検討）。

## 対応要件一覧

本ファイルは Audit-Pii API 公開インタフェースの詳細方式設計であり、以下の要件 ID に対応する。

- FR-T1-AUDIT-001〜FR-T1-AUDIT-003（Audit 機能要件、ハッシュチェーン / WORM / Seal & VerifyChain）
- FR-T1-AUDIT-001（RecordEvent とハッシュチェーン整合性）/ FR-T1-AUDIT-002（WORM 保持と冷データ退避）/ FR-T1-AUDIT-003（Seal と TSA 署名 / VerifyChain）
- FR-T1-PII-001（PII 自動マスキングとマスキング辞書管理）
- FR-T1-PII-002（PII 漏洩検出と個別エンクリプト退避）
- NFR-E-SEC-005（監査証跡の書き込み / 読み取り / 改ざん検証の責任分離）
- NFR-E-SEC-006（改ざん防止、ハッシュチェーン + 外部 TSA）
- NFR-G-PRI-001〜004（PII 自動マスキング、個人情報保護法 / GDPR 対応）
- NFR-H-LEG-001（J-SOX 7 年保管）
- NFR-H-LEG-003（GDPR Right to Erasure）
- ADR 参照: ADR-TIER1-001（Go+Rust 分担、Audit-Pii は Rust）/ ADR-TIER1-002（Protobuf gRPC 必須）/ ADR-AUDIT-001（WORM 監査採用）
- 共通契約: DS-SW-EIF-001〜016（[../01_tier1_11API方式概要.md](../01_tier1_11API方式概要.md)）
- 本ファイルで採番: DS-SW-EIF-400 〜 DS-SW-EIF-414
