---
id: plan.development_team_structure
axis: overview
phase: plan
kind: index
status: draft
depends_on:
  - plan.background_purpose
  - plan.value_experience
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# 開発体制

## 一文方針
- 5 階層論（infra / data / tier1 / tier2 / tier3）に対応する 5 階層エンジニア体制 + formal / security / ops / test の 4 横断軸専任 + プラットフォーム運営者 + 業務管理者の混合組織。tier3 担当はジュニア級、tier1 / formal / security はシニア級、cross-cutting は senior + domain expert の dual structure。

## 階層別エンジニア要件

### infra 軸エンジニア（シニア級）
- **必須スキル**: Kubernetes / Argo CD / Istio / Calico BGP / Envoy / OpenBao / Cosign / Litmus
- **責務**: cluster 構築 / 5 topology_class / 5 clock_integrity_class / 25+ Kyverno admission policy
- **想定人数**: 3-5 名

### data 軸エンジニア（シニア級）
- **必須スキル**: PostgreSQL / CloudNativePG / Kafka / Strimzi / ClickHouse / Apicurio Registry / Valkey / Rook+Ceph
- **責務**: 5 preservation_class / 4 layer 保全 / restore_drill / migration toolchain
- **想定人数**: 3-5 名

### tier1 軸エンジニア（シニア級）
- **必須スキル**: Rust（主言語）/ Go（Operator）/ C# / TypeScript / proto / Buf / OSS lifecycle
- **責務**: Server 系 5 分類 / Library 3 抽象レベル × 17 機能カテゴリ / Companion 役割 A/B / 9 適合仕様
- **想定人数**: 5-8 名

### tier2 軸エンジニア（中堅級）
- **必須スキル**: 4 言語（Rust / C# / Go / TypeScript）+ ドメイン業務 / 業界 pack
- **責務**: 業務資産（Domain Event / Workflow / 決定表 / 業務マスタ / DB schema）所有 / 拡張点定義 / atomic 三表書込
- **想定人数**: 5-10 名（業界 pack ごとに増減）

### tier3 軸エンジニア（ジュニア級）
- **必須スキル**: TypeScript + React / C# WPF / .NET Framework + WinForms / Tauri
- **責務**: 個別業務 UI / 業務シナリオ / フォームレイアウト / 業務エラー UX / 帳票レイアウト
- **想定人数**: 業界 pack × 業務領域 × tenant 数で増減

## k1s0 作者（最高権限）

### k1s0 作者
- **必須スキル**: Python 3.12 / bash / draw.io Desktop / PyYAML / cosign / YAML schema / git / Claude Code
- **責務**: toolchain 開発（docs_lint / lock_yaml 生成器 / drawio 作図ツール）/ 19 軸 registry governance（cap v1=20 維持）/ cosign signed tag による release 署名 / frontmatter schema evolve / CI pipeline 設計と改訂
- **想定人数**: 1-2 名（L1+ 単一深耕による代替不可能性を許容、代替不在は 1.0.0 ship blocker にしない）
- **非責務**: tier1-3 / infra / data / security / ops / client / test / formal の実装実務（それぞれ専任ロール）/ KEK shamir custodian（プラットフォーム運営者ロール）
- **体制上の注意**: LLM 補助による toolchain 生成は許容するが、spec merge / schema 変更 / cosign 署名は人間 dual sign-off が必須

## 横断軸専任エンジニア

### formal 軸（メタ専任、シニア級）
- **必須スキル**: TLA+ / Apalache / Stainless / Dafny / Lean 4 / Kani / CBMC / formal methods
- **責務**: 5 proof_class / 95 cell coverage / counter-example closure
- **想定人数**: 2-3 名

### security 軸（メタ専任、シニア級）
- **必須スキル**: 5⁴ cell threat model / KEK shamir / OAuth 2.1 + DPoP / OWASP / penetration testing
- **責務**: 脅威モデル / 8 secret class / 5 build_provenance_class / incident response
- **想定人数**: 2-3 名

### ops 軸（メタ専任、シニア級）
- **必須スキル**: SRE 50% rule / SLO / MWMBR alerting / Argo Rollouts / Backstage TechDocs / Tekton
- **責務**: 5 signal_class × 5 phase ops loop / on-call rotation / postmortem / runbook
- **想定人数**: 3-5 名

### test 軸（メタ専任、シニア級）
- **必須スキル**: 5 verification_class / Pact / Playwright / Litmus / mutation testing / property based test
- **責務**: 18 軸 × 5 verification_class = 90 cell coverage / regression corpus / golden snapshot
- **想定人数**: 2-3 名

## プラットフォーム運営者
- **必須スキル**: cluster 運用 + KEK shamir custodian + ops on-call + audit hash chain + 外部公証 attestation
- **責務**: production cluster 運用 / KEK shamir M-of-N ceremony / break-glass execution
- **想定人数**: 5-8 名（24h on-call 3 階層 escalation）

## 業務管理者
- **必須スキル**: 業界知識 + tier2 admin API（Backstage プラグイン）+ マスタ管理 / 監査検索
- **責務**: テナント別マスタ / 決定表編集 / 監査検索 / 緊急対応 / partner 連携設定
- **想定人数**: テナント単位

## 開発体制の至高路線
- **L1+ 単一深耕**: 各 OSS に対して深耕担当（L1+ primary）を割当、L2\* 同族保証は 2 OSS 担当
- **dual reviewer sign-off**: 全 PR は 2 reviewer 必須（LLM 単独 sign-off 禁止）
- **rotation gap zero**: ops on-call rotation の gap は 60 分 cap で auto cover
- **fatigue budget**: SRE 50% rule + 個人別 toil ≤ 50%

## tier3 ジュニア級のオンボーディング
- Backstage Software Template で tier3 リポジトリ生成 → 13 層強制機構 + contract test 自動組込
- tier3 担当は業務 UI に集中、業務不変条件は構造で保護
- contract test の skip / 改変は CI で merge 阻止

## 採用しない体制
- 1 階層エンジニア（フルスタック）の量産（質を選ぶ）
- LLM 単独 sign-off（必ず人間 dual sign-off）
- on-call 24h 連続当番（fatigue budget）
- 商用 incident management SaaS への依存（自製 escalation engine）

## 関連参照
- [背景と目的](../01_背景と目的/README.md)
- [提供する価値や体験](../02_提供する価値や体験/README.md)
- [層別エンジニア要件（要件定義）](../../02_要件定義/05_開発体制要件/01_層別エンジニア要件.md)
