# @k1s0/api-client

tier3 web の app から BFF（portal-bff / admin-bff）を呼ぶ薄い fetch wrapper。

## 利用例

```ts
import { createApiClient } from '@k1s0/api-client';
import { loadConfig } from '@k1s0/config';

const client = createApiClient({
  config: loadConfig(import.meta.env),
  getToken: () => localStorage.getItem('access_token'),
});

const value = await client.stateGet('postgres', 'user/123');
```

GraphQL も呼べる:

```ts
const data = await client.graphql<{ stateGet: { data: string } | null }>(
  'query Q($store: String!, $key: String!) { stateGet(store: $store, key: $key) { data etag } }',
  { store: 'postgres', key: 'user/123' },
);
```

## エラーハンドリング

すべての失敗は `ApiError` として throw される。`status` / `code` / `category` を持つ。

## 拡張ポイント

- リリース時点 では REST + GraphQL の最小呼出のみ
- リリース時点 で TanStack Query / Apollo Client に薄くラップして cache 層を追加する想定
