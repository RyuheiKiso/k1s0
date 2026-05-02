# CNCF Conformance テスト結果サマリ（月次更新）

本書は ADR-TEST-003（Sonobuoy + kind multi-node + Calico で月次実行）の月次 conformance 結果を時系列で記録する live document。ADR-CNCF-001（vanilla K8s + CNCF Conformance 維持）の継続証跡として採用検討組織に公開する。

## 本書の位置付け

`tools/qualify/conformance/run.sh` を `conformance.yml` workflow の月次 schedule（毎月 1 日 03:00 JST）で実行し、Sonobuoy `--mode certified-conformance` の結果を 12 ヶ月分時系列で版管理する。本書は各月の summary を要約し、results.tar.gz 本体は `tests/.conformance/<YYYY-MM>/` に git LFS で版管理する設計。

採用検討組織は本書を見ることで「k1s0 が動く環境が CNCF Conformance を継続取得しているか」を時系列で確認できる。upstream Kubernetes バージョン更新時の互換性破綻が早期検出される経路となる。

## 月次サマリ

### 2026-05（リリース時点 / 初月、Sonobuoy 実走前）

- **状態**: skeleton 配置完了、`conformance.yml` workflow 起動条件未充足（OSS 公開後の月初を待つ）
- **完了済**: `tools/local-stack/up.sh --role conformance` 追加 / `tools/qualify/conformance/run.sh` 実装 / `_reusable-conformance.yml` / `conformance.yml` 配置 / `Makefile verify-conformance` target 整備
- **kind 制約**: kind cluster は CSI / LB / multi-AZ の本番 fidelity 不足のため、Sonobuoy で skip される項目が一部存在する（採用初期で skip 項目一覧を本書に明示）
- **次月**: OSS 公開後、初回月初実行で 2026-06 entry を追記

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
