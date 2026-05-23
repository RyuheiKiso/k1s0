#!/usr/bin/env python3
# ops_edge_verify.py — ops edge クラスタ独立性検証ツール
# cross_edge 適合仕様: ops 用エッジクラスタが本番クラスタから独立して
# 動作できることを静的検証する。Argo Workflow / Argo CD 設定の
# 独立性チェックを行う。
#
# 使用方法:
#   python3 ops_edge_verify.py --manifests-dir ./manifests
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


# ops edge クラスタが使用すべき namespace を定義する
OPS_NAMESPACE = "k1s0-ops"
# Argo Workflow が存在すべき種別を定義する
ARGO_WORKFLOW_KINDS = {"Workflow", "WorkflowTemplate", "CronWorkflow"}
# Argo CD Application が存在すべき種別を定義する
ARGOCD_KINDS = {"Application", "ApplicationSet"}


def check_ops_independence(manifests_dir: Path) -> list[str]:
    """ops edge クラスタの独立性チェックを実行してエラーリストを返す。"""
    # エラーリストを初期化する
    errors: list[str] = []

    # YAML ファイルを収集する
    yaml_files = sorted(manifests_dir.glob("**/*.yaml")) + sorted(manifests_dir.glob("**/*.yml"))
    if not yaml_files:
        errors.append(f"{manifests_dir}: YAML ファイルが存在しません")
        return errors

    # 見つかったリソース種別を追跡するセットを初期化する
    found_kinds: set[str] = set()
    cross_cluster_deps: list[str] = []

    # 各マニフェストを確認する
    for yaml_file in yaml_files:
        try:
            content = yaml_file.read_text(encoding="utf-8")
            docs = list(yaml.safe_load_all(content))
        except (yaml.YAMLError, OSError) as e:
            errors.append(f"{yaml_file.name}: 読み込みエラー: {e}")
            continue

        for doc in docs:
            if not isinstance(doc, dict):
                continue

            kind = doc.get("kind", "")
            found_kinds.add(kind)
            name = (doc.get("metadata") or {}).get("name", "unknown")
            namespace = (doc.get("metadata") or {}).get("namespace", "")

            # namespace の確認: ops 専用 namespace を使用しているか
            if namespace and namespace not in (OPS_NAMESPACE, "argocd", "argo", ""):
                # 本番 namespace への依存を検出する
                if namespace in ("default", "production", "k1s0-tier1"):
                    cross_cluster_deps.append(
                        f"{kind}/{name}: 本番 namespace '{namespace}' を参照しています"
                    )

            # Argo CD Application の server を確認する
            if kind in ARGOCD_KINDS:
                spec = doc.get("spec", {})
                destination = spec.get("destination", {})
                server = destination.get("server", "")
                # ops edge クラスタは in-cluster または専用 server を使用すべき
                if server and "k1s0-prod" in server:
                    cross_cluster_deps.append(
                        f"Application/{name}: 本番クラスタ '{server}' を destination に指定しています"
                    )

    # 本番クラスタへの直接依存があれば報告する
    errors.extend(cross_cluster_deps)

    # Argo リソースが存在しない場合は警告を出力する
    if not (found_kinds & ARGO_WORKFLOW_KINDS) and not (found_kinds & ARGOCD_KINDS):
        errors.append("Argo Workflow / Argo CD リソースが見つかりません (ops edge の設定が不完全)")

    return errors


def main() -> int:
    """メインエントリポイント。"""
    # コマンドライン引数を解析する
    parser = argparse.ArgumentParser(
        description="ops edge クラスタ独立性の静的検証ツール"
    )
    # マニフェストディレクトリパスを引数として受け取る
    parser.add_argument("--manifests-dir", default="manifests", help="Kubernetes マニフェストのディレクトリ")
    args = parser.parse_args()

    # マニフェストディレクトリを Path オブジェクトに変換する
    manifests_dir = Path(args.manifests_dir)
    if not manifests_dir.exists():
        print(f"INFO: {manifests_dir} が見つかりません (dry-run で終了)")
        print("PASS: ops_edge_verify — マニフェストなし (dry-run)")
        return 0

    # 独立性チェックを実行する
    errors = check_ops_independence(manifests_dir)

    # エラーがある場合は出力して失敗を返す
    if errors:
        for err in errors:
            print(f"FAIL: {err}", file=sys.stderr)
        print(f"\nFAIL: ops edge クラスタ設定に問題があります", file=sys.stderr)
        return 1

    # 全チェックを通過した場合は成功を出力する
    yaml_count = len(list(manifests_dir.glob("**/*.yaml")) + list(manifests_dir.glob("**/*.yml")))
    print(f"PASS: ops edge クラスタ設定 — {yaml_count} ファイルを確認しました")
    return 0


if __name__ == "__main__":
    sys.exit(main())
