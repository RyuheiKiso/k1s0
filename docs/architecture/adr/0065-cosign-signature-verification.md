# ADR-0065: app-registry Cosign 署名検証の実装戦略

## ステータス

承認済み

## コンテキスト

外部技術監査（2026-04-01）の STATIC-CRITICAL-002 指摘: `app-registry` の `CosignVerifier` が常に `true` を返すスタブのままで、`AppVersion` エンティティに署名フィールドが存在せず、サプライチェーン攻撃への防御が未実装だった。

具体的な問題:
- `StubCosignVerifier` が実装の代わりに `Ok(true)` を返す
- `AppVersion` エンティティに `cosign_signature` フィールドなし
- DB テーブル `app_versions` に署名カラムなし
- `create_version` ハンドラーが署名を受け取らない
- `AppState` に `CosignVerifier` が未組み込み

## 決定

### 実装方針: CLI サブプロセス方式

`sigstore-rs` / `cosign-rs` クレートは重量級の依存（TLS スタック・OCI レジストリクライアント等）を持つため、採用しない。代わりに `cosign verify-blob` CLI を子プロセスで呼び出す `SubprocessCosignVerifier` を実装する。

### 実装内容

1. **`CosignVerifier` トレイト**: `async fn verify(artifact: &str, signature: &str) -> anyhow::Result<bool>`
2. **`StubCosignVerifier`**: 開発・テスト用スタブ（変更なし）
3. **`SubprocessCosignVerifier`**: `cosign verify-blob --key {key_path} --signature {sig_file} {artifact}` を実行
4. **`AppVersion.cosign_signature: Option<String>`**: Cosign 署名を保持するフィールド（Optional: 開発環境は省略可）
5. **DB マイグレーション**: `app_versions.cosign_signature TEXT`（マイグレーション `007`）
6. **`CreateVersionRequest.cosign_signature: Option<String>`**: API リクエストで署名を受け取る
7. **署名検証フロー**: `create_version` ハンドラーで `cosign_verifier.verify(checksum_sha256, signature)` を呼び出す
8. **設定**: `cosign.verify_enabled: bool`（デフォルト `false`）+ `cosign.public_key_path: String`
9. **AppState**: `cosign_verifier: Arc<dyn CosignVerifier>` を追加

### 署名検証のレスポンス

| 状況 | HTTP ステータス | エラーコード |
|------|---------------|------------|
| 署名なし | 通常登録 | — |
| 署名あり・検証成功 | 201 Created | — |
| 署名あり・検証失敗 | 422 Unprocessable Entity | `SYS_APPS_SIGNATURE_INVALID` |
| 検証エラー | 500 Internal Server Error | `SYS_APPS_SIGNATURE_VERIFY_ERROR` |

## 理由

- **CLI ラッパー方式の採用**: sigstore-rs は数十の依存を追加し、ビルド時間・バイナリサイズを大幅に増加させる。本プロジェクトでは既に `cosign` バイナリが CI/CD 環境にインストールされている前提があるため、CLI ラッパーで十分
- **`Optional<String>` 型**: 段階的移行を可能にするため。既存の未署名バージョンとの互換性を維持しつつ、将来的に必須化できる
- **設定ベースの切り替え**: `cosign.verify_enabled` により、開発・ステージング・本番で動作を変えられる

## 影響

**ポジティブな影響**:
- 悪意のあるバイナリがリリースされるサプライチェーン攻撃を検出・ブロックできる
- 署名情報が DB に永続化されるため、事後監査が可能
- 既存の sigstore エコシステムと互換性あり

**ネガティブな影響・トレードオフ**:
- 本番環境に `cosign` バイナリのインストールが必要
- 一時ファイル (`/tmp/cosign-sig-*.sig`) の書き込みが必要（Docker `tmpfs` マウントで対応）
- 署名なしでの登録を許可する（Optional のため）— 将来的に必須化を検討

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| sigstore-rs クレート | Rust ネイティブな Cosign 実装 | 重量級依存、ビルド時間増大 |
| HTTP Rekor API | 透明性ログへの直接クエリ | Rekor への外部依存が増える、実装複雑 |
| `cosign_signature` を必須化 | `Option<String>` ではなく `String` | 段階的移行ができない、既存クライアントが壊れる |

## 参考

- [app-registry implementation.md](../../servers/system/app-registry/implementation.md)
- [app-registry database.md](../../servers/system/app-registry/database.md)
- 外部技術監査報告書 2026-04-01: STATIC-CRITICAL-002
- Sigstore プロジェクト: https://www.sigstore.dev/

## 更新履歴

| 日付 | 変更内容 | 変更者 |
|------|---------|--------|
| 2026-04-01 | 初版作成（外部監査 STATIC-CRITICAL-002 対応） | @kiso ryuhei |
