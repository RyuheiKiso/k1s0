import subprocess
import os
from pathlib import Path

import pytest

CLI_PATH = Path(__file__).parent.parent / "CLI" / "target" / "debug" / "k1s0.exe"


@pytest.fixture
def workspace(tmp_path):
    ws = tmp_path / "workspace"
    ws.mkdir()
    return ws


@pytest.fixture
def config_dir(tmp_path):
    d = tmp_path / "config"
    d.mkdir()
    return d


def run_cli(
    selections: list[str],
    config_dir: Path,
    timeout: int = 10,
) -> subprocess.CompletedProcess:
    stdin_input = "\n".join(selections) + "\n"
    env = {
        **os.environ,
        "K1S0_STDIN_MODE": "1",
        "K1S0_CONFIG_DIR": str(config_dir),
    }
    return subprocess.run(
        [str(CLI_PATH)],
        input=stdin_input,
        capture_output=True,
        text=True,
        timeout=timeout,
        env=env,
        encoding="utf-8",
    )
