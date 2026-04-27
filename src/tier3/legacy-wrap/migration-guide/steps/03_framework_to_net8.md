# Step 3: .NET Framework → .NET 8 本格移行

Sidecar / Wrapper 形態から、.NET 8 完全移行へ進める。

## 移行パス

| パス | 対象 | 工数感 |
|---|---|---|
| A. Wrapper 化（`wrappers/` 配下） | 既存ライブラリを .NET Standard 2.0+ で参照可能なケース | 低〜中 |
| B. Pod 完全 Linux 化 | 既存資産が WCF / WPF / Forms に依存しない場合 | 中〜高 |
| C. 完全再実装 | 上記いずれも不可、または再設計の機会と判断した場合 | 高 |

## Wrapper 化の手順（パス A）

1. 既存 `.dll` を `third_party/` または社内 NuGet server に登録
2. `.NET 8` の wrapper csproj から `<Reference Include="LegacyLib">` で参照
3. wrapper が `K1s0.Sdk.Grpc` 経由で tier1 公開 API を呼ぶ
4. wrapper を Linux container として k1s0 基盤上で動作させる（Windows Node 依存解消）

## 注意

- .NET Framework 固有 API（`System.Web` / `WCF Server` / `WPF` / `Windows Forms`）を使っている場合は再実装が必要
- 文字エンコード（CP932 等）に依存しているコードは UTF-8 化を強制
- 同期 I/O は async/await に書き換え

## 検証

- 単体テスト: 既存テストを wrapper 経由で動かす（NetArchTest で依存方向を強制）
- 性能: 移行前後で p95 latency / メモリフットプリント / startup time を比較
- 互換性: 既存クライアントとの payload / API contract を Pact で検証
