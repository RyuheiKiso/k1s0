# security コーディングポリシー

全軸共通ルールは `src/CLAUDE.md` を参照。本ファイルは security 固有の制約のみ記述する。

## 配置・構成

- **Kyverno**: `src/security/kyverno/policies/`（25+ policy）
- **lock.yaml 配置先**: `src/security/lock/`（手書き禁止）
- **audit_event**: append-only WORM（git 管理外、Ceph RGW に保存）

設計パターン・モジュール構成の詳細は `docs/03_概要設計/07_security設計方針/README.md` を単一の真とする。

## コーディング制約

### audit_event

- `audit_event` テーブルの UPDATE / DELETE を発行するコード禁止（append-only WORM）
- `audit_event` の INSERT は atomic 三表書込の一部として tier2 が必ず発行する

### シークレット管理

- secret-like value を ConfigMap / Deployment env / Helm values に平文で書くことを禁止（→ OpenBao Transit）
- CI runner に cosign signing key を長期保持禁止（OpenBao Transit sign-on-demand）

### PII

- PII pattern 検出時: block + redact 必須（Vector pipeline で enforce）
- PII column への操作は envelope encryption + RLS + audit_event emit を同時に実装

### supply chain

- cosign 未署名 image / SBOM 未 attach → CI fail
- Semgrep ruleset は build artifact（手書き禁止）
- Trivy ignore は `expires_at` 全 entry 必須（期限切れ例外禁止）

### threat model

- `threat_model.lock.yaml`（625 cell）/ `mitigation_bindings.lock.yaml` は build artifact（手書き禁止）
- 新しい attack surface を実装する場合、`src/security/threat_model/catalog/cells.yaml` に対応 cell を追加してから実装する

### Kyverno policy

- policy YAML は `src/security/kyverno/policies/` に配置
- policy は build artifact として生成する（Rego 手書きは禁止ではないが、schema validate 必須）

## 関連参照

- `docs/03_概要設計/07_security設計方針/README.md` — 設計パターン
- `docs/04_詳細設計/02_強制機構/06_security強制機構.md` — CI fail 条件の詳細
- `docs/04_詳細設計/01_適合仕様/15_脅威モデル適合仕様.md` — threat model
- `docs/04_詳細設計/01_適合仕様/16_build_provenance適合仕様.md` — build provenance
