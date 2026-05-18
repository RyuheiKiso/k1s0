#!/usr/bin/env python3
# oss_inventory_sync_check/run.py — oss_inventory.lock.yaml と 01_OSS採用一覧.md の整合検証
# 08_OSSライフサイクル適合仕様.md §oss_lifecycle に準拠する
# lock yaml と docs の entry-equal を検証する CI linter
# 使用方法: python tools/oss_inventory_sync_check/run.py [--fix]

# 標準ライブラリのインポート
import argparse
import pathlib
import sys
import re

# プロジェクトルートを計算する（tools/ の 1 段上がプロジェクトルート）
PROJECT_ROOT = pathlib.Path(__file__).parent.parent.parent

# oss_inventory.lock.yaml のパス（src/tier1/lock/ に格納される build artifact）
OSS_INVENTORY_LOCK = PROJECT_ROOT / "src" / "tier1" / "lock" / "oss_inventory.lock.yaml"

# 01_OSS採用一覧.md のパス（docs の SoT ドキュメント）
OSS_LIST_DOC = PROJECT_ROOT / "docs" / "02_要件定義" / "04_技術選定" / "01_OSS採用一覧.md"


def parse_oss_inventory_lock(path: pathlib.Path) -> set[str]:
    """oss_inventory.lock.yaml から OSS パッケージ名一覧を抽出する関数"""
    # ファイルが存在しない場合はエラーを出力して終了する
    if not path.exists():
        # エラーメッセージを stderr に出力する
        print(f"ERROR: {path} が見つかりません", file=sys.stderr)
        # 異常終了する
        sys.exit(1)
    # ファイルを UTF-8 で読み込む
    content = path.read_text(encoding="utf-8")
    # package_name フィールドを正規表現で抽出する
    names: set[str] = set()
    # packageName または package_name フィールドを抽出する（camelCase / snake_case 両対応）
    for m in re.finditer(r"^\s+(?:packageName|package_name):\s+(.+)$", content, re.MULTILINE):
        # 値の前後の空白と引用符を除去して正規化する
        name = m.group(1).strip().strip('"\'')
        # 空でない場合のみ追加する
        if name:
            # セットに追加する（重複を自動除去する）
            names.add(name)
    # 抽出した名前セットを返す
    return names


def parse_oss_list_doc(path: pathlib.Path) -> set[str]:
    """01_OSS採用一覧.md から OSS パッケージ名一覧を抽出する関数"""
    # ファイルが存在しない場合はエラーを出力して終了する
    if not path.exists():
        # エラーメッセージを stderr に出力する
        print(f"ERROR: {path} が見つかりません", file=sys.stderr)
        # 異常終了する
        sys.exit(1)
    # ファイルを UTF-8 で読み込む
    content = path.read_text(encoding="utf-8")
    # Markdown テーブルの最初のカラム（パッケージ名）を抽出する
    names: set[str] = set()
    # パイプ区切りテーブルの行を正規表現でマッチする
    for m in re.finditer(r"^\|\s*([^\|]+?)\s*\|", content, re.MULTILINE):
        # セルの値を取得して前後の空白を除去する
        cell = m.group(1).strip()
        # ヘッダ行（名前 / Name / OSS 等）と区切り行（--- で構成）を除外する
        if not cell or cell.startswith("-") or cell.lower() in (
            "name", "名前", "oss", "パッケージ", "package", "ライブラリ", "library",
        ):
            # ヘッダ / 区切り行はスキップする
            continue
        # バッククォートで囲まれた名前（`name` 形式）を展開する
        cell = cell.strip("`")
        # 空でない場合のみ追加する
        if cell:
            # セットに追加する（重複を自動除去する）
            names.add(cell)
    # 抽出した名前セットを返す
    return names


def main() -> int:
    """メイン関数: oss_inventory.lock.yaml と 01_OSS採用一覧.md の整合を検証する"""
    # コマンドライン引数を解析する
    parser = argparse.ArgumentParser(
        # ツールの説明文
        description="OSS inventory sync check: lock yaml と docs の整合を検証する",
    )
    # --fix オプション: 差分がある場合に自動修正するフラグ（将来実装予定）
    parser.add_argument(
        "--fix",
        # ストアアクション: フラグが指定された場合に True を設定する
        action="store_true",
        # ヘルプ文
        help="差分がある場合に自動修正する（現バージョンでは未実装）",
    )
    # 引数を解析する
    args = parser.parse_args()

    # oss_inventory.lock.yaml を解析してパッケージ名セットを取得する
    lock_names = parse_oss_inventory_lock(OSS_INVENTORY_LOCK)
    # 解析結果をコンソールに出力する
    print(f"oss_inventory.lock.yaml: {len(lock_names)} エントリ")

    # 01_OSS採用一覧.md を解析してパッケージ名セットを取得する
    doc_names = parse_oss_list_doc(OSS_LIST_DOC)
    # 解析結果をコンソールに出力する
    print(f"01_OSS採用一覧.md: {len(doc_names)} エントリ")

    # lock にあって doc にないエントリを検出する（lock が SoT: doc が古い場合に発生する）
    lock_only = lock_names - doc_names
    # doc にあって lock にないエントリを検出する（doc が SoT: lock が古い場合に発生する）
    doc_only = doc_names - lock_names

    # lock にあって doc にないエントリを報告する
    if lock_only:
        # エラーメッセージを stderr に出力する
        print(
            f"ERROR: lock にあって doc にないエントリ ({len(lock_only)} 件): "
            f"{sorted(lock_only)}",
            file=sys.stderr,
        )
    # doc にあって lock にないエントリを報告する
    if doc_only:
        # エラーメッセージを stderr に出力する
        print(
            f"ERROR: doc にあって lock にないエントリ ({len(doc_only)} 件): "
            f"{sorted(doc_only)}",
            file=sys.stderr,
        )

    # --fix オプションが指定された場合に警告を出力する（現バージョンでは未実装）
    if args.fix and (lock_only or doc_only):
        # 未実装の警告を出力する
        print("WARN: --fix は現バージョンでは未実装です。手動で差分を解消してください。", file=sys.stderr)

    # 差分がない場合は成功メッセージを出力する
    if not lock_only and not doc_only:
        # 成功メッセージを出力する
        print("OK: oss_inventory.lock.yaml と 01_OSS採用一覧.md は整合しています")
        # 正常終了コードを返す
        return 0

    # 差分がある場合は exit code 1 を返す（CI が fail になる）
    return 1


# スクリプトとして実行された場合にメイン関数を呼び出す
if __name__ == "__main__":
    # メイン関数の戻り値を exit code として使用する
    sys.exit(main())
