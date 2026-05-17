# k1s0 monorepo root Makefile
# 全ターゲットは直列実行を想定

.PHONY: all generate validate lint test \
        kind-up kind-down \
        deploy-kyverno deploy-cnpg deploy-apicurio \
        drill-topology drill-clock drill-restore \
        release-gate help

# リポジトリルート絶対パス
REPO_ROOT := $(shell pwd)

# uv 経由で依存を揃えてから実行するプレフィックス
UV := uv run --with pyyaml --with jsonschema --with click --with rich --no-project

# デフォルトターゲット: 生成 → 検証 → lint → テスト
all: generate validate lint test

# lock.yaml 全生成（トポロジカルソート順）
generate:
	$(UV) python -m tools.lock_yaml_generator.cli all

# bit-for-bit 再現性確認（生成結果が既存ファイルと一致するか）
validate:
	$(UV) python -m tools.lock_yaml_generator.cli all --check-only

# ドキュメント lint（シェルスクリプト + Python）
lint:
	bash tools/docs_lint/run_lint.sh
	$(UV) python3 tools/docs_lint/run_lint.py

# ツール群の単体テスト
test:
	$(UV) pytest tools/ -q

# kind クラスタ起動（target + ops-edge）
kind-up:
	kind create cluster --name k1s0-target --config src/infra/kind/kind-config.yaml
	kind create cluster --name k1s0-ops-edge --config src/infra/kind/kind-config-ops-edge.yaml

# kind クラスタ削除
kind-down:
	kind delete cluster --name k1s0-target 2>/dev/null || true
	kind delete cluster --name k1s0-ops-edge 2>/dev/null || true

# Kyverno オペレータ + ポリシー適用
deploy-kyverno:
	kubectl --context kind-k1s0-target apply -f src/security/kyverno/install/install.yaml
	kubectl --context kind-k1s0-target wait --for=condition=Ready pod \
	  -l app.kubernetes.io/name=kyverno -n kyverno --timeout=120s
	kubectl --context kind-k1s0-target apply -f src/security/kyverno/policies/

# CloudNativePG オペレータ + PostgreSQL クラスタ適用
deploy-cnpg:
	kubectl --context kind-k1s0-target apply -f \
	  https://raw.githubusercontent.com/cloudnative-pg/cloudnative-pg/release-1.24/releases/cnpg-1.24.0.yaml
	kubectl --context kind-k1s0-target wait --for=condition=Ready pod \
	  -l app.kubernetes.io/name=cloudnative-pg -n cnpg-system --timeout=180s
	kubectl --context kind-k1s0-target apply -f src/data/postgresql/cluster.yaml

# Apicurio Registry マニフェスト適用
deploy-apicurio:
	kubectl --context kind-k1s0-target apply -f src/data/apicurio/manifests/

# トポロジードリル（ノード配置・ゾーン疎通確認）
drill-topology:
	bash src/infra/topology/drill.sh

# クロック整合性ドリル（NTP/PTP 同期確認）
drill-clock:
	bash src/infra/clock_integrity/drill.sh

# リストアドリル（Barman バックアップからの復元確認）
drill-restore:
	bash src/data/preservation/drill.sh

# release_gate.lock.yaml 再生成
release-gate:
	$(UV) python -m tools.lock_yaml_generator.cli release_gate

# 利用可能ターゲット一覧を表示
help:
	@grep -E '^[a-z][a-z-]*:' $(MAKEFILE_LIST) | sed 's/:.*//; s/^/  make /'
