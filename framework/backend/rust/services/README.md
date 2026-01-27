# Framework Services

framework が提供する共通マイクロサービス。

## サービス一覧

| サービス | 説明 | 所有テーブル |
|---------|------|-------------|
| auth-service | 認証・認可 | `fw_m_user`, `fw_m_role`, `fw_m_permission`, `fw_m_user_role`, `fw_m_role_permission`, `fw_t_refresh_token` |
| config-service | 動的設定（`fw_m_setting`） | `fw_m_setting` |
| endpoint-service | エンドポイント情報管理 | `fw_m_endpoint` |

## 構成規約

各サービスは以下の構成を持つ：

```
{service_name}/
├── Cargo.toml
├── README.md
├── config/
│   ├── default.yaml
│   ├── dev.yaml
│   ├── stg.yaml
│   └── prod.yaml
├── deploy/
│   ├── base/
│   └── overlays/
│       ├── dev/
│       ├── stg/
│       └── prod/
├── proto/              # gRPC 正本
├── migrations/         # DB マイグレーション正本
└── src/
    ├── application/
    ├── domain/
    ├── infrastructure/
    ├── presentation/
    └── main.rs
```

## DB 所有

- 所有サービス以外はスキーマ変更を行わない
- DDL の正本は `framework/database/table/*.sql`
- マイグレーションの正本は `{service}/migrations/`

## 関連ドキュメント

- [規約: サービス構成](../../../../docs/conventions/service-structure.md)
