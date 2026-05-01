#!/usr/bin/env bash
# cc shim: rustc 形式の target triple を zig cc 形式に変換するためのラッパ。
#
# 背景:
#   tier1 Rust ワークスペース（src/tier1/rust）の crates/decision は zen-engine 依存で
#   rquickjs（QuickJS の Rust binding、C ソース埋込）をビルドする。rquickjs の build.rs
#   は cc-rs 経由で `cc --target=x86_64-unknown-linux-gnu ...` を呼び出すが、開発環境に
#   よっては /usr/bin/cc が `zig-bootstrap` の clang ラッパで提供されており、zig 側の
#   target parser は rustc 形式の triple（`x86_64-unknown-linux-gnu`）を `UnknownOperating
#   System` として拒否する（zig は `unknown-` を含まない `x86_64-linux-gnu` 形式のみ）。
#
# 役割:
#   1. 下層 cc（K1S0_REAL_CC で上書き可、既定は `cc` の PATH 解決値）を 1 度だけ叩いて
#      zig-bootstrap であるかを判定する。
#   2. zig 配下の場合、`--target=<rustc-triple>` を `-target <zig-triple>` に変換する。
#   3. zig 以外（gcc / clang など）はそのまま透過する。
#
# 設計正典:
#   docs/05_実装/10_ビルド設計/10_Rust_Cargo_workspace/01_Rust_Cargo_workspace.md
#     - tier1 Rust ビルド方針（cargo build / cargo check）
#   docs/02_構想設計/adr/ADR-RULE-001-zen-engine.md
#     - DECISION-001 PoC の実装ベース（zen-engine → rquickjs → cc-rs）
#
# 使い方:
#   src/tier1/rust/.cargo/config.toml の [env] で
#     CC_x86_64_unknown_linux_gnu = { value = "../../../tools/build/cc-zigshim.sh", relative = true }
#   と指定して cc-rs 経由のビルドに割り込む。手元の cc が GNU/Apple-Clang の場合は
#   透過されるため CI（ubuntu-24.04 + system gcc）にも影響しない。

set -euo pipefail

# 下層 cc の解決。明示指定を優先し、未指定時は PATH の cc を採用する。
REAL_CC="${K1S0_REAL_CC:-cc}"

# zig-bootstrap 配下かを軽量判定する。--version 出力に "zig-bootstrap" が含まれるかで
# 判定する。失敗時は透過パスに倒す（fail-open）。
is_zig=0
if "$REAL_CC" --version 2>/dev/null | grep -q "zig-bootstrap"; then
  is_zig=1
fi

if [ "$is_zig" = "0" ]; then
  # 透過: GNU/Clang などは元の引数で呼び出す。
  exec "$REAL_CC" "$@"
fi

# zig パス: --target=<rustc> を -target <zig> に変換する。
# rustc triple → zig triple のマップは limited set に絞る（必要時に追加）。
translate_triple() {
  case "$1" in
    x86_64-unknown-linux-gnu)   echo "x86_64-linux-gnu" ;;
    x86_64-unknown-linux-musl)  echo "x86_64-linux-musl" ;;
    aarch64-unknown-linux-gnu)  echo "aarch64-linux-gnu" ;;
    aarch64-unknown-linux-musl) echo "aarch64-linux-musl" ;;
    x86_64-apple-darwin)        echo "x86_64-macos" ;;
    aarch64-apple-darwin)       echo "aarch64-macos" ;;
    x86_64-pc-windows-gnu)      echo "x86_64-windows-gnu" ;;
    *)                          echo "$1" ;;
  esac
}

new_args=()
for arg in "$@"; do
  case "$arg" in
    --target=*)
      triple="${arg#--target=}"
      zig_triple="$(translate_triple "$triple")"
      # zig cc は `-target <triple>` のみ受け付ける（`--target=` 形式は不可）。
      new_args+=("-target" "$zig_triple")
      ;;
    *)
      new_args+=("$arg")
      ;;
  esac
done

exec "$REAL_CC" "${new_args[@]}"
