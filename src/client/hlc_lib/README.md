# k1s0-hlc — Hybrid Logical Clock (HLC) 4 言語実装

Kulkarni et al. (2014) "Logical Physical Clocks" を Rust / TypeScript / Go / C# の 4 言語で実装した共通ライブラリ。

`src/CLAUDE.md §wall-clock TTL 禁止` に従い、tier3 (TypeScript) / tier2 (Go / C#) の全 deadline 計算をこのライブラリ経由で行う。

## ディレクトリ構成

```
src/client/hlc_lib/
├── rust/       Rust 実装 (k1s0-hlc crate)
├── typescript/ TypeScript 実装 (@k1s0/hlc-lib)
├── go/         Go 実装 (github.com/k1s0/hlc-lib-go)
└── csharp/     C# 実装 (K1s0.HlcLib)
```

## 4 言語 API 対応表

| Rust | TypeScript | Go | C# |
|---|---|---|---|
| `HlcTimestamp { wall_ms: u64, logical: u16, node_id: u16 }` | `new HlcTimestamp(wall_ms: bigint, logical: number, node_id: number)` | `HlcTimestamp{ WallMs uint64, Logical uint16, NodeId uint16 }` | `new HlcTimestamp(WallMs: ulong, Logical: ushort, NodeId: ushort)` |
| `HlcTimestamp::EPOCH` | `HlcTimestamp.EPOCH` | `hlc.Epoch` | `HlcTimestamp.Epoch` |
| `ts.format_compact()` | `ts.formatCompact()` | `ts.FormatCompact()` | `ts.FormatCompact()` |
| `HlcTimestamp::parse_compact(s)` → `Option<Self>` | `HlcTimestamp.parseCompact(s)` → `HlcTimestamp \| null` | `hlc.ParseCompact(s)` → `(HlcTimestamp, bool)` | `HlcTimestamp.ParseCompact(s)` → `HlcTimestamp?` |
| `ts.add_ms(ms: u64)` → `Self` | `ts.addMs(ms: bigint)` → `HlcTimestamp` | `ts.AddMs(ms uint64)` → `HlcTimestamp` | `ts.AddMs(ms: ulong)` → `HlcTimestamp` |
| `ts.elapsed_ms_since(ref)` → `u64` | `ts.elapsedMsSince(ref)` → `bigint` | `ts.ElapsedMsSince(ref)` → `uint64` | `ts.ElapsedMsSince(ref)` → `ulong` |
| `ts.is_expired_at(current)` → `bool` | `ts.isExpiredAt(current)` → `boolean` | `ts.IsExpiredAt(current)` → `bool` | `ts.IsExpiredAt(current)` → `bool` |
| `HlcClock::new(node_id)` | `new HlcClock(nodeId)` | `hlc.NewHlcClock(nodeId)` | `new HlcClock(nodeId)` |
| `HlcClock::from_env()` | `HlcClock.fromEnv()` | `hlc.NewHlcClockFromEnv()` | `HlcClock.FromEnv()` |
| `clock.tick()` | `clock.tick()` | `clock.Tick()` | `clock.Tick()` |
| `clock.recv(msg_ts)` | `clock.recv(msgTs)` | `clock.Recv(msgTs)` | `clock.Recv(msgTs)` |
| `clock.now()` | `clock.now()` | `clock.Now()` | `clock.Now()` |

## 環境変数

| 変数名 | 型 | 説明 |
|---|---|---|
| `HLC_NODE_ID` | u16 (0–65535) | ノード識別子。未設定時は 0 を使用する。複数インスタンス環境ではインスタンスごとに異なる値を設定する。 |

## 使用方法

### deadline 計算パターン（全言語共通）

```
// 現在の HLC タイムスタンプを取得する
current = clock.now()

// 5 分後の deadline を計算する（wall-clock 禁止: add_ms 経由で計算する）
deadline = current.add_ms(5 * 60 * 1000)

// 後で期限切れを確認する
current2 = clock.now()
if deadline.is_expired_at(current2):
    // 期限切れ処理
```

### コンパクト文字列形式

HLC タイムスタンプは `{wall_ms_hex_16}-{logical_04x}-{node_04x}` 形式でシリアライズできる。

例: `0000018d5a3b7c40-0005-0001`
- `0000018d5a3b7c40`: wall_ms (UNIX epoch ms, hex 16 桁)
- `0005`: logical (hex 4 桁)
- `0001`: node_id (hex 4 桁)

### テスト実行

```bash
# Rust
cd src/client/hlc_lib/rust && cargo test

# TypeScript (vitest)
cd src/client/hlc_lib/typescript && npm install && npm test

# Go
cd src/client/hlc_lib/go && go test ./...

# C#
cd src/client/hlc_lib/csharp && dotnet test
```
