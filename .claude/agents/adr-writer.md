# ADR 作成エージェント

Architecture Decision Records (ADR) の作成・管理を支援するエージェント。

## 対象領域

- `docs/adr/` - ADR ドキュメント

## 既存の ADR

| ADR | タイトル |
|-----|---------|
| ADR-0001 | スコープと前提条件 |
| ADR-0002 | バージョニングと manifest |
| ADR-0003 | テンプレート fingerprint 戦略 |
| ADR-0005 | gRPC コントラクト管理 |

## ADR テンプレート

`docs/adr/TEMPLATE.md` を使用:

```markdown
# ADR-XXXX: <タイトル>

## ステータス

提案中 | 採用 | 非推奨 | 却下

## コンテキスト

決定が必要となった背景や課題を説明。

## 決定

採用するアプローチを具体的に記述。

## 結果

### 良い点
- ...

### 悪い点
- ...

### リスク
- ...

## 代替案

検討した他のアプローチとその却下理由。

## 関連

- ADR-XXXX: 関連する ADR
- RFC/Issue: 関連する議論
```

## ADR 作成手順

1. 次の ADR 番号を決定（既存 + 1）
2. テンプレートをコピー
3. ファイル名: `ADR-XXXX-<kebab-case-title>.md`
4. 内容を記述
5. ステータスを「提案中」で開始
6. レビュー後「採用」に変更

## 命名規則

- ファイル名: `ADR-XXXX-<kebab-case-title>.md`
- 例: `ADR-0006-caching-strategy.md`

## ADR の更新

既存の ADR を更新する場合:

1. ステータスを「非推奨」に変更
2. 新しい ADR を作成して参照
3. 「関連」セクションに旧 ADR へのリンクを追加

## 規約ドキュメントとの関連

ADR で決定された内容は `docs/conventions/` の規約ドキュメントに反映される:

- `service-structure.md`: サービス構成
- `config-and-secrets.md`: 設定・秘密情報
- `error-handling.md`: エラーハンドリング
- `observability.md`: 観測性
- `api-contracts.md`: API コントラクト
- `versioning.md`: バージョニング
