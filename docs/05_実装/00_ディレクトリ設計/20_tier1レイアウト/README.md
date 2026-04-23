# 20. tier1 レイアウト

本章は tier1 層（`src/tier1/` + `src/contracts/` + `src/sdk/`）の物理配置を確定する。概要設計 `DS-SW-COMP-120 〜 DS-SW-COMP-134`（[../../../04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/06_パッケージ構成_Rust_Go.md](../../../04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/06_パッケージ構成_Rust_Go.md)）の論理配置を、ADR-DIR-001（contracts 昇格）と ADR-DIR-002（infra 分離）を反映した形で再整理する。

## 本章の位置付け

Phase 0 稟議承認時点の確定範囲のうち、最も実装コードに近い層の物理配置を規定する。Phase 1a の MVP-0 で tier1 Go + Rust 全 6 Pod が動作する状態まで持ち込むため、本章のレイアウトは Phase 0 時点で詳細まで固まっている必要がある。

## 構成

- [01_tier1全体配置.md](01_tier1全体配置.md) — src/tier1/ の全体像と DS-SW-COMP-120 改訂内容
- [02_contracts配置.md](02_contracts配置.md) — src/contracts/ の昇格後の配置
- [03_go_module配置.md](03_go_module配置.md) — src/tier1/go/ の Go module レイアウト
- [04_rust_workspace配置.md](04_rust_workspace配置.md) — src/tier1/rust/ の Cargo workspace レイアウト
- [05_SDK配置.md](05_SDK配置.md) — src/sdk/ の 4 言語独立配置
- [06_生成コードの扱い.md](06_生成コードの扱い.md) — buf generate の commit 方針

## 本章で採番する IMP-DIR ID

- IMP-DIR-T1-021（tier1 全体配置）
- IMP-DIR-T1-022（src/contracts/ 配置）
- IMP-DIR-T1-023（src/tier1/go/ 配置）
- IMP-DIR-T1-024（src/tier1/rust/ 配置）
- IMP-DIR-T1-025（src/sdk/ 配置）
- IMP-DIR-T1-026（Protobuf 生成コード配置）

## 本章の対応 ADR / 概要設計

- ADR-DIR-001 / ADR-DIR-002
- ADR-TIER1-001 / ADR-TIER1-002 / ADR-TIER1-003
- DS-SW-COMP-120（改訂後）/ DS-SW-COMP-121（改訂後）/ DS-SW-COMP-122 / DS-SW-COMP-123 / DS-SW-COMP-124 / DS-SW-COMP-125 / DS-SW-COMP-126 / DS-SW-COMP-129 / DS-SW-COMP-130 / DS-SW-COMP-131 / DS-SW-COMP-132

## 関連図

- [img/tier1_Go_Rust_SDK関係.drawio](img/tier1_Go_Rust_SDK関係.drawio)
- [img/contracts生成フロー.drawio](img/contracts生成フロー.drawio)
