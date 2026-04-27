# Step 2: 既存 API の段階置換

既存 .NET Framework App の重要呼出を、Sidecar 経由の k1s0 公開 API に段階的に置き換える。

## 進め方

1. 既存呼出の依存マップを作成する（DB 直接呼出 / 外部 API / file I/O 等を分類）。
2. 重要度の高いものから順に対象化する（一般に DB → 認証 → file I/O → 外部 API）。
3. 切替時は **Feature Flag**（`infra/feature-management/flagd/`）で旧呼出 / 新呼出を切替可能にする。
4. 観測性: tier1 OTel 経由で旧/新両者のレスポンス時間 / エラー率を計測し、退行が無いことを確認する。
5. 退行が出たら Flag で即時 rollback する。

## 例: DB 読込の置換

既存:

```csharp
using (var conn = new SqlConnection("...")) {
    var user = conn.QuerySingle<User>("SELECT * FROM users WHERE id = @id", new { id });
}
```

置換後（Sidecar 経由 k1s0 State）:

```csharp
var http = new HttpClient { BaseAddress = new Uri("http://localhost") };
var res = await http.GetAsync($"/api/k1s0bridge/state/users/{id}");
res.EnsureSuccessStatusCode();
var json = await res.Content.ReadAsStringAsync();
var stateValue = JsonConvert.DeserializeObject<StateValue>(json);
var user = JsonConvert.DeserializeObject<User>(stateValue.Data);
```

## ロールアウト戦略

- canary 5% → 25% → 100% を Argo Rollouts で適用（`deploy/rollouts/canary-strategies/`）
- 検証メトリクスは error rate < 1% / p95 latency < 200ms（`deploy/rollouts/analysis-templates/`）
