# k1s0 ドキュメント

本ディレクトリには、k1s0 プロジェクトの各種ドキュメントを格納する。

## ディレクトリ構成

```
docs/
├── README.md              # このファイル
├── adr/                   # Architecture Decision Records
│   ├── README.md          # ADR の運用方針
│   ├── TEMPLATE.md        # ADR テンプレート
│   └── ADR-XXXX-*.md      # 各 ADR
├── architecture/          # アーキテクチャ設計ドキュメント
│   ├── README.md          # 概要とナビゲーション
│   ├── overview.md        # システム全体像
│   ├── clean-architecture.md  # Clean Architecture の適用
│   ├── tier-system.md     # Tier システム（Tier1/2/3）
│   ├── service-mesh.md    # サービス間通信
│   └── diagrams/          # 図（Mermaid等）
├── conventions/           # 規約ドキュメント
│   ├── README.md          # 規約一覧
│   ├── service-structure.md
│   ├── config-and-secrets.md
│   ├── api-contracts.md
│   ├── observability.md
│   └── error-handling.md
├── design/                # 設計ドキュメント
│   ├── README.md          # 設計ドキュメント一覧
│   ├── cli.md             # CLI 設計
│   ├── generator.md       # Generator 設計
│   ├── lint.md            # Lint 設計
│   ├── framework.md       # Framework 設計
│   └── template.md        # Template 設計
└── operations/            # 運用ドキュメント
    ├── README.md          # 運用ドキュメント概要
    ├── deployment.md      # デプロイメント手順
    ├── monitoring.md      # モニタリング・アラート
    ├── troubleshooting.md # トラブルシューティング
    ├── sla.md             # SLA 定義
    └── runbooks/          # 運用手順書
        ├── service-restart.md
        ├── database-migration.md
        └── incident-response.md
```

## 各ディレクトリの役割

| ディレクトリ | 役割 | 対象読者 |
|-------------|------|----------|
| `adr/` | アーキテクチャ上の重要な決定を記録 | 開発者全員 |
| `architecture/` | システムアーキテクチャの設計ドキュメント | 開発者・アーキテクト |
| `conventions/` | 開発規約（コーディング、構成、API 等） | 開発者全員 |
| `operations/` | 運用手順、障害対応、デプロイ等 | 運用チーム・開発者 |

## ドキュメントの原則

1. **確定した内容のみ** をここに置く（検討中の草案は `work/` へ）
2. **規約はコードで検査** する（ドキュメントだけで終わらせない）
3. **変更は ADR で記録** する（重要な決定は理由を残す）

## クイックリンク

- [アーキテクチャ設計](architecture/README.md)
- [ADR 一覧](adr/README.md)
- [規約一覧](conventions/README.md)
- [設計ドキュメント](design/README.md)
- [運用ドキュメント](operations/README.md)
- [構想.md](../work/構想.md): 全体方針（草案）
- [プラン.md](../work/プラン.md): 実装計画（草案）
