# 40. tier3 レイアウト

本章は tier3 層（`src/tier3/`）の物理配置を確定する。tier3 はエンドアプリ層で、Web（React + TypeScript）・Native（.NET MAUI）・BFF（Backend For Frontend）・Legacy ラッパー（.NET Framework）の 4 カテゴリで構成される。

## 本章の位置付け

tier3 は顧客（JTC 業務担当者）が直接触る UI / UX の実装層である。tier2 のドメインサービスを消費し、SDK 経由で tier1 公開 API も直接呼び出せる。

## 構成

- [01_tier3全体配置.md](01_tier3全体配置.md) — src/tier3/ の全体像
- [02_web_pnpm_workspace配置.md](02_web_pnpm_workspace配置.md) — React + TypeScript pnpm workspace
- [03_maui_native配置.md](03_maui_native配置.md) — .NET MAUI
- [04_bff配置.md](04_bff配置.md) — Backend For Frontend
- [05_レガシーラップ配置.md](05_レガシーラップ配置.md) — .NET Framework ラッパー

## 本章で採番する IMP-DIR ID

- IMP-DIR-T3-056（tier3 全体配置）
- IMP-DIR-T3-057（web pnpm workspace 配置）
- IMP-DIR-T3-058（maui native 配置）
- IMP-DIR-T3-059（bff 配置）
- IMP-DIR-T3-060（legacy-wrap 配置）

## 対応 ADR / 概要設計

- ADR-TIER1-003（内部言語不可視）
- ADR-MIG-001（.NET Framework sidecar）

## 関連図

- [img/tier3全体構成.drawio](img/tier3全体構成.drawio)
