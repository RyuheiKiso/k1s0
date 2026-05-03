# CNCF Conformance テスト結果サマリ（月次更新）

本書は ADR-TEST-003（Sonobuoy + kind multi-node + Calico で月次実行）の月次 conformance 結果を時系列で記録する live document。ADR-CNCF-001（vanilla K8s + CNCF Conformance 維持）の継続証跡として採用検討組織に公開する。

## 本書の位置付け

`tools/qualify/conformance/run.sh` を `conformance.yml` workflow の月次 schedule（毎月 1 日 03:00 JST）で実行し、Sonobuoy `--mode certified-conformance` の結果を 12 ヶ月分時系列で版管理する。本書は各月の summary を要約し、results.tar.gz 本体は `tests/.conformance/<YYYY-MM>/` に git LFS で版管理する設計。

採用検討組織は本書を見ることで「k1s0 が動く環境が CNCF Conformance を継続取得しているか」を時系列で確認できる。upstream Kubernetes バージョン更新時の互換性破綻が早期検出される経路となる。

## 月次サマリ

### 2026-05（リリース時点 / 初月、Sonobuoy quick mode 実走 PASS）

- **状態**: kind cluster（k8s v1.31.4、4 nodes）+ Calico CNI で **Sonobuoy v0.57.3 quick mode が実走 PASS**（2026-05-03 00:08 JST）
- **実走結果**:
  - `sonobuoy run --mode quick --wait`: 約 1.5 分で完了
  - e2e plugin: complete / passed
  - systemd-logs plugin: complete / passed（4 nodes すべて）
  - Failed: 0 / Remaining: 0
- **動作確認した経路**:
  1. `curl -L sonobuoy_0.57.3_linux_amd64.tar.gz | tar xz` で CLI install
  2. 既存 kind cluster（multi-node + Calico）に対し `sonobuoy run --mode quick --wait`
  3. plugins (e2e + systemd-logs) が complete + passed で終了
  4. `sonobuoy delete --wait` で cleanup
- **未実走**: `--mode certified-conformance` は所要 60〜120 分のため本実走では実施せず（quick mode で経路確認）。月初 schedule trigger で `conformance.yml` が起動した際に certified-conformance で full 取得する設計
- **kind 制約**: kind cluster は CSI / LB / multi-AZ の本番 fidelity 不足のため、certified-conformance で skip される項目が一部存在する（採用初期で skip 項目一覧を本書に追記）
- **証跡保存の課題**: 本実走では retrieve 前に delete を呼んで artifact tar.gz を取り損ねた。`tools/qualify/conformance/run.sh` の冪等順序（retrieve → summary → delete）が正しい運用で、CLI 直叩き時は順序遵守が必要
- **以降**: certified-conformance を月初 schedule で実走し、tests/.conformance/<YYYY-MM>/sonobuoy-results.tar.gz を 12 ヶ月版管理開始

## 月次サマリ template

```markdown
### YYYY-MM

- **実行日**: YYYY-MM-01 03:00 JST
- **K8s version**: 1.NN.M（kind が pin する version）
- **Sonobuoy version**: vX.YY.Z
- **CNI**: Calico vX.YY.Z
- **結果**: PASS / FAIL（失敗 K 件）
- **失敗テスト一覧**: <テスト名 / 概要 / 上流 issue link>
- **kind 制約による skip**: <スキップ項目の概要>
- **artifact**: tests/.conformance/YYYY-MM/sonobuoy-results.tar.gz
- **対応**: <修正 commit / upstream issue 報告 / 設計見直し>
```

## 関連

- ADR-TEST-003（CNCF Conformance を Sonobuoy で月次実行）
- ADR-CNCF-001（vanilla K8s + CNCF Conformance 維持）
- ADR-NET-001（kind = Calico、L5 conformance も同 CNI）
- `.github/workflows/conformance.yml` / `_reusable-conformance.yml`
- `tools/qualify/conformance/run.sh`
- IMP-CI-CONF-001〜005
