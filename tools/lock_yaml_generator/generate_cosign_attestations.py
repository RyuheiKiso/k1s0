"""tools/lock_yaml_generator/generate_cosign_attestations.py

# cosign_attestations.lock.yaml 生成器。
# src/tier3/lock/cosign_attestations_input.yaml を読み込み、
# tier3 build artifact の cosign keyless 署名 manifest を lock artifact として生成する。
"""

# 標準ライブラリのインポート
from __future__ import annotations

# datetime モジュール: generated_at タイムスタンプ生成に使用
import datetime
# Path: ファイルパス操作に使用
from pathlib import Path
# Any: 型ヒントに使用
from typing import Any

# BaseGenerator と REPO_ROOT をインポートする
from tools.lock_yaml_generator.base_generator import BaseGenerator, REPO_ROOT

# SoT input YAML のパス（手書き可能な宣言ファイル）
_INPUT_YAML = REPO_ROOT / "src/tier3/lock/cosign_attestations_input.yaml"


class CosignAttestationsGenerator(BaseGenerator):
    """cosign_attestations.lock.yaml 生成器。"""

    # 出力ファイル名
    OUTPUT_NAME = "cosign_attestations.lock.yaml"
    # 必須入力なし（_INPUT_YAML が SoT）
    REQUIRED_INPUTS: list[str] = []
    # スキーマ検証なし
    SCHEMA_PATH: Path | None = None
    # 出力先ディレクトリ
    DEFAULT_OUTPUT_DIR = "src/tier3/lock"

    def load_inputs(self, lock_dir: Path) -> dict[str, Any]:
        """src/tier3/lock/cosign_attestations_input.yaml を読み込む。
        存在しない場合は空 dict を返す。
        """
        # ファイルが存在する場合のみ読み込む
        if _INPUT_YAML.exists():
            # PyYAML を使って YAML を読み込む
            import yaml  # type: ignore
            # テキストを読み込んで YAML パースする
            raw = yaml.safe_load(_INPUT_YAML.read_text(encoding="utf-8"))
            # dict であれば返す、そうでなければ空 dict を返す
            return raw if isinstance(raw, dict) else {}
        # ファイルが存在しない場合は空 dict を返す
        return {}

    def build_artifact(self, inputs: dict[str, Any]) -> dict[str, Any]:
        """cosign keyless 署名 manifest の artifact dict を返す。
        inputs が空の場合はフォールバックスケルトンを使用する。
        """
        # 現在時刻を UTC で生成する
        generated_at = datetime.datetime.now(tz=datetime.timezone.utc).strftime(
            "%Y-%m-%dT%H:%M:%SZ"
        )

        # inputs から attestations 一覧を取得する（存在しない場合は空リスト）
        attestations: list[dict[str, Any]] = inputs.get("attestations", [])

        # attestations が空の場合はフォールバックスケルトンを使用する
        if not attestations:
            # フォールバック: 3 artifact のスケルトンを生成する
            attestations = [
                # Tauri sidecar スケルトン
                {
                    "artifact_id": "tier3_tauri_sidecar",
                    "signing_method": "keyless",
                    "oidc_issuer": "https://token.actions.githubusercontent.com",
                    "fulcio_url": "https://fulcio.sigstore.dev",
                    "rekor_url": "https://rekor.sigstore.dev",
                    "slsa_level": "L3",
                    "status": "declared",
                    "artifact_path": "dist/tier3/tauri/k1s0-companion-sidecar",
                    "note": "cosign keyless で Tauri sidecar を署名する（CI pipeline で実装）",
                },
                # WPF installer スケルトン
                {
                    "artifact_id": "tier3_wpf_installer",
                    "signing_method": "keyless",
                    "oidc_issuer": "https://token.actions.githubusercontent.com",
                    "fulcio_url": "https://fulcio.sigstore.dev",
                    "rekor_url": "https://rekor.sigstore.dev",
                    "slsa_level": "L3",
                    "status": "declared",
                    "artifact_path": "dist/tier3/wpf/k1s0-wpf-installer.exe",
                    "note": "cosign keyless で WPF installer を署名する",
                },
                # MSIX bundle スケルトン
                {
                    "artifact_id": "tier3_msix_bundle",
                    "signing_method": "keyless",
                    "oidc_issuer": "https://token.actions.githubusercontent.com",
                    "fulcio_url": "https://fulcio.sigstore.dev",
                    "rekor_url": "https://rekor.sigstore.dev",
                    "slsa_level": "L3",
                    "status": "declared",
                    "artifact_path": "dist/tier3/msix/k1s0-bundle.msix",
                    "note": "cosign keyless で MSIX bundle を署名する",
                },
            ]

        # inputs から verification_template を取得する
        verification_template: dict[str, Any] = inputs.get("verification_template", {})

        # verification_template が空の場合はデフォルト値を使用する
        if not verification_template:
            # デフォルトの検証コマンドテンプレートを設定する
            verification_template = {
                # 証明書 identity の正規表現（GitHub Actions OIDC の subject claim に合わせる）
                "certificate_identity_regexp": "https://github.com/k1s0-io/k1s0/.*",
                # OIDC issuer（GitHub Actions）
                "certificate_oidc_issuer": "https://token.actions.githubusercontent.com",
                # Rekor のエンドポイント
                "rekor_url": "https://rekor.sigstore.dev",
            }

        # inputs から dual_signoff_required を取得する（デフォルト: true）
        dual_signoff_required: bool = inputs.get("dual_signoff_required", True)

        # artifact dict を構築して返す
        return {
            # 自動生成ヘッダ（手書き禁止の明示）
            "_AUTO_GENERATED": (
                "DO NOT EDIT. Generated by"
                " tools/lock_yaml_generator/generate_cosign_attestations.py"
            ),
            # 生成ツール識別子（release_gate DSL で handwritten 状態がゼロであることを確認する）
            "generated_by": "generate_cosign_attestations",
            # 生成日時
            "generated_at": generated_at,
            # tier3 build artifact の cosign keyless 署名宣言一覧
            "attestations": attestations,
            # 検証コマンドテンプレート（CI の verify ステップで使用する）
            "verification_template": verification_template,
            # dual_signoff 要件（CLAUDE.md の dual sign-off 規約に従う）
            "dual_signoff_required": dual_signoff_required,
        }
