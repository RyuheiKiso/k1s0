# ADR-0037: YAML 設定管理ライブラリの移行（serde_yaml 0.9 廃止）

## ステータス

提案

## コンテキスト

外部技術監査（M-3）において、以下の問題が指摘された。

1. **serde_yaml 0.9 が deprecated**: crates.io でアーカイブ済みの非推奨クレート。後継の `serde_yml` への移行が推奨されている。
2. **シェル変数の非展開**: `serde_yaml` は `${VAR:-default}` 形式のシェル構文を展開しない。Docker Compose 環境の `config.docker.yaml` でこのパターンを使用しているため、DB 認証失敗（C-1, C-4）の根本原因になっている。
3. **影響範囲**: `regions/system/Cargo.toml` ワークスペースの全27サーバー + 5ライブラリ（計32ファイル）が依存。

現状の暫定対応として、各サービスの startup.rs で `DATABASE_URL` 環境変数オーバーライドを実装（C-1対応）しているが、根本的な解決は設定管理の抜本的見直しが必要。

## 決定

以下の 2 段階で移行する。

### フェーズ 1: config.docker.yaml から シェル変数構文を排除（短期）

`config.docker.yaml` の `${VAR:-default}` 形式を全て固定値に変更し、環境変数参照は startup.rs の `std::env::var()` パターンに統一する。

### フェーズ 2: serde_yaml から figment への移行（長期）

`figment` クレートは環境変数レイヤーを標準でサポートし、`YAML ファイル < 環境変数` の優先順位で設定をマージできる。これにより startup.rs での個別オーバーライド実装が不要になる。

```toml
# Cargo.toml
figment = { version = "0.10", features = ["yaml", "env"] }
```

```rust
// startup.rs での使用例
let cfg: Config = Figment::new()
    .merge(Yaml::file(&config_path))
    .merge(Env::prefixed("K1S0_"))  // K1S0_DATABASE__PASSWORD 等
    .extract()?;
```

## 理由

1. **根本解決**: 環境変数レイヤーを figment が管理することで、startup.rs への個別実装が不要になる
2. **セキュリティ**: 非推奨クレートの使用を終了し、積極的にメンテナンスされているライブラリに移行
3. **一貫性**: 全32クレートで設定管理方法が統一される

## 影響

**ポジティブな影響**:
- serde_yaml の非推奨問題を解消
- 環境変数オーバーライドが自動的に処理される
- config.docker.yaml のシェル変数構文を廃止できる

**ネガティブな影響・トレードオフ**:
- 全27サーバーの startup.rs と Config 構造体の変更が必要（大規模変更）
- figment の学習コストが生じる
- フェーズ 2 は工数が大きく（2週間以上）、計画的な移行が必要

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| 案 A: serde_yml へ移行 | serde_yaml の後継クレート | 環境変数展開はサポートしないため問題の根本解決にならない |
| 案 B: config-rs | 汎用設定ライブラリ | figment と同等機能だが API が複雑 |
| 案 C: 現状維持 + 全 startup.rs 修正 | C-1 の対応パターンを全サービスに適用 | 根本解決にならず、新規サービス追加時に同じ問題が再発する |

## 参考

- 外部技術監査報告書 M-3: serde_yaml 0.9 (deprecated) の継続使用
- [figment ドキュメント](https://docs.rs/figment)
- ADR-0030: 実装状態追跡の例（本 ADR でも実装ステータス管理を行う）

## 更新履歴

| 日付 | 変更内容 | 変更者 |
|------|---------|--------|
| 2026-03-25 | 初版作成（M-3 監査対応） | - |
