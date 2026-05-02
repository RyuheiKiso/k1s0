# ADR-TIER3-003: tier3 Native アプリに .NET MAUI を採用する

- ステータス: Accepted
- 起票日: 2026-05-02
- 決定日: 2026-05-02
- 起票者: kiso ryuhei
- 関係者: tier3 Native 開発チーム / 採用検討組織 / DevEx チーム

## コンテキスト

k1s0 の tier3 には Web SPA（ADR-TIER3-002）に加えて Native アプリ（モバイル / デスクトップ）が含まれる。`src/tier3/native/apps/{Hub,Admin}/` で「業務担当者がモバイル / デスクトップから k1s0 にアクセスする」用途を想定し、tier1 公開 12 API + tier2 ドメインサービスへの HTTPS / gRPC アクセスを行う。

Native アプリの技術選定では以下の対立軸が顕在化する。

- **OS Native**（Swift / Kotlin / WinUI）: 各 OS 最適化、ただしコード重複が iOS / Android / Windows / macOS で 4 倍
- **クロスプラットフォーム**（Flutter / React Native / .NET MAUI / Tauri / Electron）: コードシェアによる開発効率、ただし各フレームワーク固有の制約
- **Web View ラッパー**（Capacitor / Cordova）: Web 資産の再利用、ただしネイティブ機能アクセスが間接的

加えて k1s0 の前提として、

- **採用組織の人材プール**: 日本企業の業務システム開発で既に大量の人材が居る言語は **C#（.NET Framework / .NET Core 系）と Java**。tier2 で .NET / Go を使っている（ADR-TIER1-001 と整合）
- **tier2 .NET 生態系との統合**: tier2 の `K1s0.Tier2.Common` / `K1s0.Sdk.Grpc` / `K1s0.Sdk.Proto` が C# で提供されているため、Native でも C# を使えばコード共有が可能
- **業界での技術安定性**: 10 年保守を前提とすると、フレームワークが消える / 大幅変更されるリスクが採用判断に影響
- **対象 OS**: Windows / Android / iOS / macOS の 4 OS で動作する想定（業務担当者は Windows + iPhone / Android が多数派）
- **業務性能要件**: 業務 UI なので 3D / 高フレームレート不要、HTTPS REST と画面遷移が中心

選択は採用組織の Native 開発体験 / リクルーティング / .NET 生態系の活用度に直結し **two-way door** だが、コードベースの書き換えコストは大きい。リリース時点で確定する。

## 決定

**tier3 Native アプリには .NET MAUI（Multi-platform App UI、.NET 8+）を採用する。**

- .NET MAUI 8 LTS、対象プラットフォーム: Android / iOS / macOS（Catalyst）/ Windows（WinUI 3）
- アーキテクチャ: MVVM（XAML + ViewModels + Services）
- tier1 / tier2 へのアクセスは `K1s0.Sdk.Grpc` / `K1s0.Sdk.Proto`（tier2 と同じ SDK）を ProjectReference
- 共通ロジック（認証 / API 呼出 / observability）は `src/tier3/native/shared/K1s0.Native.Shared/` に集約
- UI 実装は各アプリ（`Hub` / `Admin`）の `App.xaml` + `AppShell` + `Pages/` + `ViewModels/` に分散
- platform 別 native API（Biometric / SecureStorage / Push）が必要な場合は `Platforms/{Android,iOS,MacCatalyst,Windows}/` で分離実装
- リリース時点 では net8.0 単体で build 検証、platform 別実機 build / store 配布は採用初期で実施

`src/tier3/native/apps/K1s0.Native.{Hub,Admin}/` と `src/tier3/native/shared/K1s0.Native.Shared/` で確定（既存実装あり、SHIP_STATUS § tier3）。

## 検討した選択肢

### 選択肢 A: .NET MAUI（採用）

- 概要: Microsoft 製のクロスプラットフォーム フレームワーク（Xamarin.Forms の後継）、.NET 8+ で stable
- メリット:
  - **C# / XAML、tier2 .NET 生態系との完全統合**
  - tier2 SDK（`K1s0.Sdk.Grpc` / `K1s0.Sdk.Proto`）を ProjectReference で共有可能
  - 採用組織の C# 開発者（業務システム開発で大量に居る）がそのまま Native 開発に参画可能
  - Microsoft 公式、長期サポート（.NET LTS は 3 年）
  - Android / iOS / macOS Catalyst / Windows の 4 OS 対応
  - Visual Studio / VS Code / Rider のいずれでも開発可能
- デメリット:
  - Xamarin.Forms 後継として stable 化したのが 2022 年で、Flutter / React Native より歴史が浅い
  - iOS の native 機能の一部追従が遅延することがある
  - Linux 公式サポートなし（業務担当者の Linux 利用は想定外）

### 選択肢 B: Flutter

- 概要: Google 製、Dart 言語、独自描画エンジン
- メリット:
  - 独自描画で全 OS で同じ見た目、性能良好
  - ホットリロード、開発体験良
  - 業界での採用事例豊富
- デメリット:
  - **Dart 言語が採用組織の既存人材プールから乖離**（学習コスト大、リクルーティング難）
  - tier2 .NET 生態系と統合できない（SDK を Dart で別途生成する作業が要る）
  - Google が将来サポートを縮小するリスク（過去に Angular Dart 等の前例）

### 選択肢 C: React Native

- 概要: Meta 製、JavaScript / TypeScript
- メリット:
  - Web SPA（ADR-TIER3-002 が React）と言語共通
  - 業界での採用事例多
- デメリット:
  - **Web の React と native の React Native は実質別物**（Component / Routing / Style が大きく異なる）、共通化のメリットは限定的
  - tier2 .NET 生態系と統合できない（TypeScript SDK を別途必要）
  - Meta が将来サポートを縮小するリスク
  - Native bridge のパフォーマンス問題（New Architecture で改善中だが）

### 選択肢 D: Tauri Mobile

- 概要: Rust + WebView の軽量フレームワーク
- メリット:
  - Bundle size 極小
  - Rust 統合（k1s0 の tier1 Rust crates と整合する可能性）
- デメリット:
  - **モバイル対応が beta（2024 時点）**、業務利用に時期尚早
  - Rust 開発者が日本企業の業務系で少なく、採用組織の人材プールから乖離
  - tier2 .NET 統合に追加レイヤ要

### 選択肢 E: 各 OS Native（Swift / Kotlin / WinUI）

- 概要: 各 OS の純 Native 言語
- メリット:
  - 各 OS で最適なパフォーマンス・UX
  - 各 OS の最新 API 追従が最速
- デメリット:
  - **コード重複が iOS / Android / Windows / macOS の 4 倍**
  - 採用組織が 4 言語の人材を抱える必要、リクルーティング負担大
  - 業務 UI 程度の機能で Native の最適化は overkill

### 選択肢 F: Web View ラッパー（Capacitor / Cordova）

- 概要: Web 資産を Native アプリにラップ
- メリット: Web 開発資産の流用
- デメリット:
  - SecureStorage / Biometric / Push 等の native 機能アクセスが Capacitor plugin 経由で間接的
  - Web 資産は ADR-TIER3-002 で SPA に確定、業務担当者向けの専用 UX を提供しにくい
  - Cordova はメンテ縮小傾向

### 選択肢 G: Electron（デスクトップのみ）

- 概要: Web 技術でデスクトップアプリ
- メリット: Web 資産流用
- デメリット:
  - **モバイル非対応**、tier3 Native の要件を満たさない
  - Bundle size が巨大（Chrome 込み）

## 決定理由

選択肢 A（.NET MAUI）を採用する根拠は以下。

- **tier2 .NET 生態系との完全統合**: tier2 の `K1s0.Sdk.Grpc` / `K1s0.Sdk.Proto` / `K1s0.Tier2.Common` が C# で提供されており、MAUI からは ProjectReference で再利用可能。Flutter（B）/ React Native（C）/ Tauri Mobile（D）はいずれも別言語で、SDK を再生成する負債を抱える
- **採用組織の人材プールとの整合**: 日本企業の業務系開発で C# 開発者は最大級の人材プール。tier2 で .NET を採用している以上、Native も C# にすることで「同じチームが tier2 / tier3 Native の両方を扱える」体制が組める。Conway の法則を踏まえると、技術分割と組織分割が一致する
- **クロスプラットフォーム の現実性**: 業務担当者の利用 OS は Windows / Android / iOS / macOS で、4 OS 対応が必須。各 OS Native（E）は工数 4 倍で破綻、Tauri Mobile（D）は時期尚早
- **業務 UI 特性との適合**: 業務 UI は 3D / 高フレームレート不要で、画面遷移と HTTPS API 呼出が中心。MAUI の XAML + ViewModels で十分。Flutter の独自描画 / React Native の Native bridge は業務 UI には overkill
- **長期保守の信頼性**: Microsoft 公式 + .NET LTS（3 年サイクル）+ Xamarin からの継承（2014〜）で、業界での実績と保守期間の見通しが立つ。Tauri Mobile（D、beta）は時期尚早、React Native（C）は Meta の方針依存
- **退路の確保**: 将来 Native 要件が業務 UI を超えた場合（例: 高度な 3D / AR）、特定 OS のみ Native（E）で再実装する経路は残せる。MAUI は Android / iOS / Windows / macOS の標準 native API への薄い抽象なので、撤退時の Native コード再実装は機能単位で局所化できる

## 帰結

### ポジティブな帰結

- tier2 .NET 生態系と完全統合、SDK 共有でコード重複なし
- 採用組織の C# 人材プールがそのまま Native 開発に流用可能
- tier2 / tier3 Native を同じチームで扱える組織体制が成立
- Microsoft 公式 + .NET LTS で 10 年保守の見通しが立つ
- Visual Studio / VS Code / Rider のいずれでも開発可能、開発者体験の選択肢広い

### ネガティブな帰結 / リスク

- MAUI workload の install 必要（dotnet workload install maui）、CI / dev container で別途整備
- platform 別 build（android / ios / maccatalyst / windows）には platform 別 SDK が必要、CI コスト
- iOS の native 機能追従が Swift より遅延することがある（業務 UI 範囲では問題にならない見込み）
- リリース時点 では net8.0 単体 build のみ検証、platform 別実機 build / store 配布は採用初期で実装

### 移行・対応事項

- `src/tier3/native/apps/K1s0.Native.{Hub,Admin}/` で MAUI app skeleton を確定（既存実装あり）
- `src/tier3/native/shared/K1s0.Native.Shared/` で共通ロジック（認証 / API 呼出 / observability）を集約（既存実装あり）
- リリース時点 で net8.0 単体 build を CI で検証、platform 別 build は採用初期で IMP-CI-* に追加
- platform 別 native API（Biometric / SecureStorage / Push）の `Platforms/` 実装ガイドを `docs/05_実装/00_ディレクトリ設計/40_tier3レイアウト/04_native配置.md` に明文化
- store 配布手順（App Store / Play Store / MS Store）の Runbook 化は採用初期で
- MAUI バージョン追従手順（.NET LTS の 3 年サイクル追従）を Runbook 化（NFR-C-NOP-003）

## 関連

- ADR-TIER3-001（BFF パターン）— Native は BFF 不要、SDK + tier1 直接アクセス
- ADR-TIER3-002（SPA + BFF）— Web 側の構成
- ADR-TIER1-001（Go + Rust hybrid）— tier2 .NET / Go と整合
- ADR-DEV-001（Paved Road）— MAUI 雛形を Backstage Software Template 化
- ADR-MIG-001（.NET Framework サイドカー）— Legacy .NET Framework との関係（別領域）
- IMP-DIR-INFRA-* — `src/tier3/native/` 配置

## 参考文献

- .NET MAUI 公式: dotnet.microsoft.com/apps/maui
- .NET LTS Release Schedule: dotnet.microsoft.com/platform/support/policy/dotnet-core
- Xamarin.Forms から .NET MAUI への移行ガイド: learn.microsoft.com/dotnet/maui/migration/
