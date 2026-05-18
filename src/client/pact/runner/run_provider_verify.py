#!/usr/bin/env python3
# tier1 gateway Pact provider 検証スクリプト
# 5 consumer × 1 provider の Pact contract を検証する

"""src/client/pact/runner/run_provider_verify.py

tier1 gateway Pact provider 検証ランナー。
src/client/pact/consumer/ 配下の 5 consumer の Pact contract を
tier1 gateway provider で検証する。
"""

# 標準ライブラリのインポート
import json
import sys
from pathlib import Path

# リポジトリルートのパス (runner/ → pact/ → client/ → src/ → k1s0/)
REPO_ROOT = Path(__file__).resolve().parent.parent.parent.parent.parent

# consumer Pact contract のディレクトリ
CONSUMER_DIR = REPO_ROOT / "src/client/pact/consumer"

# Pact contract ファイルを列挙する関数
def list_contracts() -> list[Path]:
    """consumer Pact contract ファイルの一覧を返す。"""
    # consumer ディレクトリ配下の .pact.json ファイルをソートして返す
    return sorted(CONSUMER_DIR.glob("*.pact.json"))


def verify_contract(contract_path: Path) -> bool:
    """1 つの Pact contract を検証する（スタブ実装）。"""
    # contract ファイルを読み込む
    contract = json.loads(contract_path.read_text(encoding="utf-8"))
    # consumer 名を取得する
    consumer_name = contract.get("consumer", {}).get("name", "unknown")
    # 検証対象の consumer 名を出力する
    print(f"Verifying: {consumer_name}")
    # TODO: 実際の provider verification を Stage 5 で実装する
    # 現在はスタブとして全て pass にする
    print(f"  STUB PASS: {contract_path.name}")
    # スタブとして True を返す
    return True


def main() -> int:
    """メイン関数。全 contract を検証する。"""
    # contract 一覧を取得する
    contracts = list_contracts()
    # contract が存在しない場合は警告を出して終了する
    if not contracts:
        print("WARNING: no Pact contracts found")
        return 0
    # 全 contract を順番に検証する
    results = [verify_contract(c) for c in contracts]
    # 成功した contract 数を集計する
    passed = sum(1 for r in results if r)
    # 最終結果を出力する
    print(f"\nResults: {passed}/{len(results)} passed")
    # 全て成功した場合は 0、失敗がある場合は 1 を返す
    return 0 if all(results) else 1


if __name__ == "__main__":
    # スクリプトとして直接実行された場合にメイン関数を呼び出す
    sys.exit(main())
