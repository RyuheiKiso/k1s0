---
id: req.functional.client_sdk_functional
axis: client
phase: requirement
kind: requirement
status: draft
depends_on:
  - req.functional.functional_index
  - arch.client.client_index
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# クライアント SDK 機能要件

## 一文方針
- クライアント SDK は 5 distribution_class × 9 言語 SDK の lockstep release で機能要件として宣言する。Library / Companion / Auto-Instrumentation の三層配布、6 transport adapter、5 retry_policy_class、device_at_rest_encryption、refresh_token storage、build_provenance を必須要件とする。

## 5 distribution_class
- `v1_full_native_with_companion`: tier1 Library 直接組込（.NET 8 / Java 21 / Node.js 20 / Rust / Go / Python / Ruby）
- `v1_legacy_dotnet_framework`: レガシー .NET Framework 4.6.2+
- `v1_browser_spa_typescript`: Browser SPA（TypeScript）
- `v1_thick_native_via_tauri`: Tauri デスクトップ
- `v1_thin_business_api_only`: polyglot 環境（OpenAPI 仕様のみ）

## 9 言語 SDK lockstep release
- 同一 MAJOR.MINOR を全言語同時 release
- PATCH のみ skew 許容
- waterfall release は 30 日以内に他言語へ propagate

## 主要機能要件
- **三層配布**: Library / Companion / Auto-Instrumentation
- **6 transport adapter**: grpc_native / connect_bidi / grpc_web / web_transport / sse_paired / paired_post_sse / long_poll / webhook / messaging_bridge
- **UA-aware adapter**: 5 ua_subclass で経路分割
- **5 retry_policy_class**: v1_idempotent_safe / v1_idempotent_with_key / v1_non_idempotent / v1_long_streaming_resumable / v1_eventual_via_pending_queue
- **device_at_rest_encryption**: os_native / dpapi / webcrypto_aes_gcm / OS keychain wrap
- **refresh_token storage**: process_memory / dpapi_protected_blob / in_memory + httponly_cookie / os_keychain
- **OTel Auto-Instrumentation**: distribution_class 別 attach 方式、disable 経路なし
- **k1s0.Companion.NetFx.OTelExt**: 4 stack で JWT claim → Activity attribute 注入
- **k1s0.Connect.NetCore**: v1_inhouse_authoritative、Connect Conformance Suite 全 case green

## 詳細設計参照
- [client 設計方針 index](../../03_概要設計/09_client設計方針/README.md)（8 方針）
- [クライアント SDK 配布適合仕様](../../04_詳細設計/01_適合仕様/18_クライアントSDK配布適合仕様.md)
- [client 強制機構](../../04_詳細設計/02_強制機構/08_client強制機構.md)

## 受入条件
- 5 distribution_class × 9 言語 SDK の conformance test 全 green
- lockstep release（MAJOR.MINOR skew = 0）
- 全 SDK package が cosign signed + SBOM + SLSA provenance attestation
- Connect Conformance Suite 全 case green

## 関連参照
- [機能要件 index](README.md)
- [client 設計方針 index](../../03_概要設計/09_client設計方針/README.md)
