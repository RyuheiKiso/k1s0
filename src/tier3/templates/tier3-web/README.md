# tier3-web テンプレート

tier3 ユーザ向け Web SPA（React 18 + Vite + TypeScript strict）の最小雛形を生成する。

## 利用方法

```bash
k1s0-scaffold new tier3-web \
  --name portal \
  --owner @k1s0/web \
  --system k1s0
```

## 生成内容

- `{{name}}/package.json` — pnpm workspace 互換、React 18 + Vite + TS 5.5
- `{{name}}/tsconfig.json` — strict + ESNext + bundler resolution
- `{{name}}/vite.config.ts`
- `{{name}}/index.html`
- `{{name}}/src/main.tsx`
- `{{name}}/src/App.tsx` — placeholder ホームページ
- `{{name}}/catalog-info.yaml`
- `{{name}}/README.md`

## 関連

- [`examples/tier3-web-portal/`](../../../../examples/tier3-web-portal/)
- [`docs/05_実装/00_ディレクトリ設計/40_tier3レイアウト/`](../../../../docs/05_実装/00_ディレクトリ設計/40_tier3レイアウト/)
