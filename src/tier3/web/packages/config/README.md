# @k1s0/config

tier3 web の app / package が読む環境変数を一元化する。Vite と Node の両方で動作する。

## 利用例

```ts
import { loadConfig } from '@k1s0/config';

const cfg = loadConfig(import.meta.env);

console.log(cfg.bffUrl);     // -> "http://localhost:8080"
console.log(cfg.environment); // -> "dev" | "staging" | "prod"
```

## 必須 / 任意

- 必須: `VITE_BFF_URL` または `BFF_URL`
- 任意: `VITE_TENANT_ID` / `VITE_ENVIRONMENT` / `VITE_OTEL_COLLECTOR_URL` / `VITE_KEYCLOAK_ISSUER`

## テスト用ヘルパ

`stubConfig()` で部分 override 可能な fixture を生成できる。
