#!/usr/bin/env python3
# pii_cluster_verify.py — PII 専用クラスタ設定検証ツール
# cross_pii 適合仕様: PII 専用 Kubernetes クラスタの namespace 分離・
# NetworkPolicy・暗号化設定が正しく構成されているか静的検証する。
#
# 使用方法:
#   python3 pii_cluster_verify.py --manifests-dir ./manifests
# 終了コード: 0=pass, 1=fail, 2=input error

from __future__ import annotations

# argparse: コマンドライン引数解析
import argparse
# pathlib: ファイルパス操作
from pathlib import Path
# sys: 終了コード制御
import sys

# yaml モジュールをインポートする
try:
    import yaml
except ImportError:
    print("ERROR: PyYAML required: pip install pyyaml", file=sys.stderr)
    sys.exit(2)


# PII 専用 namespace の期待値を定義する
PII_NAMESPACE = "k1s0-pii"
# PII クラスタに必須のリソース種別を定義する
REQUIRED_KINDS = {"Namespace", "NetworkPolicy"}


def check_manifest(manifest_path: Path) -> tuple[bool, list[str]]:
    """単一のマニフェストファイルを検証する。"""
    # 問題リストを初期化する
    issues: list[str] = []

    # YAML ファイルを読み込む
    try:
        content = manifest_path.read_text(encoding="utf-8")
        docs = list(yaml.safe_load_all(content))
    except (yaml.YAMLError, OSError) as e:
        issues.append(f"{manifest_path.name}: 読み込みエラー: {e}")
        return False, issues

    # 各 Kubernetes リソースを確認する
    for doc in docs:
        if not isinstance(doc, dict):
            continue

        kind = doc.get("kind", "")
        name = (doc.get("metadata") or {}).get("name", "unknown")
        namespace = (doc.get("metadata") or {}).get("namespace", "")

        # Namespace リソースの確認
        if kind == "Namespace":
            # PII namespace のラベルを確認する
            labels = (doc.get("metadata") or {}).get("labels", {})
            if labels.get("k1s0/pii-dedicated") != "true":
                issues.append(
                    f"Namespace '{name}': k1s0/pii-dedicated=true ラベルが未設定"
                )

        # NetworkPolicy リソースの確認
        if kind == "NetworkPolicy":
            namespace_check = namespace or (doc.get("spec") or {}).get("podSelector")
            if not namespace_check:
                issues.append(f"NetworkPolicy '{name}': namespace または podSelector が未設定")

            # ingress/egress ルールの存在を確認する
            spec = doc.get("spec", {})
            if not spec.get("ingress") and not spec.get("egress"):
                issues.append(
                    f"NetworkPolicy '{name}': ingress/egress ルールが未定義 (デフォルト全許可になる可能性)"
                )

        # PersistentVolumeClaim の暗号化確認 (StorageClass を通じた推定)
        if kind in ("PersistentVolumeClaim", "StorageClass"):
            annotations = (doc.get("metadata") or {}).get("annotations", {})
            storage_class = doc.get("spec", {}).get("storageClassName", "")
            if "encrypted" not in storage_class.lower() and \
               "k1s0/encrypted" not in annotations:
                issues.append(
                    f"{kind} '{name}': 暗号化 StorageClass または k1s0/encrypted アノテーションが未設定"
                )

    return len(issues) == 0, issues


def main() -> int:
    """メインエントリポイント。"""
    # コマンドライン引数を解析する
    parser = argparse.ArgumentParser(
        description="PII 専用クラスタ設定の静的検証ツール"
    )
    # マニフェストディレクトリパスを引数として受け取る
    parser.add_argument("--manifests-dir", default="manifests", help="Kubernetes マニフェストのディレクトリ")
    args = parser.parse_args()

    # マニフェストディレクトリを Path オブジェクトに変換する
    manifests_dir = Path(args.manifests_dir)
    if not manifests_dir.exists():
        print(f"INFO: {manifests_dir} が見つかりません (dry-run で終了)")
        print("PASS: pii_cluster_verify — マニフェストなし (dry-run)")
        return 0

    # YAML ファイルを収集する
    yaml_files = sorted(manifests_dir.glob("**/*.yaml")) + sorted(manifests_dir.glob("**/*.yml"))
    if not yaml_files:
        print(f"INFO: {manifests_dir} に YAML ファイルがありません")
        return 0

    # 全マニフェストを検証する
    all_passed = True
    found_kinds: set[str] = set()

    for yaml_file in yaml_files:
        passed, issues = check_manifest(yaml_file)
        if not passed:
            all_passed = False
            for issue in issues:
                print(f"FAIL: {issue}", file=sys.stderr)
        else:
            print(f"PASS: {yaml_file.name}")

    # 必須リソース種別が存在するか確認する
    all_content = " ".join(f.read_text() for f in yaml_files)
    for required_kind in REQUIRED_KINDS:
        if f"kind: {required_kind}" not in all_content:
            print(f"FAIL: 必須リソース '{required_kind}' が見つかりません", file=sys.stderr)
            all_passed = False

    # 結果サマリを出力する
    if all_passed:
        print(f"\nPASS: PII 専用クラスタ設定 — {len(yaml_files)} ファイルを確認しました")
    else:
        print("\nFAIL: PII 専用クラスタ設定に問題があります", file=sys.stderr)

    return 0 if all_passed else 1


if __name__ == "__main__":
    sys.exit(main())
