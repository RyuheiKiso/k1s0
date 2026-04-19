# State API

本書は、tier1 が公開する State API の機能要件を定義する。tier2/tier3 のキー・バリュー状態管理とキャッシュを、Valkey（Redis 互換 Linux Foundation フォーク）バックエンドで提供する。

## API 概要

セッション、一時キャッシュ、トークンバケット（レート制限）、軽量な進行状態保持を対象とする。永続業務データ（PostgreSQL 相当）や大容量データ（MinIO 相当）は本 API の対象外。

内部実装は Dapr State Building Block の Go SDK を利用する。バックエンドは Valkey で、Phase 2 以降で切り替え余地を残す（Component YAML を書き換えるだけで影響を tier1 内に閉じる）。

## 機能要件

### FR-T1-STATE-001: 基本 Get/Set/Delete

**現状**: tier2 が Redis を直接使うと、接続文字列・認証・プール設定を各アプリで個別管理することになる。Dapr State を使えば抽象化されるが、Dapr のキー命名規約（アプリ名プレフィックス）と tenant 分離の強制が個別実装となる。

**要件達成後**: `k1s0.State.Get(key)`、`Set(key, value, options)`、`Delete(key)` で基本操作を提供する。キーの内部プレフィックスには `tenant_id` が自動付与され、テナント越境が構造的に不可能になる。値のシリアライズは JSON（デフォルト）と Protobuf（オプション）を選択可能。

**崩れた時**: tier2 開発者が tenant_id を手でキーに含める必要が生じ、書き忘れが起きる。複数 tier2 アプリがキー名空間を衝突させ、意図しない読み書きが発生する。

**受け入れ基準**:
- `Get` は未存在キーに対し `None` / `null` を返す（例外にしない）
- `Set` は TTL なしの永続書き込み、および TTL 秒数指定での自動期限切れを両方サポート
- `Delete` は未存在キーに対してエラーにしない（冪等）
- キーに自動付与される tenant_id プレフィックスは tier2 から見えない（返り値にも含まれない）

### FR-T1-STATE-002: TTL 制御

**現状**: Redis の TTL 設定は個別 API（`EXPIRE`）で行うが、Dapr State では metadata で渡す。キー設計と TTL 設定の分離で、TTL 漏れが起きやすい。

**要件達成後**: `Set` のオプション引数 `ttl_seconds` で TTL を指定する。0 または未指定の場合は永続保持。TTL 延長・短縮は `Set` を再呼び出しすることで行う（Redis の `EXPIRE` 相当は提供しない、シンプル化のため）。

**崩れた時**: TTL 管理が tier2 でバラバラになり、Valkey のメモリ圧迫や意図しない早期失効が発生する。

**受け入れ基準**:
- TTL は 1 秒〜無制限の範囲で指定可能
- TTL 0 は「即時失効」ではなく「TTL なし（永続）」として扱う
- TTL 経過後のキーは `Get` で `None` を返す（Redis の lazy expiration の挙動を踏襲）

### FR-T1-STATE-003: Bulk 操作

**現状**: 複数キーを一括で取得・設定したい場合、tier2 がループ呼び出しするとネットワーク往復で p99 が悪化する。Redis の `MGET` / `MSET` 相当が欲しい。

**要件達成後**: `BulkGet(keys)`、`BulkSet(items)`、`BulkDelete(keys)` を提供する。最大 100 キー / 呼び出しを上限とする（大きい場合は tier2 側で分割）。

**崩れた時**: tier2 がループ呼び出しで実装し、N 回の往復が発生して p99 が劣化する。

**受け入れ基準**:
- 1 回の呼び出しで最大 100 キーを処理
- 部分失敗（一部キーの読み書きが失敗）は個別結果配列で返す（全体失敗ではない）
- Phase 1a では未提供、Phase 1b で提供

### FR-T1-STATE-004: 楽観的ロック（ETag）

**現状**: 同一キーへの同時書き込みで「読んで・変更して・書き戻す」を壊れずに行うには、Redis の `WATCH/MULTI/EXEC` または Lua スクリプトが必要。Dapr State の ETag サポートで軽量に扱える。

**要件達成後**: `Get` が返す `(value, etag)` の etag を保持し、`Set(key, new_value, etag=old_etag)` で競合検出する。etag 不一致時は `K1s0Error.Conflict` を返す。

**崩れた時**: 同時書き込みの整合性が tier2 アプリでバラバラに扱われ、上書き事故が発生する。

**受け入れ基準**:
- `Get` は etag（Valkey 内部の version 番号、不透明な文字列）を返す
- `Set` の etag 指定は `None`（無条件上書き）と明示指定（楽観ロック）を両方サポート
- etag 不一致は `K1s0Error.Conflict`（異なるエラー型）で返す

### FR-T1-STATE-005: トランザクション

**現状**: 複数キーの原子的な変更は Redis の `MULTI/EXEC` で可能だが、Dapr Transaction API は限定的なバックエンドでしか動作しない。Valkey は対応可能。

**要件達成後**: `Transaction([op1, op2, ...])` で複数操作を原子実行する。Get + Set + Delete を混在可能。

**崩れた時**: 複数キーに跨る業務トランザクションが tier2 側で再現困難となる。一貫性崩れの対応コードが各所に散在する。

**受け入れ基準**:
- トランザクション内の全操作が成功するか、全て失敗するかの 2 値
- 最大 10 操作 / トランザクションを上限
- 優先度 COULD（tier2 ユースケース発見後に実装判定）

## 入出力仕様

```
k1s0.State.Get(key: string) -> (value: bytes | null, etag: string | null)
k1s0.State.Set(key: string, value: bytes, options?: {
    ttl_seconds?: int,
    etag?: string,   // None=無条件、指定=楽観ロック
    serializer?: "json" | "protobuf"
}) -> K1s0Error?

k1s0.State.Delete(key: string, options?: { etag?: string }) -> K1s0Error?

k1s0.State.BulkGet(keys: string[]) -> [(value, etag, error?)]
k1s0.State.BulkSet(items: [{key, value, options?}]) -> [error?]
k1s0.State.BulkDelete(keys: string[]) -> [error?]

k1s0.State.Transaction(operations: Op[]) -> K1s0Error?
```

エラー型には `Conflict`（etag 不一致）、`SizeExceeded`（値サイズ超過、デフォルト上限 1MB）を追加。

## 受け入れ基準（全要件共通）

- すべての操作で tenant_id プレフィックスが自動付与され、tier2 から見えない
- 値サイズ上限 1MB をデフォルトとし、Component YAML で調整可能
- 1 キー 1 秒あたりの書き込み QPS は 10,000 を超えても劣化しない（Valkey 単一ノード想定）
- API 呼び出しは自動的に Telemetry span を生成、Tempo で traceId から Valkey 呼び出しまで追跡可能

## Phase 対応

- **Phase 1a**: FR-T1-STATE-001、002（Go SDK）
- **Phase 1b**: FR-T1-STATE-003、004 を追加、C# SDK
- **Phase 1c**: FR-T1-STATE-005（トランザクション）実装判定
- **Phase 2+**: Python / Rust SDK、代替バックエンド（etcd 等）検討

## 関連非機能要件

- **NFR-B-PERF-003**: State Get p99 < 10ms（キャッシュレイテンシ目標）
- **NFR-A-CONT-001**: Valkey 単一ノード障害時のフェイルオーバー（Phase 3 以降、MVP 段階は単一ノード）
- **NFR-E-AC-003**: tenant_id クレーム検証とキー越境防止
- **NFR-B-RES-001**: Valkey メモリ水平拡張（Phase 3 以降）
