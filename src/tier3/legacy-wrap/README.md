# tier3 legacy-wrap (.NET Framework 4.8 ラッパー)

採用側組織の既存 .NET Framework 4.8 資産を k1s0 基盤に取り込むためのラッパー群。`Sidecar` と `Wrapper` の 2 形態をサポートする。

## docs 正典

- 配置: `docs/05_実装/00_ディレクトリ設計/40_tier3レイアウト/05_レガシーラップ配置.md`
- ADR: `docs/02_構想設計/adr/ADR-MIG-001-net-framework-sidecar.md`

## レイアウト

```text
src/tier3/legacy-wrap/
├── README.md
├── LegacyWrap.sln
├── Directory.Build.props
├── sidecars/
│   └── K1s0.Legacy.Sidecar/        # ASP.NET Web API（.NET Framework 4.8）
└── migration-guide/
    ├── README.md
    └── steps/
        ├── 01_sidecar_setup.md
        ├── 02_gradual_api_exposure.md
        └── 03_framework_to_net8.md
```

`wrappers/` 配下（K1s0.Legacy.PayrollWrapper など、.NET 8 への完全移行プロジェクト）は採用後の運用拡大時 で追加する。

## Sidecar パターンの構造

既存 .NET Framework プロセスは変更せず、その隣に Dapr sidecar container が同 Pod 内で動作する。.NET Framework 側は `K1s0.Legacy.Sidecar` プロジェクトの HTTP 薄ラッパーを通して Dapr の HTTP API（`localhost:3500`）を呼び、tier1 公開 API（State / PubSub / Secrets 等）にアクセスする。

```text
[既存 .NET Framework App]
        │ HTTP
        ▼
[K1s0.Legacy.Sidecar] (Web API on net48, IIS / IIS Express)
        │ HTTP
        ▼
[Dapr sidecar]  (localhost:3500、自動 inject)
        │ gRPC
        ▼
[k1s0 tier1 公開 API]
```

## ビルド

```bash
# Windows 環境（msbuild 必須）。
msbuild LegacyWrap.sln /p:Configuration=Release

# Windows container ビルド。
docker build -f sidecars/K1s0.Legacy.Sidecar/Dockerfile.windows -t ghcr.io/k1s0/legacy-sidecar:dev .
```

## 関連 ID

- IMP-DIR-T3-060
- ADR-MIG-001（.NET Framework sidecar）
- ADR-MIG-002（API Gateway）
- NFR-D-MIG-\* / 制約 8
