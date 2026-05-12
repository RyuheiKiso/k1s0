---
id: plan.tier1.scenario_sbom_cve_triage
axis: tier1
phase: plan
kind: plan_doc
status: draft
depends_on:
  - arch.tier1.tier1_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers:
    - B
    - E
  proof_classes: []
---

# SBOM / CVE 月次トリアージ

## 一文方針

tier1 担当者が月次 SBOM レビューと CVE トリアージを実施し、L1+ / L2* / L3 各 OSS の緊急 bump 判定と対応優先度を確定する。

> 朝 9 時、本社 IT 室の tier1 担当者（シニア級）が security alert dashboard を確認し、月次の SBOM review cadence が到来しており、OpenSSL の CVSS 9.8 CVE が未対応のまま残っていることに気付く。手元には `sbom_catalog.lock.yaml` と Grype スキャン結果、Mattermost 越しに security 担当者と dual reviewer 2 名がいる。

## Trigger（発火条件）

月次の SBOM review cadence 到来、または CERT / NVD / OSV.dev から重大 CVE が公開された時。

## 想定頻度 / 典型きっかけ

- 想定頻度: 月次定期（緊急 CVE はイベント駆動）
- 典型きっかけ: 「月次 review で OpenSSL の CVE-2024-XXXX が未対応のまま残っており、CVSS スコア 9.8 の緊急度評価が必要になった」「Strimzi の依存 lib に CVE が公開され L1+ 継続評価が必要になった」

## 主役 / 関与者

- 主役: tier1 担当者（シニア級）
- 関与: security 担当者（CVE 評価の dual review）
- 承認: dual reviewer（tier1 担当者 2 名、変更 PR の author 不可）

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|---|---|---|---|---|
| 主役（tier1）| シニア | 本社 IT 室 | security alert dashboard | SBOM diff 取得・CVE 分類・bump PR 作成・lock.yaml 記録 |
| 関与（security 担当者）| 中堅 | 本社 IT 室 | security alert dashboard | CVE 評価 dual review・影響範囲確認 |
| 承認（dual reviewer A）| シニア | 本社 IT 室 / リモート | GitHub PR list | lock.yaml レビュー・sign-off |
| 承認（dual reviewer B）| シニア | 本社 IT 室 / リモート | GitHub PR list | lock.yaml レビュー・sign-off |

## 前提

- Syft（SBOM 生成）+ Grype（脆弱性スキャン）が CI に組込済み
- Harbor mirror から pull した image の SBOM が `sbom_catalog.lock.yaml` に記録済み

## 流れ

1. SBOM の月次 diff を取得し新規 component と更新 component を抽出する
   - `grype sbom:./sbom_catalog.lock.yaml` で脆弱性リストを生成
   - CVSS スコア 7.0 以上を高優先度としてフィルタ
   - EPSS スコア（悪用確率）も確認し優先度を補正
2. 各 CVE を 4 分類する: 直接依存 / 間接依存 / devOnly / ベースイメージ由来
3. 直接依存 CVE は L1+ / L2* / L3 の Lv 別に対応方針を決定する
   - L1+ の CVE: 移行 toolchain 起動を検討（EOL 相当の場合）または patch 版 bump で対応
   - L2* の CVE: 同族 OSS の対応状況を確認し conformance test を再実行
   - L3 の CVE: API 互換を保ちつつ代替実装に切替可能か評価
4. CVSS 9.0 以上の CVE は緊急 bump PR を 24h 以内に作成する
   - `cargo update <crate>` / `dotnet package update` / `go get <module>@<version>` / `npm update <package>` を 4 言語で実施
   - Testcontainers conformance test が全 green になることを確認してから PR 作成
5. ベースイメージ由来の CVE は Harbor mirror の base image を更新する（infra 担当者と連携）
6. トリアージ結果を `sbom_triage.lock.yaml` に記録する（CVE ID / CVSS / 対応方針 / 期限 / 担当者）
7. security 担当者の dual review + sign-off を取得してから lock.yaml を merge
8. CVSS 9.0 以上で対応が 24h 以内に完了しない場合は Backstage ticket を起票して進捗を追跡する

## 業界 9 業務との紐付け

全 9 業務に共通基盤として影響（tier1 Library / Server は全業務の通信・認証・観測の基盤を担うため）。特に影響度が高い 2 業務:

- **警報配信**: 暗号化 / 認証ライブラリ（OpenSSL 等）の未対応 CVE は警報配信チャンネルの完全性を損ない、改竄や傍受のリスクを招く。緊急 bump 対応が警報配信の信頼性を直接支える。
- **FA 生産指示・設備操作**: 設備制御 API に利用される依存ライブラリの脆弱性は、工場の制御系への不正操作経路になりうるため、最優先 CVE 対応が安全稼働の前提条件となる。

## 関連適合仕様 / 関連 OSS

- 関連適合仕様
  - [OSS ライフサイクル適合仕様](../../../04_詳細設計/01_適合仕様/08_OSSライフサイクル適合仕様.md)
  - [tier1 強制機構](../../../04_詳細設計/02_強制機構/01_tier1強制機構.md)
- 関連 OSS
  - Syft（SBOM 生成、Anchore / Apache 2.0）
  - Grype（脆弱性スキャン、Anchore / Apache 2.0）
  - Harbor（mirror）
  - Cosign（署名検証）

## 期待結果 / 観測指標

- `sbom_triage.lock.yaml` 更新完了
- CVSS 9.0 以上の CVE は 24h 以内に bump PR 作成
- security 担当者 dual sign-off 取得
- CI（Testcontainers conformance test）green

## 失敗時の挙動 / escalation

- **CVSS 9.0 以上の CVE が 24h 以内に対応不能**: security 担当者に Mattermost `#security-incident` で即時通報（**SLA: 1h 以内**に escalate）。Backstage runbook `critical-cve-response` を起動。1.0.0 ship blocker 認定の可能性。
- **conformance test fail（bump 後）**: 旧バージョンに rollback し原因調査。旧バージョンでのリスク許容を security 担当者と協議（**SLA: 24h 以内**に方針確定）。**postmortem 期限: 3 営業日以内**。
- **ベースイメージ更新で既存 workload が起動不能**: infra 担当者に Mattermost `#infra-incident` で即時連絡（**SLA: 30 分以内**）。Backstage runbook `base-image-rollback` を参照。

## 関連参照

- [tier1 担当者シナリオ index](./README.md) — tier1 担当者シナリオ全体の構成と担当者プロフィール
- [OSS ライフサイクルイベント対応](./02_OSSライフサイクルイベント対応.md) — CVE が EOL 相当の場合の移行 toolchain シナリオ
- [supply chain 障害対応](./09_supply_chain障害対応.md) — 緊急 CVE が supply chain 全体に影響する場合のシナリオ
- [tier1 設計方針](../../../03_概要設計/02_tier1設計方針/README.md) — L1+/L2*/L3 の 3 抽象レベル定義。本シナリオの bump 判定の SoT
