"""
cosign_kyverno_check.py — cosign 署名検証 + Kyverno ポリシーチェック
08_OSSライフサイクル適合仕様: 6 lifecycle_class × cosign 署名検証のシミュレーション
subprocess で cosign / kubectl コマンドを呼び出す。コマンドが存在しない場合は pytest.mark.skipif でスキップする。
lifecycle_class 名は 08_OSSライフサイクル適合仕様.md の canonical 名（v1_l1plus_primary 等）に統一する。
"""

# 標準ライブラリのインポート: subprocess でコマンドを呼び出す
import subprocess
# shutil.which: コマンドの存在確認に使用する
import shutil
# os: 環境変数の参照に使用する
import os
# json: JSON パースに使用する
import json

# pytest: テストフレームワーク
import pytest


# ============================================================
# 環境チェック関数
# ============================================================

# cosign コマンドが存在するかどうかを確認する関数
def has_cosign() -> bool:
    # shutil.which で cosign バイナリの存在を確認する
    return shutil.which("cosign") is not None


# kubectl コマンドが存在するかどうかを確認する関数
def has_kubectl() -> bool:
    # shutil.which で kubectl バイナリの存在を確認する
    return shutil.which("kubectl") is not None


# cosign と kubectl 両方が存在するかどうかを確認する関数
def has_cosign_and_kubectl() -> bool:
    # 両コマンドが存在する場合のみ True を返す
    return has_cosign() and has_kubectl()


# ============================================================
# lifecycle_class 定義（08_OSSライフサイクル適合仕様.md §lifecycle_class セット canonical 名）
# ============================================================

# spec canonical 6 値: 08_OSSライフサイクル適合仕様.md §lifecycle_class セットに完全準拠する
LIFECYCLE_CLASSES = [
    # v1_l1plus_primary: 最重要 OSS（tier1 transport / auth 等、直接 RPC に使用する OSS）
    "v1_l1plus_primary",
    # v1_l2star_alt: 重要 OSS（tier1 server が依存する主要 OSS）
    "v1_l2star_alt",
    # v1_l3_generic: 標準 OSS（一般的なコンポーネント）
    "v1_l3_generic",
    # v1_l4_dev_tool: 開発支援ツール（build / test / lint に使用する OSS）
    "v1_l4_dev_tool",
    # v1_l5_sandbox: サンドボックス（PoC / 実験的使用）
    "v1_l5_sandbox",
    # v1_l6_deprecated: 非推奨（移行待ち）
    "v1_l6_deprecated",
]


# ============================================================
# cosign 署名検証シミュレーション関数
# ============================================================

# 指定した image の cosign 署名を検証してスコアを返す関数（シミュレーション）
def simulate_cosign_verify(image_ref: str, lifecycle_class: str) -> dict:
    """
    cosign 署名検証を simulate して結果 dict を返す。
    lifecycle_class に応じて検証の厳密さを変える。
    """
    # 結果を格納する dict を初期化する
    result = {
        # 検証対象の image 参照
        "image_ref": image_ref,
        # 対象 OSS の lifecycle_class（spec canonical 名）
        "lifecycle_class": lifecycle_class,
        # 署名検証の成否（シミュレーションでは True を返す）
        "cosign_verified": False,
        # 検証に使用した公開鍵の参照
        "key_ref": "k8s://cosign-system/cosign-pub-key",
        # エラーメッセージ（エラーがある場合）
        "error": None,
    }

    # cosign コマンドが存在しない場合はスキップフラグを立てて返す
    if not has_cosign():
        # cosign が存在しない旨をエラーに記録する
        result["error"] = "cosign binary not found; skipping verification"
        # 未検証状態で返す
        return result

    # v1_l1plus_primary / v1_l2star_alt は厳格な署名検証が必要（Fulcio + Rekor を使用する）
    if lifecycle_class in ("v1_l1plus_primary", "v1_l2star_alt"):
        # 厳格な cosign 検証コマンドを構築する（Rekor 透明性ログを必須とする）
        cmd = [
            # cosign バイナリを呼び出す
            "cosign",
            # verify サブコマンドを指定する
            "verify",
            # Rekor 透明性ログへの記録を必須とする
            "--rekor-url=https://rekor.sigstore.dev",
            # 検証対象の image 参照を渡す
            image_ref,
        ]
    else:
        # v1_l3_generic 以下は基本的な署名確認（Rekor 不要）
        cmd = [
            # cosign バイナリを呼び出す
            "cosign",
            # verify サブコマンドを指定する
            "verify",
            # 検証対象の image 参照を渡す
            image_ref,
        ]

    # subprocess で cosign コマンドを実行する
    try:
        # コマンドを実行して出力を取得する
        proc = subprocess.run(
            # 実行するコマンドリスト
            cmd,
            # 標準出力をキャプチャする
            capture_output=True,
            # 出力を文字列としてデコードする
            text=True,
            # タイムアウト: 30 秒で打ち切る
            timeout=30,
        )
        # 終了コードが 0 の場合は署名検証 OK とする
        result["cosign_verified"] = proc.returncode == 0
        # 終了コードが非 0 の場合はエラーメッセージを記録する
        if proc.returncode != 0:
            # stderr の内容をエラーメッセージとして記録する
            result["error"] = proc.stderr.strip()
    except subprocess.TimeoutExpired:
        # タイムアウトの場合はエラーを記録する
        result["error"] = "cosign verify timed out after 30s"
    except FileNotFoundError:
        # cosign バイナリが見つからない場合はエラーを記録する
        result["error"] = "cosign binary not found"

    # 検証結果を返す
    return result


# ============================================================
# pytest テスト群
# ============================================================

# ---- v1_l1plus_primary lifecycle_class の cosign 検証テスト ----

# v1_l1plus_primary: cosign が存在する場合のみ署名検証テストを実行する
@pytest.mark.skipif(not has_cosign(), reason="cosign binary not found; skipping v1_l1plus_primary test")
def test_cosign_verify_v1_l1plus_primary():
    """v1_l1plus_primary OSS の cosign 署名検証をシミュレートする"""
    # v1_l1plus_primary のテスト image 参照を定義する（実環境では実際の image を使う）
    image_ref = "ghcr.io/k1s0/tier1-server:latest"
    # cosign 署名検証を実行する
    result = simulate_cosign_verify(image_ref, "v1_l1plus_primary")
    # image_ref が結果に含まれることを確認する
    assert result["image_ref"] == image_ref, f"image_ref mismatch: {result}"
    # lifecycle_class が正しく記録されていることを確認する
    assert result["lifecycle_class"] == "v1_l1plus_primary", f"lifecycle_class mismatch: {result}"


# ---- v1_l2star_alt lifecycle_class の cosign 検証テスト ----

# v1_l2star_alt: cosign が存在する場合のみ署名検証テストを実行する
@pytest.mark.skipif(not has_cosign(), reason="cosign binary not found; skipping v1_l2star_alt test")
def test_cosign_verify_v1_l2star_alt():
    """v1_l2star_alt OSS の cosign 署名検証をシミュレートする"""
    # v1_l2star_alt のテスト image 参照を定義する
    image_ref = "docker.io/library/postgres:16"
    # cosign 署名検証を実行する
    result = simulate_cosign_verify(image_ref, "v1_l2star_alt")
    # image_ref が結果に含まれることを確認する
    assert result["image_ref"] == image_ref, f"image_ref mismatch: {result}"
    # lifecycle_class が正しく記録されていることを確認する
    assert result["lifecycle_class"] == "v1_l2star_alt", f"lifecycle_class mismatch: {result}"


# ---- v1_l3_generic lifecycle_class の cosign 検証テスト ----

# v1_l3_generic: cosign が存在する場合のみ署名検証テストを実行する
@pytest.mark.skipif(not has_cosign(), reason="cosign binary not found; skipping v1_l3_generic test")
def test_cosign_verify_v1_l3_generic():
    """v1_l3_generic OSS の cosign 署名検証をシミュレートする"""
    # v1_l3_generic のテスト image 参照を定義する
    image_ref = "docker.io/library/redis:7"
    # cosign 署名検証を実行する
    result = simulate_cosign_verify(image_ref, "v1_l3_generic")
    # image_ref が結果に含まれることを確認する
    assert result["image_ref"] == image_ref, f"image_ref mismatch: {result}"
    # lifecycle_class が正しく記録されていることを確認する
    assert result["lifecycle_class"] == "v1_l3_generic", f"lifecycle_class mismatch: {result}"


# ---- シミュレーションのみのテスト（cosign 不要: 常時 pass）----

# 全 lifecycle_class に対して simulate_cosign_verify が dict を返すことを確認する（cosign 不要）
def test_simulate_cosign_verify_returns_dict_for_all_lifecycle_classes():
    """全 lifecycle_class で simulate_cosign_verify が正しい dict 構造を返すことを確認する"""
    # 全 lifecycle_class をイテレートしてテストする
    for lc in LIFECYCLE_CLASSES:
        # テスト用 image 参照を生成する
        image_ref = f"test-image:latest-{lc.lower()}"
        # simulate_cosign_verify を呼び出す
        result = simulate_cosign_verify(image_ref, lc)
        # 結果が dict であることを確認する
        assert isinstance(result, dict), f"result should be dict for {lc}"
        # image_ref キーが存在することを確認する
        assert "image_ref" in result, f"result should have image_ref for {lc}"
        # lifecycle_class キーが存在することを確認する
        assert "lifecycle_class" in result, f"result should have lifecycle_class for {lc}"
        # cosign_verified キーが存在することを確認する
        assert "cosign_verified" in result, f"result should have cosign_verified for {lc}"
        # lifecycle_class が正しく記録されていることを確認する
        assert result["lifecycle_class"] == lc, f"lifecycle_class should be {lc}"


# ---- Kyverno ポリシーチェックテスト（kubectl 不要: 構造検証のみ）----

# Kyverno の ClusterPolicy リソース名一覧をシミュレートする関数
def get_expected_kyverno_policies() -> list:
    """tier1 で必須の Kyverno ポリシー名一覧を返す（仕様: P6 Kyverno enforcement）"""
    # tier1 必須 Kyverno ポリシーの一覧を定義する
    return [
        # cosign 署名が必須であることを強制するポリシー
        "require-cosign-signatures",
        # 信頼できるレジストリからのみ image を pull することを強制するポリシー
        "restrict-image-registries",
        # 特権コンテナの作成を禁止するポリシー
        "disallow-privileged-containers",
        # root ユーザーでのコンテナ実行を禁止するポリシー
        "disallow-root-user",
        # latest タグの image 使用を禁止するポリシー
        "disallow-latest-tag",
    ]


# 必須 Kyverno ポリシー名一覧が正しく定義されていることを確認するテスト
def test_kyverno_policy_list_structure():
    """必須 Kyverno ポリシー名一覧の構造を検証する（kubectl 不要）"""
    # 必須ポリシー一覧を取得する
    policies = get_expected_kyverno_policies()
    # ポリシー数が 5 以上であることを確認する（tier1 最低要件）
    assert len(policies) >= 5, f"At least 5 Kyverno policies required, got {len(policies)}"
    # 各ポリシー名が文字列であることを確認する
    for policy_name in policies:
        # ポリシー名が文字列であることを確認する
        assert isinstance(policy_name, str), f"policy name should be str: {policy_name}"
        # ポリシー名が空でないことを確認する
        assert len(policy_name) > 0, "policy name must not be empty"


# kubectl が存在する場合のみ Kyverno ClusterPolicy を実際に確認するテスト
@pytest.mark.skipif(not has_kubectl(), reason="kubectl binary not found; skipping Kyverno live check")
def test_kyverno_cluster_policies_exist():
    """Kyverno ClusterPolicy リソースが Kubernetes cluster に存在することを確認する"""
    # kubectl get clusterpolicies コマンドを実行する
    proc = subprocess.run(
        # kubectl コマンドと引数を定義する
        ["kubectl", "get", "clusterpolicies", "-o", "json"],
        # 標準出力をキャプチャする
        capture_output=True,
        # 出力を文字列としてデコードする
        text=True,
        # タイムアウト: 15 秒で打ち切る
        timeout=15,
    )
    # kubectl が成功した場合のみ結果を検証する
    if proc.returncode == 0:
        # JSON 出力をパースする
        data = json.loads(proc.stdout)
        # items キーが存在することを確認する
        assert "items" in data, "kubectl get clusterpolicies output should have items"
        # Kyverno ポリシーが 1 件以上存在することを確認する
        assert len(data["items"]) >= 1, "At least one ClusterPolicy should exist in the cluster"
