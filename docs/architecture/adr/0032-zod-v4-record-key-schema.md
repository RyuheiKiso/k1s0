# ADR-0032: zod v4 API移行方針 — record キースキーマ必須化

## ステータス

採用済み (2026-03-25)

## 背景

zod ライブラリが v3 から v4 へアップグレードされた際、`z.record()` APIに破壊的変更が導入された。
v3 では `z.record(valueSchema)` という形式でキースキーマを省略可能だったが、
v4 では `z.record(keySchema, valueSchema)` という形式でキースキーマの明示的な指定が必須となった。

影響を受けたファイル:
- `regions/service/activity/client/react/activity/src/types/activity.ts`

```typescript
// v3 (旧)
metadata: z.record(z.unknown()).nullable(),

// v4 (新)
metadata: z.record(z.string(), z.unknown()).nullable(),
```

## 決定内容

プロジェクト全体で `z.record()` を使用する際は、キースキーマを明示的に `z.string()` として指定する。
JSON オブジェクトのキーは常に文字列であるため、`z.record(z.string(), valueSchema)` が意味的にも正確である。

## 理由

1. **型安全性の向上**: キースキーマを明示することで、TypeScript の型チェックがより厳密になる
2. **zod v4 準拠**: v4 のAPI変更に追従し、将来のアップグレードを妨げない
3. **意味的正確性**: JSON オブジェクトのキーは仕様上 `string` 型であるため、`z.string()` の明示は正確

## 影響

- `z.record(z.unknown())` を含む全 TypeScript ファイルを `z.record(z.string(), z.unknown())` に変更する
- `z.record(z.any())` も同様に `z.record(z.string(), z.any())` に変更する
- 既存のバリデーションロジックに変更なし（キースキーマの追加のみ）

## 代替案

- **zod v3 に固定する**: セキュリティ更新を受けられなくなるため却下
- **keyof typeof でキーを列挙する**: 動的なキーには対応できないため却下

## 参考資料

- [zod v4 migration guide](https://zod.dev/changelog)
- 影響ファイル: `regions/service/activity/client/react/activity/src/types/activity.ts`
