# infra コーディングポリシー

全軸共通ルールは `src/CLAUDE.md` を参照。本ファイルは infra 固有の制約のみ記述する。

## 配置・構成

- **GitOps**: Argo CD（state は git のみが真）
- **IaC**: Helm / Kustomize / Tilt の 4 レイヤ
- **lock.yaml 配置先**: `src/infra/lock/`（手書き禁止）
- **Helm**: `src/infra/helm/k1s0/`
- **Backstage Templates**: `src/infra/backstage/templates/`
- **eBPF**: `src/infra/ebpf/`（C/Rust）

設計パターン・モジュール構成の詳細は `docs/03_概要設計/05_infra設計方針/README.md` を単一の真とする。

## コーディング制約

### GitOps 強制

- GUI（Argo CD UI / `kubectl apply` 直接実行）での state 変更禁止 → git PR のみ
- `topology_class.lock.yaml` の状態変更は GitOps + cosign signed commit で記録

### Kyverno policy の帰属

- Kyverno admission policy（25+ policy）は `src/security/kyverno/policies/` が管理
- `src/infra/` 配下に直接 Kyverno YAML を置かない（security 軸の責務）

### シークレット・鍵管理

- cosign 未署名 image の deploy 禁止（Harbor admission policy が enforce）
- Helm template / Kubernetes manifest に secret-like value を平文で書くことを禁止（→ OpenBao Transit）
- CI runner に cosign signing key を長期保持禁止（OpenBao Transit sign-on-demand）

### policy 検証

- PR 段: Conftest Rego で OPA policy check（Required Status Check）
- deploy 段: Kyverno admission（runtime enforcement）

### eBPF

- eBPF コードは `src/infra/ebpf/`（C/Rust）に配置
- eBPF probe の安全性は `src/formal/kani/harness/` の bounded model check と bind

## 関連参照

- `docs/03_概要設計/05_infra設計方針/README.md` — 設計パターン
- `docs/04_詳細設計/02_強制機構/04_infra強制機構.md` — CI fail 条件の詳細
- `docs/04_詳細設計/01_適合仕様/12_クラスタ位相適合仕様.md` — topology_class
- `docs/04_詳細設計/01_適合仕様/13_時刻整合適合仕様.md` — 時刻整合
