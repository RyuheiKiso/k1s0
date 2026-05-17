.PHONY: verify clean

# verify: Docker 経由で Stainless による形式検証を実行するターゲット
verify:
	# epfl-lara/stainless-builder:0.9.8 イメージでコンテナを起動して検証する
	docker run --rm -v $(PWD):/workspace \
		epfl-lara/stainless-builder:0.9.8 \
		stainless src/main/scala/k1s0/data/AtomicThreeTableWrite.scala

# clean: コンパイル済みクラスファイルを削除するターゲット
clean:
	# .class ファイルを再帰的に検索して削除する
	find . -name "*.class" -delete
