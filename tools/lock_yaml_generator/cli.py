"""tools/lock_yaml_generator/cli.py

lock.yaml generator CLI エントリポイント。

使用例:
  python -m tools.lock_yaml_generator.cli axis_registry
  python -m tools.lock_yaml_generator.cli release_gate --output src/_meta/lock/release_gate.lock.yaml
  python -m tools.lock_yaml_generator.cli all
  python -m tools.lock_yaml_generator.cli all --check-only
"""

from __future__ import annotations

import hashlib
import sys
from pathlib import Path

try:
    import click
except ImportError:
    print("ERROR: click required: pip install click", file=sys.stderr)
    sys.exit(2)

# リポジトリルートを sys.path に追加
REPO_ROOT = Path(__file__).resolve().parent.parent.parent
sys.path.insert(0, str(REPO_ROOT))

from tools.lock_yaml_generator.lock_registry import (
    ALL_GENERATORS,
    TOPO_ORDER,
    get_generator,
)


@click.group()
def cli() -> None:
    """k1s0 lock.yaml generator suite."""


def _run_generator(name: str, output_dir: Path, check_only: bool) -> bool:
    """1 つの generator を実行し、成功なら True を返す。

    check_only=True の場合は bit-for-bit 検証のみ行い、ファイルを更新しない。
    """
    gen = get_generator(name)
    if gen is None:
        click.echo(f"ERROR: generator '{name}' not found", err=True)
        return False

    if check_only:
        # 既存ファイルと生成内容を比較
        import io
        import contextlib

        gen_instance = gen()
        inputs = gen_instance.load_inputs(output_dir)
        artifact = gen_instance.build_artifact(inputs)
        expected = gen_instance.dump_deterministic(artifact)
        existing_path = output_dir / gen_instance.OUTPUT_NAME
        if not existing_path.exists():
            click.echo(f"FAIL: {existing_path} not found (check-only mode)", err=True)
            return False
        actual = existing_path.read_text(encoding="utf-8")
        if hashlib.sha256(expected.encode()).hexdigest() != hashlib.sha256(actual.encode()).hexdigest():
            click.echo(f"FAIL: {existing_path} is not bit-for-bit reproducible", err=True)
            return False
        click.echo(f"OK (bit-for-bit): {existing_path}")
        return True
    else:
        gen_instance = gen()
        output_path = gen_instance.emit(output_dir)
        click.echo(f"Generated: {output_path}")
        return True


# 各 generator のサブコマンドを動的登録
def _make_subcommand(gen_name: str) -> click.Command:
    @click.command(name=gen_name)
    @click.option("--output-dir", default=None,
                  help="出力先ディレクトリ（デフォルト: src/_meta/lock/ 等 generator 依存）")
    @click.option("--check-only", is_flag=True, default=False,
                  help="bit-for-bit reproducibility の確認のみ（ファイル非更新）")
    def _cmd(output_dir: str | None, check_only: bool) -> None:
        """Generator subcommand."""
        gen_cls = get_generator(gen_name)
        if gen_cls is None:
            click.echo(f"ERROR: generator '{gen_name}' not found", err=True)
            sys.exit(1)
        gen = gen_cls()
        # デフォルト出力先はリポジトリルートからの相対で決まる
        if output_dir:
            out = Path(output_dir)
        else:
            out = REPO_ROOT / gen.DEFAULT_OUTPUT_DIR  # type: ignore[attr-defined]
        ok = _run_generator(gen_name, out, check_only)
        if not ok:
            sys.exit(1)

    _cmd.name = gen_name
    return _cmd


for _gen_name in ALL_GENERATORS:
    cli.add_command(_make_subcommand(_gen_name))


@cli.command("all")
@click.option("--check-only", is_flag=True, default=False,
              help="bit-for-bit reproducibility の確認のみ（ファイル非更新）")
def cmd_all(check_only: bool) -> None:
    """全 generator を topological sort 順で実行する。"""
    failed: list[str] = []
    for name in TOPO_ORDER:
        gen_cls = get_generator(name)
        if gen_cls is None:
            continue
        gen = gen_cls()
        out = REPO_ROOT / gen.DEFAULT_OUTPUT_DIR  # type: ignore[attr-defined]
        ok = _run_generator(name, out, check_only)
        if not ok:
            failed.append(name)
    if failed:
        click.echo(f"FAILED: {failed}", err=True)
        sys.exit(1)
    click.echo("all generators OK")


if __name__ == "__main__":
    cli()
