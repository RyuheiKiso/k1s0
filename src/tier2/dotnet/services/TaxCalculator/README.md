# TaxCalculator

tier2 の税計算サービス（.NET 8 + ASP.NET Core minimal API、Onion 4 層、stateless 計算）。

## エンドポイント

`POST /api/tax/calculate`

リクエスト:

```json
{
  "mode": "EXCLUSIVE",
  "currency": "JPY",
  "minorAmount": 1000,
  "rateBasisPoints": 1000
}
```

`mode` は `EXCLUSIVE`（税抜から税込算出）/ `INCLUSIVE`（税込から税抜逆算）。`rateBasisPoints` は `100 = 1%`（消費税 10% は `1000`）。

正常レスポンス例:

```json
{
  "taxableMinorAmount": 1000,
  "taxMinorAmount": 100,
  "totalMinorAmount": 1100,
  "currency": "JPY",
  "appliedRateBasisPoints": 1000
}
```

## 設計判断

- 浮動小数点を使わず、`long` minor unit + `int` basis points の整数演算で丸め誤差を排除。
- 丸めは半端値「half-to-even」（銀行家丸め）固定。リリース時点 で half-up / half-down も選択可能にする予定。
- 永続化なし（pure compute）。`Infrastructure` 層は将来の税率テーブル DB 読込のための placeholder として保持。

## エラーコード

| コード | カテゴリ | 説明 |
|---|---|---|
| `E-T2-TAX-001` | VALIDATION | 入力不正（mode 不明 / minorAmount 負数 / rate 範囲外 等） |

## ビルド

```bash
dotnet build services/TaxCalculator/TaxCalculator.sln -c Release
dotnet test services/TaxCalculator/TaxCalculator.sln
```
