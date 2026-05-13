---
id: env.overview.business_admin_lint_format
axis: overview
phase: env_setup
kind: enforcement
status: draft
depends_on:
  - env.overview.business_admin_test_environment
covered_by:
  defense_in_depth_layers: [B]
  proof_classes: []
---

# lint と format 適用

## 一文方針

- 業務管理者が送信した tier2 admin API リクエストの schema validation はサーバ側が enforcement し、決定表の形式 lint は Backstage プラグインが行うため、業務管理者はクライアント側では lint を手動実行しない。

## tier2 admin API の schema validation

tier2 admin API はリクエスト受信時に JSON schema validation を行い、不正なデータを拒否する。

```json
// 正しいリクエスト例
{
  "name": "demo-tenant-001",
  "tier": "tier1",
  "description": "演習用テナント"
}

// バリデーションエラーの例（tier に不正値）
{
  "name": "demo-tenant-001",
  "tier": "invalid-tier"
}
// → HTTP 422 Unprocessable Entity が返る
```

業務管理者はエラーレスポンスの `detail` フィールドを確認し、入力値を修正する。

## 決定表の形式 lint（Backstage プラグイン）

決定表エディタは保存時に自動で形式 lint を実行する。

```
lint チェック項目:
- 必須フィールドの存在確認（rule_id / condition / action）
- condition の型整合（文字列 / 数値 / 真偽値）
- action の enum 値確認
- rule_id の重複チェック
```

lint が失敗した場合、エディタがインラインでエラーを表示する。修正してから再度保存する。

## docs lint（WSL2 利用時）

docs/ 配下のドキュメントを変更した場合は docs lint を実行する。

```bash
bash tools/docs_lint/run_lint.sh
python3 tools/docs_lint/run_lint.py
```

## 検収コマンド

```bash
# API schema validation の確認（意図的なエラーリクエストを送り、422 が返ることを確認）
curl -s -X POST \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"name":"test","tier":"invalid"}' \
  https://api.staging.example.internal/v2/admin/tenants \
  | jq '.status'
# 期待: 422 または error メッセージ
```

## 関連参照

- [07_テスト検証環境](07_テスト検証環境.md)
- [09_docs_lint実行手順](09_docs_lint実行手順.md)
