#!/usr/bin/env python3
# protection_layer_check.py — SLO 4 層保護設定検証ツール
# cross_slo 適合仕様: Envoy rate_limit / Kafka quota / KEDA scale / pgBouncer pool の
# 4 層 SLO 保護が全て設定されているか静的検証する。
#
# 使用方法:
#   python3 protection_layer_check.py --config-dir .
# 終了コード: 0=all 4 layers pass, 1=fail, 2=input error

from __future__ import annotations

# argparse: コマンドライン引数解析
import argparse
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


def check_layer_envoy(config_dir: Path) -> tuple[bool, str]:
    """Layer 1: Envoy rate_limit フィルタが設定されているか確認する。"""
    # Envoy 設定ファイルを探す
    envoy_configs = list(config_dir.glob("envoy/*.yaml")) + list(config_dir.glob("envoy/*.yml"))
    if not envoy_configs:
        return False, "Layer 1 (Envoy): 設定ファイルが見つかりません"

    # 設定ファイルに rate_limit フィルタが含まれているか確認する
    for cf in envoy_configs:
        try:
            raw = yaml.safe_load(cf.read_text(encoding="utf-8")) or {}
        except yaml.YAMLError:
            continue
        # envoy.filters.http.ratelimit の存在を確認する
        content = cf.read_text(encoding="utf-8")
        if "ratelimit" in content.lower() or "rate_limit" in content.lower():
            return True, f"Layer 1 (Envoy): rate_limit フィルタを確認 ({cf.name})"

    return False, "Layer 1 (Envoy): rate_limit フィルタが未設定"


def check_layer_kafka(config_dir: Path) -> tuple[bool, str]:
    """Layer 2: Kafka quota 設定が存在するか確認する。"""
    # Kafka 設定ファイルを探す
    kafka_configs = list(config_dir.glob("kafka/*.yaml")) + list(config_dir.glob("kafka/*.yml"))
    if not kafka_configs:
        return False, "Layer 2 (Kafka): 設定ファイルが見つかりません"

    # quota が設定されているか確認する
    for cf in kafka_configs:
        content = cf.read_text(encoding="utf-8")
        if "quota" in content.lower() or "client.quota" in content.lower():
            return True, f"Layer 2 (Kafka): quota 設定を確認 ({cf.name})"

    return False, "Layer 2 (Kafka): quota 設定が未設定"


def check_layer_keda(config_dir: Path) -> tuple[bool, str]:
    """Layer 3: KEDA ScaledObject が定義されているか確認する。"""
    # KEDA 設定ファイルを探す
    keda_configs = list(config_dir.glob("keda/*.yaml")) + list(config_dir.glob("keda/*.yml"))
    if not keda_configs:
        return False, "Layer 3 (KEDA): 設定ファイルが見つかりません"

    # ScaledObject が定義されているか確認する
    for cf in keda_configs:
        try:
            raw = yaml.safe_load(cf.read_text(encoding="utf-8"))
        except yaml.YAMLError:
            continue
        if isinstance(raw, dict) and raw.get("kind") == "ScaledObject":
            return True, f"Layer 3 (KEDA): ScaledObject を確認 ({cf.name})"

    return False, "Layer 3 (KEDA): ScaledObject が未定義"


def check_layer_pgbouncer(config_dir: Path) -> tuple[bool, str]:
    """Layer 4: pgBouncer pool 設定が存在するか確認する。"""
    # pgBouncer 設定ファイルを探す
    pgbouncer_configs = (
        list(config_dir.glob("pgbouncer/*.yaml"))
        + list(config_dir.glob("pgbouncer/*.ini"))
        + list(config_dir.glob("pgbouncer/*.yml"))
    )
    if not pgbouncer_configs:
        return False, "Layer 4 (pgBouncer): 設定ファイルが見つかりません"

    # pool_size が設定されているか確認する
    for cf in pgbouncer_configs:
        content = cf.read_text(encoding="utf-8")
        if "pool_size" in content or "max_client_conn" in content:
            return True, f"Layer 4 (pgBouncer): pool 設定を確認 ({cf.name})"

    return False, "Layer 4 (pgBouncer): pool_size が未設定"


def main() -> int:
    """メインエントリポイント。"""
    # コマンドライン引数を解析する
    parser = argparse.ArgumentParser(
        description="SLO 4 層保護設定の静的検証ツール"
    )
    # 設定ディレクトリパスを引数として受け取る (デフォルトはカレントディレクトリ)
    parser.add_argument("--config-dir", default=".", help="設定ファイルを含むディレクトリ")
    args = parser.parse_args()

    # 設定ディレクトリを Path オブジェクトに変換する
    config_dir = Path(args.config_dir)

    # 4 層の検証チェックを定義するリストを作成する
    checks = [
        check_layer_envoy,
        check_layer_kafka,
        check_layer_keda,
        check_layer_pgbouncer,
    ]

    # 全検証を実行して結果を収集する
    all_passed = True
    for check in checks:
        passed, message = check(config_dir)
        # 結果を出力する
        prefix = "PASS" if passed else "FAIL"
        print(f"{prefix}: {message}")
        if not passed:
            all_passed = False

    # 全 4 層が通過した場合は成功メッセージを出力する
    if all_passed:
        print("\nPASS: SLO 4 層保護設定 — 全 4 レイヤーを確認しました")
    else:
        print("\nFAIL: SLO 4 層保護設定 — 不足しているレイヤーがあります", file=sys.stderr)

    return 0 if all_passed else 1


if __name__ == "__main__":
    sys.exit(main())
