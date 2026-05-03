# =============================================================================
# k1s0 トップレベル Makefile（軽量タスクランナー）
#
# 設計: plan/03_Contracts実装/01_buf設定.md（主作業 6: Makefile / just / mise 統合）
#       plan/02_開発環境整備/05_pre-commit_hooks有効化.md
#       plan/02_開発環境整備/12_コミットメッセージ規約.md
#
# 提供する target は最小限。複雑な build / test は各言語の native build tool
# (cargo / go / dotnet / pnpm) に任せ、本 Makefile は **横断的な orchestration** のみ。
# =============================================================================

.DEFAULT_GOAL := help
.PHONY: help codegen codegen-check openapi openapi-check grpc-docs grpc-docs-check lint pre-commit lint-proto verify verify-quick verify-cones verify-conformance verify-portability doctor clean \
	e2e-owner-full e2e-owner-platform e2e-owner-observability e2e-owner-security e2e-owner-ha-dr e2e-owner-upgrade e2e-owner-sdk-roundtrip e2e-owner-tier3-web e2e-owner-perf \
	e2e-user-smoke e2e-user-full

help: ## このヘルプを表示
	@awk 'BEGIN {FS = ":.*##"; printf "Usage:\n  make \033[36m<target>\033[0m\n\nTargets:\n"} /^[a-zA-Z_-]+:.*?##/ { printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2 }' $(MAKEFILE_LIST)

codegen: ## buf generate で 4 言語 SDK を一括生成
	@./tools/codegen/buf/run.sh

codegen-check: ## buf generate 後の差分検出（CI 用）
	@./tools/codegen/buf/run.sh --check

openapi: ## proto から OpenAPI v2（Swagger）を docs/ に export
	@./tools/codegen/openapi/run.sh

openapi-check: ## OpenAPI 生成後の差分検出（CI 用）
	@./tools/codegen/openapi/run.sh --check

grpc-docs: ## proto から gRPC reference docs（Markdown）を docs/ に生成
	@./tools/codegen/grpc-docs/run.sh

grpc-docs-check: ## gRPC docs 生成後の差分検出（CI 用）
	@./tools/codegen/grpc-docs/run.sh --check

lint-proto: ## buf lint
	@./tools/codegen/buf/run.sh --lint

pre-commit: ## pre-commit run --all-files
	@pre-commit run --all-files

doctor: ## Dev Container の役別 toolchain 診断（自動 role 検出）
	@./tools/devcontainer/doctor.sh

verify-cones: ## sparse-checkout cone 定義の構文・整合性検証（10 役）
	@./tools/sparse/verify.sh

lint: lint-proto pre-commit ## proto + pre-commit を一括 lint

verify: ## CI と同等の検査を全 tier / 全言語で実行（push 前の最終ゲート）
	@./tools/ci/verify-local.sh full

verify-quick: ## origin/main からの差分スコープのみ verify（高速イテレーション用）
	@./tools/ci/verify-local.sh quick

verify-conformance: ## CNCF Conformance（Sonobuoy）実行（kind multi-node + Calico、所要 60-120 分）
	@./tools/qualify/conformance/run.sh

verify-portability: ## L6 portability 検証（multipass + kubeadm + Calico、kind 以外の vanilla K8s 実装で 3-node cluster を立てて Ready 確認）
	@./tools/qualify/portability/run.sh

# =============================================================================
# e2e テスト target（ADR-TEST-008 owner / user 二分構造）
# 詳細仕様: docs/05_実装/30_CI_CD設計/35_e2e_test_design/
# =============================================================================

# owner suite: 48GB host 専用、host OS の WSL2 native shell から実行（multipass 制約）
# 全 8 部位は cluster 既存利用が前提（make e2e-owner-full は cluster 起動 + 全件 + cleanup）
E2E_OWNER_DATE := $(shell date -u +%Y-%m-%d)
E2E_OWNER_DIR := tests/.owner-e2e/$(E2E_OWNER_DATE)
E2E_OWNER_GO_FLAGS := -tags=owner_e2e -v -json -count=1

e2e-owner-full: ## owner suite 全 8 部位実行 — multipass × 5 起動 + 全件 + cleanup（約 1 時間 45 分、48GB host 専用）
	@./tools/e2e/owner/up.sh
	@trap './tools/e2e/owner/down.sh' EXIT; \
	  mkdir -p $(E2E_OWNER_DIR); \
	  cd tests/e2e/owner && \
	    go test $(E2E_OWNER_GO_FLAGS) -timeout=120m ./... \
	    | tee ../../.owner-e2e/$(E2E_OWNER_DATE)/full-result.json
	@./tools/qualify/owner-e2e/archive.sh $(E2E_OWNER_DATE)
	@./tools/qualify/owner-e2e/update-results.sh $(E2E_OWNER_DATE)

e2e-owner-platform: ## owner platform/ のみ実行（既存 cluster 前提、約 8 分）
	@./tools/e2e/owner/check.sh
	@mkdir -p $(E2E_OWNER_DIR)/platform
	@cd tests/e2e/owner && go test $(E2E_OWNER_GO_FLAGS) -timeout=10m ./platform/... \
	  | tee ../../.owner-e2e/$(E2E_OWNER_DATE)/platform/result.json

e2e-owner-observability: ## owner observability/ のみ（5 検証、約 5 分）
	@./tools/e2e/owner/check.sh
	@mkdir -p $(E2E_OWNER_DIR)/observability
	@cd tests/e2e/owner && go test $(E2E_OWNER_GO_FLAGS) -timeout=15m ./observability/... \
	  | tee ../../.owner-e2e/$(E2E_OWNER_DATE)/observability/result.json

e2e-owner-security: ## owner security/ のみ（Kyverno / NetPol / SPIRE / mTLS / CVE、約 5 分）
	@./tools/e2e/owner/check.sh
	@mkdir -p $(E2E_OWNER_DIR)/security
	@cd tests/e2e/owner && go test $(E2E_OWNER_GO_FLAGS) -timeout=10m ./security/... \
	  | tee ../../.owner-e2e/$(E2E_OWNER_DATE)/security/result.json

e2e-owner-ha-dr: ## owner ha-dr/ のみ（HA fail-over + 4 経路 DR 復旧、約 15 分）
	@./tools/e2e/owner/check.sh
	@mkdir -p $(E2E_OWNER_DIR)/ha-dr
	@cd tests/e2e/owner && go test $(E2E_OWNER_GO_FLAGS) -timeout=20m ./ha-dr/... \
	  | tee ../../.owner-e2e/$(E2E_OWNER_DATE)/ha-dr/result.json

e2e-owner-upgrade: ## owner upgrade/ のみ（kubeadm N→N+1 minor upgrade、約 30 分）
	@./tools/e2e/owner/check.sh
	@mkdir -p $(E2E_OWNER_DIR)/upgrade
	@cd tests/e2e/owner && go test $(E2E_OWNER_GO_FLAGS) -timeout=40m ./upgrade/... \
	  | tee ../../.owner-e2e/$(E2E_OWNER_DATE)/upgrade/result.json

e2e-owner-sdk-roundtrip: ## owner sdk-roundtrip/ のみ（4 言語 × 12 RPC = 48 cross-product、約 12 分）
	@./tools/e2e/owner/check.sh
	@mkdir -p $(E2E_OWNER_DIR)/sdk-roundtrip
	@cd tests/e2e/owner && go test $(E2E_OWNER_GO_FLAGS) -timeout=20m ./sdk-roundtrip/... \
	  | tee ../../.owner-e2e/$(E2E_OWNER_DATE)/sdk-roundtrip/result.json

e2e-owner-tier3-web: ## owner tier3-web/ のみ（chromedp で headless Chrome 駆動、約 8 分）
	@./tools/e2e/owner/check.sh
	@mkdir -p $(E2E_OWNER_DIR)/tier3-web
	@cd tests/e2e/owner && go test $(E2E_OWNER_GO_FLAGS) -timeout=15m ./tier3-web/... \
	  | tee ../../.owner-e2e/$(E2E_OWNER_DATE)/tier3-web/result.json

e2e-owner-perf: ## owner perf/ のみ（k6 spawn ラップ、約 10 分）
	@./tools/e2e/owner/check.sh
	@mkdir -p $(E2E_OWNER_DIR)/perf
	@cd tests/e2e/owner && go test $(E2E_OWNER_GO_FLAGS) -timeout=15m ./perf/... \
	  | tee ../../.owner-e2e/$(E2E_OWNER_DATE)/perf/result.json

# user suite: 16GB host OK、devcontainer 内可、PR + nightly CI で機械検証
# target 毎に cluster を新規起動 + cleanup する設計（kind 起動コストが小さいため）
E2E_USER_DATE := $(shell date -u +%Y-%m-%d)
E2E_USER_DIR := tests/.user-e2e/$(E2E_USER_DATE)
E2E_USER_GO_FLAGS := -tags=user_e2e -v -json -count=1 -parallel=4

e2e-user-smoke: ## user smoke/ のみ（PR 5 分以内、kind 起動 + minimum stack + smoke + cleanup）
	@./tools/e2e/user/up.sh
	@trap './tools/e2e/user/down.sh' EXIT; \
	  mkdir -p $(E2E_USER_DIR)/smoke; \
	  cd tests/e2e/user && \
	    go test $(E2E_USER_GO_FLAGS) -timeout=10m ./smoke/... \
	    | tee ../../.user-e2e/$(E2E_USER_DATE)/smoke/result.json

e2e-user-full: ## user smoke + examples 全件（nightly 30〜45 分）
	@./tools/e2e/user/up.sh
	@trap './tools/e2e/user/down.sh' EXIT; \
	  mkdir -p $(E2E_USER_DIR); \
	  cd tests/e2e/user && \
	    go test $(E2E_USER_GO_FLAGS) -timeout=45m ./... \
	    | tee ../../.user-e2e/$(E2E_USER_DATE)/full-result.json

clean: ## Python / pytest キャッシュのみ削除（Rust target/ は cargo clean を使うこと）
	@# Rust の target/ は容量大かつ再ビルド負荷大のため、本 target からは除外。
	@# 必要時は各 Cargo workspace で `cargo clean` を実行する。
	@find . -type d \( -name "__pycache__" -o -name ".pytest_cache" -o -name ".mypy_cache" -o -name ".ruff_cache" \) -prune -exec rm -rf {} + 2>/dev/null || true
	@echo "  cleaned: __pycache__ / .pytest_cache / .mypy_cache / .ruff_cache"
