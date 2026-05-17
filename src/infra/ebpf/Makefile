# k1s0 eBPF プログラムのビルド Makefile
# BPF C プログラムを clang でコンパイルし、skeleton ヘッダを生成する

# コンパイラとリンカの設定: clang を使用して BPF オブジェクトを生成する
CC       := clang
# BPF プログラム用コンパイラフラグ: ターゲットアーキテクチャと最適化を指定する
BPF_CFLAGS := -O2 -g -target bpf -D__TARGET_ARCH_x86

# ソースファイルとターゲットファイルのパスを定義する
SRC_DIR  := .
# ビルド成果物の出力先ディレクトリを定義する
OUT_DIR  := build

# BPF C ソースファイルのリストを自動検出する
BPF_SRCS := $(wildcard $(SRC_DIR)/*.bpf.c)
# BPF オブジェクトファイルのリストを生成する（.bpf.c -> .bpf.o に変換する）
BPF_OBJS := $(patsubst $(SRC_DIR)/%.bpf.c,$(OUT_DIR)/%.bpf.o,$(BPF_SRCS))
# BPF skeleton ヘッダファイルのリストを生成する（.bpf.o -> .skel.h に変換する）
BPF_SKELS := $(patsubst $(OUT_DIR)/%.bpf.o,$(OUT_DIR)/%.skel.h,$(BPF_OBJS))

# インクルードパスを定義する（libbpf と カーネルヘッダを参照する）
INCLUDES := -I/usr/include/bpf -I/usr/include/linux

# デフォルトターゲット: all で全ビルド成果物を生成する
.PHONY: all
all: $(OUT_DIR) $(BPF_OBJS) $(BPF_SKELS)
	# 全 BPF オブジェクトと skeleton ヘッダのビルドが完了した
	@echo "BPF build complete: $(BPF_OBJS)"

# 出力ディレクトリを作成するターゲット
$(OUT_DIR):
	# ビルド成果物の出力ディレクトリを作成する
	mkdir -p $(OUT_DIR)

# BPF C ソースファイルを BPF オブジェクトファイルにコンパイルするルール
$(OUT_DIR)/%.bpf.o: $(SRC_DIR)/%.bpf.c
	# clang で BPF オブジェクトファイルをコンパイルする
	$(CC) $(BPF_CFLAGS) $(INCLUDES) -c $< -o $@

# BPF オブジェクトから skeleton ヘッダを生成するルール
$(OUT_DIR)/%.skel.h: $(OUT_DIR)/%.bpf.o
	# bpftool でユーザ空間ロード用の skeleton ヘッダを生成する
	bpftool gen skeleton $< > $@

# テストターゲット: eBPF probe のロードテストを実行する
.PHONY: test
test: all
	# tests/clock_skew_test.sh を実行して eBPF probe を検証する
	bash tests/clock_skew_test.sh

# クリーンターゲット: ビルド成果物を削除する
.PHONY: clean
clean:
	# ビルド出力ディレクトリを削除する
	rm -rf $(OUT_DIR)
	# クリーン完了メッセージを出力する
	@echo "Cleaned build artifacts."
