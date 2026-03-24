# bff-proxy

バックエンド・フォー・フロントエンド（BFF）プロキシサーバー。React SPA および Flutter クライアントからの HTTP/WebSocket リクエストを受け付け、system tier の各 gRPC サービスへルーティングする。JWT の `iss`/`aud` 検証もこの層で実施し、フロントエンド向けのレスポンス整形を行う。

## 技術スタック

- **言語**: Go
- **フレームワーク**: Chi（HTTP ルーター）+ gRPC クライアント
- **設計**: クリーンアーキテクチャ + DDD

## ディレクトリ構造

```
.
├── cmd/            # エントリポイント
├── internal/
│   ├── domain/     # ドメインモデル
│   ├── usecase/    # ユースケース層
│   ├── adapter/    # HTTP ハンドラー・gRPC クライアント
│   └── config/     # 設定読み込み
└── go.mod
```

## ローカル起動

```bash
# 依存サービスを起動
just local-up-dev

# サービス単体をビルド
go build ./cmd/...
```

## テスト実行

```bash
# ユニットテスト + 統合テスト
go test ./... -race -count=1
```

## 設計書

- 設計書: [`docs/servers/system/bff-proxy/`](../../../../../docs/servers/system/bff-proxy/)
- API 定義: [`api/proto/k1s0/system/`](../../../../../api/proto/k1s0/system/)
