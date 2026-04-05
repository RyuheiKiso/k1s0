# このディレクトリについて

M-030 監査対応: buf generate を実行して Rust コードを生成してください。

```bash
cd api/proto
buf generate
```

service/business tier の Rust サーバーは `build.rs` でローカル生成を使用しているため、
このディレクトリのファイルは現在空です。統一的なコード管理については ADR-0079 を参照してください。
