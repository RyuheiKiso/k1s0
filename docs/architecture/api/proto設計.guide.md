# Proto 設計 ガイド

> **仕様**: サービス定義・メッセージ型・buf 設定は [proto設計.md](./proto設計.md) を参照。

## Protobuf / gRPC 採用の背景

### 選定理由

k1s0 のマイクロサービス間通信に Protobuf / gRPC を採用した理由は以下の通り。

| 目的 | 説明 |
| --- | --- |
| サービス間高速通信 | HTTP/2 ベースのバイナリプロトコルにより、REST API 比で低レイテンシ・高スループットを実現する |
| 型安全なインターフェース | Protobuf スキーマから Go / Rust / TypeScript のコードを自動生成し、型不一致を防止する |
| スキーマ進化の管理 | buf による lint・破壊的変更検出で、安全なスキーマ進化を保証する |
| Kafka イベントスキーマの統一 | メッセージング基盤のイベント型も Protobuf で定義し、Schema Registry で互換性を管理する |

REST API ではスキーマの逸脱が起きやすく、複数言語間でのインターフェース同期コストが高い。Protobuf による IDL ファーストアプローチにより、Go / Rust / TypeScript のコードを単一の `.proto` ファイルから生成し、一貫性を担保する。

### バージョニング戦略の設計意図

proto パッケージは [API設計.md](./API設計.md) D-009 の命名規則に従い、メジャーバージョンをパッケージ名に含める。

```
k1s0.{tier}.{domain}.v{major}
```

初期バージョンは `v1` とし、後方互換性を破壊する変更が必要な場合のみ `v2` パッケージを新設する。この方式を採用した理由は、gRPC のパッケージ名がクライアントコードのインポートパスに直接反映されるため、メジャーバージョンを明示的に分離することで、既存クライアントへの影響を完全にゼロにできるためである。

---

## Rust ローカルコード生成（tonic-build）の設計判断

Rust サーバーは `buf.gen.yaml` による一括生成とは別に、各サービスの `build.rs` で `tonic-build` を用いてローカルコード生成する方式を採用している。

### なぜ buf generate ではなく tonic-build なのか

- **ビルドシステム統合**: Cargo のビルドパイプラインに組み込まれるため、`cargo build` だけで proto コード生成が完了する
- **サーバー専用オプション**: `build_server(true)` + `build_client(false)` により、サーバー側に不要なクライアントコードを除外できる
- **proto パスの柔軟性**: `compile_protos` で参照先を指定できるため、`api/proto/` の共有 proto を直接参照可能

proto ファイルはすべて `api/proto/k1s0/system/` を参照する（`regions/system/proto/` は廃止済み）。

```rust
// build.rs（例: auth サーバー）
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .out_dir("src/proto")
        .compile_protos(
            &["../../../../../api/proto/k1s0/system/auth/v1/auth.proto"],
            &["../../../../../api/proto"],  // api/proto を include パスとして指定
        )?;
    Ok(())
}
```

---

## バージョニング・後方互換性の設計指針

### 後方互換の判断基準

proto の後方互換性はワイヤーフォーマット（バイナリエンコーディング）の互換性に基づいて判断する。proto3 ではフィールド番号がエンコーディングキーとなるため、フィールド名の変更はワイヤーフォーマットに影響しない。

#### 後方互換（バージョンアップ不要）

| 変更種別 | 説明 |
| --- | --- |
| フィールド追加 | 新しいフィールド番号で追加。既存のデシリアライズに影響なし |
| 新規 RPC メソッド追加 | サービス定義に新メソッドを追加。既存クライアントは影響なし |
| 新規 enum 値追加 | 既存の enum に新しい値を追加 |
| フィールド名変更 | ワイヤーフォーマットは番号ベースのため互換性維持 |

#### 後方互換性を破壊する変更（メジャーバージョンアップ）

| 変更種別 | 説明 |
| --- | --- |
| フィールドの削除・番号変更 | 既存のデシリアライズが失敗する |
| フィールドの型変更 | ワイヤーフォーマットが変わる |
| RPC メソッドのシグネチャ変更 | リクエスト/レスポンス型の変更 |
| メッセージ名の変更 | JSON マッピング・リフレクションに影響 |

### 削除時のフィールド番号予約

フィールドを削除する場合は `reserved` で番号を予約し、再利用を防止する。これにより、将来の開発者が同じ番号を異なる型で再利用してしまう事故を防ぐ。

```protobuf
message Example {
  reserved 2, 5;
  reserved "old_field_name";
  string id = 1;
  string name = 3;
}
```

### buf breaking による自動検証

CI パイプラインで `buf breaking` を実行し、意図しない破壊的変更を検出する。

```bash
# main ブランチとの比較
buf breaking api/proto --against '.git#branch=main'
```

破壊的変更が検出された場合は CI が失敗する。意図的な変更であれば新しいバージョンパッケージ（`v2`）として作成する。

---

## 関連ドキュメント

- [proto設計.md](./proto設計.md) -- サービス定義・メッセージ型・buf 設定（仕様）
- [API設計.md](./API設計.md) -- gRPC サービス定義パターン (D-009)・gRPC バージョニング (D-010)
- [認証認可設計.md](../auth/認証認可設計.md) -- JWT Claims 構造・RBAC ロール定義
- [メッセージング設計.md](../../architecture/messaging/メッセージング設計.md) -- Kafka トピック・イベントスキーマ
