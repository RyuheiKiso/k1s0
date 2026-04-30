<!--
PR タイトルは Conventional Commits 形式（commitlint で検証されます）。
type(scope): subject の形式で 50 文字以内、lowercase か日本語で開始、句点なし。
type:  feat / fix / docs / style / refactor / perf / test / build / ci / chore / revert
scope: contracts / sdk-{dotnet,go,rust,typescript} / tier1-{go,rust} / tier2 /
       tier3-{web,native,bff,legacy} / platform / infra / deploy / ops / tools /
       docs / tests / security / deps / release

ヒント: branch 名を <type>/<scope>/<subject>（例: feat/tools/add-verify-target）
にすると、PR 作成時に .github/workflows/pr-title-autofix.yml が title を
自動補正します。違反時は形式案内のコメントが付きます。
-->

## 概要

<!-- 何を達成する PR か、1〜3 行で記述 -->

## 動機 / 背景

<!-- なぜこの変更が必要か。関連 Issue / ADR / 要件 ID を挙げる -->

- 関連 Issue: #
- 関連 ADR: ADR-
- 関連 要件 ID:

## 変更内容

<!-- 主要な変更点を箇条書きで -->

-
-

## 影響範囲

<!-- 影響する tier / コンポーネント / API contract -->

- [ ] tier1 / tier2 / tier3 / sdk / contracts / infra / deploy / docs / tools のいずれを変更したか明示
- [ ] 後方互換性を壊す変更か（壊す場合は ADR 必須、Superseded 旧 ADR 明示）

## チェックリスト

### コード品質

- [ ] `make pre-commit` が通る
- [ ] 1 ファイル 500 行以内（src/ 配下、docs 例外）
- [ ] コメントは日本語、各行の 1 行上に説明、ファイル先頭に説明コメント
- [ ] 自動生成ファイル以外、新規追加ファイルにライセンスヘッダ無しを確認

### contracts / SDK 変更時のみ

- [ ] `make codegen` を実行し、生成物を git add / commit した
- [ ] `buf lint` が通る
- [ ] `buf breaking` で破壊的変更が検出されないことを確認（破壊的変更時は ADR）

### docs 変更時のみ

- [ ] `markdownlint-cli2` が通る
- [ ] drawio 編集時は SVG export 済（`tools/_export_svg.py`）
- [ ] リンク切れがない（`tools/_link_check.py`）

### tier1 / tier2 / tier3 / SDK 変更時のみ

- [ ] 該当言語の native ビルドが通る（cargo build / go build / dotnet build / pnpm build）
- [ ] テストを追加・更新した（unit / integration / contract のいずれか）
- [ ] 関連する SHIP_STATUS.md のマチュリティ記述を更新した（必要時）

### local-stack / SoT 変更時のみ（ADR-POL-002）

- [ ] `tools/local-stack/up.sh` の `apply_*` 関数を新設・改名・廃止した場合、
      `infra/security/kyverno/block-non-canonical-helm-releases.yaml` の
      `deny.conditions.all[0].value` allow-list を**同 PR で**更新した
- [ ] `tools/local-stack/known-releases.sh` の出力と Kyverno policy allow-list の diff が空
      （`.github/workflows/drift-check.yml` sync-check job が PR で機械検証する。事前に
      `diff <(./tools/local-stack/known-releases.sh \| sort -u) <(yq ... policy)` で確認推奨）
- [ ] `up.sh` 改修時、`tools/local-stack/verify-cluster.sh` も期待値（namespace 名 / 重要 release 名 /
      ApplicationSet 名）を更新した

## ブレイクダウン

<!-- どのファイル変更が何を意味するか、レビュアの読み順を提示 -->

## テスト方法

<!-- レビュア / 自動テストが動作確認に使う手順 -->

```bash
# 例
make test-tier1-go
```

## スクリーンショット / ログ（該当時のみ）

<!-- UI 変更時 / CI 出力 / ベンチ結果 / etc. -->
