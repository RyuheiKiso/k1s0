# tier2 OPA bundle ビルド Makefile
# OPA CLI を使って bundle のテストとビルドを実行する

# OPA CLI のパス (環境変数 OPA_PATH で上書き可能)
OPA ?= opa

# .PHONY ターゲット宣言
.PHONY: test build bench

# OPA policy テストの実行
test:
	# OPA テストを実行する
	$(OPA) test ./policies/ ./tests/ -v

# OPA bundle のビルド
build:
	# bundle をアーカイブに変換する
	$(OPA) build -b ./ -o tier2-abac-bundle.tar.gz

# OPA benchmark の実行
bench:
	# delegation ポリシーのベンチマークを実行する
	$(OPA) bench --data ./policies/ ./tests/
