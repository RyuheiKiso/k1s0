from pathlib import Path

import yaml
import pytest

from conftest import run_cli

SCENARIOS_PATH = Path(__file__).parent / "scenarios" / "create_project.yaml"


def _setup_workspace(config_dir, workspace):
    """ワークスペースを事前設定する"""
    run_cli(["1", "1", str(workspace), "2", "2"], config_dir)


# --- データ駆動テスト ---

with open(SCENARIOS_PATH, encoding="utf-8") as f:
    scenarios = yaml.safe_load(f)


@pytest.mark.parametrize(
    "scenario",
    scenarios,
    ids=[s["name"] for s in scenarios],
)
def test_create_project_scenario(scenario, config_dir, workspace):
    """プロジェクト作成シナリオ (データ駆動)"""
    _setup_workspace(config_dir, workspace)
    result = run_cli(scenario["selections"], config_dir)
    assert result.returncode == 0
    for expected in scenario.get("expect_stdout", []):
        assert expected in result.stdout, (
            f"'{expected}' not found in stdout:\n{result.stdout}"
        )


# --- 個別テスト ---


def test_system_library_rust(config_dir, workspace):
    """System Region → Library → Rust の完全フロー"""
    _setup_workspace(config_dir, workspace)
    result = run_cli([
        "0",    # メインメニュー → プロジェクト作成
        "0",    # リージョン → system-region
        "0",    # プロジェクト種別 → Library
        "0",    # 言語 → Rust
        "2",    # メインメニュー → 終了
    ], config_dir)
    assert result.returncode == 0
    assert "ワークスペース" in result.stdout
    assert "チェックアウト" in result.stdout


def test_business_new_region_service_go(config_dir, workspace):
    """Business Region → 新規 → Service → Go の完全フロー"""
    _setup_workspace(config_dir, workspace)
    result = run_cli([
        "0",            # メインメニュー → プロジェクト作成
        "1",            # リージョン → business-region
        "sales",        # 領域名入力 (空リストなので直接入力)
        "1",            # プロジェクト種別 → Service
        "1",            # 言語 → Go
        "2",            # メインメニュー → 終了
    ], config_dir)
    assert result.returncode == 0
    assert "チェックアウト" in result.stdout


def test_business_empty_region_name_rejected(config_dir, workspace):
    """Business Region で空の領域名は拒否される"""
    _setup_workspace(config_dir, workspace)
    result = run_cli([
        "0",    # メインメニュー → プロジェクト作成
        "1",    # リージョン → business-region
        "",     # 空の領域名
        "2",    # メインメニュー → 終了
    ], config_dir)
    assert result.returncode == 0
    assert "領域名が不正です" in result.stdout


def test_checkout_failure_shows_error(config_dir, workspace):
    """git sparse-checkout 失敗時にエラーメッセージが表示される"""
    _setup_workspace(config_dir, workspace)
    result = run_cli([
        "0",    # メインメニュー → プロジェクト作成
        "0",    # リージョン → system-region
        "0",    # プロジェクト種別 → Library
        "0",    # 言語 → Rust
        "2",    # メインメニュー → 終了
    ], config_dir)
    assert result.returncode == 0
    # ワークスペースは実際のgitリポジトリではないので、
    # checkout失敗のエラーメッセージが表示される
    assert "チェックアウトに失敗しました" in result.stdout


def test_service_region_no_business_regions(config_dir, workspace):
    """Service Region で部門領域が存在しない場合のエラー"""
    _setup_workspace(config_dir, workspace)
    result = run_cli([
        "0",    # メインメニュー → プロジェクト作成
        "2",    # リージョン → service-region
        "2",    # メインメニュー → 終了
    ], config_dir)
    assert result.returncode == 0
    assert "部門固有領域が存在しません" in result.stdout
