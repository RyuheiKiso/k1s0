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

- `tools/lock_yaml_generator/generate_release_gate.py` を実行して `release_gate.lock.yaml` が出力されることが、lock_yaml 生成器の環境確認条件である。全 20 cell が `status: red` の初期 lock が出力されれば正常。

## generate_release_gate.py の実行

```bash
source .venv/bin/activate
python3 tools/lock_yaml_generator/generate_release_gate.py
```

出力先: `tools/lock_yaml_generator/samples/release_gate.lock.yaml`

現状 skeleton（`実装 status: skeleton` コメントあり）であり、全 20 cell が `status: red` の YAML を生成する。

## 20 cell catalog

生成スクリプト内 `RELEASE_GATE_CELLS` で定義された 20 cell:

| # | cell 名 | 担当軸 |
|---|---|---|
| 1-5 | formal 関連 5 cell | formal |
| 6-9 | tier1 関連 4 cell | tier1 |
| 10 | tier2 1 cell | tier2 |
| 11 | tier3 1 cell | tier3 |
| 12 | client 1 cell | client |
| 13-14 | infra 2 cell | infra |
| 15 | data 1 cell | data |
| 16-17 | security 2 cell | security |
| 18 | ops 1 cell | ops |
| 19 | test 1 cell | test |
| 20 | meta docs_lint 1 cell | meta |

AND-gate が green（全 20 cell green）にならないと cosign signed tag を打てない。

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

全行 `status: red` が出力されることを確認する。

## 関連参照

- [13_署名とReleaseGate](13_署名とReleaseGate.md)
- [tools/lock_yaml_generator/README.md](../../../tools/lock_yaml_generator/README.md)
