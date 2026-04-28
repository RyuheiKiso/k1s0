# 05. Binding API 方式

本ファイルは IPA 共通フレーム 2013 の 7.1.2.2「外部及びコンポーネント間のインタフェース方式設計（外部側）」のうち、tier1 が公開する 11 API の 5 番目である Binding API の外部契約を個別に定義する。共通契約は親ファイル [../01_tier1_11API方式概要.md](../01_tier1_11API方式概要.md) の DS-SW-EIF-001〜016 に従い、本ファイルは Binding API 固有の Input / Output 双方向抽象と、外部プロトコル（MQTT / HTTP / Kafka / Cron / SMTP / S3 互換）との変換契約に絞る。

## 本ファイルの位置付け

Binding API は tier2 / tier3 アプリが、外部プロトコルを意識せずに「入力を受け取る」「出力を送る」の 2 方向の通信を行うための抽象層である。Service Invoke が「tier1 内部のマイクロサービス間の同期呼び出し」であるのに対し、Binding は「tier1 外部の異質なプロトコル」を対象にする点で性質が異なる。Dapr Binding コンポーネントの枠組みをそのまま採用し、facade-binding Pod（Go）で薄くラップする。構想設計 [ADR-TIER1-001](../../../../02_構想設計/02_tier1設計/) で facade-binding は Go に分類済みであり、要件定義 FR-T1-BINDING-001〜005 で Input / Output 双方向の要件が確立している。

Binding API の設計上の難所は、対象プロトコル毎に「配信保証」「冪等性」「エラー回復」の挙動が異なる点である。MQTT は QoS 設定次第で at-most-once / at-least-once が切り替わり、HTTP Webhook は net 分断時のリトライ責務が受信側にあり、Cron はトリガ重複を防ぐ排他制御が必須である。これらの差異を隠蔽しつつ、アプリ側に「どのプロトコルでも冪等に処理できる規律」を強制することが本ファイルの目的となる。そのため各設計項目では「プロトコル固有の挙動」を明示しつつ、それらを tier1 側でどこまで吸収するかの境界線を引く。

## 公開形式と Protobuf 契約

Binding API は Output（アプリから外部へ）が unary、Input（外部からアプリへ）がサーバサイドストリーミングという非対称な構造を取る。Output は「呼ぶだけで終わる」副作用操作、Input は「待ち受ける」イベント駆動であり、性質が異なるためメソッドを分ける。

**設計項目 DS-SW-EIF-300 Protobuf Service 定義**

Protobuf Service は `k1s0.public.binding.v1.Binding` として 2 メソッドを提供する。`InvokeOutput(InvokeOutputRequest) returns (InvokeOutputResponse)` は Output binding の単発送信、`Subscribe(SubscribeRequest) returns (stream InputEvent)` は Input binding のサーバサイドストリーミング購読である。`Subscribe` は Dapr の Input Binding が「コンポーネントからアプリへの HTTP 呼び出し」として実装されている仕組みを、tier1 ファサードで gRPC ストリームへ反転変換する。クライアントはストリームを開いたまま Ack を都度返送し、Ack されないイベントは一定時間後に再配信される at-least-once 保証となる。

**設計項目 DS-SW-EIF-301 Binding 種別とプロトコル対応表**

Input と Output で対応するプロトコルを整理する。根拠セルは単なる採用宣言ではなく「何故このプロトコルで、何故この段階か」を併記する。

| 方向 | プロトコル | 用途 | 適用段階 | 根拠 |
|------|------------|------|-------|------|
| Output | MQTT (Mosquitto) | エッジデバイスへのコマンド送出 | 採用初期 | 採用側組織の現場センサ連携で実測採用、[04_エッジIoTインタフェース方式.md](../04_エッジIoTインタフェース方式.md) DS-SW-EIF-115〜117 と整合 |
| Output | HTTP/1.1 + HTTPS | 外部 SaaS / レガシー REST への POST | 採用初期 | 既存 採用側組織の取引先連携の主力、ベストエフォートで p99 200ms |
| Output | Kafka | tier1 内 PubSub 以外の外部 Kafka クラスタ（顧客 Kafka 互換ミラー）へのブリッジ | 採用初期 | tier1 内の Kafka は PubSub API に専念させ、外部 Kafka は Binding として責務分離 |
| Output | S3 互換（MinIO） | バッチ成果物 / エクスポートファイルのオブジェクトストア書き込み | 採用初期 | 監査 WORM とは別経路、MinIO の AGPL は Pod 境界で隔離済み（[ADR-0003](../../../../02_構想設計/05_法務とコンプライアンス/)） |
| Output | SMTP | 通知メール送出 | 採用後の運用拡大時 | 送信レート制御と DKIM 署名のため独立 binding 化、運用蓄積後 |
| Input | MQTT (Mosquitto) | エッジセンサからのイベント受信 | 採用初期 | QoS 1 固定で at-least-once、デバイス側再送前提 |
| Input | HTTP Webhook | 外部 SaaS からのコールバック（Stripe / GitHub 等互換） | 採用初期 | HMAC 署名検証とタイムスタンプ窓で改ざん防止 |
| Input | Cron | 定期ジョブトリガ | 採用初期 | 業務バッチの起点として必須、UTC 基準で標準化 |
| Input | Kafka | 外部 Kafka クラスタからの購読ブリッジ | 採用後の運用拡大時 | リリース時点 は PubSub で完結 / で外部連携解放 |

各プロトコルは Dapr Binding Components の公式実装を一次候補とする。公式実装の安定度が不十分なプロトコル（SMTP や S3 互換）は、facade-binding 内で独自 wrapper を書かずに Dapr Component の SPI を実装して OSS に貢献する方針を採る。これは [../../../../03_要件定義/60_事業契約/07_OSSライセンス管理](../../../../03_要件定義/60_事業契約/) で宣言済みの OSS 還元方針と整合する。

## Input Binding の詳細契約

Input Binding はアプリが受け身で待つため、tier1 側で購読セッション管理・Ack 待ち合わせ・DLQ 転送を担う必要がある。ここが手薄だと、アプリ障害時にイベントを喪失する。

**設計項目 DS-SW-EIF-302 MQTT Input の QoS と再送**

MQTT Input は QoS 1 固定とする。QoS 0 は at-most-once で業務要件を満たさず、QoS 2 は exactly-once だが MQTT ブローカ実装の差異が大きく相互運用性が劣る。QoS 1 は at-least-once で、アプリ側に冪等性責務を押し付けるが、業界で最も実装が安定している。アプリは `Idempotency-Key` を MQTT ペイロード先頭 36 バイト UUID で受領し、重複排除する。購読トピックは `k1s0/<tenant_id>/<app_id>/in/<topic>` と階層化し、テナント境界を subscription ACL で強制する。エッジデバイス側の契約は [04_エッジIoTインタフェース方式.md](../04_エッジIoTインタフェース方式.md) の DS-SW-EIF-115〜117 を参照する。

**設計項目 DS-SW-EIF-303 HTTP Webhook Input の署名検証と時刻窓**

HTTP Webhook Input は HMAC-SHA256 署名検証と時刻窓 ±5 分の 2 条件を必須とする。署名は `X-K1s0-Signature: sha256=<hex>` ヘッダで、署名対象は `<timestamp>.<raw_body>` の連結、キーは OpenBao から `secret/<tenant_id>/<app_id>/webhook-hmac-key` として取得する。時刻窓 ±5 分の根拠は NTP 同期誤差（通常 1 秒以内）に加え、クライアント側クロックドリフトと再送遅延の両方を吸収するため Stripe の公式ガイドライン（5 分）を参考に採用した。窓外のリクエストは 401 で拒否し、リプレイ攻撃を防ぐ。加えて、過去 5 分以内に受領済みの署名をテナント別 Valkey Set にキャッシュし、同一署名の再受領も拒否する（二重防御）。

**設計項目 DS-SW-EIF-304 Cron Input の構文と時刻基準**

Cron Input の構文は標準 5 フィールド（`分 時 日 月 曜日`）に加え、先頭に秒フィールドを許容する拡張 6 フィールドを併用する。秒フィールド拡張は Dapr Cron Component の公式仕様に準拠する。時刻基準は UTC 固定とし、JST や tenant timezone での指定は許容しない。これは DST（夏時間）を持つ地域への将来展開時のトリガ重複 / 欠落を防ぐための割り切りで、tenant 毎の timezone 変換はアプリ側で行うことを規約とする。Cron トリガの重複実行防止は facade-binding 側で Valkey に `cron-lock:<tenant_id>:<binding_name>:<fire_time>` のキーを TTL 60 秒で置き、同一 fire time の二重実行を防ぐ。

## Output Binding の詳細契約

Output Binding は副作用を伴うため、冪等性・リトライ・DLQ の 3 点を tier1 側で規律する。ここが手薄だと、ネットワーク一時障害時に二重送信が発生する。

**設計項目 DS-SW-EIF-305 Output の冪等キー伝搬**

`InvokeOutput` 呼び出し時、クライアントは `Idempotency-Key` gRPC metadata を任意で設定できる（共通契約 DS-SW-EIF-005 を継承）。facade-binding は受領したキーを対象プロトコルへ伝搬する。HTTP Output は `Idempotency-Key` ヘッダをそのまま転送、Kafka Output は Kafka Header `idempotency-key` として設定、MQTT Output はペイロード先頭 36 バイトに UUID として埋め込む（MQTT v3 にヘッダが無いため）、SMTP Output は `Message-ID` として使用、S3 Output はオブジェクトキーに `?idempotency=<key>` クエリ相当のメタデータを付与する。受信側が冪等キーを解釈できない場合は至らぬとみなし、採用後の運用拡大時 で受信側互換性確保のオプションを検討する。

**設計項目 DS-SW-EIF-306 Output 失敗時のリトライと DLQ**

Output Binding が一時的失敗（ネットワーク断・5xx・タイムアウト）を検出した場合、facade-binding は指数バックオフで最大 3 回リトライする。初回 1 秒、2 回目 3 秒、3 回目 10 秒の間隔で、合計 14 秒以内に収束する。この数値は HTTP Output の p99 200ms SLO と「呼び出し側がタイムアウトする前に全リトライを諦めさせない」バランスから導出した。4 回目以降の失敗は DLQ（Kafka トピック `k1s0-binding-output-dlq`）に enqueue し、呼び出し元には `BACKEND_UNAVAILABLE` を返す。DLQ は PubSub API の DLQ（DS-SW-EIF-066 系）と同一基盤を再利用し、監査 / 再送は同じツールで一元化する。恒久的失敗（4xx 系）はリトライせず即座に `VALIDATION_FAILED` を返す。

**設計項目 DS-SW-EIF-307 Output best-effort SLO の計測境界**

Output Binding の SLO は親ファイル DS-SW-EIF-013 で p99 200ms と定めているが、外部エンドポイントの応答性能は tier1 の制御外にある。したがって SLO の計測境界は「facade-binding が外部エンドポイントへの最初のバイト送信を開始するまで」とし、外部応答待ち時間は SLO 対象外とする。ただし呼び出し元には外部応答を含めた end-to-end のレイテンシを返すため、計測境界と観測値は乖離する。この乖離は Grafana ダッシュボードで別メトリクス（`k1s0_binding_output_internal_latency_ms` / `k1s0_binding_output_external_latency_ms`）として分離表示し、SRE が判別できるようにする。

## エラー契約と監査

Binding API 固有のエラーコードは、共通契約 DS-SW-EIF-006 の K1s0Error 体系を継承しつつ、プロトコル固有のコードを追加する。

**設計項目 DS-SW-EIF-308 エラーコード体系**

Binding API 固有のエラーコードは以下 6 種に限定する。`BINDING_NOT_FOUND`（404、指定された binding 名が未登録）、`INPUT_ALREADY_SUBSCRIBED`（409、同一 binding の重複購読）、`SIGNATURE_INVALID`（401、HTTP Webhook の HMAC 署名検証失敗）、`TIMESTAMP_OUT_OF_WINDOW`（401、時刻窓外）、`BACKEND_UNAVAILABLE`（503、外部エンドポイントへのリトライ枯渇後の DLQ 転送失敗、または bind 先 Kafka/S3 の障害）、`QUOTA_EXCEEDED`（429、tier1 側レートリミット超過）。`VALIDATION_FAILED` は共通コードとして扱い、プロトコル毎の詳細（Cron 構文エラー、MQTT トピック不正、HTTP URL 不正）を `details[]` に埋める。

**設計項目 DS-SW-EIF-309 監査連携と PII マスキング**

全 Binding 呼び出しは Audit-Pii API に自動連携する。ただし Output ペイロードは大容量（S3 Output では MB オーダー）になり得るため、ペイロード本体は記録せず、SHA-256 ハッシュと先頭 256 バイト（PII マスキング後）を保存する。HTTP Webhook Input は受領ヘッダ全体（Authorization / Cookie は自動マスキング）を記録し、改ざん監査に備える。この方針は [10_Audit_Pii_API方式.md](10_Audit_Pii_API方式.md) の PII マスキング規約と整合する。

## 採用段階別の提供範囲

**設計項目 DS-SW-EIF-310 段階別機能解放**

採用初期は Output の MQTT / HTTP の 2 種のみ提供する。Input は リリース時点 では未提供で、アプリ側から外へ出る一方向に限定する。この割り切りは、Input 側の at-least-once 保証と DLQ 運用の検証期間を確保するためである。採用初期は Input の MQTT / HTTP Webhook / Cron の 3 種を解放し、Output に Kafka / S3 互換を追加、`Subscribe` メソッドを有効化する。採用初期は Output に SMTP、Input に Kafka を追加し 9 種全対応を完了する。採用後の運用拡大時に業界需要に応じて RabbitMQ / Azure Service Bus / AWS SNS 互換の追加を ADR 起票で検討する。

## 対応要件一覧

本ファイルは tier1 Binding API の外部インタフェース設計であり、要件 ID → 設計 ID の 1:1 対応を以下の表で固定する。表形式併記は DR-COV-001 への緩和策として、CI スクリプトでの機械検証の一次入力となる。

| 要件 ID | 要件タイトル | 対応設計 ID | カバー状況 |
|---|---|---|---|
| FR-T1-BINDING-001 | Output Binding 抽象化（MinIO / Kafka / MQTT / HTTP） | DS-SW-EIF-300, DS-CTRL-MSG-001 | 完全 |
| FR-T1-BINDING-002 | SMTP Output Binding | DS-SW-EIF-301 | 完全 |
| FR-T1-BINDING-003 | HTTP Output Binding | DS-SW-EIF-302 | 完全 |
| FR-T1-BINDING-004 | Input Binding 定期実行（Cron） | DS-SW-EIF-303 | 完全 |
| FR-EXT-IOT-001 | MQTT エッジ連携（外部 IoT） | DS-SW-EIF-304 | 完全 |
| FR-EXT-WEBHOOK-001 | 外部 SaaS Webhook 受信 | DS-SW-EIF-305 | 完全 |
| NFR-B-PERF-001 | tier1 API p99 500ms 内で Binding Output を実装（固有目標は リリース時点 実測後に追加検討） | DS-SW-EIF-306, DS-NFR-PERF-001 | 部分 |
| NFR-C-OPS-003 | DLQ 運用と可視化 | DS-SW-EIF-307, DS-NFR-OPS-003 | 完全 |

表に載せた要件数は FR-T1-BINDING-*4 件 + FR-EXT-* 2 件 + NFR 2 件 = 計 8 件。NFR-B-PERF-001 は `部分` 状態であり、リリース時点 実測値をもとに Binding 固有の性能要件を追加する計画である（Product Council 提示時に差分 PR 予定）。

補助参照は以下のとおり。

- ADR 参照: ADR-TIER1-001（Go+Rust 分担、facade-binding は Go）/ ADR-TIER1-002（Protobuf gRPC 必須）
- 連携設計: [04_エッジIoTインタフェース方式.md](../04_エッジIoTインタフェース方式.md) DS-SW-EIF-115〜117（MQTT エッジ契約）
- 本ファイルで採番: DS-SW-EIF-300 〜 DS-SW-EIF-310
