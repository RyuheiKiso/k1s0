# ADR-0008: Python バックエンドサポートの追加

## ステータス

承認済み

## コンテキスト

k1s0 は v0.2.0 で C# サポートを追加し、Rust・Go・C# の 3 言語バックエンドをサポートしている。しかし、データサイエンスや ML チーム、プロトタイピング用途で Python を使用するチームが増加しており、k1s0 プラットフォームへの統合需要がある。特に以下の要因がある:

- データサイエンス・ML チームが FastAPI でモデルサービングを行うケースの増加
- プロトタイピングやスクリプト系マイクロサービスでの Python 利用
- Python の広範なエコシステム（scikit-learn, pandas, numpy 等）を活用したい需要

## 決定

k1s0 v0.2.1 において、`backend-python` テンプレートタイプを追加し、FastAPI ベースの Python バックエンドサポートを導入する。

### 具体的な変更内容

1. **CLI 拡張**: `new-feature` および `new-domain` コマンドで `--type backend-python` を選択可能にする
2. **テンプレート追加**: `CLI/templates/backend-python/` に feature テンプレートと domain テンプレートを作成する
3. **フレームワークパッケージ**: `framework/backend/python/` に共通パッケージを提供する
   - Tier 1: k1s0-error, k1s0-config, k1s0-validation
   - Tier 2: k1s0-observability, k1s0-grpc-server, k1s0-grpc-client, k1s0-health, k1s0-db
4. **Lint 対応**: K010/K011/K020/K021/K022 の既存 lint ルールを Python コードに対応させる
5. **CI/CD**: `python.yml` ワークフローを追加する
6. **Clean Architecture 準拠**: 他言語と同じ 4 層構造を Python パッケージ構成で実現する

### 技術選定

| 機能 | 技術 |
|------|------|
| Web フレームワーク | FastAPI 0.115+ |
| ASGI サーバー | Uvicorn |
| バリデーション | Pydantic v2 |
| ORM | SQLAlchemy 2.0 + asyncpg |
| gRPC | grpcio + grpcio-tools |
| テスト | pytest + pytest-asyncio + httpx |
| ログ/トレース | OpenTelemetry Python SDK |
| パッケージ管理 | uv (pyproject.toml) |
| フォーマット/リント | Ruff |
| 型チェック | mypy |
| Python バージョン | 3.12+ |

### プロジェクト構成

```
{feature_name}/
├── .k1s0/manifest.json
├── pyproject.toml
├── src/{feature_name_snake}/
│   ├── domain/
│   ├── application/
│   ├── infrastructure/
│   └── presentation/
├── tests/
├── config/
├── deploy/
└── Dockerfile
```

## 理由

- **データサイエンス・ML 需要**: Python は ML/AI 領域のデファクトスタンダードであり、モデルサービングやデータパイプラインの構築に不可欠
- **FastAPI の選定**: 型ヒント・Pydantic との統合が優れており、Clean Architecture パターンとの親和性が高い。非同期対応も標準
- **uv の選定**: Rust 製の高速パッケージマネージャで、pip/poetry より高速かつ再現性が高い
- **Ruff の選定**: Rust 製の高速リンター/フォーマッターで、flake8/black/isort を統合

## 結果

### ポジティブ

- データサイエンス・ML チームが k1s0 プラットフォームを直接利用可能になる
- 4 言語サポートにより、ほぼすべてのバックエンドチームをカバーできる
- Python の豊富なライブラリエコシステムを k1s0 規約の下で活用可能

### ネガティブ

- 4 つ目のバックエンド言語のメンテナンスコストが発生する
- Python の動的型付けにより、静的言語ほどの型安全性が確保できない（mypy で軽減）
- パフォーマンス面で Rust/Go に劣る可能性がある（ML 推論等では問題にならない）

### 関連 ADR

- [ADR-0001](ADR-0001-scope-and-prerequisites.md): スコープと前提条件
- [ADR-0006](ADR-0006-three-layer-architecture.md): 三層アーキテクチャ
- [ADR-0007](ADR-0007-csharp-backend-support.md): C# バックエンドサポート
