---
id: env.meta.author_drawio_toolchain
axis: meta
phase: env_setup
kind: enforcement
status: draft
depends_on:
  - env.meta.author_os_prerequisite
  - env.meta.author_markdownlint_textlint
covered_by:
  defense_in_depth_layers: [B]
  proof_classes: []
---

# drawio 作図ツール手順

## 一文方針

- `.claude/skills/drawio-authoring/bin/` の 3 スクリプト（drawio-lint / drawio-export / svg-postcheck）を手元で実行できることが drawio 作図環境の検収条件である。

## 3 スクリプトの概要

| スクリプト | 役割 | Python 必須バージョン |
|---|---|---|
| `drawio-lint` | `.drawio` XML に対して 15 規則を検証 | 3.10+ |
| `drawio-export` | WSL 経由で draw.io.exe を呼び出し SVG エクスポート | bash（Python 不要） |
| `svg-postcheck` | エクスポートした SVG を 5 規則で検証 | 3.10+ |

スクリプト配置:

```
.claude/skills/drawio-authoring/bin/drawio-lint
.claude/skills/drawio-authoring/bin/drawio-export
.claude/skills/drawio-authoring/bin/svg-postcheck
```

## drawio-lint の動作確認

```bash
python3 .claude/skills/drawio-authoring/bin/drawio-lint --help
```

15 検証規則の概要:

1. 白背景 rect の存在
2. 白 (`#ffffff`) のストローク色禁止
3. waypoint の不要配置禁止
4. エッジ交差禁止
5. ラベル間隔の確保
6. `<diagram>` 要素が 1 つだけ（1 図 1 ページ）
7. WCAG AA コントラスト比（4.5:1 以上）
8. レイヤーパレット規約
9. 以降 15 番まで drawio-authoring/SKILL.md を参照

`--strict` フラグで厳格モードを有効化できる。

## draw.io Desktop のパス設定

```bash
ls -la "/mnt/c/Program Files/draw.io/draw.io.exe"
```

存在しない場合は `DRAWIO_BIN` 環境変数で上書きする。

```bash
export DRAWIO_BIN="/mnt/c/Users/<username>/AppData/Local/Programs/draw.io/draw.io.exe"
```

## drawio-export の動作確認

```bash
DRAWIO_BIN="${DRAWIO_BIN:-/mnt/c/Program Files/draw.io/draw.io.exe}" \
  bash .claude/skills/drawio-authoring/bin/drawio-export --help 2>&1 || true
```

デフォルトオプション: `border=8` / `embed-svg-fonts=true`。

## svg-postcheck の動作確認

```bash
python3 .claude/skills/drawio-authoring/bin/svg-postcheck --help
```

5 検証項目: SVG サイズ < 1MB / 白背景 rect 存在 / 空 `<text>` なし / 白ストロークなし / エクスポートメタデータ整合。

## 作図ワークフロー

1. `.drawio` ファイル編集
2. `drawio-lint`（+ `--strict`）で検証
3. `drawio-export` で SVG 生成
4. `svg-postcheck` で生成 SVG 検証
5. GitHub の dark-theme でレンダリング目視確認

テンプレートは `.claude/skills/drawio-authoring/templates/canvas.drawio`（1200×800 白背景）を使う。

## 検収コマンド

```bash
python3 .claude/skills/drawio-authoring/bin/drawio-lint --help 2>&1 | head -1
python3 .claude/skills/drawio-authoring/bin/svg-postcheck --help 2>&1 | head -1
```

## 関連参照

- [02_前提OS環境](02_前提OS環境.md)
- [00_k1s0作者 index](README.md)
