---
id: env.meta.author_lock_yaml_generator
axis: meta
phase: env_setup
kind: enforcement
status: draft
depends_on:
  - env.meta.author_docs_lint
covered_by:
  defense_in_depth_layers: [B]
  proof_classes: []
---

# lock_yaml 生成器手順

## 一文方針

- `tools/lock_yaml_generator/generate_release_gate.py` を実行して `release_gate.lock.yaml` が出力されることが、lock_yaml 生成器の環境確認条件である。全 98 cell が `status: red` の初期 lock が出力されれば正常。

## generate_release_gate.py の実行

> **pre-P0 現状**: generator は P0 deliverable であり実体ゼロ（未物理化）。P0 完了後は以下のコマンドで実行する。

```bash
source .venv/bin/activate
python3 tools/lock_yaml_generator/generate_release_gate.py
```

出力先: `tools/lock_yaml_generator/samples/release_gate.lock.yaml`

物理化後は全 98 cell が `status: red` の初期 lock を生成する。

## cell catalog（98 cell）

生成スクリプト内 `RELEASE_GATE_CELLS` で定義される 98 cell。代表 cell の担当軸分類（詳細は `docs/04_詳細設計/05_lock_yaml体系/03_release_gate体系.md` の cell catalog が SoT）:

| 軸 | cell 数（代表） |
|---|---|
| formal（summary cell） | 10 |
| test | 3 |
| ops | 2 |
| security | 3 |
| infra | 2 |
| data | 2 |
| tier1/tier2/tier3/client | 7 |
| meta | 4 |
| cross-cutting cluster | 13 |
| formal proof_class 内訳展開 | 52 |
| **合計** | **98** |

AND-gate が green（全 98 cell green）にならないと cosign signed tag を打てない。

## 将来生成器の slot

`00_軸登録適合仕様` で宣言された各適合仕様ごとに対応する生成器が必要になる。作者は以下の命名規約で生成器を追加する。

```
tools/lock_yaml_generator/generate_<axis>_<spec_short>.py
```

例: `generate_tier1_transport.py` → `capabilities.lock.yaml` を生成。

## samples/ ディレクトリの扱い

`tools/lock_yaml_generator/samples/` 配下は commit 対象。生成器の出力サンプルとして管理し、実運用 lock.yaml は `src/` 配下の各軸に配置する（配置規約は 04_詳細設計/05_lock_yaml体系 を参照）。

## 検収コマンド

```bash
python3 tools/lock_yaml_generator/generate_release_gate.py && \
  echo "生成成功" && \
  cat tools/lock_yaml_generator/samples/release_gate.lock.yaml | grep "status:" | head -5
```

先頭 5 行が `status: red` であることを確認する（98 cell 全行 red が正常）。

## 関連参照

- [13_署名とReleaseGate](13_署名とReleaseGate.md)
- [tools/lock_yaml_generator/README.md](../../../tools/lock_yaml_generator/README.md)
