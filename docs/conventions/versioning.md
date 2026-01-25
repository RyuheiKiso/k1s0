# バージョニング規約

本ドキュメントは、k1s0 におけるバージョニングと互換性ポリシーを定義する。

## 1. 単一バージョン管理

k1s0 はリポジトリ全体で **単一バージョン（SemVer）** を持つ。

**バージョンファイル:**
```
k1s0-version.txt
```

**対象:**
- CLI（`k1s0` バイナリ）
- k1s0-generator crate
- templates（backend-rust, frontend-react 等）
- framework crates（k1s0-config, k1s0-error 等）
- framework services（auth-service, config-service 等）

**目的:**
- 導入・アップグレード時に整合する組み合わせを常に 1 つにする
- 複数バージョンの管理による複雑性を回避する

## 2. SemVer の定義

### MAJOR（破壊的変更）

後方互換性がない変更。

例:
- 公開 API の破壊（HTTP/gRPC/GraphQL の契約変更）
- framework crates の公開 API 破壊（公開関数/型の削除）
- テンプレートの構造的変更（既存サービスがビルド不能になる）
- DB の破壊（列削除・型変更）
- manifest.json のスキーマ破壊的変更

### MINOR（機能追加）

後方互換性がある機能追加。

例:
- 任意フィールドの追加
- エンドポイントの追加
- `#[deprecated]` の導入
- 新しいテンプレートの追加
- manifest.json への任意キー追加

### PATCH（不具合修正）

後方互換性がある不具合修正・小改善。

例:
- バグ修正
- 互換性維持の依存更新
- テンプレートの軽微な安全修正
- ドキュメント修正

## 3. バージョン更新のルール

### CLI でのバージョン参照

```rust
// k1s0-version.txt を埋め込み
const VERSION: &str = include_str!("../../../k1s0-version.txt").trim();
```

### リリース時

1. `k1s0-version.txt` を更新
2. CHANGELOG.md を更新
3. Git タグを作成（`v{version}`）
4. CI でリリースビルドを実行

## 4. 破壊的変更の手順

破壊的変更（MAJOR）は原則禁止。必要な場合は以下を必須とする：

1. **ADR**: `docs/adr/ADR-XXXX-*.md`
   - なぜ必要か
   - 代替案
   - 影響範囲
   - 移行計画

2. **UPGRADE.md** または **リリースノート**
   - 影響一覧
   - 移行手順
   - ロールバック方法

3. **CLI 支援**
   - `k1s0 upgrade --check` が破壊箇所を検知できること
   - `k1s0 upgrade` が安全に止まれること

## 5. manifest.json のスキーマバージョン

manifest.json 自体のスキーマ変更を管理するため、`schema_version` を導入。

**互換性維持:**
- MINOR: 任意キーの追加（既存 manifest は有効）
- MAJOR: 必須キーの追加・削除・型変更

**マイグレーション:**
- CLI は古い `schema_version` の manifest を読み込み可能にする
- 必要に応じてマイグレーションを自動実行

## 6. framework crates の互換性

### 公開 API

- `pub` で公開される型/関数/trait は SemVer の互換性対象
- `pub(crate)` や非公開モジュールは互換性対象外

### 非推奨化

```rust
#[deprecated(since = "0.2.0", note = "Use `new_function` instead")]
pub fn old_function() { ... }
```

- 削除ではなく `#[deprecated]` で段階移行
- 移行期間（少なくとも 1 MINOR）を設ける
- MAJOR リリース時のみ削除可能

### 実験的 API

```rust
#[cfg(feature = "experimental")]
pub fn experimental_function() { ... }
```

- `experimental` feature flag で opt-in
- 互換性は保証しない

## 7. テンプレートの互換性

### managed_paths

- CLI が自動更新してよい領域
- MINOR でも変更可能（ただし既存ファイルを壊さない）

### protected_paths

- ビジネスロジック領域
- CLI は変更しない

### 構造的変更（MAJOR）

- ディレクトリ構成の変更
- 必須ファイルの追加・削除
- ADR + UPGRADE.md + CLI 支援が必須

## 関連ドキュメント

- [ADR-0002](../adr/ADR-0002-versioning-and-manifest.md): バージョニングと manifest の型の固定
- [構想.md](../../work/構想.md): 8. framework のアップグレード支援
