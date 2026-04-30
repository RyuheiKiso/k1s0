# TaxCalculator Dockerfile（multi-stage）。
# build context: docker build -f services/TaxCalculator/Dockerfile src/tier2/dotnet/

FROM mcr.microsoft.com/dotnet/sdk:8.0 AS builder
WORKDIR /workspace
COPY Directory.Build.props Directory.Packages.props nuget.config ./
COPY services/TaxCalculator/ ./services/TaxCalculator/
COPY shared/ ./shared/
RUN dotnet restore services/TaxCalculator/TaxCalculator.sln
RUN dotnet publish services/TaxCalculator/src/TaxCalculator.Api/TaxCalculator.Api.csproj \
    -c Release -o /publish --no-restore

FROM mcr.microsoft.com/dotnet/aspnet:8.0-bookworm-slim AS runtime
USER app
WORKDIR /app
ENV ASPNETCORE_URLS=http://0.0.0.0:8080 \
    DOTNET_ENVIRONMENT=Production
COPY --from=builder /publish .
EXPOSE 8080
ENTRYPOINT ["dotnet", "K1s0.Tier2.TaxCalculator.Api.dll"]
