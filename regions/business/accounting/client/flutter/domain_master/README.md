# Domain Master Client (Flutter)

ドメインマスタ管理用Flutterクライアント。マスタカテゴリ・アイテムのCRUD操作、バージョン履歴閲覧、テナント拡張管理を提供する。

## 技術スタック

- Flutter 3.24+
- Riverpod (状態管理)
- go_router (ルーティング)
- Dio (HTTP通信)
- Material Design 3

## 画面構成

- カテゴリ一覧 (`/`) - マスタカテゴリの管理
- アイテム一覧 (`/categories/:code/items`) - 階層構造のアイテム管理
- バージョン履歴 (`/categories/:code/items/:item_code/versions`) - 変更履歴
- テナント拡張 (`/tenants/:tenant_id/extensions`) - テナント固有カスタマイズ

## 開発

```bash
flutter pub get
flutter run -d chrome
```

## ビルド

```bash
docker build -t domain-master-client .
docker run -p 8080:80 domain-master-client
```
