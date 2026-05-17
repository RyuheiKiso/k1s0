#!/usr/bin/env python3
# Kyverno ポリシー自動生成スクリプト
# src/infra/topology/classes.yaml および src/infra/clock_integrity/classes.yaml を読み込み
# topology_class / clock_integrity_class の許可値を Kyverno policy YAML に埋め込む

# 標準ライブラリのインポート
import sys
# パス操作に使用する pathlib をインポートする
from pathlib import Path
# YAML 読み書きに使用する yaml をインポートする
import yaml
# 現在時刻取得に使用する datetime をインポートする
from datetime import datetime, timezone

# リポジトリルートのパスを計算する（このスクリプトの 4 階層上が root）
REPO_ROOT = Path(__file__).resolve().parents[4]
# topology クラス定義ファイルのパスを設定する
TOPOLOGY_CLASSES_FILE = REPO_ROOT / "src" / "infra" / "topology" / "classes.yaml"
# clock integrity クラス定義ファイルのパスを設定する
CLOCK_CLASSES_FILE = REPO_ROOT / "src" / "infra" / "clock_integrity" / "classes.yaml"
# 生成した policy の出力先ディレクトリパスを設定する
OUTPUT_DIR = REPO_ROOT / "src" / "security" / "kyverno" / "policies"

# 自動生成ヘッダーコメントのテンプレートを定義する
AUTO_GENERATED_HEADER = """\
# _AUTO_GENERATED: このファイルは build_policies.py によって自動生成されました
# 手動編集は次回生成時に上書きされます
# 生成日時: {timestamp}
# 生成コマンド: python3 src/security/kyverno/generator/build_policies.py
"""


def load_topology_classes(file_path: Path) -> list[str]:
    """topology クラス定義ファイルから class_id の一覧を読み込む。

    Args:
        file_path: topology クラス定義 YAML ファイルのパス

    Returns:
        topology class_id の文字列リスト
    """
    # ファイルが存在するかを確認する
    if not file_path.exists():
        # ファイルが存在しない場合はエラーメッセージを出力して終了する
        print(f"ERROR: {file_path} が見つかりません", file=sys.stderr)
        sys.exit(1)
    # YAML ファイルを読み込む
    with open(file_path, encoding="utf-8") as f:
        # YAML をパースして辞書に変換する
        data = yaml.safe_load(f)
    # topology_classes キーから class_id の一覧を抽出して返す
    return [cls["class_id"] for cls in data.get("topology_classes", [])]


def load_clock_integrity_classes(file_path: Path) -> list[str]:
    """clock integrity クラス定義ファイルから class_id の一覧を読み込む。

    Args:
        file_path: clock integrity クラス定義 YAML ファイルのパス

    Returns:
        clock integrity class_id の文字列リスト
    """
    # ファイルが存在するかを確認する
    if not file_path.exists():
        # ファイルが存在しない場合はエラーメッセージを出力して終了する
        print(f"ERROR: {file_path} が見つかりません", file=sys.stderr)
        sys.exit(1)
    # YAML ファイルを読み込む
    with open(file_path, encoding="utf-8") as f:
        # YAML をパースして辞書に変換する
        data = yaml.safe_load(f)
    # clock_integrity_classes キーから class_id の一覧を抽出して返す
    return [cls["class_id"] for cls in data.get("clock_integrity_classes", [])]


def build_topology_class_policy(class_ids: list[str]) -> str:
    """topology-class annotation ポリシーの YAML 文字列を生成する。

    Args:
        class_ids: 許可する topology class_id のリスト

    Returns:
        生成した Kyverno ClusterPolicy の YAML 文字列
    """
    # 許可値を "|" で結合してパターン文字列を作成する
    allowed_pattern = "|".join(class_ids)
    # 現在時刻のタイムスタンプを取得する
    timestamp = datetime.now(timezone.utc).isoformat()
    # 自動生成ヘッダーを作成する
    header = AUTO_GENERATED_HEADER.format(timestamp=timestamp)
    # ポリシー YAML 本体を構築する
    policy_yaml = f"""{header}
# topology-class annotation 必須化ポリシー（自動生成）
# 許可値は src/infra/topology/classes.yaml から自動的に生成される
apiVersion: kyverno.io/v1
# ClusterPolicy はクラスタ全体に適用されるポリシーリソース
kind: ClusterPolicy
metadata:
  # ポリシー名称: topology-class annotation の要求（自動生成版）
  name: require-topology-class-annotation
  annotations:
    # このポリシーが準拠する enforcement 仕様
    k1s0.io/enforcement-spec: detail.infra.cluster_topology_conformance
    # 自動生成されたことを示す annotation
    k1s0.io/auto-generated: "true"
    # 生成元ファイルを示す annotation
    k1s0.io/generated-from: src/infra/topology/classes.yaml
    # ポリシーの説明
    policies.kyverno.io/description: "k1s0.io/topology-class annotation がない workload を reject する（自動生成）"
spec:
  # Enforce モード: 違反リソースの作成を拒否する
  validationFailureAction: Enforce
  # バックグラウンドスキャンを無効化（admission 時のみ検証）
  background: false
  rules:
    # ルール名称: topology-class annotation の確認
    - name: check-topology-class
      # 適用対象: Pod リソース
      match:
        any:
          - resources:
              kinds:
                # Pod リソースに適用する
                - Pod
      # 除外対象: system namespace は enforcement 対象外
      exclude:
        any:
          - resources:
              namespaces:
                # Kyverno 自身の namespace を除外
                - kyverno
                # Kubernetes システム namespace を除外
                - kube-system
                # cert-manager namespace を除外
                - cert-manager
      # topology-class annotation の検証ルール
      validate:
        # 違反時のエラーメッセージ
        message: "k1s0.io/topology-class annotation は必須です。許可値: {allowed_pattern}"
        # 許可された topology class 値のパターン（自動生成）
        pattern:
          metadata:
            annotations:
              # topology class は定義された値のいずれかである必要がある
              k1s0.io/topology-class: "{allowed_pattern}"
"""
    # 生成したポリシー YAML 文字列を返す
    return policy_yaml


def build_clock_integrity_class_policy(class_ids: list[str]) -> str:
    """clock-integrity-class annotation ポリシーの YAML 文字列を生成する。

    Args:
        class_ids: 許可する clock integrity class_id のリスト

    Returns:
        生成した Kyverno ClusterPolicy の YAML 文字列
    """
    # 許可値を "|" で結合してパターン文字列を作成する
    allowed_pattern = "|".join(class_ids)
    # 現在時刻のタイムスタンプを取得する
    timestamp = datetime.now(timezone.utc).isoformat()
    # 自動生成ヘッダーを作成する
    header = AUTO_GENERATED_HEADER.format(timestamp=timestamp)
    # ポリシー YAML 本体を構築する
    policy_yaml = f"""{header}
# clock-integrity-class annotation 必須化ポリシー（自動生成）
# 許可値は src/infra/clock_integrity/classes.yaml から自動的に生成される
apiVersion: kyverno.io/v1
# ClusterPolicy はクラスタ全体に適用されるポリシーリソース
kind: ClusterPolicy
metadata:
  # ポリシー名称: clock-integrity-class annotation の要求（自動生成版）
  name: require-clock-integrity-class-annotation
  annotations:
    # このポリシーが準拠する enforcement 仕様
    k1s0.io/enforcement-spec: detail.infra.clock_integrity_conformance
    # 自動生成されたことを示す annotation
    k1s0.io/auto-generated: "true"
    # 生成元ファイルを示す annotation
    k1s0.io/generated-from: src/infra/clock_integrity/classes.yaml
    # ポリシーの説明
    policies.kyverno.io/description: "k1s0.io/clock-integrity-class annotation がない workload を reject する（自動生成）"
spec:
  # Enforce モード: 違反リソースの作成を拒否する
  validationFailureAction: Enforce
  # バックグラウンドスキャンを無効化（admission 時のみ検証）
  background: false
  rules:
    # ルール名称: clock-integrity-class annotation の確認
    - name: check-clock-integrity-class
      # 適用対象: Pod リソース
      match:
        any:
          - resources:
              kinds:
                # Pod リソースに適用する
                - Pod
      # 除外対象: system namespace は enforcement 対象外
      exclude:
        any:
          - resources:
              namespaces:
                # Kyverno 自身の namespace を除外
                - kyverno
                # Kubernetes システム namespace を除外
                - kube-system
                # cert-manager namespace を除外
                - cert-manager
      # clock-integrity-class annotation の検証ルール
      validate:
        # 違反時のエラーメッセージ
        message: "k1s0.io/clock-integrity-class annotation は必須です。許可値: {allowed_pattern}"
        # 許可された clock integrity class 値のパターン（自動生成）
        pattern:
          metadata:
            annotations:
              # clock integrity class は定義された値のいずれかである必要がある
              k1s0.io/clock-integrity-class: "{allowed_pattern}"
"""
    # 生成したポリシー YAML 文字列を返す
    return policy_yaml


def write_policy(output_dir: Path, filename: str, content: str) -> None:
    """ポリシーファイルを出力ディレクトリに書き込む。

    Args:
        output_dir: 出力先ディレクトリのパス
        filename: 出力ファイル名
        content: 書き込む YAML 文字列
    """
    # 出力先ファイルのパスを構築する
    output_path = output_dir / filename
    # ファイルに内容を書き込む
    output_path.write_text(content, encoding="utf-8")
    # 書き込み完了メッセージを出力する
    print(f"Generated: {output_path}")


def main() -> None:
    """メイン関数: ポリシーの生成と出力を実行する。"""
    # 処理開始メッセージを出力する
    print("Kyverno ポリシー自動生成を開始します...")
    # topology クラス一覧を読み込む
    topology_classes = load_topology_classes(TOPOLOGY_CLASSES_FILE)
    # 読み込んだ topology クラス数を出力する
    print(f"topology クラス数: {len(topology_classes)} -> {topology_classes}")
    # clock integrity クラス一覧を読み込む
    clock_classes = load_clock_integrity_classes(CLOCK_CLASSES_FILE)
    # 読み込んだ clock integrity クラス数を出力する
    print(f"clock integrity クラス数: {len(clock_classes)} -> {clock_classes}")
    # 出力ディレクトリが存在しない場合は作成する
    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
    # topology-class ポリシーを生成して書き込む
    topology_policy = build_topology_class_policy(topology_classes)
    write_policy(OUTPUT_DIR, "require-topology-class-annotation.yaml", topology_policy)
    # clock-integrity-class ポリシーを生成して書き込む
    clock_policy = build_clock_integrity_class_policy(clock_classes)
    write_policy(OUTPUT_DIR, "require-clock-integrity-class-annotation.yaml", clock_policy)
    # 処理完了メッセージを出力する
    print("ポリシー生成が完了しました。")


# スクリプトが直接実行された場合にメイン関数を呼び出す
if __name__ == "__main__":
    main()
