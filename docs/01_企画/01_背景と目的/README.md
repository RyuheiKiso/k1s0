---
id: plan.background_purpose
axis: overview
phase: plan
kind: index
status: draft
depends_on: []
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# 背景と目的

## 一文方針
- 本企画は、業界 pack 並立 + 19 軸同型構造 + 5 階層論 + L1+ 単一深耕 + 移行コミットメントの哲学に基づき、業務エンジニアが業務不変条件を踏み抜く経路を構造で塞ぎ、ジュニア級 tier3 担当でも事故が起きにくい業務プラットフォームを構築する。

## 課題認識
- 既存業務プラットフォームの問題点:
  - 業務不変条件（テナント越境 / 楽観 lock / atomic 三表書込 / Domain Event emit）の実装漏れがあらゆる layer で発生
  - OSS のライセンス変更（Vault / Redis / Confluent）への対応で業務コードが破綻
  - 業界拡張（業界 pack 追加）時に業界横断層が業界固有概念で汚染
  - tier3 ジュニア級が業務不変条件を踏み抜く経路が構造的に塞がれていない
  - SLO / 監査 / 認可 が文章運用で空文化
  - 形式検証 / 物理 enforcement が文章で「することにする」レベルに留まる

## 目的
- 上記課題に対し、19 軸同型構造（tier1 / tier2 / tier3 / infra / data / security / ops / client / test / formal + meta-axis）+ 5 階層論（infra / data / tier1 / tier2 / tier3）+ L1+ 単一深耕 + 移行コミットメントで構造的に解決する
- 業務不変条件は CI / lint / 公開 API snapshot / Kyverno admission policy / HSM zeroize / 外部公証 attestation の多重防御で物理 enforce する
- 業界 pack 並立は命名禁則 + 依存方向 + 第二業界 stub conformance の 3 種機械的担保で構造的に enforce する
- ジュニア級 tier3 担当でも業務不変条件を踏み抜けない構造を提供する

## 1.0.0 ship スコープ
- 業界 pack: 製造業のみ（ただし業界並立構造は day-1 から有効）
- アプリケーション形態: Web SPA / デスクトップ exe / レガシー .NET Framework 4.6.2+ の 3 形態
- 19 軸全てが完成、`release_gate.lock.yaml` の全 cell green、cosign signed tag が物理 prerequisite

## 至高路線における立ち位置
- CLAUDE.md ポリシー「運用コスト度外視 / 1.0.0 で完璧 / 段階的 release 禁止 / 機能削減なし」を全軸で物理 enforce
- 「至高を目指す判断」を採り、業務品質 / セキュリティ / 監査 / 認可 / 形式検証を文章運用ではなく物理機構で担保する

## 関連参照
- [提供する価値や体験](../02_提供する価値や体験/README.md)
- [競合との差別化](../03_競合との差別化/README.md)
- [法務確認](../04_法務確認/README.md)
- [ターゲットと利用シナリオ](../05_ターゲットと利用シナリオ/README.md)
- [業界 pack 戦略](../07_業界pack戦略/README.md)
- [03_概要設計 アーキテクチャ概観](../../03_概要設計/01_アーキテクチャ概観/README.md)
