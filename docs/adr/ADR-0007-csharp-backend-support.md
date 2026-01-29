# ADR-0007: C# バックエンドサポートの追加

## ステータス

承認済み

## コンテキスト

k1s0 は現在、バックエンド言語として Rust と Go をサポートしている。しかし、エンタープライズ顧客の多くが C#/.NET を主要な開発言語として使用しており、k1s0 の採用における障壁となっていた。特に以下の要因がある:

- エンタープライズ環境では C#/.NET が広く採用されている
- 既存の .NET チームが k1s0 プラットフォームを利用できない
- 競合ツールが .NET サポートを提供している

## 決定

k1s0 v0.2.0 において、`backend-csharp` テンプレートタイプを追加し、ASP.NET Core 8.0 ベースの C# バックエンドサポートを導入する。

### 具体的な変更内容

1. **CLI 拡張**: `new-feature` および `new-domain` コマンドで `--type backend-csharp` を選択可能にする
2. **テンプレート追加**: `CLI/templates/backend-csharp/` に feature テンプレートと domain テンプレートを作成する
3. **フレームワークパッケージ**: Rust/Go のフレームワークライブラリと同等の 8 つの NuGet パッケージを提供する
   - Tier 1: K1s0.Error, K1s0.Config, K1s0.Validation
   - Tier 2: K1s0.Observability, K1s0.Grpc.Server, K1s0.Grpc.Client, K1s0.Health, K1s0.Db
4. **Lint 対応**: K020（環境変数禁止）等の既存 lint ルールを C# コードに対応させる
5. **CI/CD**: `csharp.yml` ワークフローを追加する
6. **Clean Architecture 準拠**: Rust/Go と同じ 4 層構造（Domain, Application, Infrastructure, Presentation）を .NET プロジェクト構成で実現する

### プロジェクト構成

```
{Name}.sln
src/
├── {Name}.Domain/           # ドメイン層
├── {Name}.Application/      # アプリケーション層
├── {Name}.Infrastructure/   # インフラストラクチャ層
└── {Name}.Presentation/     # プレゼンテーション層 (ASP.NET Core)
config/
deploy/
.k1s0/manifest.json
```

## 理由

- **エンタープライズ需要**: C#/.NET は企業向け開発で最も広く使われている言語の一つであり、サポートすることで k1s0 の採用範囲を大幅に拡大できる
- **一貫したアーキテクチャ**: ASP.NET Core 8.0 は Clean Architecture パターンとの親和性が高く、既存の Rust/Go テンプレートと同等の構造を自然に実現できる
- **既存パターンの再利用**: Rust/Go で確立したフレームワークパッケージ構成（Tier 1/2/3）をそのまま NuGet パッケージに適用できる
- **ASP.NET Core 8.0 選定理由**: LTS リリースであり、gRPC・OpenTelemetry・Health Check のネイティブサポートが充実している

## 結果

### ポジティブ

- エンタープライズ顧客が k1s0 を採用しやすくなる
- .NET 開発者コミュニティからの貢献が期待できる
- 3 言語サポートにより、k1s0 のプラットフォームとしての信頼性が向上する

### ネガティブ

- 3 つ目のバックエンド言語のメンテナンスコストが発生する
- テンプレート更新時に 3 言語分の整合性を維持する必要がある
- フレームワークパッケージの新機能追加時に 3 言語での実装が必要になる
- CI/CD パイプラインの実行時間が増加する

### 関連 ADR

- [ADR-0001](ADR-0001-scope-and-prerequisites.md): スコープと前提条件
- [ADR-0006](ADR-0006-three-layer-architecture.md): 三層アーキテクチャ
