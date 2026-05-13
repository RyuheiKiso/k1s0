---
id: env.formal.formal_lint_format
axis: formal
phase: env_setup
kind: enforcement
status: draft
depends_on:
  - env.formal.formal_test_environment
covered_by:
  defense_in_depth_layers: [B]
  proof_classes: []
---

# lint と format 適用

## 一文方針

- formal 軸エンジニアは各 proof_class に対応する syntax check コマンドを push 前に実行し、構文エラーがゼロの状態を維持する。lint の CI 配線は k1s0 作者の責務だが、手元での実行は formal 軸エンジニアの規律である。

## TLA+ syntax check（tlc）

```bash
tlc -syntax HelloSpec.tla
# 構文エラーがあれば行番号と内容が表示される
```

TLA+ Toolbox の IDE では保存時に自動 syntax check が走る。CLI 実行の場合は上記コマンドを使う。

## Dafny syntax-only check

```bash
dafny /compile:0 Sample.dfy
# /compile:0 はコンパイルせず検証のみ実行（syntax + verification）
```

`dafny verify` と等価だが、コンパイル成果物を生成しないため高速。CI での利用を推奨する。

## Lean 4 check

```bash
lean --check Sample.lean
# コンパイルとゴールチェックを行いエラーを出力する
```

`lake build` をプロジェクト全体に対して実行することで、全 `.lean` ファイルを一括チェックできる。

```bash
lake build
```

## Kani check（cargo kani）

```bash
cargo kani
# Rust コードの形式検証を実行する
```

`--tests` フラグを付けると通常の cargo test と組み合わせて実行できる。

## CBMC check

```bash
cbmc cbmc_sample.c --bounds-check --pointer-check
# 境界チェックとポインタ検査を有効にして実行
```

## Stainless check

```bash
stainless-dotty Sample.scala
# Scala コードの形式検証を実行する
```

## docs lint との統合

proof spec ファイル（.tla / .dfy / .lean / Rust）は `docs/` 配下には置かない。`src/formal/` に配置し、docs lint の対象外とする。

```bash
# docs lint は docs/ 配下のみを対象とするため、src/ の spec は個別に実行
bash tools/docs_lint/run_lint.sh
python3 tools/docs_lint/run_lint.py
```

## 検収コマンド

```bash
dafny /compile:0 Sample.dfy 2>&1 | tail -3
lean --check Sample.lean 2>&1 | tail -3
cbmc --version
```

## 関連参照

- [07_テスト検証環境](07_テスト検証環境.md)
- [09_docs_lint実行手順](09_docs_lint実行手順.md)
