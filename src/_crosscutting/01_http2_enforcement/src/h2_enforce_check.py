#!/usr/bin/env python3
# h2_enforce_check.py — HTTP/2 強制 Envoy 設定検証ツール
# cross_http2 適合仕様 (docs/04_詳細設計/03_クロスカッティング適合仕様/01_HTTP2_enforcement.md) の
# 物理 enforcement を確認する。Envoy の filter_chain に http2_protocol_options が設定されており、
# HTTP/1.1 リクエストを拒否することを YAML レベルで静的検証する。
#
# 使用方法:
#   python3 h2_enforce_check.py <envoy_config.yaml>
# 終了コード: 0=pass, 1=fail, 2=input error

from __future__ import annotations

# argparse: コマンドライン引数解析
import argparse
# pathlib: ファイルパス操作
from pathlib import Path
# sys: 終了コード制御
import sys
# typing: 型ヒント
from typing import Any

# yaml モジュールをインポートする (PyYAML が必要)
try:
    import yaml
except ImportError:
    # PyYAML がインストールされていない場合はエラーを出力して終了する
    print("ERROR: PyYAML required: pip install pyyaml", file=sys.stderr)
    sys.exit(2)


def check_listener_http2(listener: dict[str, Any]) -> list[str]:
    """Envoy リスナー設定に HTTP/2 強制が設定されているか確認する。
    違反がある場合はエラーメッセージのリストを返す。"""
    # エラーメッセージを収集するリストを初期化する
    errors: list[str] = []

    # filter_chains を取得する
    filter_chains = listener.get("filter_chains", [])

    # filter_chain が空の場合はエラーを追加する
    if not filter_chains:
        errors.append(f"listener '{listener.get('name', 'unknown')}': filter_chains が空")
        return errors

    # 各 filter_chain を確認する
    for i, chain in enumerate(filter_chains):
        # transport_socket を確認する (TLS 必須)
        transport_socket = chain.get("transport_socket")
        if not transport_socket:
            errors.append(
                f"listener '{listener.get('name')}' chain[{i}]: "
                "transport_socket (TLS) が未設定"
            )

        # filters 内の http_connection_manager を確認する
        filters = chain.get("filters", [])
        hcm_found = False
        http2_options_found = False

        for f in filters:
            # http_connection_manager フィルタを探す
            if f.get("name") not in (
                "envoy.filters.network.http_connection_manager",
                "http_connection_manager",
            ):
                continue
            hcm_found = True

            # typed_config または config を取得する
            config = f.get("typed_config") or f.get("config", {})

            # http2_protocol_options が設定されているか確認する
            if config.get("http2_protocol_options") is not None:
                http2_options_found = True

            # codec_type が HTTP2 または AUTO に設定されているか確認する
            codec_type = config.get("codec_type", "").upper()
            if codec_type not in ("HTTP2", "AUTO"):
                errors.append(
                    f"listener '{listener.get('name')}' chain[{i}]: "
                    f"codec_type='{codec_type}' は HTTP2 強制に不十分"
                )

        # http_connection_manager が見つからない場合はエラーを追加する
        if not hcm_found:
            errors.append(
                f"listener '{listener.get('name')}' chain[{i}]: "
                "http_connection_manager フィルタが未設定"
            )

        # http2_protocol_options が設定されていない場合は警告を追加する
        if hcm_found and not http2_options_found:
            errors.append(
                f"listener '{listener.get('name')}' chain[{i}]: "
                "http2_protocol_options が未設定 (HTTP/1.1 が受け入れられる可能性あり)"
            )

    return errors


def check_envoy_config(config_path: Path) -> bool:
    """Envoy 設定ファイルを読み込み HTTP/2 強制を確認する。
    全チェックを通過した場合は True を返す。"""
    # 設定ファイルを読み込む
    try:
        raw = yaml.safe_load(config_path.read_text(encoding="utf-8"))
    except (yaml.YAMLError, OSError) as e:
        # ファイル読み込みエラーをメッセージ付きで出力する
        print(f"ERROR: {config_path} の読み込みに失敗しました: {e}", file=sys.stderr)
        return False

    # static_resources.listeners を取得する
    listeners = (raw or {}).get("static_resources", {}).get("listeners", [])

    # リスナーが設定されていない場合は警告を出力する
    if not listeners:
        print(f"WARNING: {config_path}: listeners が未設定", file=sys.stderr)
        return True

    # 全エラーを収集するリストを初期化する
    all_errors: list[str] = []

    # 各リスナーを確認する
    for listener in listeners:
        errors = check_listener_http2(listener)
        all_errors.extend(errors)

    # エラーがある場合は出力して失敗を返す
    if all_errors:
        for err in all_errors:
            print(f"FAIL: {err}", file=sys.stderr)
        return False

    # 全チェックを通過した場合は成功を出力する
    print(f"PASS: {config_path} — HTTP/2 強制設定を確認しました ({len(listeners)} リスナー)")
    return True


def main() -> int:
    """メインエントリポイント。"""
    # コマンドライン引数を解析する
    parser = argparse.ArgumentParser(
        description="Envoy HTTP/2 強制設定の静的検証ツール"
    )
    # 検証対象の設定ファイルパスを引数として受け取る
    parser.add_argument("config", nargs="+", help="Envoy 設定ファイルのパス")
    args = parser.parse_args()

    # 全ファイルの検証結果を追跡する変数を初期化する
    all_passed = True

    # 各設定ファイルを確認する
    for config_path_str in args.config:
        config_path = Path(config_path_str)
        # ファイルが存在しない場合はエラーを出力する
        if not config_path.exists():
            print(f"ERROR: {config_path} が見つかりません", file=sys.stderr)
            all_passed = False
            continue
        # 設定ファイルを確認する
        if not check_envoy_config(config_path):
            all_passed = False

    # 全チェックが通過した場合は 0、そうでない場合は 1 を返す
    return 0 if all_passed else 1


if __name__ == "__main__":
    sys.exit(main())
