# tests/fixtures — テスト共有 fixture

全テストカテゴリ（e2e / contract / integration / fuzz / golden）で共有する test data / seed / 試験用 TLS 証明書を集約する。

## 構造

```text
fixtures/
├── README.md
├── seed-data/             # テナント / ユーザ / フィクスチャの JSON
│   ├── tenants.json
│   └── users.json
├── tls-certs/             # 試験用 CA / server / client 証明書（リリース時点 雛形）
│   ├── test-ca.pem
│   └── test-server.pem
└── openapi-samples/       # OpenAPI ベースのサンプルレスポンス（contract test 用）
```

## 注意事項

- **本ディレクトリの内容は test 専用**: 試験用 TLS 証明書は `test-*.pem` の命名で本番証明書と区別する。本番 issuer（Let's Encrypt prod）からの証明書を本ディレクトリに置くことは禁止する。
- **seed-data の管理**: `tenants.json` / `users.json` は擬似データのみ。実顧客データの含有禁止。`tools/git-hooks/` の secret-scan で個人情報パターンを検出する。
- **更新方法**: e2e / integration が必要とする最小セットを採用初期 段階で埋め、変更は PR レビューで全テストカテゴリへの影響を確認する。

## リリース時点 のスコープ

`seed-data/` / `tls-certs/` / `openapi-samples/` の各サブディレクトリを placeholder で配置する。実データは採用初期 で埋める。
