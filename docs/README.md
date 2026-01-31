# k1s0 ドキュメント

本ディレクトリには、k1s0 プロジェクトの各種ドキュメントを格納する。

## ディレクトリ構成

```
docs/
├── README.md              # このファイル
├── GETTING_STARTED.md     # 入門ガイド
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
│   ├── error-handling.md
│   ├── domain-boundaries.md   # Domain 境界ガイドライン
│   ├── deprecation-policy.md  # 非推奨化ポリシー
│   └── versioning.md          # バージョニング規約
├── design/                # 設計ドキュメント
│   ├── README.md          # 設計ドキュメント一覧
│   ├── cli/               # CLI 設計（ディレクトリ）
│   ├── generator.md       # Generator 設計
│   ├── lint/              # Lint 設計（ディレクトリ）
│   ├── framework.md       # Framework 設計
│   ├── framework/         # Framework 設計（分割版）
│   ├── domain.md          # Domain 層設計
│   └── template/          # Template 設計
├── guides/                # 開発ガイド
│   ├── domain-development.md  # Domain 開発ガイド
│   ├── domain-versioning.md   # Domain バージョン管理ガイド
│   └── migration-to-three-tier.md  # 3層構造移行ガイド
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
| `design/` | 各機能の設計ドキュメント | 開発者・アーキテクト |
| `guides/` | 開発ガイド（Domain 開発・移行等） | 開発者全員 |
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
- [開発ガイド](guides/)
- [Getting Started](GETTING_STARTED.md)
- [運用ドキュメント](operations/README.md)
- [Observability Stack](../observability/README.md): OTEL Collector / Jaeger / Loki / Prometheus / Grafana
- [構想.md](../work/構想.md): 全体方針（草案）
- [プラン.md](../work/プラン.md): 実装計画（草案）
