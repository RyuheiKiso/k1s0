# 04. エッジ IoT インタフェース方式

本ファイルは tier1 と工場 PLC / ビル設備 / POS 端末などのエッジ IoT デバイスを接続する外部インタフェースを固定化する。採用側組織の製造・流通領域では既に数千台規模のエッジ機器が稼働しており、これらを k1s0 の価値抽象（State / PubSub）に統合する方式を確立することが本ファイルの目的である。

## 本ファイルの位置付け

採用側組織の情シス基盤にとって IoT / OT（Operational Technology）の統合は避けて通れない課題である。工場 PLC は Modbus、ビル設備は BACnet、POS 端末は独自プロトコルで通信し、これらを k1s0 のクラウドネイティブ API に素朴に直結させると、バージョニング・認証・セキュリティの制約が両立しない。本ファイルはエッジ Gateway を中間層として配置し、tier1 PubSub / State / Binding API の抽象に変換することで構造的に解決する。

エッジ IoT の特殊性は「低帯域・断続的接続・長寿命機器・物理的アクセス可能性」の 4 点にある。クラウド基盤の前提（常時接続・高帯域・機器の短寿命・論理的隔離）がそのまま通用しないため、インタフェース設計でこの差分を吸収する必要がある。本ファイルはこの吸収方針を MUST 要件として書き下す。

## エッジ統合のアーキテクチャ

エッジ IoT から tier1 への経路は 4 段で構成する。現場機器 → エッジ Gateway → MQTT Broker → Dapr Binding → tier1 PubSub API。各段で担う責務を分離することで、機器側の多様性を抽象化しつつ tier1 側の一貫性を守る。

**設計項目 DS-SW-EIF-110 4 段構成による責務分離**

4 段の責務は以下のとおり。

1. **現場機器（PLC / ビル設備 / POS / センサ）**: 物理値の取得、Modbus / OPC UA / BACnet / CoAP などの現場プロトコルで通信。k1s0 側の仕様変更の影響を受けない立場。
2. **エッジ Gateway（Eclipse Kura または EdgeX Foundry）**: 現場プロトコルを MQTT 5 / HTTP に正規化、ローカルバッファリング、証明書管理、メタデータ付与。運用蓄積後は Helm チャート化された標準構成を配布。
3. **MQTT Broker（Eclipse Mosquitto または EMQ X）**: MQTT メッセージの配送、QoS 制御、認証、トピックベースルーティング。k1s0 クラスタ内部に配置。
4. **Dapr Binding（Input）**: MQTT Broker からメッセージを受信し、PubSub API トピックへ再 Publish。テナント識別とスキーマ変換を実施。

この 4 段は リリース時点 で完成、採用後の運用拡大時 で OPC UA UA Server ・BACnet Gateway の個別サポートを追加する。リリース時点 は MQTT のみの小規模 PoC で済ませる。

**設計項目 DS-SW-EIF-111 プロトコル選定と採用根拠**

エッジ Gateway が外向きに喋るプロトコルは以下 4 種とし、これ以外は tier1 側で受け付けない。

| プロトコル | 用途 | 採用根拠 |
|-----------|------|---------|
| MQTT 5 | IoT センサ、リアルタイム発報 | pub-sub モデルが PubSub API と自然に接続、低帯域で動作 |
| OPC UA | 工場 PLC、SCADA | 製造業デファクト、セキュリティ強（証明書ベース） |
| Modbus TCP | レガシー PLC、BACnet 設備 | 既存資産互換、エッジで OPC UA に変換後 Broker へ |
| CoAP | 電池駆動センサ（WAN 経由） | DTLS で軽量、RESTful、UDP ベースで低消費電力 |

HTTP ポーリングは「エッジ機器がサーバに定期問い合わせ」するパターンで許容するが、低帯域環境ではコスト高となるため推奨しない。gRPC のエッジ直結は接続維持コストと証明書ローテの難しさから禁止する。

## エッジ Gateway の設計

エッジ Gateway は現場プロトコルを正規化する重要な役割を担う。エッジ側に配置する OSS は Eclipse Kura または EdgeX Foundry から採用する。両 OSS は「現場プロトコルアダプタ」の豊富さで知られ、採用側組織の既存機器ラインナップをカバーできる。

**設計項目 DS-SW-EIF-112 エッジ OSS 採用方針**

採用候補は 2 つ。どちらを標準とするかは リリース時点 着手時に PoC で決定する。

- **Eclipse Kura**: OSGi ベース、Java Runtime、モジュール型アダプタ、EPL 2.0。軽量なエッジ機器向け（ARM、2GB RAM から稼働）。
- **EdgeX Foundry**: Linux Foundation ホスト、Go/Python ベース、マイクロサービス型、Apache 2.0。中規模エッジ（4GB RAM 以上）向け、Kubernetes Edge 対応。

ライセンスは両者とも AGPL-3.0 非該当で、設計原則 8 の隔離要件に抵触しない。標準採用決定後は Helm チャート化して配布し、現場ごとの構築コストを削減する。

**設計項目 DS-SW-EIF-113 エッジでのローカルバッファリング**

エッジ Gateway はローカルストレージで最大 24 時間分のメッセージをバッファする。ネットワーク切断時はバッファへ蓄積、復旧時に順次送信する。バッファサイズの根拠は「日本国内の典型的なデータセンタ〜工場間回線で 24 時間以上の連続断は運用リスクとして明示的に検知すべき水準」から導出する。24 時間を超える切断は現場作業員への通知と代替手段（USB 持込み、衛星回線）を発動する。

バッファは SQLite または RocksDB を選択し、電源断でも破損しない WAL 構成とする。リリース時点 標準構成では SQLite、トランザクション量が多い工場 PLC 向けには RocksDB をオプションで採用する。

**設計項目 DS-SW-EIF-114 タイムスタンプとセンサ値のフォーマット**

エッジで付与するタイムスタンプは RFC 3339 形式（`2026-04-12T10:30:45.123Z`）、UTC 必須とする。現場機器のローカルタイムを直接送ることは禁止する。エッジ Gateway は NTP 同期を必須とし、時刻ドリフトが 1 秒を超えた場合は警告メトリクスを発報する。

センサ値は SI 単位（m、kg、s、K、A、mol、cd）および組立単位（Pa、W、V など）で送信する。機器メーカ独自単位（インチ、華氏温度など）はエッジで SI 単位に変換して送信する。Protobuf メッセージでは `unit` フィールドに単位記号を明示する。

## MQTT Broker の設計

MQTT Broker は k1s0 クラスタ内部に配置し、エッジ Gateway との接続エンドポイントを公開する。外向きは mTLS で認証し、内部は Dapr Binding へ接続する。

**設計項目 DS-SW-EIF-115 MQTT Broker 選定**

リリース時点 時点では Eclipse Mosquitto（EPL 2.0）を標準採用する。軽量、運用実績豊富、Kubernetes 対応の Helm チャートが公式提供されている点が採用根拠。スケール要件で不足する場合（10,000 接続以上）は EMQ X Community（Apache 2.0）への切替を 採用後の運用拡大時 で検討する。EMQ X の Enterprise 版はプロプライエタリ機能があるため採用不可。

**設計項目 DS-SW-EIF-116 トピック命名と権限分離**

トピック命名は `edge/<tenant_id>/<site_id>/<device_id>/<sensor_type>` 階層とする。例: `edge/T001/factory-A/plc-001/temperature`。Broker 側で ACL を設定し、エッジ機器は自機器プレフィクスへの publish のみ許可、subscribe は禁止。

tier1 Dapr Binding は `edge/+/+/+/+` にサブスクライブし、受信メッセージを `k1s0.pubsub.v1.topic=edge-events` へ再 Publish する。ここで再 Publish 時に Dapr 側で tenant_id を検証し、Broker 側 ACL を突破するスプーフィングを二重防御する。

**設計項目 DS-SW-EIF-117 MQTT QoS 方針**

MQTT QoS レベルは用途に応じて使い分ける。

| QoS | 配送保証 | 用途 | 採用方針 |
|-----|---------|------|---------|
| 0 | at-most-once | 高頻度センサ値（温度、湿度） | 1 秒 1 件、欠損許容 |
| 1 | at-least-once | 状態遷移イベント（ドア開閉、エラー発生） | 重複はエッジ側冪等キーで解決 |
| 2 | exactly-once | 会計トランザクション、POS 売上 | Broker 負荷大、限定利用 |

QoS 1 を標準とし、重複排除は tier1 PubSub API の冪等キー機構に委譲する。QoS 2 は Broker・クライアント双方で 4-way handshake が必要で性能影響が大きいため、本当に重複を許容できないトランザクションのみ用いる。

## 認証とセキュリティ

エッジ機器は物理的にアクセス可能な環境に置かれることが多く、証明書盗難・改ざんリスクが通常のサーバサイドより高い。短命証明書と検知機構の 2 段で対処する。

**設計項目 DS-SW-EIF-118 エッジ機器の証明書認証**

エッジ機器ごとに SPIFFE SVID を発行する。trust domain は `edge.k1s0.internal.example.jp`（tier1 内部 `k1s0.internal.example.jp` と分離）。SVID は `spiffe://edge.k1s0.internal.example.jp/tenant/T001/site/factory-A/device/plc-001` 形式とし、テナント・サイト・機器を SVID に埋め込む。

TTL は 6 時間、自動ローテーション間隔 3 時間とする。tier1 内部サービス（TTL 1 時間）より長いのは、ネットワーク瞬断時の SVID 更新失敗で機器が孤立する事象を避けるためである。ただしセキュリティ事故発生時は即時失効できるよう、CRL / OCSP Responder を運用する。

**設計項目 DS-SW-EIF-119 境界防御と WAF**

外向き MQTT エンドポイントは Istio Ingress Gateway で受信する。TLS 1.3 必須、クライアント証明書必須の mTLS モード。WAF（Coraza）で MQTT CONNECT パケットの異常パターンを検出し、ブルートフォース攻撃や不正 clientId を遮断する。

MQTT 以外のポート（SSH、Telnet など）はエッジ機器側でも閉塞する。エッジ機器の設定変更は OTA（Over-The-Air）方式で配信し、物理アクセスによる設定変更を禁止する。リリース時点 時点で全機器に設定を強制適用する。

## スキーマとデータ変換

エッジから届くメッセージは現場機器固有のスキーマを持つが、tier1 側は標準 Protobuf スキーマで受け取る必要がある。エッジ Gateway でスキーマ変換を完結させる。

**設計項目 DS-SW-EIF-120 エッジでの Protobuf 化**

エッジ Gateway は現場プロトコル（Modbus レジスタ値、OPC UA NodeId など）を Protobuf メッセージ `k1s0.edge.v1.EdgeEvent` に変換して MQTT に publish する。`EdgeEvent` の必須フィールドは `tenant_id` / `site_id` / `device_id` / `event_type` / `occurred_at` / `payload`（device 固有 Any 型）。

device 固有 payload はさらに `k1s0.edge.v1.SensorReading` / `k1s0.edge.v1.StateTransition` / `k1s0.edge.v1.Alarm` のいずれかで wrap する。これら型定義は `src/tier1/contracts/edge/v1/` で集中管理し、バージョニング方針（[02_APIバージョニング方式.md](02_APIバージョニング方式.md)）に従って進化させる。

**設計項目 DS-SW-EIF-121 at-least-once 配送とエッジ側冪等キー**

エッジからの配送保証は at-least-once とする。重複が起こる代わりに欠損しないことを優先する。エッジ Gateway は各メッセージに冪等キー `edge_event_id`（UUIDv7）を付与し、tier1 側で重複を検出できるようにする。

tier1 PubSub API は `edge-events` トピックの受信側で Valkey に `edge_event_id` を TTL 7 日で記録し、重複メッセージは廃棄する。この Valkey は State Store と同居させるが、key prefix `idem:edge:` で分離する。

## オフライン対応と信頼性

切断は日常的に発生する前提で設計する。切断中のデータ喪失ゼロと、復旧後の自動再送を保証する。

**設計項目 DS-SW-EIF-122 切断検知と復旧シーケンス**

エッジ Gateway は MQTT Broker との接続を 30 秒間隔で keepalive する。3 連続 keepalive 失敗（90 秒無応答）で切断とみなし、以下のシーケンスに移行する。

1. ローカルバッファへの書き込みモードに切替。
2. 30 秒ごとに再接続試行（backoff なし、現場作業員が物理的に復旧できる前提）。
3. 接続復旧後、バッファのメッセージを古い順に送信。送信完了後、通常モードへ戻る。

この挙動はエッジ Gateway の構成ファイルで調整可能とするが、既定値を標準テンプレートで配布する。現場ごとの細かい調整を避ける方針とする。

**設計項目 DS-SW-EIF-123 エッジメトリクスの tier1 送信**

エッジ Gateway 自身のメトリクス（CPU 使用率、バッファ残量、送信レート、接続断回数）は Prometheus Remote Write で tier1 の Prometheus に送る。リリース時点 時点では一部機種のみ対応、採用後の運用拡大時 で全機種標準化する。メトリクスを通じて現場の健全性を SRE が遠隔把握でき、採用側の小規模運用原則と整合する。

## 対応要件一覧

本ファイルはエッジ IoT 統合の外部インタフェース設計であり、以下の要件 ID に対応する。

- FR-T1-014（PubSub at-least-once）
- FR-T1-015（PubSub DLQ）
- FR-T1-019（Binding Input MQTT）
- FR-T1-020（Binding Input HTTP/Cron/Kafka）
- FR-EXT-MON-001（外部センサメトリクス取り込み）
- NFR-E-SIR-012（エッジ境界防御）
- NFR-B-PERF-008（バースト時のオフライン耐性）
- ADR 参照: ADR-TIER1-001（言語分担）/ ADR-SEC-002（SPIFFE）
- 本ファイルで採番: DS-SW-EIF-110 〜 DS-SW-EIF-123
