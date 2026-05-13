---
id: env.overview.environment_setup_index
axis: overview
phase: env_setup
kind: index
status: draft
depends_on:
  - plan.development_team_structure
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# 環境構築 index

## 一文方針

- 本フォルダは役割ごとに分離された環境構築手順を集約する。CI 強制機構（docs_lint / drawio-lint / lock_yaml 生成器）を手元で完全再現できる状態を各ロールの受け入れ基準とし、push 前に全 green を確認することを規律とする。

## 至高路線における立ち位置

- 環境構築は実装前提であり、段階的構築は許容しない。clone 直後に全 lint が green になる状態を「環境が整った」とする唯一の定義とする。
- 各ロールの手順書は CI job との 1:1 trace を持ち、機械的に green を確認できる検収基準を明示する。
- ロール間の diff（k1s0 作者 vs tier1 vs …）は手順書の構成上の差として分離し、共通前提は 05_環境構築/README.md（本ファイル）が宣言する。

## 共通前提（全ロール共通）

- **OS**: WSL2 + Ubuntu 22.04 LTS 以上（Windows 11 host）
- **git**: 2.40 以上
- **bash**: 5.0 以上 + GNU coreutils（find / awk / grep / sort / uniq / sed / mktemp）
- **clone 後の初回検証コマンド**: `bash tools/docs_lint/run_lint.sh`

## 配下サブフォルダ

| 番号 | フォルダ | 対象ロール |
|---|---|---|
| 00 | [00_k1s0作者/](00_k1s0作者/README.md) | k1s0 作者（toolchain authoring + 軸登録 governance + release 署名） |
| 01 | [01_tier1/](01_tier1/README.md) | tier1 軸エンジニア（Rust / Go / C# / TypeScript / proto / Buf） |
| 02 | [02_tier2/](02_tier2/README.md) | tier2 軸エンジニア（4 言語 + 業界 pack / 業務資産 / 決定表） |
| 03 | [03_tier3/](03_tier3/README.md) | tier3 軸エンジニア（TypeScript+React / WPF / WinForms / Tauri） |
| 04 | [04_infra/](04_infra/README.md) | infra 軸エンジニア（k8s / Argo CD / Istio / Calico / OpenBao） |
| 05 | [05_data/](05_data/README.md) | data 軸エンジニア（PostgreSQL / Kafka / ClickHouse / Apicurio / Rook+Ceph） |
| 06 | [06_security/](06_security/README.md) | security 軸エンジニア（KEK shamir / OAuth 2.1 / OWASP / penetration testing） |
| 07 | [07_ops/](07_ops/README.md) | ops 軸エンジニア（SRE / SLO / Argo Rollouts / Backstage / Grafana） |
| 08 | [08_client/](08_client/README.md) | client 軸エンジニア（SDK 配布 / 5 distribution_class / browser matrix） |
| 09 | [09_test/](09_test/README.md) | test 軸エンジニア（Pact / Playwright / Litmus / mutation / property-based） |
| 10 | [10_formal/](10_formal/README.md) | formal 軸エンジニア（TLA+ / Apalache / Stainless / Dafny / Lean / Kani / CBMC） |
| 11 | [11_プラットフォーム運営者/](11_プラットフォーム運営者/README.md) | プラットフォーム運営者（KEK custodian / on-call / break-glass / audit hash chain） |
| 12 | [12_業務管理者/](12_業務管理者/README.md) | 業務管理者（tenant マスタ / 決定表 / 監査検索 / partner 連携） |

## 上位フェーズとの bind

### 上位フェーズへの依存
- [08_開発体制](../01_企画/08_開発体制/README.md): 各ロール定義の source of truth。本フォルダのサブディレクトリ名と 08_開発体制 のロール一覧が 1:1 対応する

## 横断軸との bind

なし（環境構築は全軸に先立つ前提層であり、特定軸への bind を持たない）

## 読み筋

- 初読: 本ファイル → 自分のロールのサブフォルダ README → 01_責務とスコープ → 02 以降を順に実行
- 実装前チェック: 本ファイル共通前提 → 自ロール README の検収確認
- CI green が取れない場合: 自ロール `12_CI完全再現` を参照

## 関連参照

- [docs/ 全フェーズ index](../README.md)
- [01_企画/08_開発体制/README.md](../01_企画/08_開発体制/README.md)
- [docs/00_format/README.md](../00_format/README.md)
