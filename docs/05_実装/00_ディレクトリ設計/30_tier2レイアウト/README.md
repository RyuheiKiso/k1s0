# 30. tier2 レイアウト

本章は tier2 層（`src/tier2/`）の物理配置を確定する。tier2 はドメイン共通業務ロジック層で、C# .NET と Go の両言語でサービスを実装する。リリース時点 時点では雛形テンプレートのみ、運用蓄積後で 採用側組織の業務の実稼働サービスを追加する。

## 本章の位置付け

tier1（基盤）と tier3（エンドアプリ）の中間層として、以下の役割を担う。

- 業務ドメインの共通ロジック（承認フロー・帳票生成・税計算など）の提供
- tier1 公開 API を SDK 経由でラップし、業務文脈を加えたサービスとして tier3 に提供
- .NET 資産の活用（採用側組織の既存 .NET 資産を段階的に移行する受け皿）

## 構成

- [01_tier2全体配置.md](01_tier2全体配置.md) — src/tier2/ の全体像
- [02_dotnet_solution配置.md](02_dotnet_solution配置.md) — .NET ソリューション構成
- [03_go_services配置.md](03_go_services配置.md) — Go サービス構成
- [04_サービス単位の内部構造.md](04_サービス単位の内部構造.md) — Onion Architecture
- [05_テンプレート配置.md](05_テンプレート配置.md) — 雛形 CLI 参照テンプレ
- [06_依存管理.md](06_依存管理.md) — NuGet / go.mod 方針

## 本章で採番する IMP-DIR ID

- IMP-DIR-T2-041（tier2 全体配置）
- IMP-DIR-T2-042（dotnet solution 配置）
- IMP-DIR-T2-043（go services 配置）
- IMP-DIR-T2-044（サービス内部 Onion Architecture）
- IMP-DIR-T2-045（テンプレート配置）
- IMP-DIR-T2-046（依存管理）

## 対応 ADR / 概要設計

- ADR-TIER1-003（内部言語不可視）
- DS-SW-COMP-019（採用後の運用拡大時 再評価条件）

## 関連図

- `img/tier2サービス内部構造.drawio`（後続作業で作成予定）
