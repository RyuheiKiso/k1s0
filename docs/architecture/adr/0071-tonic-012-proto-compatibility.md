# ADR-0071: tonic 0.12 互換のための Proto 生成コード対応方針

## ステータス

承認済み

## コンテキスト

`api/proto/gen/rust/` 配下には buf によって事前生成された Rust 向け protobuf
コードが格納されている。このコードは tonic 0.13 API を前提とした以下の
シンボルを参照している:

- `tonic_prost::ProstCodec`
- `tonic::body::Body`

しかし Rust workspace の `Cargo.toml` では `tonic = "0.12"` を使用しており、
tonic 0.13 で新設・変更されたシンボルが見つからないためビルドが失敗していた。
特に graphql-gateway は生成済み proto コードを利用しており、このサービスの
ビルドが完全に停止した状態であった。

tonic 0.12 と 0.13 の間には以下の API 変更がある:

| tonic 0.13 (生成コード) | tonic 0.12 (workspace) |
|------------------------|------------------------|
| `tonic_prost::ProstCodec` | `tonic::codec::ProstCodec` |
| `tonic::body::Body` | `tonic::body::BoxBody` |

buf.gen.yaml の neoeinstein-tonic プラグインはバージョンが固定されておらず、
最新版（0.13 互換）が使用されていたことが不一致の原因であった。

## 決定

全面的な tonic 0.13 へのアップグレードは現時点では行わず、生成済み proto コードを
tonic 0.12 互換に修正する方針を採用する。具体的な対応:

1. **生成済みコードの修正**:
   - `tonic_prost::ProstCodec` → `tonic::codec::ProstCodec` に置換
   - `tonic::body::Body` → `tonic::body::BoxBody` に置換

2. **buf.gen.yaml のバージョンピン留め**:
   - neoeinstein-tonic プラグインのバージョンを tonic 0.12 互換版（v0.3.0）に明示固定する
   - 再生成時に同様の不一致が再発しないようにする

tonic 0.13 への全面移行は別途 ADR を立案して計画的に実施する。

## 理由

tonic 0.13 への全面アップグレードを選択しなかった理由:

- workspace 内の全サービス（Go/Rust 双方）で tonic 依存の互換性検証が必要
- Rust サービスは graphql-gateway 以外にも複数あり、全サービスの結合テストが必要
- 現時点でのアップグレードは影響範囲が広く、工数・リスクが大きい

生成済みコード修正を選択した理由:

- 即時ビルド失敗を最小限の変更で解消できる
- tonic 0.12 の API は安定しており、修正内容が明確で限定的
- buf.gen.yaml のバージョンをピン留めすることで将来の再発を防止できる
- tonic 0.13 移行を別 ADR で計画的に実施できる

## 影響

**ポジティブな影響**:

- graphql-gateway のビルド失敗を即時解消できる
- buf.gen.yaml のバージョンピン留めにより、再生成後も一貫性を保てる
- tonic 0.13 移行の計画・実施を焦らず実行できる

**ネガティブな影響・トレードオフ**:

- 生成ファイルを手動修正する必要があり、buf による再生成と乖離が生じる
  （ただし buf.gen.yaml のバージョンピン留めにより、再生成後も同じ出力が得られる）
- tonic 0.12 は将来的にサポート終了になるため、移行 ADR を別途立案する必要がある
- buf プラグインのバージョンをピン留めすることで、セキュリティパッチの適用が
  手動管理になる

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| tonic 0.13 へ全面アップグレード | workspace の `tonic` を 0.13 に更新し、全サービスを対応させる | 全サービスの互換性検証・統合テストが必要で工数が多い。ビルド失敗の即時解消を妨げる |
| 生成コードを workspace から除外 | `api/proto/gen/rust/` を .gitignore に追加し、CI でビルド時に毎回生成する | CI ビルド時間が増加する。生成ツールのバージョン管理が複雑になる |

## 参考

- [tonic 0.12 リリースノート](https://github.com/hyperium/tonic/releases/tag/v0.12.0)
- [tonic 0.13 変更点](https://github.com/hyperium/tonic/releases/tag/v0.13.0)
- [neoeinstein-tonic buf プラグイン](https://buf.build/neoeinstein/tonic)
- [ADR-0006: Protobuf バージョニング戦略](./0006-proto-versioning.md)

## 更新履歴

| 日付 | 変更内容 | 変更者 |
|------|---------|--------|
| 2026-04-02 | 初版作成 | kiso ryuhei |
