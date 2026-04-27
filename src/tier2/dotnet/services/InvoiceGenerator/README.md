# InvoiceGenerator

tier2 の請求書生成サービス（.NET 8 + ASP.NET Core minimal API、Onion 4 層）。

## エンドポイント

| メソッド | パス | 役割 |
|---|---|---|
| `POST` | `/api/invoices` | 行リストから請求書を生成 |
| `GET` | `/healthz` | liveness |
| `GET` | `/readyz` | readiness |

リクエスト例:

```json
{
  "customer": "Acme Corp",
  "lines": [
    { "description": "apple", "quantity": 3, "currency": "JPY", "unitMinorAmount": 100 },
    { "description": "banana", "quantity": 2, "currency": "JPY", "unitMinorAmount": 200 }
  ]
}
```

正常レスポンス（201 Created）には `id` / `customer` / `totalMinorAmount` / `currency` / `issuedAt` を含む。同 invoice 内の通貨は統一されている必要がある（混在は VALIDATION エラー）。

## エラーコード

| コード | カテゴリ | 説明 |
|---|---|---|
| `E-T2-INVOICE-001` | VALIDATION | 入力不正（lines 空 / quantity ≤ 0 等） |
| `E-T2-INVOICE-002` | VALIDATION | 通貨混在 |

## レイアウト

ApprovalFlow と同じ Onion 4 層 + 3 種テスト（Domain / Application / Architecture）。

## ビルド

```bash
dotnet build services/InvoiceGenerator/InvoiceGenerator.sln -c Release
dotnet test services/InvoiceGenerator/InvoiceGenerator.sln
```
