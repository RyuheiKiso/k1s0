---
id: arch.client.client_index
axis: client
phase: architecture
kind: index
status: draft
depends_on:
  - req.overview.provided_scope
  - req.overview.non_scope
  - req.overview.oss_catalog
covered_by:
  defense_in_depth_layers: [A, B, C, D, E]
  proof_classes:
    - v1_program_correctness_proof
---

# client 設計方針 index

## 一文方針
- client 層は tier1 Library / tier2 Service / tier3 SPA を 9 言語 SDK + Companion + Auto-Instrumentation の三層 package として全 distribution_class に同型投影する単一 facade である。SDK が独自の意味論を持たず、tier1 / tier2 / tier3 / infra / data / security / formal の 18 軸を SDK API surface に 1:1 投影する。

## 位置づけ
- client は本企画の「投影層」（projection layer）。tier1 04_Library で定義された Library 抽象と、tier3 で定義された 4 layer reducer / conflict tree を、9 言語 × 5 distribution_class へ機械的に転写する役割を持つ
- SDK はビジネス意味論を SDK 内で再定義しない。retry / SLO / quota / observability / capability / pii の規律はすべて proto annotation + Buf workflow + 18 軸の適合仕様が単一の真として宣言し、SDK は codegen で固定される
- client が物理的に唯一の SDK 配布 facade。「言語ごとに勝手に SDK を書く」「distribution_class 毎に retry semantics が違う」状態は CI 不変条件で物理 reject

## 5 distribution_class（v1）
- `v1_full_native_with_companion`: tier1 Library を直接組み込めるサーバ側 / 太い client（.NET 8 / Java 21 / Node.js 20 / Rust / Go / Python / Ruby）
- `v1_legacy_dotnet_framework`: レガシー .NET Framework 4.6.2+ exe / Service
- `v1_browser_spa_typescript`: Browser SPA（TypeScript / 任意 UI framework 非依存）
- `v1_thick_native_via_tauri`: Tauri デスクトップ Native（Rust + WebView）
- `v1_thin_business_api_only`: Library / Companion を組み込めない polyglot 環境（OpenAPI 仕様のみ）

bundle 関係は [クライアント SDK 配布適合仕様](../../04_詳細設計/01_適合仕様/18_クライアントSDK配布適合仕様.md) が単一所有。

## 設計原則
- **distribution_class は bundle**: class 1 値が language_runtime / distribution_channel / transport_capability_class / auto_instrumentation_mode / companion_provision_mode / device_at_rest_encryption_mode / refresh_token_storage_mode を一意に導出
- **dimension override 禁止**: 「Browser SPA の refresh_token を localStorage に変えたい」要望は新 class を切る形のみ
- **dead spec を CI で殺す**: classes / scenarios / capability cell / SDK package のいずれも参照消失で CI fail
- **lockstep SemVer**: 全 9 言語 SDK の MAJOR.MINOR を同期、PATCH のみ skew 許容
- **defense-in-depth は 5 層**: compile / lint / contract test / runtime / 物理（OS keychain / WebCrypto / Cosign / Kyverno）
- **手書き SDK code path 禁止**: protoc + Buf による単一 codegen pipeline のみ

## 8 方針構成
- [01_SDK 配布構成方針](01_SDK配布構成方針.md) — Library / Companion / Auto-Instrumentation の三層、5 distribution_class
- [02_Transport 適応方針](02_Transport適応方針.md) — 6 adapter / 4 capability_class、UA aware 経路選択
- [03_オフライン耐性方針](03_オフライン耐性方針.md) — 4 layer / pending queue durable / IndexedDB encrypted / OS keychain
- [04_認証コンテキスト伝播方針](04_認証コンテキスト伝播方針.md) — AuthContext / SessionContext / token lifecycle / mTLS
- [05_配布 SemVer 方針](05_配布SemVer方針.md) — lockstep release / 1.0.0 完璧 / deprecation_window 180 日
- [06_リトライバックオフ方針](06_リトライバックオフ方針.md) — 5 retry_policy_class / circuit breaker / retry budget
- [07_観測 Auto-Instrumentation 方針](07_観測Auto-Instrumentation方針.md) — distribution_class 別 attach / OTel SemConv / 二段 Collector
- [08_コード生成方針](08_コード生成方針.md) — protoc + Buf 単一 pipeline / method annotation / Apicurio 連携

## 詳細設計への参照
- [クライアント SDK 配布適合仕様](../../04_詳細設計/01_適合仕様/18_クライアントSDK配布適合仕様.md)
- [client 強制機構](../../04_詳細設計/02_強制機構/08_client強制機構.md)
- [client 運用 UI](../../04_詳細設計/04_運用UI開発者体験/05_client運用UI.md)
- [Companion OTel 拡張](../../04_詳細設計/03_クロスカッティング適合仕様/11_companion_otel_extension.md)
- [Connect-RPC UA-aware adapter](../../04_詳細設計/03_クロスカッティング適合仕様/12_UA_aware_adapter.md)
- [.NET 8 LTS 向け Connect-RPC 自製実装](../../04_詳細設計/03_クロスカッティング適合仕様/13_dotnet8_connect_inhouse.md)

## 採用しない選択肢
- 言語ごとの独立 SDK 開発（手書き stub 含む）
- distribution_class 毎の retry / SLO / observability semantics 差異
- SDK 公開 API での auto-instr 完全 disable
- 自製 crypto / 生 HTTP getter / access_token getter の SDK 公開 API 露出
- localStorage / SessionStorage への PII 平文書込
- 言語別 release date skew / PATCH での API 追加

## 至高路線における立ち位置
- 「言語数を絞れば運用が楽」を理由に SDK class を削減しない。1.0.0 で 5 class × 9 言語を全 release
- 「Auto-Instrumentation が大きいから外す」を採らない。distribution_class 全てで default on
- 「Python だけ release を遅らせる」を採らない。lockstep 必須
- SDK 規律は文章でなく CI / lint / contract test / property test / Cosign signed package + OS 物理機構で多重防御
