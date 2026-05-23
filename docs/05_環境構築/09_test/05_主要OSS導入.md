---
id: env.test.test_oss_install
axis: test
phase: env_setup
kind: enforcement
status: draft
depends_on:
  - env.test.test_repository_acquisition
covered_by:
  defense_in_depth_layers: [B]
  proof_classes: []
---

# 主要 OSS 導入

## 一文方針

- test 軸エンジニアは pytest + Hypothesis / Vitest + fast-check / Pact broker（docker）/ Playwright / k6 / Litmus CLI / pitest（Maven）/ mutmut / Stryker / cargo-mutants / AFL++ / Testcontainers を導入し、全ツールの `--version` 応答またはコンテナ起動確認が取れることを本ページの検収条件とする。

> **pre-P0 注記**: `src/` は P10 deliverable（pre-P0 時点で実体ゼロ）。以下の手順は P10 完了後に有効。

## pytest + Hypothesis

03_必須ランタイム でインストール済み。確認コマンド:

```bash
pytest --version
python3 -c "import hypothesis; print('Hypothesis', hypothesis.__version__)"
```

## Vitest + fast-check

```bash
pnpm add -D vitest fast-check
pnpm vitest --version
node -e "const fc = require('fast-check'); console.log('fast-check OK')"
```

## Pact broker（docker）

```bash
# Pact broker + PostgreSQL をコンテナで起動
docker pull pactfoundation/pact-broker:latest
docker pull postgres:15-alpine

# docker compose で起動
cat > /tmp/pact-compose.yml << 'EOF'
version: "3.9"
services:
  postgres:
    image: postgres:15-alpine
    environment:
      POSTGRES_USER: pact
      POSTGRES_PASSWORD: pact
      POSTGRES_DB: pact
  pact-broker:
    image: pactfoundation/pact-broker:latest
    ports:
      - "9292:9292"
    environment:
      PACT_BROKER_DATABASE_URL: postgres://pact:pact@postgres/pact
      PACT_BROKER_BASIC_AUTH_USERNAME: pact
      PACT_BROKER_BASIC_AUTH_PASSWORD: pact
    depends_on:
      - postgres
EOF
docker compose -f /tmp/pact-compose.yml up -d
```

## Playwright

```bash
pnpm add -D @playwright/test
pnpm playwright install --with-deps
pnpm playwright --version
```

## k6

```bash
# apt 経由
sudo gpg --no-default-keyring --keyring /usr/share/keyrings/k6-archive-keyring.gpg \
  --keyserver hkp://keyserver.ubuntu.com:80 \
  --recv-keys C5AD17C747E3415A3642D57D77C6C491D6AC1D69
echo "deb [signed-by=/usr/share/keyrings/k6-archive-keyring.gpg] https://dl.k6.io/deb stable main" | \
  sudo tee /etc/apt/sources.list.d/k6.list
sudo apt update && sudo apt install -y k6
k6 version
```

## Litmus CLI

03_必須ランタイム でインストール済み。確認コマンド:

```bash
litmus version
```

## pitest（Maven）

pitest は Maven plugin として使うため独立インストール不要。`pom.xml` に追加する。

```xml
<plugin>
  <groupId>org.pitest</groupId>
  <artifactId>pitest-maven</artifactId>
  <version>1.15.0</version>
  <configuration>
    <mutationThreshold>80</mutationThreshold>
  </configuration>
</plugin>
```

```bash
mvn org.pitest:pitest-maven:mutationCoverage
```

## mutmut（Python mutation）

```bash
pip install mutmut
mutmut --version
```

## Stryker（JavaScript mutation）

```bash
pnpm add -D @stryker-mutator/core @stryker-mutator/vitest-runner
pnpm stryker --version
```

## cargo-mutants（Rust mutation）

03_必須ランタイム でインストール済み。確認コマンド:

```bash
cargo-mutants --version
```

## AFL++（fuzzing）

```bash
cargo install afl
cargo afl --version
```

## Testcontainers

```bash
pip install testcontainers
python3 -c "import testcontainers; print('Testcontainers OK')"
```

## 検収コマンド

```bash
pytest --version
pnpm vitest --version
pnpm playwright --version
k6 version
litmus version
cargo-mutants --version
mutmut --version
```

## 関連参照

- [04_リポジトリ取得手順](04_リポジトリ取得手順.md)
- [06_開発エディタIDE設定](06_開発エディタIDE設定.md)
- [07_テスト検証環境](07_テスト検証環境.md)
