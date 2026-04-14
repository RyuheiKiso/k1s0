# Architecture Decision Records (ADR)

k1s0 における確定済みの技術決定を記録する。
フォーマットは [`../../00_format/ADR-xxxx.md`](../../00_format/ADR-xxxx.md) に準拠する。

---

## ADR 一覧

| 番号 | タイトル | ステータス | 決定日 |
|---|---|---|---|
| [ADR-0001](./ADR-0001-use-rust-edition-2024.md) | Rust Edition 2024 の採用 | Accepted | 2026-04-14 |
| [ADR-0002](./ADR-0002-tier1-language-hybrid.md) | tier1 の内部言語ハイブリッド (ファサード = Go / 自作領域 = Rust) | Accepted | 2026-04-14 |
| [ADR-0003](./ADR-0003-kustomize-helm-strategy.md) | Kustomize + Helm の使い分け方針 | Accepted | 2026-04-14 |
| [ADR-0004](./ADR-0004-kubeadm-adoption.md) | Kubernetes ディストリビューションとして kubeadm を採用 | Accepted | 2026-04-14 |
| [ADR-0005](./ADR-0005-dapr-adoption-and-encapsulation.md) | Dapr の採用と tier2/tier3 への隠蔽 | Accepted | 2026-04-14 |
| [ADR-0006](./ADR-0006-keycloak-sso.md) | SSO 基盤として Keycloak を採用 | Accepted | 2026-04-14 |

---

## 運用ルール (要点)

詳細は [`../../00_format/ADR-xxxx.md`](../../00_format/ADR-xxxx.md) を参照。要点のみ再掲する。

- ファイル名は `ADR-NNNN-<短いタイトル>.md` (4 桁ゼロ埋め連番 + ケバブケース)。
- 1 つの ADR では 1 つの決定のみを扱う。
- **決定済み ADR は原則として書き換えない**。覆す場合は新規 ADR を起票し、旧 ADR のステータスを `Superseded by ADR-NNNN` に更新する。
- 新規追加時は本 README の一覧表にも追記する。
- 図表が必要な場合はアスキーアートを使用せず、`img/` 配下の drawio から出力した SVG を埋め込む。

---

## ステータスの定義

| ステータス | 説明 |
|---|---|
| `Proposed` | 提案中。レビューおよび議論の対象 |
| `Accepted` | 承認済み。実装・運用の指針となる |
| `Deprecated` | 廃止。新規採用は禁止だが、既存の記録として残す |
| `Superseded` | 別 ADR によって置き換えられた。置換先を `Superseded by ADR-NNNN` として明記する |
