# 03. owner-e2e-results.md entry template

本ファイルは ADR-TEST-011 で確定した `docs/40_運用ライフサイクル/owner-e2e-results.md` の entry フォーマットを実装段階の正典として固定する。各 entry の必須フィールド、cut.sh が parse する keyword、part ごとの PASS/FAIL 記録形式、過去 entry との時系列整合を ID として採番する。

## 本ファイルの位置付け

ADR-TEST-011 で release tag ゲートが owner-e2e-results.md の最新 entry を必須参照することを確定した。本ファイルでは entry のフィールド構成を機械可読な形で固定し、cut.sh が確実に parse できる構造を運用契約として残す。entry のフォーマット drift があると cut.sh の正規表現が失敗し、release tag が切れなくなる事故を防ぐ。

## owner-e2e-results.md の全体構造

`docs/40_運用ライフサイクル/owner-e2e-results.md` は live document として、owner full e2e の実走結果を時系列で記録する。各 entry は実走日（YYYY-MM-DD）ごとに `### YYYY-MM-DD` heading で区切る。

```markdown
# owner full e2e 実走結果

本書は ADR-TEST-008 / 009 / 010 / 011 で確定した owner full e2e の実走結果を時系列で記録する live document。`tools/release/cut.sh` の release tag ゲート（ADR-TEST-011）が本書の最新 PASS entry を必須参照する。

## 月次サマリ

### 2026-05-15

- 実走者: kiso ryuhei
- 判定: PASS
- 各部位 PASS 数:
  - platform: 12/12
  - observability: 5/5（検証 1 trace propagation real / 検証 2 cardinality real / 検証 3 log↔trace real / 検証 4 SLO alert real / 検証 5 dashboard goldenfile real）
  - security: 18/18
  - ha-dr: 4/4
  - upgrade: 1/1
  - sdk-roundtrip: 48/48（4 言語 × 12 RPC）
  - tier3-web: 7/7
  - perf: 8/8
- artifact sha256: 7a3f9c5e1b2d8a6f4c2e9d1b7a5f3c8e6d2a9b4f1e7c5a3d8f6e2c9b4a1f5e3d
- artifact path: tests/.owner-e2e/2026-05-15/full-result.tar.zst
- 実走環境:
  - host RAM: 48GB / WSL2 Ubuntu 24.04
  - multipass version: 1.13.x
  - kubeadm version: v1.30.x
  - Cilium version: v1.15.x
  - Longhorn version: v1.6.x
  - MetalLB version: v0.14.x
- 所要時間: 1 時間 47 分
- 失敗詳細: なし

### 2026-04-15

- 実走者: kiso ryuhei
- 判定: FAIL（部位 ha-dr）
- 各部位 PASS 数:
  - platform: 12/12
  - observability: 5/5
  - security: 18/18
  - ha-dr: 3/4（etcd snapshot 復旧 FAIL: snapshot file path mismatch、root cause: tools/local-stack/up.sh で etcd backup volume mount 不足）
  - upgrade: 1/1
  - sdk-roundtrip: 48/48
  - tier3-web: 7/7
  - perf: 8/8
- artifact sha256: ...
- artifact path: tests/.owner-e2e/2026-04-15/full-result.tar.zst
- 実走環境: ...
- 所要時間: 1 時間 35 分
- 失敗詳細:
  - ha-dr/etcd_snapshot_recovery_test.go: FAIL
  - root cause: tools/local-stack/up.sh の `--role owner-e2e` で etcd backup volume が control-plane VM に mount されていない
  - 修正対応: tools/local-stack/up.sh に backup volume mount を追加（PR #142 で対応）
  - 次回再実走予定: PR #142 merge 後の 2026-04-22

## 月次サマリ template

（以下、新 entry 追加時の template）
```

## 必須フィールド（cut.sh が parse する）

各 entry は以下の必須フィールドを持つ。cut.sh が機械的に parse するため、フィールド名と value 形式は固定する。

| フィールド | 形式 | 必須 | 用途 |
|---|---|---|---|
| `実走者` | 自然文 | 必須 | 実走者 attribution |
| `判定` | `PASS` または `FAIL（部位 X）` | 必須 | cut.sh が `判定: PASS` を grep |
| `各部位 PASS 数` | bullet list で 8 部位列挙 | 必須 | 部分 FAIL の特定 |
| `artifact sha256` | `: <64 文字 HEX>` | 必須 | cut.sh が抽出して sha256sum 比較 |
| `artifact path` | `: tests/.owner-e2e/<YYYY-MM-DD>/full-result.tar.zst` | 必須 | cut.sh が読む artifact ファイル位置 |
| `実走環境` | bullet list（host RAM / multipass / kubeadm / Cilium / Longhorn / MetalLB version）| 必須 | 環境再現性 |
| `所要時間` | `<H 時間 M 分>` 形式 | 必須 | trend 監視 |
| `失敗詳細` | `なし` または bullet list | 必須 | FAIL 時の root cause / 対応 |

cut.sh の parse は以下の正規表現で行う:

```bash
# 最新 entry の date 抽出
grep -m1 "^### " owner-e2e-results.md | sed 's/^### //'

# 判定 PASS 確認
grep -A 20 "^### $latest_date" | grep -q "^- 判定: PASS"

# artifact sha256 抽出
grep -A 20 "^### $latest_date" | grep "^- artifact sha256:" | sed 's/^- artifact sha256: //'
```

正規表現が壊れない形式を維持するため、フィールドの prefix（`- 判定:` / `- artifact sha256:`）は厳密に統一する。

## entry 追加時の手順

`make e2e-owner-full` 実行後、新 entry を以下の手順で追加する。

1. owner-e2e-results.md の `## 月次サマリ` セクション直下（最新 entry の上）に新 entry を挿入
2. template に従ってフィールドを埋める（実走者 / 判定 / 各部位 PASS 数 / artifact sha256 / 実走環境 / 所要時間 / 失敗詳細）
3. artifact sha256 は `sha256sum tests/.owner-e2e/<YYYY-MM-DD>/full-result.tar.zst | cut -d' ' -f1` で取得
4. 各部位の PASS 数は `tests/.owner-e2e/<YYYY-MM-DD>/{部位}/result.json` を集計
5. git add owner-e2e-results.md + tests/.owner-e2e/<YYYY-MM-DD>/ で staging
6. commit message: `chore(e2e): owner full PASS 記録 - YYYY-MM-DD（部位 N/M）`
7. push（採用初期では release tag 切る前に必ず本記録 commit を済ませる）

`tools/qualify/owner-e2e/update-results.sh`（採用初期で新設）が手順 1〜5 を自動化する。リリース時点では手動更新を Runbook で案内する。

## FAIL entry の取り扱い

owner full で 1 部位でも FAIL した場合、entry の判定は `FAIL（部位 X）` と記録する。`cut.sh` は最新 entry が FAIL なら release tag を切らない。

FAIL entry でも `失敗詳細` に root cause + 修正対応 + 次回再実走予定を必ず記載する。これは「FAIL を記録だけして放置」を防ぎ、次回実走の追跡可能性を担保する。

PASS に戻すには、修正対応を実装 → owner full を再実走 → 新 PASS entry を追加、の 3 step を踏む。途中の状態（修正対応済だが未実走）で release tag を切ろうとすると cut.sh が「30 日以内に PASS なし」または「最新が FAIL」で fail する。

## 過去 entry との時系列整合

最新 entry は **必ず最上位**（`## 月次サマリ` 直下）に配置する。古い entry は下に積み上がる。これは cut.sh が `grep -m1 "^### "` で最新 entry を取得する経路と整合する設計。

時系列を逆転させる挿入（古い entry を上に追加）は禁止する。発見された場合は revert で修正する。

## entry の長さ制限

各 entry の本文は概ね 30〜50 行に収める。FAIL の詳細記録で 100 行を超える場合は、別ファイル（`tests/.owner-e2e/<YYYY-MM-DD>/failure-detail.md`）に切り出し、entry からは link のみとする。owner-e2e-results.md 全体が 5,000 行を超えると git diff / blame の応答が劣化するため、12 ヶ月超の古い entry は採用初期で年次 archive ファイル（`owner-e2e-results-2026.md` 等）に切り出す。

## IMP ID

| ID | 内容 | 配置 |
|---|---|---|
| IMP-CI-E2E-016 | owner-e2e-results.md entry フォーマット規約（必須フィールド + cut.sh parse 経路） | 本ファイル |

## 対応 ADR / 関連設計

- ADR-TEST-011（release tag ゲート代替保証）— 本ファイルの起源
- ADR-TEST-008（owner / user 二分構造）— 8 部位の PASS 数記録
- ADR-TEST-009（観測性 5 検証）— observability の 5 検証記録
- `01_cut_sh_拡張.md`（同章）— cut.sh の parse 経路
- `02_artifact_保管.md`（同章）— artifact sha256 source
- `docs/40_運用ライフサイクル/owner-e2e-results.md` — 本テンプレートが投影される実ファイル
- `tools/qualify/owner-e2e/update-results.sh`（採用初期で新設）— entry 追加自動化
