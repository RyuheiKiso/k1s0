# K1s0.System.Dlq

DLQ (Dead Letter Queue) 管理サーバーへの REST HTTP クライアント SDK。

## 機能

- DLQ メッセージの一覧取得 (ページネーション対応)
- メッセージ詳細取得
- メッセージ再処理 / 一括再処理
- メッセージ削除
- エラーレスポンスの型付きハンドリング

## 使い方

```csharp
services.AddK1s0DlqClient(new DlqConfig(
    BaseUrl: "http://dlq-server:8080",
    TimeoutSeconds: 30));
```

```csharp
public class MyService(IDlqClient dlqClient)
{
    public async Task ProcessDeadLetters()
    {
        var response = await dlqClient.ListMessagesAsync("orders.v1", page: 1, pageSize: 50);
        foreach (var msg in response.Messages)
        {
            await dlqClient.RetryMessageAsync(msg.Id);
        }
    }
}
```

## API エンドポイント

| メソッド | パス | 説明 |
|---------|------|------|
| GET | `/api/v1/dlq/:topic` | トピック別メッセージ一覧 |
| GET | `/api/v1/dlq/messages/:id` | メッセージ詳細 |
| POST | `/api/v1/dlq/messages/:id/retry` | 再処理 |
| DELETE | `/api/v1/dlq/messages/:id` | 削除 |
| POST | `/api/v1/dlq/:topic/retry-all` | 一括再処理 |

## テスト

```bash
dotnet test tests/
```
