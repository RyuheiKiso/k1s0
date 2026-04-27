# tier2 .NET サービス Golden Path 完動例の Dockerfile（multi-stage、distroless dotnet）。
FROM mcr.microsoft.com/dotnet/sdk:8.0 AS builder
WORKDIR /workspace
# csproj だけ先にコピーして restore（キャッシュ効率最大化、ProjectReference 含む）
COPY examples/tier2-dotnet-service/ExampleDotnetService.csproj examples/tier2-dotnet-service/
COPY src/sdk/dotnet/Directory.Build.props src/sdk/dotnet/
COPY src/sdk/dotnet/src/ src/sdk/dotnet/src/
RUN dotnet restore examples/tier2-dotnet-service/ExampleDotnetService.csproj

# 全ソースをコピーして publish
COPY examples/tier2-dotnet-service/ examples/tier2-dotnet-service/
RUN dotnet publish examples/tier2-dotnet-service/ExampleDotnetService.csproj \
    -c Release -o /out --no-restore

# 実行ステージ（distroless aspnet ランタイム）
FROM mcr.microsoft.com/dotnet/aspnet:8.0-distroless
WORKDIR /app
COPY --from=builder /out .
USER 1654:1654
EXPOSE 8080
ENV ASPNETCORE_URLS=http://+:8080
ENTRYPOINT ["dotnet", "K1s0.Examples.Tier2DotnetService.dll"]
