# Full Native SDK (9 言語)

full_native sdk_distribution_class の実装。
9 言語 (Rust / Go / C# / TypeScript / Python / Ruby / Java / Browser / Tauri) の wrapper facade。

## 主要特性
- 9 言語 SDK の MAJOR.MINOR 同期を維持する (PATCH のみ skew 許容)
- 1 言語のみリリースを先行させることを禁止する (lockstep 規約)
- transport_capability_class: http2_native (gRPC bidi 完全 + HTTP/3 + WebTransport optional)
- platform_keystore (OS keychain) による device_at_rest_encryption を実施する
- agent_attach (.NET / Java) および library_auto (他言語) による OTel auto_instrumentation を使用する
- companion は optional (サーバ側 / 太い client 向け)

## 対応言語
| 言語 | ランタイム | 配布チャネル |
|---|---|---|
| Rust | rust stable | cargo |
| Go | go 1.22+ | go module |
| C# | .NET 8+ | nuget |
| TypeScript | node 20 | npm |
| Python | python 3.12 | pypi |
| Ruby | ruby 3.3 | gems |
| Java | openjdk 21 | maven |
| Browser | chromium/firefox/safari | npm |
| Tauri | rust + webview | cargo + npm |

## 対応仕様
- spec 18: client SDK 配布適合仕様 (class: full_native)
