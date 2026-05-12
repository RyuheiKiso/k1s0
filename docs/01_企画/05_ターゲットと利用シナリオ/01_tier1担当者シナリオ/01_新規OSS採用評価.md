---
id: plan.tier1.scenario_new_oss_adoption
axis: tier1
phase: plan
kind: plan_doc
status: draft
depends_on:
  - arch.tier1.tier1_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# 新規 OSS 採用評価

## 一文方針

新規 OSS を 3 抽象レベルのいずれに割付けるかを評価し、AGPL/SSPL 系を物理拒否しつつ、dual reviewer sign-off + CI green + Harbor mirror 確立を merge 条件として tier1 facade の品質を守る。

> 朝 10 時、本社 IT 室の tier1 担当者（シニア級）が GitHub PR list を確認し、コミュニティから「Kafka から Redpanda への移行コスト評価」提案が上がっていることに気付く。手元には Backstage Catalog と buf CI ダッシュボード、Mattermost 越しに dual reviewer の tier1 担当者 2 名がいる。

## Trigger（発火条件）

tier1 担当者またはコミュニティから「現行 L1+ OSS の移行 or 新カテゴリへの OSS 採用」提案が挙がった時。

## 想定頻度 / 典型きっかけ

- 想定頻度: 四半期〜年次
- 典型きっかけ: 「pgvector の ANN 性能が不足し競合 OSS（Qdrant）が tier1 採用候補として上がった」「Kafka から Redpanda への移行コストを評価する提案がコミュニティから来た」

## 主役 / 関与者

- 主役: tier1 担当者（シニア級）
- 関与: dual reviewer（tier1 担当者 2 名、同一人物不可）

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|---|---|---|---|---|
| 主役（tier1）| シニア | 本社 IT 室 | GitHub PR list | OSS 評価・ライセンス判定・Harbor mirror 登録・PR 提出 |
| 関与（dual reviewer A）| シニア | 本社 IT 室 / リモート | GitHub PR list | OSS 評価レビュー・sign-off |
| 関与（dual reviewer B）| シニア | 本社 IT 室 / リモート | GitHub PR list | OSS 評価レビュー・sign-off |

## 前提

- 現行 `04_提供機能カテゴリ.md` のカテゴリ × Lv 割付が存在する
- Harbor mirror が稼働しており、supply chain lint が CI に組み込まれている
- `oss_lifecycle.lock.yaml` が管理されている

## 流れ

1. **抽象レベル判定**: OSS を [3 抽象レベル](../../../03_概要設計/02_tier1設計方針/02_Library.md)（L3 OSS 中立 / L2* 同族保証 / L1+ 単一深耕）のいずれに割付けるか判定する。評価軸は以下の通り:
   - ライセンス: AGPL / SSPL 該当の有無（Business Source License 型も含め採用しない判断を記録）
   - API 安定性: SemVer の遵守状況 / breaking change 頻度
   - コミュニティ存続性: メンテナ人数 / コミット頻度 / スポンサー状況
   - 他言語バインディング有無: Rust / C# / Go / TypeScript 各言語での利用可否

2. **L2* 採用時の手順**:
   - 同族 OSS 2 実装を選定し、共通インタフェースを tier1 Library facade として定義
   - Testcontainers conformance test を作成し、2 実装で全テスト green を merge 条件とする
   - `04_提供機能カテゴリ.md` に L2* として Lv 割付と露出概念 allowlist を追記

3. **L1+ 採用時の手順**:
   - 移行 toolchain 計画を draft する（schema diff ツール選定 / dual-write 期間と戦略 / observability 連続性 / 業務コード移行ガイド）
   - `oss_lifecycle.lock.yaml` 候補として登録し年次 dry-run 計画を設定
   - `04_提供機能カテゴリ.md` に L1+ として Lv 割付と露出概念 allowlist を追記

4. **AGPL / SSPL 系の拒否**: AGPL / SSPL 系（Business Source License 型含む）を採用しない判断を文書に記録する。強制機構（CI の依存導入 lint）が物理拒否するため、仮に PR が来ても merge 不可となる。

5. **`04_提供機能カテゴリ.md` 更新**: 採用決定後に Lv 割付・露出概念 allowlist を追記し、dual reviewer（tier1 2 名）の sign-off を取得する。

6. **Harbor mirror 登録**: OSS image / package を Harbor mirror に登録し supply chain mirror 経路を確立する。cosign 署名と SBOM 生成も合わせて実施する。

7. **CI 更新とグリーン確認**: 依存導入 lint に新 OSS を追加し CI green を確認する。4 言語の Library コード生成が全て通ることを検証する。

## 業界 9 業務との紐付け

全 9 業務に共通基盤として影響（tier1 Library / Server は全業務の通信・認証・観測の基盤を担うため）。特に影響度が高い 2 業務:

- **SCADA テレメトリ**: Kafka 系 OSS 採用評価はリアルタイムテレメトリ配信パイプラインの信頼性・スループットに直結し、採用判定が設備データ収集基盤全体に波及する。
- **警報配信**: Message Broker 系 OSS の採用・移行可否が、アラート配信レイテンシと信頼到達性に直接影響する。

## 関連適合仕様 / 関連 OSS

**関連適合仕様**:
- [OSSライフサイクル適合仕様](../../../04_詳細設計/01_適合仕様/08_OSSライフサイクル適合仕様.md)
- [tier1強制機構](../../../04_詳細設計/02_強制機構/01_tier1強制機構.md)

**関連 OSS**:
- Buf: proto 管理（proto ファイルの lint / breaking change 検出）
- Harbor: supply chain mirror（image / package の内部 mirror）
- Testcontainers: L2* conformance test の実行基盤

## 期待結果 / 観測指標

- `04_提供機能カテゴリ.md` が更新され Lv 割付と露出概念 allowlist が追記されている
- dual reviewer（tier1 2 名）の sign-off が PR に記録されている
- CI の依存導入 lint が green（AGPL/SSPL 混入なし）
- Harbor mirror に新 OSS の image / package が登録され、cosign 署名と SBOM が存在する
- L2* の場合: Testcontainers conformance test が 2 実装ともに green
- L1+ の場合: `oss_lifecycle.lock.yaml` に新 OSS が登録され、年次 dry-run 計画が存在する

## 失敗時の挙動 / escalation

- **AGPL/SSPL 検出**: CI の依存導入 lint が fail し merge 阻止。担当者は依存を削除して再 PR を作成する。escalate 先: tier1 担当者 dual reviewer へ Mattermost `#tier1-incident` で報告（SLA: 検出後 2 時間以内に次アクション）。Backstage runbook `oss-license-violation` を参照。
- **L1+ 移行 toolchain 計画未提出**: 採用 PR に移行 toolchain draft が添付されていない場合、dual reviewer が sign-off を拒否し merge 不可。escalate 先: tier1 担当者 dual reviewer（SLA: 24 時間以内に draft を提出）。
- **Harbor mirror 登録未完了**: supply chain lint が fail し merge 阻止。mirror 経路確立まで PR は open のまま。escalate 先: ops 担当者へ Mattermost `#tier1-incident` で連絡（SLA: 4 時間以内に mirror 経路確立）。Backstage runbook `harbor-mirror-registration` を参照。
- **Testcontainers conformance test fail（L2*）**: 2 実装のうち 1 つでも fail した場合、merge 不可。OSS の選定を見直す。escalate 先: tier1 担当者 dual reviewer（SLA: 48 時間以内に代替 OSS 選定）。

## 関連参照

- [tier1 設計方針](../../../03_概要設計/02_tier1設計方針/README.md)
- [tier1 担当者シナリオ index](README.md)
- [層別エンジニア要件](../../../02_要件定義/05_開発体制要件/01_層別エンジニア要件.md)
