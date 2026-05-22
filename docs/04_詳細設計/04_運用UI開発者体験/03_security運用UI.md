---
id: detail.security.ops_dx
axis: security
phase: detail
kind: ops_dx
status: draft
depends_on:
  - arch.security.security_index
  - arch.security.threat_model_policy
  - arch.security.incident_response_policy
  - detail.security.security_enforcement
covered_by:
  defense_in_depth_layers: [D, E]
  proof_classes: []
trace:
  fr_ids:
  - FR-security-004

---

# security 運用 UI と開発者体験

## 一文方針
- 運用 UI は Backstage（infra 13 と共有）+ Perses（audit query / SLO / drill 状態 dashboard）+ 内部 custom plugin（threat_model coverage / mitigation binding visualization）の 3 OSS 経由のみで提供し、開発者は IDE 拡張 + pre-commit hook + Testcontainers で同等の security check を local で完結できる。

## 運用 UI（operator 視点）

### Backstage TechDocs
- `threat_model.lock.yaml` の visualization（actor × capability × surface × asset の heatmap、各 cell の mitigation 列挙）
- PII class taxonomy 一覧 + 列レベル mapping
- secret class lifecycle / rotation status
- incident playbook（class 別、6 phase 全 step）
- `drill_progress.lock.yaml` の cadence 状況

### Perses dashboard
- audit_event ingestion lag（ClickHouse query）
- SLO 状況（patch_window / drill / audit_chain health）
- admission policy decision rate（allow / deny / audit）
- CVE open critical count（severity 別）
- drill `last_green_at` 一覧

### 自製 escalation engine（Argo Workflows + Mattermost）
- 班 rotation 状況、escalation history、active incident の page log
- postmortem PR の merge 状況（GitHub integration）

### 内部 custom plugin（Backstage）
- `threat_model_coverage_visualizer`: 未 mitigation cell を highlight、PR diff で coverage gap before/after 比較
- `mitigation_pointer_navigator`: mitigation pointer から物理機構（policy yaml / rule / 設定）への deep link

## 開発者 UI（developer 視点）

### IDE 拡張
- VSCode 拡張（自社 / OSS、v1 では README + 設定 guide のみ提供、v2 で自社拡張検討）:
    - schema 編集時に PII class 宣言 lint
    - secret-like pattern 警告（gitleaks integration）
    - threat_model coverage hint（編集中 column が登録されている threat_id を表示）

### pre-commit hook（git pre-commit、自社 boilerplate 提供）
- gitleaks（secret scan）
- trufflehog（secret scan、補強）
- Semgrep（SAST、軽量 ruleset）
- schema lint（pii_class 宣言 check）
- cargo deny / govulncheck / pnpm audit（dep audit、変更があった package のみ）

### PR 上の自動 comment
- PR で touched された PII column / threat_class の変化点 summary
- mitigation pointer の bind 状況（before/after）
- drift 警告（policy が手書き、build artifact と乖離）

### Testcontainers ベース local
- 全 Kyverno admission policy を local cluster（k3d / kind）で同一 build artifact から適用、local development で本番と同一 admission decision を再現
- OpenBao dev mode で secret 配信 path を local mock、long-lived secret に汚染されない経路を確保
- Falco ruleset を local syscall に対して dry-run 適用、開発 commit 前に detection rule の動作確認

## 自動化の規律
- 全 dashboard / TechDocs / Perses panel は build artifact（`dashboards/` 配下 yaml）から生成、UI 上の手書き編集は drift として CI 検出 + Argo CD self-heal
- threat_model 編集 PR は CODEOWNERS で security 層 reviewer 必須、自由 merge 経路を持たない
- PII class 宣言の変更 PR は security 層 + 業務 owner の二人 reviewer 必須

## アクセシビリティ
- 全 dashboard の色指定は drawio 規約 / figure-layer-convention と整合（colorblind-safe palette）、本層自身の audit dashboard も同 palette
- UI 操作の全認証は OIDC + WebAuthn 必須、password-only 経路ゼロ

## 採用しない方針
- 商用 dashboard SaaS（DataDog / New Relic）: 採用しない
- UI 上の policy 直接編集（Kyverno UI でのトポロジ操作）: 禁止、PR 経由のみ
- 個人 development 環境への production secret 同期: 禁止
- `threat_model.lock.yaml` の手書き: 禁止
- dashboard の personalization: 禁止
- 開発者の独自 secret store 持ち込み: 禁止

## audit / 不可逆性
- 全 operator action（break-glass / approval / override）は audit_event subject に `v1_security_<action>` fact emit
- ChatOps bot は dual reviewer signoff を要する operation を `cosign signed approval` で物理 enforce

## 関連参照
- [security 設計方針 index](../../03_概要設計/07_security設計方針/README.md)
- [脅威モデル適合仕様](../01_適合仕様/15_脅威モデル適合仕様.md)
- [security 強制機構](../02_強制機構/06_security強制機構.md)
- [インシデント対応方針](../../03_概要設計/07_security設計方針/07_インシデント対応方針.md)
