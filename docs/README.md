---
id: meta.docs_index
axis: meta
phase: detail
kind: index
status: draft
depends_on: []
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# docs/ 全フェーズドキュメント

本ドキュメント体系は 4 フェーズ構造（企画 / 要件定義 / 概要設計 / 詳細設計）+ knowledge 補助層と 19 軸同型構造の cross-product で組まれる。

## フェーズ別ナビゲーション

| Phase | ディレクトリ | 役割 |
|---|---|---|
| 0 | [00_format/](00_format/) | テンプレート / 規約 / frontmatter schema / lint 規約 |
| 1 | [01_企画/](01_企画/README.md) | 背景 / 価値 / 競合 / 法務 / ターゲット / OSS 公開 / 業界 pack 戦略 / 開発体制 / 用語集 |
| 2 | [02_要件定義/](02_要件定義/README.md) | スコープ / 機能要件 / 非機能要件 / 技術選定 / 開発体制 / 制約と前提 |
| 3 | [03_概要設計/](03_概要設計/README.md) | アーキ概観 / 軸別設計方針 × 10 軸 / クロスカッティング設計 |
| 4 | [04_詳細設計/](04_詳細設計/README.md) | 適合仕様 / 強制機構 / クロス適合仕様 / 運用 UI 開発者体験 / lock.yaml 体系 |
| 5 | [05_環境構築/](05_環境構築/README.md) | 役割別環境構築手順（k1s0 作者 / tier1 / …）|

## 19 軸 matrix

| 軸 | 概要設計 | 詳細設計（適合仕様） | cross-cutting |
|---|---|---|---|
| tier1 | [02_tier1設計方針](03_概要設計/02_tier1設計方針/README.md) | [01_Bidi適合仕様](04_詳細設計/01_適合仕様/01_Bidi適合仕様.md) / [02_移行Pair適合仕様](04_詳細設計/01_適合仕様/02_移行Pair適合仕様.md) / [03_観測適合仕様](04_詳細設計/01_適合仕様/03_観測適合仕様.md) / [04_認証適合仕様](04_詳細設計/01_適合仕様/04_認証適合仕様.md) / [05_鍵管理適合仕様](04_詳細設計/01_適合仕様/05_鍵管理適合仕様.md) / [06_スキーマ進化適合仕様](04_詳細設計/01_適合仕様/06_スキーマ進化適合仕様.md) / [07_SLO適合仕様](04_詳細設計/01_適合仕様/07_SLO適合仕様.md) / [08_OSSライフサイクル適合仕様](04_詳細設計/01_適合仕様/08_OSSライフサイクル適合仕様.md) / [09_テナント容量適合仕様](04_詳細設計/01_適合仕様/09_テナント容量適合仕様.md) | [01_HTTP2_enforcement](04_詳細設計/03_クロスカッティング適合仕様/01_HTTP2_enforcement.md) / [02_KEK_shamir_distribution](04_詳細設計/03_クロスカッティング適合仕様/02_KEK_shamir_distribution.md) / [03_apicurio_gitops_sot](04_詳細設計/03_クロスカッティング適合仕様/03_apicurio_gitops_sot.md) / [13_dotnet8_connect_inhouse](04_詳細設計/03_クロスカッティング適合仕様/13_dotnet8_connect_inhouse.md) |
| tier2 | [03_tier2設計方針](03_概要設計/03_tier2設計方針/README.md) | [10_テナント分離適合仕様](04_詳細設計/01_適合仕様/10_テナント分離適合仕様.md) | [04_protoc_gen_go_fsm](04_詳細設計/03_クロスカッティング適合仕様/04_protoc_gen_go_fsm.md) / [05_SLO_protection_layers](04_詳細設計/03_クロスカッティング適合仕様/05_SLO_protection_layers.md) |
| tier3 | [04_tier3設計方針](03_概要設計/04_tier3設計方針/README.md) | [11_クライアント状態適合仕様](04_詳細設計/01_適合仕様/11_クライアント状態適合仕様.md) | [06_BFF_auth_edge](04_詳細設計/03_クロスカッティング適合仕様/06_BFF_auth_edge.md) / [07_Tauri_companion_sidecar](04_詳細設計/03_クロスカッティング適合仕様/07_Tauri_companion_sidecar.md) |
| infra | [05_infra設計方針](03_概要設計/05_infra設計方針/README.md) | [12_クラスタ位相適合仕様](04_詳細設計/01_適合仕様/12_クラスタ位相適合仕様.md) / [13_時刻整合適合仕様](04_詳細設計/01_適合仕様/13_時刻整合適合仕様.md) | [10_ops_edge_cluster](04_詳細設計/03_クロスカッティング適合仕様/10_ops_edge_cluster.md) |
| data | [06_data設計方針](03_概要設計/06_data設計方針/README.md) | [14_データ保全適合仕様](04_詳細設計/01_適合仕様/14_データ保全適合仕様.md) | [08_PII_dedicated_cluster](04_詳細設計/03_クロスカッティング適合仕様/08_PII_dedicated_cluster.md) |
| security | [07_security設計方針](03_概要設計/07_security設計方針/README.md) | [15_脅威モデル適合仕様](04_詳細設計/01_適合仕様/15_脅威モデル適合仕様.md) / [16_build_provenance適合仕様](04_詳細設計/01_適合仕様/16_build_provenance適合仕様.md) | [09_audit_ingest_gap_monitor](04_詳細設計/03_クロスカッティング適合仕様/09_audit_ingest_gap_monitor.md) |
| ops | [08_ops設計方針](03_概要設計/08_ops設計方針/README.md) | [17_運用ループ適合仕様](04_詳細設計/01_適合仕様/17_運用ループ適合仕様.md) | - |
| client | [09_client設計方針](03_概要設計/09_client設計方針/README.md) | [18_クライアントSDK配布適合仕様](04_詳細設計/01_適合仕様/18_クライアントSDK配布適合仕様.md) | [11_companion_otel_extension](04_詳細設計/03_クロスカッティング適合仕様/11_companion_otel_extension.md) / [12_UA_aware_adapter](04_詳細設計/03_クロスカッティング適合仕様/12_UA_aware_adapter.md) |
| test | [10_test設計方針](03_概要設計/10_test設計方針/README.md) | [19_検証規律適合仕様](04_詳細設計/01_適合仕様/19_検証規律適合仕様.md) | - |
| formal | [11_formal設計方針](03_概要設計/11_formal設計方針/README.md) | [20_形式検証適合仕様](04_詳細設計/01_適合仕様/20_形式検証適合仕様.md) | 全軸の proof obligation |
| meta（軸登録）| [12_クロスカッティング設計](03_概要設計/12_クロスカッティング設計/README.md) | - | release_gate.lock.yaml |

## アーキテクチャ概観
- [03_概要設計/01_アーキテクチャ概観/](03_概要設計/01_アーキテクチャ概観/README.md): 5 階層論 / 19 軸論 / defense-in-depth 6 層 / 5 proof_class 論 / 軸間依存図
- [03_概要設計/12_クロスカッティング設計/](03_概要設計/12_クロスカッティング設計/README.md): 認証 / 観測 / スキーマ進化 / 鍵管理 / OSS lifecycle / Bidi 適応経路 / 時刻整合 HLC / 数学的 enforcement

## lock.yaml 体系
- [04_詳細設計/05_lock_yaml体系/](04_詳細設計/05_lock_yaml体系/README.md): proof_artifact / counter_example / release_gate / artifact_lock 命名規約 / immutable archive

## 補助
- [90_knowledge/](90_knowledge/): 技術学習用 reference

## 用語
- [01_企画/09_用語集/](01_企画/09_用語集/README.md): 19 軸 / 5 階層論 / L1+ / 業界 pack / 4 layer state / BusinessConflict subtype / KEK shamir / atomic 三表書込 / proof_class 等の主要術語

## 関連
- [リポジトリ root README](../README.md)
- [CLAUDE.md](../CLAUDE.md): プロジェクトポリシー（至高路線）
- [LICENSE](../LICENSE)
