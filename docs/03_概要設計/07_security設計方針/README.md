---
id: arch.security.security_index
axis: security
phase: architecture
kind: index
status: draft
depends_on: []
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# security 設計方針 index

## 一文方針
- 本フォルダは security 軸（13 軸 + meta-spec の上位に立つ「直交 invariant」管掌 layer）の概要設計を集約する。security は新規物理機構を持ち込まず、5 階層 × 13 軸 grid の defense-in-depth 層 D / E に登場する全 block 機構（RLS FORCE / Kyverno admission / mTLS / cosign verify / cryptographic erasure 等）を脅威モデル下で cross-axis に bind することで「漏れなく重複なく組み合わさっていること」を機械検証する meta-layer として位置づく。

## 至高路線における立ち位置
- security は canonical axis 06 として 15_脅威モデル / 16_build_provenance の 2 spec を所有し 00_軸登録適合仕様 の meta-registry に entry 登録される。infra は canonical axis 04 として 12_クラスタ位相 / 13_時刻整合 の 2 spec を所有する（cap v1=20 中 19 / 20、残 1 スロット）。
- 13 軸が「層別 × 機能別」軸であるのに対し、security は「脅威別 × 軸横断」軸として直交方向に位置する。脅威モデル class（actor / capability / surface / asset / mitigation）を class 軸として持ち、各 class が複数軸に mitigation を要求する fan-out 構造を持つ。
- security 自身は新規物理機構を持ち込まない。例外は監査 SoR の immutability と SBOM/SLSA など本質的に cross-cut な道具立てのみ。
- 「現実には起こりにくいから登録しない」「mitigation が高コストだから緩める」判断は採らない。脅威の重みは coverage 内で SLO や budget で表現するが、登録自体は省略しない。

## test 軸 / formal 軸との同型直交
- test 軸: 18 axis × 5 verification_class の coverage matrix
- formal 軸: 19 axis × 5 proof_class の proof matrix
- security 軸: 5⁴ = 625 cell（actor × capability × surface × asset）の threat catalog に対し mitigation pointer
- 三軸とも「全 cell 物理 enforce」の構造的整合体として 1.0.0 ship blocker

## 配下ドキュメント

| 番号 | ドキュメント | 主題 | 該当する 5 軸 |
|---|---|---|---|
| 01 | [脅威モデル方針](01_脅威モデル方針.md) | 5⁴ cell の threat catalog + mitigation pointer | actor / capability / surface / asset / mitigation |
| 02 | [境界制御方針](02_境界制御方針.md) | 6 surface × 4 mitigation = 24 cell の trust boundary | surface |
| 03 | [秘密管理方針](03_秘密管理方針.md) | 8 secret class の lifecycle + rotation | mitigation |
| 04 | [脆弱性管理方針](04_脆弱性管理方針.md) | CVE 5 検知経路 + severity 別 patch SLO | mitigation |
| 05 | [監査方針](05_監査方針.md) | audit_event subject の immutability 三重化 | audit |
| 06 | [PII保護方針](06_PII保護方針.md) | 5 PII class × 6 軸 cross-axis bind | asset |
| 07 | [インシデント対応方針](07_インシデント対応方針.md) | 7 incident class × 6 phase playbook | mitigation |
| 08 | [セキュリティ訓練方針](08_セキュリティ訓練方針.md) | 7 drill class の cadence + success criteria | mitigation |

## 5 軸の class enum

### actor 軸（5 class）
- `v1_external_unauth`: 未認証外部攻撃者
- `v1_external_auth`: 認証済み外部 actor（tenant 越境含む）
- `v1_insider_application`: tier1 / tier2 / tier3 application identity の compromise
- `v1_insider_operator`: cluster operator / SRE 権限 identity の compromise
- `v1_supply_chain`: build / image / dep 経路からの compromise

### capability 軸（5 class、actor とは独立）
- `v1_read` / `v1_write` / `v1_delete` / `v1_dos` / `v1_lateral`

### surface 軸（5 class、actor が触れる外面）
- `v1_north_south` / `v1_east_west` / `v1_control_plane` / `v1_persistence` / `v1_supply_chain_surface`
- 加えて `v1_build_time`（境界制御方針で 6 surface に拡張）

### asset 軸（5 class、被害対象）
- `v1_pii` / `v1_business_data` / `v1_audit_log` / `v1_credential` / `v1_availability`

### mitigation 軸（5 class、物理機構の class）
- `v1_authn_authz` / `v1_crypto` / `v1_isolation` / `v1_admission_block` / `v1_audit_detect`

## 上位フェーズへの依存
- [提供スコープ](../../02_要件定義/01_スコープ/01_提供スコープ.md) / [非提供スコープ](../../02_要件定義/01_スコープ/02_非提供スコープ.md): security の責務 / 非責務
- [セキュリティ要件](../../02_要件定義/03_非機能要件/03_セキュリティ要件.md): 要件側 view
- [OSS 採用一覧](../../02_要件定義/04_技術選定/01_OSS採用一覧.md): security 軸採用 OSS

## 下位フェーズへの委譲
- [脅威モデル適合仕様](../../04_詳細設計/01_適合仕様/15_脅威モデル適合仕様.md): structural spec、build artifact 化
- [build_provenance 適合仕様](../../04_詳細設計/01_適合仕様/16_build_provenance適合仕様.md): supply chain integrity の structural spec
- [security 強制機構](../../04_詳細設計/02_強制機構/06_security強制機構.md): 5 層 defense-in-depth + 25+ Kyverno admission policy
- [security 運用 UI](../../04_詳細設計/04_運用UI開発者体験/03_security運用UI.md): Backstage + Perses + Backstage custom plugin
- [audit_ingest_gap_monitor](../../04_詳細設計/03_クロスカッティング適合仕様/09_audit_ingest_gap_monitor.md): hash chain の honest 補完

## 横断軸との bind
- [tier1 認証適合仕様](../../04_詳細設計/01_適合仕様/04_認証適合仕様.md) / [tier1 鍵管理適合仕様](../../04_詳細設計/01_適合仕様/05_鍵管理適合仕様.md): authn / key の cross-axis bind
- [tier2 テナント分離適合仕様](../../04_詳細設計/01_適合仕様/10_テナント分離適合仕様.md): RLS FORCE の cross-axis assertion
- [data 保全適合仕様](../../04_詳細設計/01_適合仕様/14_データ保全適合仕様.md): WORM / cryptographic erasure
- [infra 強制機構](../../04_詳細設計/02_強制機構/04_infra強制機構.md): Kyverno / NetworkPolicy / Istio
- [formal 設計方針](../11_formal設計方針/README.md): cryptographic invariant の数学的証明

## 読み筋
- 初読: README.md → 01_脅威モデル方針 → 02_境界制御方針 → 09 audit
- 実装時参照: 各方針 → [脅威モデル適合仕様](../../04_詳細設計/01_適合仕様/15_脅威モデル適合仕様.md)
- インシデント対応: 07_インシデント対応方針 → 11 ChatOps + 自製 escalation engine

## 関連参照
- [親フォルダ index](../README.md)
- [規約層 index](../../00_format/README.md)
