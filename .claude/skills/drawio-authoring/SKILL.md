---
name: drawio-authoring
description: drawio 図の作図規約。白背景・矢印重なり禁止・交差判定・ラベル間隔・1 図 1 ページ・コントラスト下限を強制する。WSL からの SVG エクスポート手順、検証スクリプト drawio-lint と svg-postcheck の併用、テンプレート参照を含む。
---

# drawio 作図 / SVG エクスポート規約

drawio 図を新規作成または編集する時は、本 Skill の全項目を満たしてから SVG エクスポートする。SVG が正典資産として md に埋め込まれる以上、生成物の質は再生成なしで読み手に届く品質でなければならない。

過去 3 セッションで透過背景 / 矢印重なり / レイヤ責務取り違えの事故が再発済 (`8253bbdd` / `2f723e66` / `7694a878`)。本規約はその再発防止のために存在する。

## なぜ SVG 一択か

- **拡大耐性**: GitHub の任意ズームでも文字が潰れない。
- **再加工可能**: `.drawio` を編集して再エクスポートできる。
- **テキスト検索可能**: SVG 内の `<text>` が検索ヒットする。

PNG/JPG ラスタ出力は禁止。

## 作業工程（厳守）

drawio は「**生成 → 検証 → エクスポート**」の 3 段階で進める。

1. **生成** — `.claude/skills/drawio-authoring/templates/canvas.drawio` を雛形に取り、drawio XML を書く。
2. **検証** — `drawio-lint` をエラーゼロにし、自己チェックリストを全項目通す。違反があれば**矢印の迂回ではなく要素の配置自体を見直す**。
3. **エクスポート** — `drawio-export` で SVG を出力し、続けて `svg-postcheck` をエラーゼロにし、最後に GitHub のダークテーマで別タブ目視確認する。

検証ステップを省略して `orthogonalEdgeStyle` の自動ルーティングに依存すると、矢印が要素を覆い隠して読解不能になる。

## ファイル配置と命名

- md と同階層に `img/` ディレクトリを作り、`.drawio`（編集ソース）と `.svg`（エクスポート結果）の両方を格納する。
- ファイル名は `<topic>_<subtopic>_<concept>` のスネークケース（例: `rust_basics_ownership.drawio` / `rust_basics_ownership.svg`）。
- md からは `.svg` を埋め込み、`alt` 属性必須・空 alt 禁止。`.drawio` は編集ソースとして併置する。
- **1 ファイル 1 ページ**: `<diagram>` 要素は 1 つだけ。図を分割したい時は別 `.drawio` を作る。
- `.drawio` を更新したら必ず `.svg` を再エクスポートし、両者を**同一コミットに含める**。古い `.svg` を残してはならない。
- `/knowledge` skill 経由で md を生成する場合も同じ `img/` 配下規律に従う。

## drawio XML の必須要素

### 白背景矩形（必須・第一要素）

`<root>` 直下、`id="0"` `id="1"` の基底 cell に続く**最初の vertex として**ページ全体を覆う白矩形を配置する。`mxGraphModel` の `background` 属性は SVG エクスポート時に無視される。

```xml
<mxCell id="bg" value="" style="rounded=0;whiteSpace=wrap;html=1;fillColor=#FFFFFF;strokeColor=none;" vertex="1" parent="1">
  <mxGeometry x="0" y="0" width="{pageWidth}" height="{pageHeight}" as="geometry" />
</mxCell>
```

### 推奨スタイル定数（迷ったらこれをコピー）

vertex 既定:

```
rounded=1;whiteSpace=wrap;html=1;fontFamily=Helvetica;fontSize=12;fontColor=#333333;strokeColor=#333333
```

edge 既定:

```
endArrow=classic;endFill=1;html=1;strokeColor=#333333;edgeStyle=none
```

### コントラスト下限

線・文字色は `#333333` 以上の濃さ（背景白に対し WCAG AA 相当）。`#000000` は GitHub ダークで潰れる。レイヤ色は `figure-layer-convention` のパレットに従う。

### 矢印

- 白系ストローク（`#FFFFFF` / `white`）禁止。背景白で消える。
- 矢印がボックス・テキスト・他の矢印と重ならないよう、経由点を**明示指定**する（`orthogonalEdgeStyle` の自動ルーティングに経路判断を任せない）。
- 経由点は `<mxGeometry>` 内の `<Array as="points">` に書く:

```xml
<mxCell id="e1" edge="1" parent="1" source="A" target="B"
        style="endArrow=classic;endFill=1;html=1;strokeColor=#333333;edgeStyle=none;">
  <mxGeometry relative="1" as="geometry">
    <Array as="points">
      <mxPoint x="240" y="120" />
      <mxPoint x="240" y="280" />
    </Array>
  </mxGeometry>
</mxCell>
```

### ラベル

- ラベルが矢印自体を覆い隠してはならない。
- ラベル付き矢印の接続元と接続先の間隔は、ラベル幅近似値の **1.5 倍以上**を確保する。
- ラベル幅近似値（px）= `全角文字数 × fontSize + 半角文字数 × fontSize × 0.55`。
- 間隔不足は要素の配置を見直して解消する（矢印の迂回では解決しない）。

### 形状ボキャブラリ

形状の意味付け（角丸矩形 / 楕円 / 菱形 / 円柱 / 平行四辺形）は `figure-layer-convention` Skill の役割表に従う。同じ形・色で異なる責務レイヤのコンポーネントを並べてはならない。

## 検証（drawio-lint）

XML 生成後、エクスポート前に必ず実行する:

```bash
.claude/skills/drawio-authoring/bin/drawio-lint <input.drawio>
```

機械検証項目:

基本ルール:

1. 白背景矩形が `<root>` 直下の最初の vertex に存在し (0,0) を起点にしている。
2. 全 edge が白系ストロークでない。
3. `orthogonalEdgeStyle` を使う edge は `<Array as="points">` で経由点が明示されている。
4. 全 edge セグメントが **source/target/それらを含むコンテナ/bg 以外**の vertex bbox と交差していない。
5. ラベル付き edge の source–target 間距離がラベル幅近似値の 1.5 倍以上ある。
6. `<diagram>` 要素は 1 つだけ。

拡張ルール:

7. **ラベル bbox の干渉禁止**: edge ラベルの bbox（中点起点・幅 = ラベル幅近似値、高さ = fontSize × 1.4）が、自身以外の vertex や他 edge セグメントと重ならない。
8. **text vertex 同士の重なり禁止**: 透明注釈テキストの bbox が他 text vertex と重ならない（WARN）。
9. **ページ枠はみ出し検出**: `mxGraphModel` の `pageWidth × pageHeight` から vertex bbox がはみ出していない（コンテナ子要素は除外）。
10. **重複 ID / 孤立 edge 検出**: 同一 ID の cell が複数存在しない。edge の source/target が解決可能であるか、解決不能なら `sourcePoint`/`targetPoint` で座標補完されている。
11. **自己ループ / ゼロ長 edge 検出**: source==target、または polyline 総長 < 4px の edge を禁止。
12. **parent 階層整合**: vertex の `parent` 属性が `0`/`1` または既存 vertex を指していること。コンテナ親が指定された場合、子 bbox が親 bbox 内に収まっているか（WARN）。
13. **コントラスト下限 (WCAG AA)**: 背景白に対する `fontColor` および edge の `strokeColor` のコントラスト比が 4.5:1 以上。レイヤパレットの装飾枠色（`#d79b00` / `#6c8ebf` / `#666666` / `#9673a6`）は許容例外。`#333333` 相当（>= 7.5:1）に達しない場合は WARN。
14. **レイヤパレット遵守**: vertex の `(fillColor, strokeColor)` ペアが `figure-layer-convention` の 4 ペアのいずれか、または中性色（white/none/`#333333`/`#666666`）。逸脱は WARN。
15. **ラスタ画像埋め込み禁止**: `shape=image` および `image=data:` URI の検出。SVG 一択ポリシー違反は ERROR。

ERROR が 1 つでも残っている間は SVG エクスポートしない。WARN（透明背景テキストとの交差、レイヤパレット逸脱、コントラスト推奨値割れ等）は許容するが、`--strict` でエラー扱いに引き上げ可。

## 複数レイヤが登場する場合

アプリ層・ネットワーク層・インフラ層・データ層のうち 2 つ以上が同一図に登場する場合は、**`figure-layer-convention` Skill の色・線種・配置 3 軸の規約を併せて遵守する**。同じビジュアルで責務レイヤの異なるコンポーネントを並べてはならない。凡例ブロックの雛形は `templates/legend.xml` を参照。

## SVG エクスポート手順（WSL）

専用ラッパーを使う:

```bash
.claude/skills/drawio-authoring/bin/drawio-export <input.drawio>
# → <input>.svg を同階層に出力（border=8 / embed-svg-fonts=true）
```

主要オプション:

| オプション | 意味 | 既定 |
|---|---|---|
| `-o, --output <path>` | 出力先 SVG パス | `<input>.svg` |
| `-b, --border <px>` | 図の外周マージン | `8` |
| `--crop` | 余白を自動詰めて出力 | off |
| `--page-index <N>` | マルチページ図の特定ページ | `0` |

CLI の場所が異なる環境では `DRAWIO_BIN` 環境変数で上書きする。生 CLI を直接叩く場合は:

```bash
"/mnt/c/Program Files/draw.io/draw.io.exe" --export --format svg \
  --embed-svg-fonts true --border 8 \
  --output <出力先.svg> <入力.drawio>
```

## エクスポート後検証（svg-postcheck）

```bash
.claude/skills/drawio-authoring/bin/svg-postcheck <output.svg>
```

機械検証項目:

- ファイルサイズ < 1 MB（embed-svg-fonts 暴走の検知）。
- 文書ルート付近に白塗り `<rect>` が存在（白背景の反映確認）。
- 空 `<text>` 要素が存在しない（フォント未解決の文字化け検知）。
- 白系 stroke で消えている要素がない。

最後に GitHub ダークテーマで別タブ目視確認する。

## 自己チェックリスト（エクスポート前に必ず通す）

1. [ ] `drawio-lint` が ERROR ゼロで通った。
2. [ ] `<root>` 直下の最初の vertex がページ全体を覆う白矩形である。
3. [ ] 白系ストロークの矢印を使っていない。
4. [ ] 矢印がボックス・テキストに重なっていない（lint の交差判定が緑）。
5. [ ] `orthogonalEdgeStyle` 使用時は `<Array as="points">` で経由点を明示指定している。
6. [ ] ラベル付き矢印の接続元と接続先の間隔がラベル幅近似値の 1.5 倍以上ある。
7. [ ] ラベルが矢印自体を覆っていない。
8. [ ] 線・文字に `#333333` 以上のコントラストを確保している。
9. [ ] 同じビジュアル（形・色）で異なる責務レイヤのコンポーネントを並べていない。
10. [ ] 複数レイヤ図の場合、`figure-layer-convention` の色・線種・凡例配置を遵守している。
11. [ ] `<diagram>` 要素は 1 つだけ（マルチページではない）。
12. [ ] ファイルが md と同階層の `img/` 配下に `<topic>_<subtopic>_<concept>.{drawio,svg}` で配置されている。
13. [ ] `.drawio` 更新と `.svg` 再エクスポートを同一コミットに含める。
14. [ ] `drawio-export` で出力した SVG が `svg-postcheck` を ERROR ゼロで通った。
15. [ ] GitHub ダークテーマで別タブ目視確認した。

## 同梱資産

- `bin/drawio-lint` — `.drawio` XML の規約検証（Python 3.10+）
- `bin/svg-postcheck` — エクスポート済 SVG の規約検証（Python 3.10+）
- `bin/drawio-export` — WSL から draw.io.exe を呼ぶ統一ラッパー
- `templates/canvas.drawio` — 白背景＋タイトル付き 1200×800 雛形
- `templates/legend.xml` — 複数レイヤ図用の凡例ブロック雛形
