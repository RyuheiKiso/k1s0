from conftest import run_cli


def test_exit(config_dir):
    """起動 → 終了"""
    result = run_cli(["2"], config_dir)
    assert result.returncode == 0
    assert "終了します" in result.stdout


def test_banner_displayed(config_dir):
    """バナーが表示される"""
    result = run_cli(["2"], config_dir)
    assert "k1s0" in result.stdout


def test_create_project_without_workspace(config_dir):
    """ワークスペース未設定でプロジェクト作成を選択"""
    result = run_cli([
        "0",  # メインメニュー → プロジェクト作成
        "2",  # メインメニュー → 終了
    ], config_dir)
    assert result.returncode == 0
    assert "未設定" in result.stdout
