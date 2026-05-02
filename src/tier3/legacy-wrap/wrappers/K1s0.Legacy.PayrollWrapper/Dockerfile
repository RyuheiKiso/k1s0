# K1s0.Legacy.PayrollWrapper の Linux container 用 Dockerfile。
#
# docs 正典:
#   docs/05_実装/00_ディレクトリ設計/40_tier3レイアウト/05_レガシーラップ配置.md
#
# 注意:
#   Sidecar (Windows container, Windows Node 必須) と異なり、本 wrapper は
#   .NET 8 / Linux container で動作するため、通常の k1s0 Linux Node に乗る。
#   Windows Node 依存からの脱出が wrapper 化の最大の意義。

# stage 1: build。
FROM mcr.microsoft.com/dotnet/sdk:8.0 AS builder
WORKDIR /workspace
COPY . .
RUN dotnet restore K1s0.Legacy.PayrollWrapper.csproj
RUN dotnet publish K1s0.Legacy.PayrollWrapper.csproj \
    -c Release \
    -o /publish \
    --no-restore

# stage 2: runtime（nonroot 化）。
FROM mcr.microsoft.com/dotnet/aspnet:8.0 AS runtime
WORKDIR /app
COPY --from=builder /publish ./
USER 1000:1000
ENV ASPNETCORE_URLS=http://+:8080
EXPOSE 8080
ENTRYPOINT ["dotnet", "K1s0.Legacy.PayrollWrapper.dll"]
