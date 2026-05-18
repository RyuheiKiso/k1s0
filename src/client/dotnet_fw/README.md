# .NET Framework SDK

dotnet_fw sdk_distribution_class の実装。
.NET Framework 4.6.2+ 対象のレガシー向け SDK。

## 主要特性
- DPAPI (Windows Data Protection API) によるストレージ暗号化を行う
- refresh_token は DPAPI protected blob として保管する
- WinHttpHandler による HTTP/1.1 + gRPC fallback (sse_paired) を使用する
- Companion 起動時に WinHttpHandler の ALPN h2 probe を実行する
  - probe 成功時: 業務 listener (h2/h3) に昇格して v1_full_native_with_companion 相当経路を使用する
  - probe 失敗時: v1_legacy_http11 専用 listener (port 8443-legacy) に固定する
- CLR profiler attach による auto_instrumentation を使用する

## 対応仕様
- spec 18: client SDK 配布適合仕様 (class: dotnet_fw)
