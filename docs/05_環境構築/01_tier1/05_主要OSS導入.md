---
id: env.tier1.tier1_oss_install
axis: tier1
phase: env_setup
kind: enforcement
status: draft
depends_on:
  - env.tier1.tier1_repository_acquisition
covered_by:
  defense_in_depth_layers: [B]
  proof_classes: []
---

# 主要 OSS 導入

## 一文方針

- tier1 の proto / schema エコシステムである Buf CLI / protoc / Apicurio Registry / ts-proto を導入し、`buf --version` と `protoc --version` が応答することを本ページの検収とする。

## Buf CLI

Go ツールチェーンでインストールする。

```bash
go install github.com/bufbuild/buf/cmd/buf@latest
buf --version   # 1.x.x
```

`$(go env GOPATH)/bin` が `PATH` に含まれていることを確認する。

## protoc

```bash
sudo apt update
sudo apt install -y protobuf-compiler
protoc --version   # libprotoc 3.x 以上
```

## Apicurio Registry（Docker）

Schema Registry として Apicurio を使用する。

```bash
docker pull apicurio/apicurio-registry-mem:latest
docker run -d -p 8080:8080 --name apicurio apicurio/apicurio-registry-mem:latest
# 起動確認
curl http://localhost:8080/apis/registry/v2/system/info
```

本番では docker compose で Kafka / postgres と合わせて起動する。

## pnpm + ts-proto

TypeScript の gRPC stub 生成に使用する。

```bash
pnpm add -g ts-proto
# または proto 生成を buf gen で行う場合
pnpm add -D ts-proto
```

## protoc-gen-go / protoc-gen-go-grpc（Go stub 生成）

```bash
go install google.golang.org/protobuf/cmd/protoc-gen-go@latest
go install google.golang.org/grpc/cmd/protoc-gen-go-grpc@latest
```

## protoc-gen-csharp（C# stub 生成）

C# / gRPC は `Grpc.Tools` NuGet パッケージが `dotnet build` 時に自動生成する。追加インストール不要。

## 検収コマンド

```bash
buf --version
protoc --version
docker ps | grep apicurio
```

## 関連参照

- [04_リポジトリ取得手順](04_リポジトリ取得手順.md)
- [06_開発エディタIDE設定](06_開発エディタIDE設定.md)
- [11_軸固有環境設定](11_軸固有環境設定.md)
