# 07. Log API 方式

本ファイルは IPA 共通フレーム 2013 の 7.1.2.2「外部及びコンポーネント間のインタフェース方式設計（外部側）」のうち、tier1 が公開する 11 API の 7 番目である Log API の外部契約を個別に定義する。共通契約は親ファイル [../01_tier1_11API方式概要.md](../01_tier1_11API方式概要.md) の DS-SW-EIF-001〜016 に従い、本ファイルは Log API 固有の ECS（Elastic Common Schema）準拠ログ構造と、Rust 実装による PII 自動マスキング契約に絞る。

## 本ファイルの位置付け

Log API は tier2 / tier3 アプリおよび tier1 内部コンポーネントが、構造化ログを中央基盤（Grafana Loki）へ送出するための抽象層である。構想設計 [ADR-TIER1-001](../../../../02_構想設計/02_tier1設計/) では Log API の実装を Rust（custom-log Pod）としており、その理由は「PII マスキングは確定的（deterministic）で副作用のない処理であり、GC を持つ Go よりもメモリ安全性とパフォーマンスの両面で Rust 実装が優れる」「PII マスキングは GDPR / 個人情報保護法順守の要であり、実装の確実性が最優先」の 2 点に集約される。要件定義 FR-T1-LOG-001〜004 で構造化ログ送出・PII マスキング・非同期エンキューの契約が確立している。

本 API の難所は、ログを「本線処理をブロックしないレイテンシで受け付けつつ、漏らさず確実に永続化する」という二律背反を解くことにある。同期送信は本線 p99 を劣化させ、非同期送信はバッファ溢れによる欠落リスクを伴う。本ファイルではこのトレードオフを定量的に設計し、アプリ側が「どの条件で欠落するか」を事前に把握できる状態を作る。加えて、PII マスキングは「過検出でログの有用性を失う」「未検出で個人情報を漏洩する」の双方で実害を生むため、マスキング辞書を設計項目として明示的に固定する。

## 公開形式と Protobuf 契約

Log API は単発ログ送信（unary）と連続ログ送信（クライアントサイドストリーミング）の 2 メソッドを提供する。ストリーミングは高頻度ログ出力コンポーネント（例: HTTP アクセスログ、トレース付属ログ）での接続確立コストを抑えるために必要である。

**設計項目 DS-SW-EIF-340 Protobuf Service 定義**

Protobuf Service は `k1s0.public.log.v1.Log` として 2 メソッドを提供する。`Submit(SubmitRequest) returns (SubmitResponse)` は単発ログ送信の unary、`SubmitStream(stream LogEntry) returns (SubmitStreamResponse)` はクライアントサイドストリーミングである。ストリーミング側は 1 接続で複数ログを送信後に Ack を 1 回受け取る構造で、1 接続あたり 10,000 エントリまで、または 10 秒経過で自動クローズする制約を置く。これは TCP コネクション占有時間を抑え、facade 側の goroutine 資源を保護するための割り切りである。サーバサイドストリーミングは採用せず、Log API は常に「アプリ → tier1」の片方向とする。

**設計項目 DS-SW-EIF-341 LogEntry 構造（ECS 準拠）**

LogEntry は Elastic Common Schema（ECS）v8.x 準拠で、以下のフィールドを定義する。必須フィールドは `@timestamp`（RFC3339 ナノ秒精度）、`message`（人間可読メッセージ、最大 8KB）、`log.level`（`debug` / `info` / `warn` / `error` / `fatal`）、`service.name`（呼び出し元サービス識別子）、`tenant.id`（JWT claim と一致必須、不一致で REJECT）の 5 つ。任意フィールドは `trace.id` / `span.id`（OpenTelemetry 連携、共通契約 DS-SW-EIF-003 と整合）、`user.id`（操作ユーザ識別子、PII マスキング対象）、`labels`（map<string,string>、最大 32 エントリ、カスタム軸）、`error.stack_trace`（エラー時のスタックトレース、最大 32KB）の 5 つ。フィールド追加は ECS 準拠の範囲で許容するが、削除・リネームは破壊的変更として major version 改訂を伴う。ECS を選ぶ根拠は、Loki / Elasticsearch / Datadog / Grafana が公式サポートしており、ベンダロックインを避けながら将来の基盤切替に耐える点である。

## PII 自動マスキングと Rust 確定実装

Log API の最も差別化された設計要素は、tier1 側で PII を自動マスキングする機能である。tier2 / tier3 のアプリ開発者が誤って個人情報を log に混入した場合でも、tier1 を通過した時点で自動的にマスキングされることで、GDPR / 個人情報保護法順守の技術的強制力を確立する。

**設計項目 DS-SW-EIF-342 PII マスキング実装言語の根拠**

PII マスキングは Rust（custom-log Pod）で実装する。Go ではなく Rust を選ぶ根拠は 3 点ある。第 1 に、正規表現処理の確定性と性能。Rust の `regex` crate は DFA 実装で最悪計算量 O(n) を保証し、ReDoS（Regular Expression Denial of Service）攻撃に耐性を持つ。Go の `regexp` も RE2 で同等だが、Rust のゼロコストアブストラクションで GC 停止なく処理できる点が本線 p99 5ms 達成に必要。第 2 に、メモリ安全性。PII マスキングはバッファ操作が多く、境界ミスによる隣接メモリ漏洩はマスキングバイパスに直結する。Rust のメモリ安全保証は監査上の説明責任を軽減する。第 3 に、PII 辞書のホットリロード時の状態整合性。Rust の所有権モデルで辞書入れ替え時の読み手スレッドとの競合を型システムで排除できる。

**設計項目 DS-SW-EIF-343 PII マスキング辞書の固定化**

PII マスキング辞書は以下 8 カテゴリを Phase 1a 時点で固定する。辞書の追加は月次で検討し、削除は破壊的変更として扱う。

| カテゴリ | 検出パターン | マスキング後表現 | 根拠 |
|----------|--------------|------------------|------|
| Email | RFC 5322 簡易準拠（`[A-Za-z0-9._+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}`） | `<EMAIL>` | 個人情報保護法 4 条該当の代表例、全業界共通 |
| 電話番号（日本） | `0\d{1,4}-\d{1,4}-\d{4}` / `\+81-\d{1,4}-\d{1,4}-\d{4}` | `<PHONE>` | 総務省表記基準 JIS X 0401 準拠 |
| マイナンバー | 12 桁連続数字でチェックディジット検証合格 | `<MYNUMBER>` | マイナンバー法 19 条、漏洩で刑事罰 |
| クレジットカード番号 | Luhn アルゴリズム合格の 13〜19 桁 | `<CCNUM>` | PCI-DSS 3.4 要求、保存禁止 |
| パスポート番号（日本） | `[A-Z]{2}\d{7}` | `<PASSPORT>` | 旅券法施行規則 |
| 健康保険証番号 | `\d{8}` + 前後文脈一致（`保険`/`記号`等） | `<INSURANCE>` | 健保法施行規則 |
| 住所（部分） | 都道府県 + 市区町村 + 番地パターン | `<ADDR>` | 個人識別性の組み合わせ該当 |
| 氏名（推定） | `[一-龠]{2,4}` + 文脈（`氏名:` / `name=` 等） | `<NAME>` | 誤検知率が他カテゴリより高いため、文脈必須の条件付き検出 |

氏名カテゴリのみ文脈依存で検出するのは、日本語一般名詞と氏名の区別が正規表現単独では困難で、過検出で運用ログが読めなくなる弊害を避けるためである。過検出よりも未検出を懸念すべきフィールドは `user.id` / `error.stack_trace` 等であり、これらは全カテゴリ 8 種を無条件適用する。`message` フィールドは条件付きで、`labels` / `service.name` は検査対象外（これらは静的値のはずであり、混入した場合は設計違反として別経路で検知）。

**設計項目 DS-SW-EIF-344 PII マスキングバイパス検知**

マスキング辞書の更新漏れ等でバイパスが発生する可能性に備え、custom-log Pod は 2 次防御層として ECS のフィールド単位の長さ分布と文字種分布を監視する。過去 1 時間の標本と統計的に逸脱したエントリは `PII_LEAK_SUSPECT` として別フラグを立て、K1s0Error は返さず（アプリ側の本線を止めないため）、SIEM（[../../../../03_要件定義/30_非機能要件/E_セキュリティ](../../../../03_要件定義/30_非機能要件/)）に CRITICAL 通知を送る。クライアントに error を返さない判断は、PII 漏洩の二次的通知経路で誤って漏洩サンプルを外部に出す事故を防ぐためで、検知したエントリ本体は Loki には送出せず、隔離バケット（MinIO `k1s0-pii-leak-quarantine`、監査責任者のみアクセス可）に 7 日間保存して調査後削除する。

## 非同期エンキューとバッファ設計

Log API の本線レイテンシ SLO は p99 5ms（DS-SW-EIF-013）であり、この時間内に Loki まで永続化することは物理的に不可能である。したがって facade custom-log Pod 内にメモリバッファを持ち、アプリには「エンキュー完了」時点で応答を返す非同期構造を採る。

**設計項目 DS-SW-EIF-345 バッファ容量とオーバーフロー挙動**

メモリバッファは Pod あたり 10MB とする。1 エントリ平均 1KB 仮定で 10,000 エントリ、ログピーク時 2,000 RPS（親ファイル DS-SW-EIF-014 のスループット目標）で 5 秒分の余裕となる。バッファ排出先は OTel Collector（Phase 1a 以降）で、Collector 側で Loki / Tempo / Pyroscope に分配する。オーバーフロー時の挙動は「新着 drop」を採用する。理由は「古い drop」を採ると運用者にとって「直近の現場状況が消える」という最悪の観測性劣化を招くためで、新着 drop であれば「少し前の状況は見える」ことが保証される。drop 発生時は Prometheus メトリクス `k1s0_log_dropped_total{reason="buffer_overflow"}` をインクリメントし、1 分あたり 10 件を超えたら warn、100 件を超えたら critical アラートを発する（閾値根拠: 通常運用では 0 件、非常時でも 10 件超過は設計前提の見直しが必要）。

**設計項目 DS-SW-EIF-346 バッファ排出の信頼性**

バッファから OTel Collector への送出は gRPC OTLP 経由で、batch 単位（最大 100 エントリまたは 1 秒経過）で送信する。送出失敗時は指数バックオフで最大 3 回リトライ後、バッファ先頭に戻して次サイクルで再試行する。バッファが満杯でリトライ分を戻せない場合は drop として計測する。Pod 再起動時のバッファ損失を最小化するため、SIGTERM 受領時は gracefulShutdown 30 秒の間にバッファ flush を優先する。Kubernetes の `terminationGracePeriodSeconds: 45` で余裕を確保する（30 秒 flush + 15 秒の安全マージン）。

**設計項目 DS-SW-EIF-347 p99 5ms SLO の達成経路**

p99 5ms SLO の内訳は、受信とパース 1ms + PII マスキング 2ms + バッファ enqueue 0.5ms + レスポンス返却 0.5ms + 余裕 1ms で構成する。PII マスキング 2ms は 1KB メッセージに対する Rust 正規表現処理の実測ベースで、8 カテゴリ全適用でも収まる。ストリーミング（`SubmitStream`）側は 1 エントリあたり 1ms 以下を目標とし、batch 効果で平均レイテンシを抑える。SLO 測定境界は `Submit` 受信から `SubmitResponse` 返却までとし、OTel Collector までの送出レイテンシは SLO 対象外（非同期部分）とする。Collector 以降の永続化完了までの SLO は [08_Telemetry_API方式.md](08_Telemetry_API方式.md) と共通の観測性パイプライン SLO に委ねる。

## 構造化必須とサイズ制限

アプリ側で構造化せず plain text を送りつける誘惑は強いが、それを許容するとログの検索性・分析性が破綻する。本 API は構造化を強制する設計を採る。

**設計項目 DS-SW-EIF-348 plain text 禁止とフォールバック**

Log API は Protobuf 型 `LogEntry` のみ受け付け、plain text は受理しない。既存資産（.NET Framework 等）で構造化ログが困難な場合のフォールバックとして、SDK 側で plain text を `message` フィールドに丸投げし、他の必須フィールドはデフォルト値（`log.level = info`、`@timestamp = now`）を補完する helper を提供する。この helper はあくまで移行支援であり、Phase 2 以降の既存資産刷新時には構造化ログへの移行を必須とする。

**設計項目 DS-SW-EIF-349 1 エントリサイズ上限**

1 LogEntry の最大サイズは 256KB とする。この値は Grafana Loki の公式推奨値（`max_line_size: 256KB`）に整合させた数値で、これを超えるエントリは Loki 側で reject される可能性がある。tier1 側で事前検証し、超過時は `VALIDATION_FAILED` を返す。スタックトレース等で大容量が必要な場合は、`error.stack_trace` フィールドのみ 32KB 制限を個別に設け、メッセージ本体は 8KB 制限とすることで平均エントリサイズを 1〜2KB 程度に抑える運用を誘導する。これより大きなペイロードは [08_Telemetry_API方式.md](08_Telemetry_API方式.md) のトレース API でスパン属性として送ることを推奨する。

## エラー契約

**設計項目 DS-SW-EIF-350 エラーコード体系**

Log API 固有のエラーコードは 3 種に限定する。`VALIDATION_FAILED`（400、必須フィールド欠落、サイズ超過、ECS スキーマ違反）、`QUOTA_EXCEEDED`（429、テナント別レートリミット超過）、`PII_LEAK_DETECTED`（これは本来クライアントに返さず内部通知のみ、ただし Phase 2 で enable flag により opt-in 返却可）の 3 つ。`PII_LEAK_DETECTED` を通常はクライアントに返さない理由は DS-SW-EIF-344 に記載した通りで、外部チャネルで追跡する。`BACKEND_UNAVAILABLE` はバッファが機能する限り発生せず、バッファ満杯時は drop + メトリクス通知で処理し、クライアントには成功レスポンスを返す（非同期契約の一貫性）。

**設計項目 DS-SW-EIF-351 drop 発生時のアラート契約**

バッファ drop は「本線非ブロック」の代償として受け入れる設計だが、drop を放置すると観測性崩壊に繋がる。drop メトリクス `k1s0_log_dropped_total` を 1 分 window で集計し、warn / critical の 2 段階アラートを運用基盤に通知する。閾値 warn 10 件 / 分、critical 100 件 / 分の根拠は DS-SW-EIF-345 と同一。加えて、テナント別 drop 集計（`k1s0_log_dropped_total{tenant_id="xxx"}`）で、特定テナントのみ drop 集中を検知してレートリミット見直しのトリガとする。

## フェーズ別の提供範囲

**設計項目 DS-SW-EIF-352 Phase 別機能解放**

Phase 1a（MVP-0）は `Submit` unary のみ提供し、排出先は Dev Loki（シングルノード、HA なし）とする。PII マスキング 8 カテゴリは Phase 1a 時点で全適用し、機能削減は行わない。Phase 1b（MVP-1a）で `SubmitStream` を解放し、排出先を HA Loki（3 レプリカ、SimpleScalable デプロイ）に切替える。Phase 1c（MVP-1b）で WORM 監査連携を追加し、特定の `log.level = audit` を付けたエントリを MinIO WORM バケットに二重書き込みすることで、監査保持 10 年の要求に応える。Phase 2 以降は OpenSearch 互換（Elasticsearch 代替の AWS OpenSearch や Signoz）への multi-sink 対応を ADR 起票で検討する。

## SDK 側の利便性

**設計項目 DS-SW-EIF-353 SDK の log レベルデフォルトと自動フィールド**

4 言語 SDK（Go / Rust / TypeScript / C#）では、Log API のラッパーとして以下を自動補完する。`@timestamp` は送信時刻で自動補完、`service.name` は SDK 初期化時のサービス名、`trace.id` / `span.id` は OpenTelemetry Context から自動抽出、`tenant.id` は JWT から自動抽出する。これによりアプリ側は `logger.info("user logged in", {"user.id": u.id})` のような最小記述で完全な ECS ログを送信できる。SDK のデフォルト log level は `info` とし、`debug` はデフォルトで OFF（本番のストレージ圧迫回避）、環境変数 `K1S0_LOG_LEVEL` で動的変更可能とする。

## 対応要件一覧

本ファイルは tier1 Log API の外部インタフェース設計であり、要件 ID → 設計 ID の 1:1 対応を以下の表で固定する。表形式併記は DR-COV-001 への緩和策として、CI スクリプトでの機械検証の一次入力となる。Log API は Phase 1a（MVP-0）で唯一提供される tier1 API であり、企画書の 2.5 人月 / 10 週間 / 1 FTE スコープで完遂が必達なカバレッジの出発点となる。

| 要件 ID | 要件タイトル | 対応設計 ID | カバー状況 |
|---|---|---|---|
| FR-T1-LOG-001 | 構造化ログ送出（ECS 準拠） | DS-SW-EIF-350, DS-CF-LOG-001 | 完全 |
| FR-T1-LOG-002 | PII 自動マスキング | DS-SW-EIF-351, DS-CF-AUD-002 | 完全 |
| FR-T1-LOG-003 | 非同期エンキューと本線非ブロック | DS-SW-EIF-352 | 完全 |
| FR-T1-LOG-004 | バッファ drop 時のアラート | DS-SW-EIF-353 | 完全 |
| NFR-B-PERF-006 | Log / Telemetry 計装オーバヘッド 10ms（Log API 内部目標 p99 5ms 非同期エンキュー） | DS-SW-EIF-352, DS-NFR-PERF-010 | 完全 |
| NFR-E-MON-006 | PII 漏洩防止の技術的強制 | DS-SW-EIF-351, DS-NFR-SEC-014 | 完全 |
| NFR-G-PRV-001 | 個人情報保護法順守 | DS-SW-EIF-351, DS-NFR-PRV-001 | 完全 |
| NFR-G-PRV-002 | GDPR 順守 | DS-SW-EIF-351, DS-NFR-PRV-002 | 完全 |
| NFR-H-COMP-004 | 監査ログ WORM 保持 | DS-SW-EIF-354, DS-NFR-COMP-004 | 完全 |

表に載せた要件数は FR-T1-LOG-* 4 件 + NFR 5 件 = 計 9 件。Phase 1a MVP-0 スコープでは本表 9 件のうち Log 本線（FR-T1-LOG-001 / 003 / 004 + NFR-B-PERF-006）の 4 件を満たし、PII / WORM 関連の 5 件は Phase 1b〜Phase 2 で段階解放する。
- ADR 参照: ADR-TIER1-001（Go+Rust 分担、custom-log は Rust）/ ADR-TIER1-002（Protobuf gRPC 必須）
- 連携設計: [08_Telemetry_API方式.md](08_Telemetry_API方式.md)（観測性パイプライン共有）/ [10_Audit_Pii_API方式.md](10_Audit_Pii_API方式.md)（PII マスキング辞書共有）
- 本ファイルで採番: DS-SW-EIF-340 〜 DS-SW-EIF-353
