# Secrets API

本書は、tier1 が公開する Secrets API の機能要件を定義する。tier2/tier3 の秘密情報（API キー、DB 認証情報、暗号鍵）を、OpenBao（HashiCorp Vault の MPL 2.0 Linux Foundation フォーク）バックエンドで管理する。

## API 概要

tier2 / tier3 が必要とする秘密情報を、ハードコード・環境変数直指定ではなく API 経由で取得する設計を強制する。静的 Secret（外部 SaaS の API キー等）と動的 Secret（PostgreSQL の一時パスワード TTL 1 時間）の両方を扱う。暗号化鍵は Transit 機能で一元管理する。

内部実装は Dapr Secrets Building Block と OpenBao の直接連携（動的 Secret）を併用する。Phase 1b で Secrets API を提供開始、Phase 1c で Secret ローテーション自動化を確立する。

## 機能要件

### FR-T1-SECRETS-001: 静的 Secret 取得

**現状**: tier2 は k8s Secret を環境変数にマウントするか、ConfigMap ベースで直接埋め込む。変更には Pod 再起動が必要で、共有 Secret の更新が全アプリに即時伝搬しない。

**要件達成後**: `k1s0.Secrets.Get(name)` で静的 Secret を取得する。tier1 ファサードが OpenBao の KV ストアから読み込み、取得結果は短時間（30 秒）のインメモリキャッシュで高速化する。Secret 更新は次回キャッシュ失効後に自動反映される。

**崩れた時**: Secret 更新で tier2 全 Pod の再起動が発生し、業務断が生じる。コードリポジトリへの Secret 混入事故が起きる。

**受け入れ基準**:
- `Get` は Secret 名から値（バイト列）を返す
- キャッシュ TTL（デフォルト 30 秒）は Component YAML で調整可能
- Secret 未存在時は `K1s0Error.NotFound` を返す
- tenant_id による Secret 名前空間分離（他テナントの Secret に到達不可）

### FR-T1-SECRETS-002: 動的 Secret 発行（PostgreSQL 等）

**現状**: PostgreSQL 接続情報を長期不変のパスワードで管理すると、漏えい時の影響範囲が大きい。定期ローテーションは運用負荷が高く、実質回らない。

**要件達成後**: `k1s0.Secrets.GetDynamic("postgres", "role_name")` で TTL 1 時間の動的 Secret を発行する。OpenBao の Database Engine が PostgreSQL ユーザを都度生成し、TTL 経過で自動失効。tier2 は取得直後にその認証情報で接続し、長期保持しない。

**崩れた時**: 長期パスワードの使い回しで漏えい検出が遅れ、影響範囲の特定に時間がかかる。ローテーションが人手に依存し、1 年以上放置される Secret が発生する。

**受け入れ基準**:
- TTL はロール定義で指定、デフォルト 1 時間、最大 24 時間
- TTL 経過時は OpenBao が自動的に PostgreSQL ユーザを DROP
- tier2 の接続エラー時に自動再取得する SDK ヘルパを提供
- Phase 1b で PostgreSQL のみ、Phase 2+ で Kafka ACL / MinIO STS 等追加

### FR-T1-SECRETS-003: Transit 暗号化（API 経由）

**現状**: 業務データの一部フィールド（個人情報、機密文書）を暗号化したい場合、tier2 が AES 実装・鍵管理を自前で行う必要がある。鍵のローテーションやバージョン管理が煩雑。

**要件達成後**: `k1s0.Secrets.Encrypt(key_name, plaintext)`、`Decrypt(key_name, ciphertext)` で OpenBao Transit を呼び出す。鍵そのものは tier2 に露出せず、OpenBao 内で版管理される。鍵ローテーションは Transit の rekey 機能で実施し、既存暗号文は旧版鍵でも復号可能。

**崩れた時**: tier2 アプリ間で鍵実装がバラつき、暗号化 / 復号の相互運用性が失われる。鍵漏えい時の影響範囲が不明となる。

**受け入れ基準**:
- 暗号化アルゴリズムは AES-256-GCM 固定
- 鍵名は `<tenant_id>.<key_label>` で tier1 が自動プレフィックス
- 鍵バージョン管理が自動で、復号時は暗号文中のバージョン番号から適切な鍵を選択
- 優先度 COULD（tier2 ユースケース顕在化後に判定）

### FR-T1-SECRETS-004: Secret ローテーション自動化

**現状**: 静的 Secret（外部 SaaS API キー等）のローテーションは人手で実施されることが多く、ローテ忘れや実施ミスが監査指摘の常連。

**要件達成後**: Phase 1c で OpenBao のローテーションスケジューラを活用し、一定期間（例: 90 日）で Secret の再発行を自動化する。対象 tier2 アプリは次回 Get でキャッシュ更新により新 Secret を取得する。古い Secret は一定猶予期間後に失効する。

**崩れた時**: Secret 長期使い回しが継続し、監査部門から指摘される。漏えい検出時の影響範囲が広がる。

**受け入れ基準**:
- ローテーション周期は Component YAML で 30〜365 日の範囲で設定
- ローテーション時刻は業務時間外で自動スケジュール
- 猶予期間中は旧 Secret・新 Secret 両方で取得可能（無停止切替）
- Phase 1c で提供、Phase 1b では手動ローテーション Runbook のみ

## 入出力仕様

```
k1s0.Secrets.Get(name: string) -> (value: bytes, error: K1s0Error?)

k1s0.Secrets.GetDynamic(
    engine: string,   // "postgres" | "kafka" | "minio" 等
    role: string
) -> (credentials: map<string, string>, lease_id: string, ttl_seconds: int, error: K1s0Error?)

k1s0.Secrets.Encrypt(key_name: string, plaintext: bytes) -> (ciphertext: bytes, error: K1s0Error?)
k1s0.Secrets.Decrypt(key_name: string, ciphertext: bytes) -> (plaintext: bytes, error: K1s0Error?)

k1s0.Secrets.RenewLease(lease_id: string) -> error
```

エラー型には `NotFound`、`LeaseExpired`、`RotationInProgress` を追加。

## 受け入れ基準（全要件共通）

- Secret 値はログ出力されない（tier1 ファサードで自動マスキング）
- OpenBao 障害時は Secret 取得が失敗するが、tier2 のキャッシュ有効期間内は継続稼働（degrade）
- Secret の取得操作は Audit API に自動記録される（NFR-E-MON-002）

## Phase 対応

- **Phase 1a**: 未提供
- **Phase 1b**: FR-T1-SECRETS-001、002（静的・動的）
- **Phase 1c**: FR-T1-SECRETS-004（ローテーション自動化）
- **Phase 2+**: FR-T1-SECRETS-003（Transit）、その他動的 Secret（Kafka / MinIO）

## 関連非機能要件

- **NFR-E-ENC-001**: Secret の暗号化保管（OpenBao の seal 機構）
- **NFR-E-AC-004**: Secret 取得の最小権限原則
- **NFR-E-MON-002**: Secret 取得操作の Audit 記録
- **NFR-A-CONT-002**: OpenBao 障害時の tier2 稼働継続（キャッシュ有効期間内）
