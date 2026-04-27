# ApprovalFlow .NET サービスの multi-stage Dockerfile。
#
# docs 正典:
#   docs/05_実装/00_ディレクトリ設計/30_tier2レイアウト/02_dotnet_solution配置.md
#
# build context 要件:
#   docker build は src/tier2/dotnet/ をルートとして実行する前提で書いている。
#   CI / 開発者は次のコマンドで build する:
#     docker build -f src/tier2/dotnet/services/ApprovalFlow/Dockerfile src/tier2/dotnet/

# stage 1: build。
FROM mcr.microsoft.com/dotnet/sdk:8.0 AS builder

# 作業ディレクトリ。
WORKDIR /workspace

# Directory.Build.props / Directory.Packages.props / nuget.config を先に COPY。
COPY Directory.Build.props Directory.Packages.props nuget.config ./

# サービス本体と shared を COPY する。
COPY services/ApprovalFlow/ ./services/ApprovalFlow/
COPY shared/ ./shared/

# 依存を解決する。
RUN dotnet restore services/ApprovalFlow/ApprovalFlow.sln

# Api 層を publish する。
RUN dotnet publish services/ApprovalFlow/src/ApprovalFlow.Api/ApprovalFlow.Api.csproj \
    -c Release \
    -o /publish \
    --no-restore \
    -p:PublishReadyToRun=true

# stage 2: runtime（aspnet 8.0 minimal）。
FROM mcr.microsoft.com/dotnet/aspnet:8.0-bookworm-slim AS runtime

# 非 root user（aspnet image は app user UID=1654 を提供）。
USER app

# 作業ディレクトリ。
WORKDIR /app

# Kestrel の listen URL を Compose / K8s から override 可能にしておく。
ENV ASPNETCORE_URLS=http://0.0.0.0:8080 \
    DOTNET_ENVIRONMENT=Production

# build artifact を取り込む。
COPY --from=builder /publish .

# 公開ポート。
EXPOSE 8080

# エントリポイント。
ENTRYPOINT ["dotnet", "K1s0.Tier2.ApprovalFlow.Api.dll"]
