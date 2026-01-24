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
├── architecture/          # アーキテクチャ設計ドキュメント（予定）
├── conventions/           # 規約ドキュメント
│   ├── README.md          # 規約一覧
│   ├── service-structure.md
│   ├── config-and-secrets.md
│   ├── api-contracts.md
│   ├── observability.md
│   └── error-handling.md
└── operations/            # 運用ドキュメント（予定）
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

- [ADR 一覧](adr/README.md)
- [規約一覧](conventions/README.md)
- [構想.md](../work/構想.md): 全体方針（草案）
- [プラン.md](../work/プラン.md): 実装計画（草案）
