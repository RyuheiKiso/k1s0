# owner full e2e 実走結果

本書は ADR-TEST-008 / 009 / 010 / 011 で確定した owner full e2e の実走結果を時系列で記録する **live document**。`tools/release/cut.sh` の release tag ゲート（ADR-TEST-011）が本書の最新 PASS entry を必須参照するため、entry のフォーマット規約は厳格に維持する。

## 本書の位置付け

owner full e2e は ADR-TEST-008 で「48GB host 専用 / 不定期実走 / CI 不可」と決定された。CI で機械検証されない代わりに ADR-TEST-011 の release tag ゲートが代替保証する。具体的には `tools/release/cut.sh` が本書の最新 entry の `判定: PASS` + `artifact sha256:` + 鮮度（既定 30 日以内）を必須検証し、不通なら release tag を切れない。

owner（k1s0 起案者）は以下の契機で owner full e2e を実走する:

- release tag を切る前
- k8s minor version upgrade 前
- 4 言語 SDK の major 改訂後
- tier1 12 service の互換破壊変更後

各実走後、本書に新 entry を追記する（`tools/qualify/owner-e2e/update-results.sh` で半自動化）。

## entry フォーマット規約（cut.sh が parse する）

各 entry は以下の必須フィールドを持つ。フィールド名 / value 形式は固定する（cut.sh の正規表現が壊れないため）。

| フィールド | 形式 | 必須 |
|---|---|---|
| `実走者` | 自然文 | ✅ |
| `判定` | `PASS` または `FAIL（部位 X）` | ✅ |
| `各部位 PASS 数` | 8 部位の bullet list | ✅ |
| `artifact sha256` | 64 文字 HEX | ✅ |
| `artifact path` | `tests/.owner-e2e/<YYYY-MM-DD>/full-result.tar.{zst,gz}` | ✅ |
| `実走環境` | host RAM / multipass / kubeadm / Cilium / Longhorn / MetalLB version | ✅ |
| `所要時間` | `<H 時間 M 分>` | ✅ |
| `失敗詳細` | `なし` または bullet list（root cause + 修正対応 + 次回再実走予定） | ✅ |

詳細な記述ルールは [docs/05_実装/30_CI_CD設計/35_e2e_test_design/40_release_tag_gate/03_owner_e2e_results_template.md](../05_実装/30_CI_CD設計/35_e2e_test_design/40_release_tag_gate/03_owner_e2e_results_template.md) を参照。

## 月次サマリ

（リリース時点では実走 entry なし。`make e2e-owner-full` 実走後、`tools/qualify/owner-e2e/update-results.sh <YYYY-MM-DD>` で本セクション直下に entry を追加する。手順:）

```bash
# 1. owner full 実走（host OS の WSL2 native shell から）
make e2e-owner-full
# 2. 結果 entry を本書に追加（判定 / artifact sha256 / 各部位 PASS 数を埋める）
./tools/qualify/owner-e2e/update-results.sh "$(date -u +%Y-%m-%d)"
# 3. 所要時間 + 失敗詳細を手動で埋める
$EDITOR docs/40_運用ライフサイクル/owner-e2e-results.md
# 4. commit + release tag を切る
git add docs/40_運用ライフサイクル/owner-e2e-results.md tests/.owner-e2e/
git commit -m "chore(e2e): owner full PASS 記録 - $(date -u +%Y-%m-%d)"
./tools/release/cut.sh v0.1.0
```

## entry template

新 entry を手動で追加する場合の参考フォーマット:

```markdown
### YYYY-MM-DD

- 実走者: <owner 名>
- 判定: PASS
- 各部位 PASS 数:
  - platform: 12/12
  - observability: 5/5
  - security: 18/18
  - ha-dr: 4/4
  - upgrade: 1/1
  - sdk-roundtrip: 48/48
  - tier3-web: 7/7
  - perf: 8/8
- artifact sha256: <64 文字 HEX>
- artifact path: tests/.owner-e2e/YYYY-MM-DD/full-result.tar.zst
- 実走環境:
  - host RAM: 48GB / WSL2 Ubuntu 24.04
  - multipass version: 1.13.x
  - kubeadm version: v1.31.0
  - Cilium version: v1.16.5
  - Longhorn version: v1.7.2
  - MetalLB version: v0.14.9
- 所要時間: 1 時間 47 分
- 失敗詳細: なし
```

## 関連

- [ADR-TEST-008](../02_構想設計/adr/ADR-TEST-008-e2e-owner-user-bisection.md) — owner / user 二分構造
- [ADR-TEST-011](../02_構想設計/adr/ADR-TEST-011-release-tag-gate-as-owner-e2e-alternative.md) — release tag ゲート代替保証
- [ops/runbooks/RB-TEST-OWNER-E2E-FULL.md](../../ops/runbooks/RB-TEST-OWNER-E2E-FULL.md) — owner full 実走 Runbook
- [tools/release/cut.sh](../../tools/release/cut.sh) — release tag ゲート本体
- [tools/qualify/owner-e2e/](../../tools/qualify/owner-e2e/) — archive.sh + update-results.sh
- [docs/05_実装/30_CI_CD設計/35_e2e_test_design/](../05_実装/30_CI_CD設計/35_e2e_test_design/) — 実装規約一式
