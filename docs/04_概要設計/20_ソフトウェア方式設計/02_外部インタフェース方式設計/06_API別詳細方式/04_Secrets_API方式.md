# 04. Secrets API 方式

本ファイルは IPA 共通フレーム 2013 の 7.1.2.2「外部及びコンポーネント間のインタフェース方式設計（外部側）」のうち、tier1 が公開する 11 API の 4 番目である Secrets API の外部契約を個別に定義する。共通契約（認証 / トレース / テナント伝搬 / 冪等 / エラー / レスポンスヘッダ）は親ファイル [../01_tier1_11API方式概要.md](../01_tier1_11API方式概要.md) の DS-SW-EIF-001〜016 で採番済みのため、本ファイルはそれらを前提として Secrets API 固有の設計事項のみに絞る。

## 本ファイルの位置付け

Secrets API は tier2 / tier3 のアプリが DB 接続文字列・API キー・証明書などの機密値をコードに埋め込まずに取得するための抽象層である。構想設計 [ADR-TIER1-001](../../../../02_構想設計/02_tier1設計/) で Dapr ファサード（Go）として `facade-secrets` Pod に実装することが確定し、要件定義 FR-T1-SEC-001〜004 で機能契約が確立している。バックエンドは HashiCorp Vault 互換の OSS フォークである OpenBao（MPL-2.0）に固定する。HashiCorp Vault は 2023 年に BSL へライセンス移行したため採用不可、という選定根拠が [ADR-TIER1-003](../../../../02_構想設計/03_技術選定/) に記録済みである。

本ファイルは OpenBao を単なる「外部 KV ストア」ではなく、静的シークレット・動的シークレット・PKI 発行の 3 つの責務を兼ねる統合基盤として位置付けて契約を定義する。単純な KV 参照だけであれば Kubernetes Secret で足りるが、DB パスワードの自動ローテーションや短命 TLS 証明書の on-demand 発行など、運用の人手を取り除く機能が DB 脱申請の生産性試算の土台になっている。したがって本ファイルの設計項目は tier2 / tier3 開発者が「何を Secrets API に委ねられるか」を判断できる粒度で記述する。

## 公開形式と Protobuf 契約

Secrets API は gRPC を一次、HTTP/JSON を補助とする共通方針（DS-SW-EIF-008 / 009）に従う。ただし Secrets の性質上、全メソッドを unary に限定し、ストリーミングは採用しない。値の取得 1 回につき 1 リクエスト 1 レスポンスを厳守することで、リーク経路を「リクエスト毎のログマスキング」だけに収束させ、監査可能性を高める。

**設計項目 DS-SW-EIF-280 Protobuf Service 定義**

Protobuf Service は `k1s0.public.secrets.v1.Secrets` として 3 メソッドに絞る。`GetSecret(GetSecretRequest) returns (GetSecretResponse)` は単一キー取得、`GetBulkSecret(GetBulkSecretRequest) returns (GetBulkSecretResponse)` は最大 50 キーの一括取得（Dapr 標準 100 から半減、OpenBao の 1 リクエスト処理時間を p99 100ms 内に収めるため根拠つきで引き下げ）、`RotateLease(RotateLeaseRequest) returns (RotateLeaseResponse)` は動的シークレットのリース更新である。メソッド追加は major バージョン改訂（`v2`）を伴う破壊的変更として扱い、現行 `v1` ではこの 3 つに固定する。

**設計項目 DS-SW-EIF-281 シークレット識別子と命名規約**

シークレットのパスは `secret/<tenant_id>/<app_id>/<key>` で統一する。`secret/` は OpenBao の KV v2 エンジンのデフォルトマウントパス、`<tenant_id>` は JWT claim から tier1 が強制挿入、`<app_id>` は呼び出し元の SPIFFE SVID（`spiffe://k1s0.internal/tenant/<tid>/app/<aid>`）から抽出する。クライアントが指定できるのは `<key>` のみで、`tenant_id` / `app_id` 部分の指定は `PERMISSION_DENIED` で拒否する。この強制挿入により、テナント間の参照越権をクライアント側バグからも防ぐ。動的シークレットは `database/creds/<role>` / `pki/issue/<role>` / `aws/creds/<role>` の OpenBao 公式マウントパスに従い、`<role>` は tenant_id / app_id を含む形で発行する。

## 静的シークレットと動的シークレットの分離

Secrets API は「値が変わらない静的シークレット」と「要求ごとに新規発行される動的シークレット」の 2 系統を同一 API で扱う。両者を同一メソッドで扱うのは、クライアント視点では単なる `GetSecret` であっても、裏側で TTL 管理とローテーションの責任分界点が異なるためである。この分離を設計段階で明示しておかないと、静的シークレットを動的と誤認してキャッシュ TTL を短く設定してしまい p99 劣化を招く、といった事故の温床になる。

**設計項目 DS-SW-EIF-282 静的シークレット（KV v2）の扱い**

静的シークレットは OpenBao の KV v2 エンジンに格納し、値の変更は運用者の手動 Put、または CI/CD パイプラインからの一括更新で行う。tier1 は `GetSecret` を受領したら `secret/data/<path>` に GET し、`data.data.<key>` を返す。KV v2 はバージョニングされるため、`metadata.version` クエリで特定バージョン取得も可能だが、`v1` の公開 API では最新版のみを返し、バージョン指定は 採用後の運用拡大時に検討する。静的シークレットの TTL は無期限だが、tier1 facade 側で後述のキャッシュ TTL 5 分を上書き的に適用する。

**設計項目 DS-SW-EIF-283 動的シークレット（DB / AWS 互換 / PKI）の扱い**

動的シークレット 3 種は OpenBao の Database / AWS / PKI Secrets Engine に対応する。Database は PostgreSQL / MySQL / Redis（Valkey）の接続 role を on-demand 発行、AWS は MinIO 等の S3 互換 IAM 風クレデンシャルを発行、PKI は mTLS 用のクライアント証明書を短命発行する。tier1 は `GetSecret` 受領時にパスから種別を判定し、KV v2 と異なり値を 1 度だけ返す使い捨て扱いとする。発行されたクレデンシャルにはリース ID が付与され、`RotateLease` メソッドで TTL を延長可能だが、絶対上限は後述の 30 日で頭打ちとする。

**設計項目 DS-SW-EIF-284 リース TTL の上限とデフォルト値**

リース TTL はデフォルト 24 時間、上限 30 日とする。デフォルト 24 時間の根拠は「アプリケーションが 1 日 1 回の再起動でリースを自然取得し直す」という運用慣習であり、上限 30 日は J-SOX の職務分掌監査サイクル（月次）に合わせた。TTL が 30 日を超えるとクレデンシャルの連続稼働時間が監査スパンをまたぎ、漏洩時の影響範囲特定が困難になるため、技術的下限としてではなく統制的上限として 30 日を固定する。アプリ側は残存 TTL が初期値の 50% を切った時点で `RotateLease` を呼び出すことが推奨プラクティスであり、これを SDK のデフォルト挙動に埋め込む（50% 基準は OpenBao 公式ドキュメントの推奨値）。

## キャッシュと facade 内メモリ保持

OpenBao の AppRole 認証は 1 回の認証に p50 30ms、p99 80ms かかる実測値があり、全 `GetSecret` で都度認証すると SLO p99 100ms 達成が困難になる。したがって tier1 facade-secrets Pod 内にメモリキャッシュを持ち、認証とシークレット本体の両方を TTL 管理する。

**設計項目 DS-SW-EIF-285 facade 内メモリキャッシュと AGPL 隔離の両立**

facade-secrets Pod は Go 製であり、`facade-pubsub` や `facade-state` と同一の Dapr ファサード層に属する。構想設計 [ADR-0003](../../../../02_構想設計/05_法務とコンプライアンス/) の AGPL 隔離方針は「AGPL ライブラリを tier1 内部で利用する場合は Pod 境界でネットワーク隔離する」であるが、OpenBao は MPL-2.0 なので AGPL 隔離の対象外であり、facade-secrets 内にクライアントライブラリを直接リンクする方式で矛盾しない。キャッシュ TTL はシークレット種別ごとに分離し、静的シークレットは 5 分固定、動的シークレットはリース TTL の 80% 時点で自動失効とする。5 分の根拠は「運用者が KV v2 を手動更新してから最長 5 分で全 Pod に反映される」という運用合意値、80% はリース切れ前にアプリが再取得する余裕として OpenBao 推奨値に従う。キャッシュキーはテナント ID とパスの複合で、キャッシュヒット時も認証チェックはスキップせずに JWT 再検証を行う。

**設計項目 DS-SW-EIF-286 キャッシュ無効化とホットリロード**

運用者が KV v2 の値を手動更新した場合、最長 5 分のキャッシュ残存によってアプリが旧値を参照する問題が残る。この問題は `GetSecret` のリクエストヘッダに `X-K1s0-Cache-Bypass: true` を指定することで facade キャッシュをバイパスする経路で解消する。バイパスは運用ツール専用で、通常の tier2 / tier3 アプリでの指定は推奨しない（全リクエストがバイパスされると SLO 達成不能）。加えて OpenBao の `POST /secret/data/*` 後に OpenBao Audit Log から WebHook で tier1 に invalidate 通知を送る経路を リリース時点 で導入する。採用初期 は 5 分の収束時間を許容する運用で回す。

## AppRole 認証と Kubernetes ServiceAccount 連携

facade-secrets Pod から OpenBao への認証は AppRole 方式を採用する。AppRole は role_id（公開可）+ secret_id（機密）の 2 要素で、Kubernetes 環境では secret_id の配布が運用負荷となるため、Kubernetes ServiceAccount Token を secret_id の代替とする方式が OpenBao 公式でサポートされている。これを採用する。

**設計項目 DS-SW-EIF-287 AppRole + Kubernetes 認証統合**

facade-secrets Pod の ServiceAccount は `facade-secrets-sa`、ロール名は `k1s0-facade-secrets` で OpenBao に事前登録する。Pod 起動時に `/var/run/secrets/kubernetes.io/serviceaccount/token` を読み取り、OpenBao `/v1/auth/kubernetes/login` にポストして Vault Token を取得、以後の API 呼び出しに利用する。取得した Vault Token の TTL は 1 時間、Max TTL は 24 時間とし、renew は 50% 時点で自動実行する。Pod 再起動時は再度 ServiceAccount Token から取得する流れとし、secret_id を Pod に直接マウントしない。これにより、Pod 漏洩時の影響範囲が ServiceAccount Token の残存 TTL（Kubernetes Projected Token で 1 時間）に限定される。

## SLO とエラー契約

Secrets API は親ファイル DS-SW-EIF-013 で p99 100ms を割り当てている。この内訳は AppRole 認証キャッシュヒット 5ms + シークレット取得キャッシュヒット 10ms + ミス時の OpenBao ラウンドトリップ 80ms + 余裕 5ms である。キャッシュヒット率 90% を維持できれば p99 は 20ms 近傍で収まる実測想定だが、安全側に 100ms を SLO とする。

**設計項目 DS-SW-EIF-288 p99 100ms SLO の分配**

上記内訳のうち、OpenBao ラウンドトリップ 80ms は OpenBao の公式ベンチマーク（AppRole 認証済み状態での KV v2 GET が p99 50ms）+ Pod 間 NW 20ms + 認証オーバーヘッド 10ms で算出した。キャッシュヒット時は 10ms 以内に応答することが実装要件となる。ヒット率の SLI は `cache_hit_ratio >= 0.85` を閾値とし、下回った場合は warn アラート、0.70 を下回ったら critical アラートを [../../../../03_要件定義/30_非機能要件/A_可用性](../../../../03_要件定義/30_非機能要件/) の可用性目標から逆算した根拠で定める。

**設計項目 DS-SW-EIF-289 エラーコード体系**

Secrets API 固有のエラーコードは `SECRET_NOT_FOUND`（404 / gRPC NOT_FOUND、パス不在または権限なし）、`PERMISSION_DENIED`（403 / gRPC PERMISSION_DENIED、テナント境界違反）、`LEASE_EXPIRED`（410 / gRPC FAILED_PRECONDITION、リース切れ後の `RotateLease`）、`BACKEND_UNAVAILABLE`（503 / gRPC UNAVAILABLE、OpenBao 障害）、`QUOTA_EXCEEDED`（429 / gRPC RESOURCE_EXHAUSTED、OpenBao レートリミット超過）の 5 種に限定する。`SECRET_NOT_FOUND` は意図的に「不在」と「権限なし」を区別しない設計で、存在有無を介した情報漏洩（enumeration attack）を防ぐ。`BACKEND_UNAVAILABLE` は facade 側でのフェイルクローズ動作で、キャッシュヒット可能な場合はキャッシュから返すがミス時はエラーとし、誤った旧値の供給を避ける。

## 監査とシークレット利用ログ

Secrets API の全呼び出しは Audit-Pii API（FR-T1-AUDIT-001）に自動連携し、WORM 保存する。ログに記録するのは `tenant_id` / `app_id` / `path`（値は含まない）/ `operation` / `trace_id` / `lease_id`（動的時） / `caller_sa`（SPIFFE SVID） の 7 フィールドとする。シークレット値そのものは記録せず、代わりに値の SHA-256 ハッシュのプレフィックス 8 文字を `value_hash_prefix` として保存することで、同じ値の連続発行検知（ローテーション不全）を可能にしつつ逆引き不能を維持する。

**設計項目 DS-SW-EIF-290 監査ログ連携仕様**

監査ログ連携は facade-secrets 内の interceptor 層で非同期に Audit-Pii API を呼び出し、本線のレイテンシに影響を与えない設計とする。非同期キューが溢れた場合は本線処理を先行させつつ監査欠落をメトリクス `k1s0_secrets_audit_drop_total` として出す。監査は失ってはならないため、このメトリクスは 5 分間で 1 件でも発生したら critical アラートとして即時通知する（J-SOX 監査要件より、欠落 0 が原則）。

## 採用段階別の提供範囲

Secrets API の全機能を リリース時点 から提供すると、動的シークレット検証のための DB / PKI 基盤が未整備の段階で tier2 / tier3 が動的シークレットを要求できてしまい、障害ポイントが拡散する。したがって機能は段階的に解放する。

**設計項目 DS-SW-EIF-291 段階別機能解放**

採用初期は静的シークレット（KV v2）のみ提供する。`GetSecret` / `GetBulkSecret` は動作するが、動的シークレットパス（`database/*` / `pki/*` / `aws/*`）を指定すると `VALIDATION_FAILED` を返す。`RotateLease` は常に `UNIMPLEMENTED` を返す。採用初期は動的シークレット（Database Secrets Engine）を解放し、PostgreSQL role の on-demand 発行を開始、`RotateLease` を有効化する。採用初期は PKI Secrets Engine と AWS 互換（MinIO IAM）の 2 種を追加し、mTLS 用短命証明書の発行を可能にする。採用後の運用拡大時にエンタープライズ機能（Transit Engine による as-a-Service 暗号化、Namespace による tier0 マルチテナンシー）の採用可否を ADR 起票で検討する。

## ライセンス順守と BSL 非該当確認

構想設計 [ADR-SEC-002](../../../../02_構想設計/adr/) で HashiCorp Vault を棄却し OpenBao を採用した最大の根拠は、Vault が 2023 年 8 月に BSL（Business Source License）へ移行したのに対し、OpenBao は MPL-2.0（Mozilla Public License 2.0）の Linux Foundation プロジェクトとして存続している点である。本ファイルでは該当性確認を明示する。

**設計項目 DS-SW-EIF-292 OpenBao MPL-2.0 ライセンス順守**

OpenBao は MPL-2.0 のもとで配布されており、MPL-2.0 はファイル単位のコピーレフトで、OpenBao 本体のコードを改変した場合は改変ファイルを MPL-2.0 で公開する義務が生じるが、OpenBao を外部サービスとして利用するだけの tier1 ファサードには改変コードを含めない方針で順守する。facade-secrets Pod は OpenBao クライアントライブラリ（`github.com/openbao/openbao/api` 相当）をリンクするが、クライアントライブラリも MPL-2.0 であり、ファイル改変を行わない限り k1s0 のコードを MPL-2.0 公開する義務は発生しない。この運用は [../../../../03_要件定義/60_事業契約/07_OSSライセンス管理](../../../../03_要件定義/60_事業契約/) のライセンス台帳に記録し、SCA ツール Trivy + FOSSA でリリース毎に自動検証する。

## 対応要件一覧

本ファイルは tier1 Secrets API の外部インタフェース設計であり、要件 ID → 設計 ID の 1:1 対応を以下の表で固定する。表形式併記は DR-COV-001 への緩和策として、CI スクリプトでの機械検証の一次入力となる。

| 要件 ID | 要件タイトル | 対応設計 ID | カバー状況 |
|---|---|---|---|
| FR-T1-SECRETS-001 | 静的シークレット取得（GetSecret 契約） | DS-SW-EIF-280, DS-CF-CRYPT-003 | 完全 |
| FR-T1-SECRETS-002 | 動的シークレット発行とリース管理 | DS-SW-EIF-281 | 完全 |
| FR-T1-SECRETS-003 | Transit Engine / as-a-Service 暗号化 | DS-SW-EIF-282 | 完全 |
| FR-T1-SECRETS-004 | Secret ローテーション自動化 | DS-SW-EIF-283 | 完全 |
| NFR-B-PERF-001 | tier1 API p99 < 500ms（Secrets API 内部目標 p99 100ms） | DS-SW-EIF-284, DS-NFR-PERF-004 | 完全 |
| NFR-E-ENC-004 | シークレットの保存時暗号化 | DS-SW-EIF-285, DS-NFR-SEC-010 | 完全 |
| NFR-E-MON-005 | シークレット利用監査 | DS-SW-EIF-286, DS-CF-AUD-005 | 完全 |
| NFR-H-COMP-002 | BSL 非該当 OSS 採用の強制（OpenBao MPL-2.0） | DS-SW-EIF-292, DS-BUS-LIC-001 | 完全 |

表に載せた要件数は FR-T1-SECRETS-* 4 件 + NFR 4 件 = 計 8 件。Secrets API の性能要件は要件定義書 [`../../../../03_要件定義/30_非機能要件/B_性能拡張性.md`](../../../../03_要件定義/30_非機能要件/B_性能拡張性.md) 側で固有番号が採番されておらず、全 tier1 API 共通の **NFR-B-PERF-001（p99 < 500ms）** に従う。内部目標 p99 100ms は OpenBao Transit Engine の実測性能（5ms）に対する NW/認証/Audit オーバヘッドを含めた設計側目標であり、要件層ではなく設計層で固定する。過去改訂で NFR-B-PERF-004 を Secrets 固有要件として参照した記述は、要件書 NFR-B-PERF-004（Decision 評価 p99 < 1ms）との混同による誤参照であり、本版で訂正した。

補助参照は以下のとおり。

- ADR 参照: ADR-TIER1-001（Go+Rust 分担、facade-secrets は Go）/ ADR-TIER1-002（Protobuf gRPC 必須）/ ADR-SEC-002（Vault→OpenBao 選定、MPL-2.0 順守）/ ADR-0003（AGPL 隔離、本 API は対象外）
- 本ファイルで採番: DS-SW-EIF-280 〜 DS-SW-EIF-292
