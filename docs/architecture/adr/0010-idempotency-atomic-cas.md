# ADR-0010: Idempotency Redis ストアの TOCTOU 競合状態修正 — Lua スクリプトによるアトミック CAS 採用

## ステータス

承認済み（2026-03-26 更新: Rust 版対応完了）

## コンテキスト

`regions/system/library/go/idempotency/redis.go` の `MarkCompleted` / `MarkFailed` メソッドは、
以下の非アトミックなパターンで実装されていた。

```
Step 1: GET  key         → record
Step 2: record.Status = StatusCompleted  (メモリ上で更新)
Step 3: SET  key = record                (上書き保存)
```

この **Get → Modify → Set** シーケンスは TOCTOU（Time-Of-Check to Time-Of-Use）競合状態を引き起こす。
複数の goroutine が同じキーに対して並行して操作した場合、以下の問題が発生しうる。

| 問題 | 内容 |
|------|------|
| Lost Update | goroutine A と B が同時に GET し、それぞれ別の値で SET すると一方の更新が失われる |
| 中間状態の観測 | GET と SET の間に別の goroutine が読み取ると、更新前の古いデータを取得する |

特に高トラフィック環境やリトライロジックが絡む場面では、冪等性保証の根幹を揺るがす問題となる。

また、旧実装の `saveUpdatedRecord` は `TTL=0` の場合に `Del` + `ExpiredError` を返す処理を含んでいたが、
`KEEPTTL` オプションを使用する新実装では既存の TTL を透過的に保持できる。

## 決定

`MarkCompleted` / `MarkFailed` の実装を **Redis Lua スクリプトによるアトミック CAS（Compare-and-Swap）** に置換する。

- `redis.NewScript(luaScript)` で Lua スクリプトを定数として定義する
- `Script.Run(ctx, client, keys, args...)` で実行する
- Lua スクリプト内で `GET → cjson.decode → フィールド更新 → cjson.encode → SET KEEPTTL` を
  単一のアトミック操作として実行する
- `go-redis/v9` の `Script.Run` は EVALSHA → EVAL フォールバックを自動的に行う

Lua スクリプトの設計：

```lua
-- MarkCompleted 用
local raw = redis.call('GET', KEYS[1])
if raw == false then
  return redis.error_reply('not_found')
end
local record = cjson.decode(raw)
record['status'] = 'completed'
if ARGV[1] ~= '' then
  record['response'] = ARGV[1]   -- base64 エンコード済みレスポンス
else
  record['response'] = nil
end
record['status_code'] = tonumber(ARGV[2])
record['error'] = nil
redis.call('SET', KEYS[1], cjson.encode(record), 'KEEPTTL')
return 1
```

Go の `encoding/json` は `[]byte` フィールドを base64 エンコードして JSON に格納するため、
Lua スクリプトへ渡す `response` 引数も `base64.StdEncoding.EncodeToString` 済みの文字列を使用する。
これにより Go 側での `json.Unmarshal` 時にデコードが正常に行われる。

## 理由

### Lua スクリプトを選んだ理由

Redis は単一スレッドでスクリプトを実行するため、Lua スクリプト内の処理は **原子的に実行されることが保証される**。
`GET` と `SET` の間に他のコマンドが割り込む余地がない。

### WATCH/MULTI トランザクションを採用しなかった理由

`WATCH/MULTI/EXEC` パターンはオプティミスティックロックを提供するが、以下の欠点がある。

- WATCH した後にキーが変更された場合、EXEC は `nil` を返しクライアント側でリトライが必要
- ネットワークラウンドトリップが増える（WATCH → GET → MULTI → SET → EXEC = 5往復）
- `redis.Cmdable` インターフェース経由では WATCH/MULTI/EXEC を直接使用しにくい（パイプライン管理が複雑）

Lua スクリプトは 1回のネットワーク往復で完結し、クライアント側のリトライ実装が不要である。

## 影響

**ポジティブな影響**:

- `MarkCompleted` / `MarkFailed` の並行安全性が保証される
- Lost Update 問題が解消され、冪等性保証の信頼性が向上する
- `KEEPTTL` を使用することで既存 TTL が透過的に保持される
- ネットワーク往復回数が旧実装（GET + SET = 2往復）と同数（Lua = 1往復）に削減される

**ネガティブな影響・トレードオフ**:

- `IdempotencyRecord` の JSON フィールドを変更・追加する際は、対応する Lua スクリプトも更新する必要がある
- Lua スクリプトのデバッグは通常の Go コードより難易度が高い
- `KEEPTTL` は Redis 6.0 以降で利用可能。それ以前の Redis バージョンではサポートされない
- `response` フィールドの base64 エンコード/デコード規則を Lua スクリプトと Go コード間で一致させる必要がある

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| WATCH/MULTI/EXEC | オプティミスティックロックによるトランザクション | ネットワーク往復が多く、クライアント側リトライが必要。`redis.Cmdable` との相性が悪い |
| 分散ロック (Redlock) | Redlock アルゴリズムで排他制御 | 実装が複雑、追加の Redis キーが必要、ロック取得失敗時の処理が必要 |
| SET with version field | レコードにバージョン番号を付与し楽観的ロック | スキーマ変更が必要、クライアント側でバージョン管理とリトライが必要 |
| 専用ロックキー | `SET NX PX` で一時ロックキーを作成し排他制御 | ロックの取得/解放の2往復が必要、ロック保持中クラッシュ時の処理が必要 |

## 参考

- [Redis Lua Scripting Documentation](https://redis.io/docs/latest/develop/interact/programmability/lua-api/)
- [Redis EVAL command](https://redis.io/docs/latest/commands/eval/)
- [go-redis Script.Run](https://pkg.go.dev/github.com/redis/go-redis/v9#Script.Run)
- [TOCTOU Race Condition (Wikipedia)](https://en.wikipedia.org/wiki/Time-of-check_to_time-of-use)
- [Redis Atomicity Guarantees](https://redis.io/docs/latest/develop/interact/programmability/#atomicity-of-scripts)
- 実装ファイル: `regions/system/library/go/idempotency/redis.go`
- Rust 実装ファイル: `regions/system/library/rust/idempotency/src/store.rs`
- ライブラリ設計書: `docs/libraries/resilience/idempotency.md`

## 追記: Rust 版アトミック CAS 対応完了（M-3 監査対応）

**更新日:** 2026-03-26

外部技術監査（M-3）の指摘を受け、Rust 版 `RedisIdempotencyStore` の `mark_completed` / `mark_failed` も
Lua スクリプトによるアトミック CAS に置換した。

### 変更内容

`regions/system/library/rust/idempotency/src/store.rs` の Redis 実装において、
従来の非アトミックな GET → 更新 → SET パターンを Lua スクリプトに置換した。

| 変更前 | 変更後 |
|--------|--------|
| `self.get(key)` → フィールド更新 → `conn.set(...)` の 2 往復 | Lua スクリプト内で GET → decode → 更新 → encode → SET KEEPTTL の 1 往復 |

### Rust 版 Lua スクリプトの注意点

- `deadpool_redis::redis::Script::prepare_invoke()` を使用して引数を組み立てる
- `Script::invoke_async(&mut *conn)` でアトミック実行する
- Go 版とのスキーマ差異: Rust の serde デフォルトでは `IdempotencyStatus` が PascalCase
  （`"Completed"`, `"Failed"`）で JSON に格納されるため、Lua スクリプト内も `'Completed'`, `'Failed'` とする
- `ARGV[2]` に `"0"` が渡された場合は `response_status` を JSON null として扱う
