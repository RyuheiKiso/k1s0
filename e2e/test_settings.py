from pathlib import Path

from conftest import run_cli


def test_set_and_show_workspace(config_dir, workspace):
    """ワークスペース設定 → 確認 → 戻る → 終了"""
    result = run_cli([
        "1",                # メインメニュー → 設定
        "1",                # 設定メニュー → パス設定
        str(workspace),     # パス入力
        "0",                # 設定メニュー → パス確認
        "2",                # 設定メニュー → 戻る
        "2",                # メインメニュー → 終了
    ], config_dir)
    assert result.returncode == 0
    assert "保存しました" in result.stdout
    assert str(workspace) in result.stdout


def test_workspace_persists_across_runs(config_dir, workspace):
    """ワークスペース設定がTOMLファイルに永続化される"""
    # 1回目: 設定
    run_cli([
        "1", "1", str(workspace), "2", "2",
    ], config_dir)

    # 2回目: 確認
    result = run_cli([
        "1",    # メインメニュー → 設定
        "0",    # 設定メニュー → パス確認
        "2",    # 設定メニュー → 戻る
        "2",    # メインメニュー → 終了
    ], config_dir)
    assert result.returncode == 0
    assert str(workspace) in result.stdout

    # TOMLファイルが存在する
    config_file = config_dir / "config.toml"
    assert config_file.exists()
    content = config_file.read_text()
    assert str(workspace).replace("\\", "\\\\") in content or str(workspace) in content


def test_show_workspace_when_not_set(config_dir):
    """ワークスペース未設定時の確認"""
    result = run_cli([
        "1",    # メインメニュー → 設定
        "0",    # 設定メニュー → パス確認
        "2",    # 設定メニュー → 戻る
        "2",    # メインメニュー → 終了
    ], config_dir)
    assert result.returncode == 0
    assert "未設定" in result.stdout


def test_set_invalid_workspace_relative_path(config_dir):
    """相対パスは拒否される"""
    result = run_cli([
        "1",                # メインメニュー → 設定
        "1",                # 設定メニュー → パス設定
        "relative/path",    # 相対パス入力
        "2",                # 設定メニュー → 戻る
        "2",                # メインメニュー → 終了
    ], config_dir)
    assert result.returncode == 0
    assert "無効" in result.stdout


def test_set_invalid_workspace_empty(config_dir):
    """空パスは拒否される"""
    result = run_cli([
        "1",    # メインメニュー → 設定
        "1",    # 設定メニュー → パス設定
        "",     # 空入力
        "2",    # 設定メニュー → 戻る
        "2",    # メインメニュー → 終了
    ], config_dir)
    assert result.returncode == 0
    assert "無効" in result.stdout
