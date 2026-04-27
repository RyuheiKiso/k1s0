# 移行ガイド

採用側組織の既存 .NET Framework 4.8 資産を k1s0 基盤上で運用しつつ、段階的に .NET 8 / .NET MAUI へ移行するための実装ガイド。

## 段階

| 段階 | 目的 | 主たる成果物 |
|---|---|---|
| 1. Sidecar 接続 | 既存資産を変更せず Dapr sidecar 経由で k1s0 公開 API を利用可能にする | `K1s0.Legacy.Sidecar` Pod 起動、`/api/k1s0bridge/healthz` 200 |
| 2. API 段階置換 | 既存 API 呼出を sidecar 経由に書き換え、観測性 / 認可を k1s0 基盤に揃える | 重要 API から順次切替、ABテスト |
| 3. .NET 8 本格移行 | Sidecar / Wrapper 形態から .NET 8 完全移行へ | `wrappers/` 配下の .NET 8 ラッパー、または完全再実装 |

詳細は `steps/` 配下を参照。

## 関連 ID

- ADR-MIG-001（.NET Framework sidecar）
- ADR-MIG-002（API Gateway）
- NFR-D-MIG-\* / 制約 8
