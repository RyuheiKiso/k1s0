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
.PHONY: help codegen codegen-check lint pre-commit lint-proto verify-cones doctor clean

help: ## このヘルプを表示
	@awk 'BEGIN {FS = ":.*##"; printf "Usage:\n  make \033[36m<target>\033[0m\n\nTargets:\n"} /^[a-zA-Z_-]+:.*?##/ { printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2 }' $(MAKEFILE_LIST)

codegen: ## buf generate で 4 言語 SDK を一括生成
	@./tools/codegen/buf/run.sh

codegen-check: ## buf generate 後の差分検出（CI 用）
	@./tools/codegen/buf/run.sh --check

lint-proto: ## buf lint
	@./tools/codegen/buf/run.sh --lint

pre-commit: ## pre-commit run --all-files
	@pre-commit run --all-files

doctor: ## Dev Container の役別 toolchain 診断（自動 role 検出）
	@./tools/devcontainer/doctor.sh

verify-cones: ## sparse-checkout cone 定義の構文・整合性検証（10 役）
	@./tools/sparse/verify.sh

lint: lint-proto pre-commit ## proto + pre-commit を一括 lint

clean: ## Python / pytest キャッシュのみ削除（Rust target/ は cargo clean を使うこと）
	@# Rust の target/ は容量大かつ再ビルド負荷大のため、本 target からは除外。
	@# 必要時は各 Cargo workspace で `cargo clean` を実行する。
	@find . -type d \( -name "__pycache__" -o -name ".pytest_cache" -o -name ".mypy_cache" -o -name ".ruff_cache" \) -prune -exec rm -rf {} + 2>/dev/null || true
	@echo "  cleaned: __pycache__ / .pytest_cache / .mypy_cache / .ruff_cache"
