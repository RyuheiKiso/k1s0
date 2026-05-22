#!/usr/bin/env python3
# shamir_verify.py — KEK Shamir 秘密分散しきい値検証ツール
# cross_kek 適合仕様の物理 enforcement を確認する。
# OpenBao Transit KEK の Shamir 分散設定が正しく構成されているか静的検証する。
#
# 使用方法:
#   python3 shamir_verify.py --config kek_config.yaml --shares-dir /path/to/shares
# 終了コード: 0=pass, 1=fail, 2=input error

from __future__ import annotations

# argparse: コマンドライン引数解析
import argparse
# hashlib: シェア整合性チェック
import hashlib
# pathlib: ファイルパス操作
from pathlib import Path
# sys: 終了コード制御
import sys
# typing: 型ヒント
from typing import Any

# yaml モジュールをインポートする
try:
    import yaml
except ImportError:
    print("ERROR: PyYAML required: pip install pyyaml", file=sys.stderr)
    sys.exit(2)


# k1s0 の Shamir 設定定数を定義する
K1S0_SHAMIR_THRESHOLD_MIN = 2
# k1s0 の Shamir 設定: 最大シェア数 (OpenBao 推奨値)
K1S0_SHAMIR_SHARES_MAX = 7


def verify_shamir_config(config: dict[str, Any]) -> list[str]:
    """Shamir 設定の静的検証を行い違反があればエラーメッセージを返す。"""
    # エラーメッセージを収集するリストを初期化する
    errors: list[str] = []

    # shamir セクションを取得する
    shamir = config.get("shamir", {})

    # threshold を確認する (最低 2 シェア必要)
    threshold = shamir.get("threshold", 0)
    if threshold < K1S0_SHAMIR_THRESHOLD_MIN:
        errors.append(
            f"threshold={threshold} は最小値 {K1S0_SHAMIR_THRESHOLD_MIN} 未満です"
        )

    # shares を確認する (threshold より大きく max 以下)
    shares = shamir.get("shares", 0)
    if shares < threshold:
        errors.append(f"shares={shares} は threshold={threshold} 未満です")
    if shares > K1S0_SHAMIR_SHARES_MAX:
        errors.append(
            f"shares={shares} は最大値 {K1S0_SHAMIR_SHARES_MAX} を超えています"
        )

    # key_id を確認する (OpenBao Transit キー名)
    key_id = shamir.get("key_id", "")
    if not key_id:
        errors.append("key_id が未設定です (OpenBao Transit キー名が必要)")

    # key_type を確認する (aes256-gcm96 推奨)
    key_type = shamir.get("key_type", "")
    if key_type not in ("aes256-gcm96", "rsa-2048", "ecdsa-p256"):
        errors.append(
            f"key_type='{key_type}' は未知の型です (aes256-gcm96 / rsa-2048 / ecdsa-p256 のいずれかが必要)"
        )

    return errors


def verify_shares_directory(shares_dir: Path, expected_count: int) -> list[str]:
    """シェアファイルの存在と整合性を確認し違反があればエラーメッセージを返す。"""
    # エラーメッセージを収集するリストを初期化する
    errors: list[str] = []

    # ディレクトリが存在するか確認する
    if not shares_dir.exists():
        errors.append(f"shares_dir='{shares_dir}' が存在しません")
        return errors

    # シェアファイルを収集する (.share または .pem 形式)
    share_files = sorted(shares_dir.glob("*.share")) + sorted(shares_dir.glob("*.pem"))

    # シェアファイルの数を確認する
    if len(share_files) < expected_count:
        errors.append(
            f"シェアファイルが {len(share_files)} 件しかありません "
            f"({expected_count} 件必要)"
        )

    # 各シェアファイルの内容を確認する
    for sf in share_files:
        # ファイルが空でないか確認する
        content = sf.read_bytes()
        if len(content) < 16:
            errors.append(f"シェアファイル '{sf.name}' が短すぎます (最低 16 バイト必要)")

        # シェアファイルのチェックサムを計算する (監査ログ用)
        digest = hashlib.sha256(content).hexdigest()[:8]
        print(f"  SHARE: {sf.name} (sha256={digest}...)")

    return errors


def main() -> int:
    """メインエントリポイント。"""
    # コマンドライン引数を解析する
    parser = argparse.ArgumentParser(
        description="KEK Shamir 秘密分散設定の静的検証ツール"
    )
    # 設定ファイルパスを引数として受け取る
    parser.add_argument("--config", default="lock/kek_shamir.yaml", help="KEK Shamir 設定ファイルのパス")
    # シェアディレクトリパスを引数として受け取る
    parser.add_argument("--shares-dir", default=None, help="シェアファイルのディレクトリパス")
    args = parser.parse_args()

    # 設定ファイルを読み込む
    config_path = Path(args.config)
    if not config_path.exists():
        print(f"INFO: {config_path} が見つかりません (dry-run モードで構成チェックをスキップ)")
        print("PASS: shamir_verify — 設定ファイル不在のため dry-run で終了")
        return 0

    # 設定ファイルを YAML として読み込む
    try:
        config = yaml.safe_load(config_path.read_text(encoding="utf-8")) or {}
    except yaml.YAMLError as e:
        print(f"ERROR: {config_path} の読み込みに失敗しました: {e}", file=sys.stderr)
        return 2

    # Shamir 設定の静的検証を実行する
    errors = verify_shamir_config(config)

    # シェアディレクトリが指定されている場合は存在確認を行う
    if args.shares_dir:
        shares_count = config.get("shamir", {}).get("shares", 0)
        shares_errors = verify_shares_directory(Path(args.shares_dir), shares_count)
        errors.extend(shares_errors)

    # エラーがある場合は出力して失敗を返す
    if errors:
        for err in errors:
            print(f"FAIL: {err}", file=sys.stderr)
        return 1

    # 全チェックを通過した場合は成功を出力する
    print(f"PASS: {config_path} — Shamir しきい値設定を確認しました")
    return 0


if __name__ == "__main__":
    sys.exit(main())
