# domain-master React クライアント

domain-master サービスの React フロントエンドクライアント。

## 技術スタック

| ライブラリ | バージョン | 用途 |
|---|---|---|
| React | 19 | UIフレームワーク |
| TanStack Router | 1.x | クライアントサイドルーティング |
| TanStack Query | 5.x | サーバー状態管理・キャッシュ |
| Zustand | 5.x | クライアント状態管理 |
| Zod | 3.x | スキーマバリデーション |
| axios | 1.x | HTTPクライアント |
| MSW | 2.x | APIモック (テスト用) |
| Vitest | 2.x | テストランナー |

## 機能

- **カテゴリ管理**: マスタカテゴリの一覧表示・作成・編集・削除
- **アイテム管理**: カテゴリ配下のアイテムの CRUD・階層ツリー表示
- **バージョン履歴**: アイテム変更履歴の before/after 差分表示
- **テナント拡張**: テナント固有のアイテムカスタマイズ管理

## ディレクトリ構成

```
src/
  types/          # 型定義 + Zod スキーマ
  lib/            # axios クライアント / QueryClient 設定
  hooks/          # TanStack Query フック (全エンドポイント)
  features/       # 機能別コンポーネント
    categories/
    items/
    versions/
    tenant-extensions/
  app/            # ルーター・App コンポーネント
tests/
  testutil/       # MSW セットアップ
```

## セットアップ

```bash
npm install
npm run dev
```

## テスト

```bash
npm test
```

## ビルド

```bash
npm run build
```

## API 接続

BFF 経由でアクセスします。開発時は `vite.config.ts` のプロキシ設定により
`/bff` リクエストが `http://localhost:8080` に転送されます。

認証は HttpOnly Cookie を使用します（`withCredentials: true`）。

## Docker

```bash
docker build -t domain-master-react .
docker run -p 80:80 domain-master-react
```
