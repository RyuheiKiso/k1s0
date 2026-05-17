#!/usr/bin/env python3
"""src/data/audit_chain/verify.py

audit_event テーブルの hash chain divergence を検証するスクリプト。
divergence 検出時は exit code 1 で終了する。
"""

# 標準ライブラリのインポート（外部依存を最小化する）
import argparse
# hashlib: SHA-256 ハッシュ計算に使用する
import hashlib
# json: 結果の JSON 出力とペイロード処理に使用する
import json
# os: 環境変数の読み取りに使用する
import os
# sys: exit code の制御と標準出力に使用する
import sys
# dataclasses: 検証結果を型安全に格納するデータクラスに使用する
from dataclasses import dataclass, field
# datetime: タイムスタンプ処理に使用する
from datetime import datetime, timezone
# typing: 型ヒントの定義に使用する
from typing import Optional

# psycopg2: PostgreSQL への接続に使用するドライバー（外部ライブラリ）
try:
    # psycopg2 のインポートを試みる
    import psycopg2
    # psycopg2 の型変換モジュール
    import psycopg2.extras
except ImportError:
    # psycopg2 が見つからない場合はエラーメッセージを出力して終了する
    print("ERROR: psycopg2 がインストールされていません。pip install psycopg2-binary を実行してください。", file=sys.stderr)
    # 前提条件不足のため exit code 2 で終了する
    sys.exit(2)


# ============================================================
# データクラス定義
# ============================================================


@dataclass
class AuditEvent:
    """audit_event テーブルの 1 行を表すデータクラス。"""

    # 監査イベントの一意識別子（UUID 文字列）
    event_id: str
    # イベント記録時刻（タイムゾーン付き datetime）
    event_at: datetime
    # テナント識別子（UUID 文字列）
    tenant_id: str
    # 監査イベント種別
    event_kind: str
    # イベントペイロード（JSON 文字列として格納）
    payload: str
    # 前のイベントのハッシュ値（None = チェーンの先頭）
    previous_hash: Optional[str]
    # このイベントのハッシュ値（PostgreSQL が GENERATED ALWAYS AS で計算）
    current_hash: str


@dataclass
class ChainVerificationResult:
    """hash chain の検証結果を格納するデータクラス。"""

    # テナント識別子
    tenant_id: str
    # 検証対象イベントの総数
    total_events: int
    # hash chain が正常なイベント数
    valid_count: int
    # hash chain に破断があるイベント数（0 = 正常）
    broken_count: int
    # 破断が検出されたイベントの詳細リスト
    broken_events: list = field(default_factory=list)
    # 検証開始時刻
    verified_at: str = field(default_factory=lambda: datetime.now(timezone.utc).isoformat())


# ============================================================
# ハッシュ計算関数
# ============================================================


def compute_expected_hash(
    previous_hash: Optional[str],
    event_id: str,
    event_kind: str,
    payload: str,
) -> str:
    """期待される current_hash を Python 側で独立計算する。

    PostgreSQL の GENERATED ALWAYS AS 式と同一のアルゴリズムで計算して
    データベース側のハッシュと比較することで改ざんを検知する。

    Args:
        previous_hash: 前のイベントのハッシュ値（None の場合は空文字列として扱う）
        event_id: 対象イベントの UUID 文字列
        event_kind: 監査イベント種別文字列
        payload: イベントペイロードの JSON 文字列

    Returns:
        SHA-256 ハッシュの hex 文字列（PostgreSQL の current_hash と一致するはず）
    """
    # previous_hash が None の場合は空文字列として扱う（PostgreSQL の coalesce と同等）
    prev = previous_hash if previous_hash is not None else ""

    # PostgreSQL の concatenation と同一の順序で文字列を連結する
    # coalesce(previous_hash, '') || event_id || event_kind || payload::text
    raw = prev + event_id + event_kind + payload

    # UTF-8 エンコードして SHA-256 ハッシュを計算する（PostgreSQL の convert_to と同等）
    digest = hashlib.sha256(raw.encode("utf-8")).hexdigest()

    # hex 文字列を返す（PostgreSQL の encode(..., 'hex') と同等）
    return digest


# ============================================================
# PostgreSQL 接続と監査ログ取得
# ============================================================


def get_db_connection(dsn: str):
    """PostgreSQL に接続して接続オブジェクトを返す。

    Args:
        dsn: PostgreSQL 接続文字列（libpq 形式）

    Returns:
        psycopg2 の接続オブジェクト

    Raises:
        SystemExit: 接続に失敗した場合は exit code 3 で終了する
    """
    try:
        # psycopg2 で PostgreSQL に接続する
        conn = psycopg2.connect(dsn)
        # オートコミットを無効化してトランザクション管理を明示的に行う
        conn.autocommit = False
        # 接続成功をログに出力する
        print("INFO: PostgreSQL への接続が成功しました", file=sys.stderr)
        # 接続オブジェクトを返す
        return conn
    except psycopg2.OperationalError as e:
        # 接続失敗のエラーメッセージを出力する
        print(f"ERROR: PostgreSQL への接続に失敗しました: {e}", file=sys.stderr)
        # 接続失敗のため exit code 3 で終了する
        sys.exit(3)


def fetch_audit_events(conn, tenant_id: str, limit: int = 10000) -> list[AuditEvent]:
    """指定テナントの監査イベントを時系列順に取得する。

    Args:
        conn: psycopg2 の接続オブジェクト
        tenant_id: 検証対象のテナント識別子
        limit: 取得する最大レコード数（大量データ時のメモリ保護）

    Returns:
        AuditEvent のリスト（event_at, event_id の昇順）
    """
    # カーソルを作成して SQL クエリを実行する（RealDictCursor で列名アクセスを有効化）
    with conn.cursor(cursor_factory=psycopg2.extras.RealDictCursor) as cur:
        # GUC を設定して RLS のテナント分離を有効化する
        cur.execute("SET app.tenant_id = %s", (tenant_id,))

        # 監査イベントを時系列順に取得する SQL を実行する
        cur.execute(
            """
            SELECT
                event_id::text,
                event_at,
                tenant_id::text,
                event_kind,
                payload::text AS payload,
                previous_hash,
                current_hash
            FROM k1s0.audit_event
            WHERE tenant_id = %s::uuid
            ORDER BY event_at ASC, event_id ASC
            LIMIT %s
            """,
            (tenant_id, limit),
        )

        # 全行を取得してリストに変換する
        rows = cur.fetchall()

    # 各行を AuditEvent データクラスに変換する
    events = []
    for row in rows:
        # 各フィールドを適切な型に変換して AuditEvent を作成する
        events.append(
            AuditEvent(
                event_id=row["event_id"],
                event_at=row["event_at"],
                tenant_id=row["tenant_id"],
                event_kind=row["event_kind"],
                payload=row["payload"],
                previous_hash=row["previous_hash"],
                current_hash=row["current_hash"],
            )
        )

    # 取得したイベント数をログに出力する
    print(f"INFO: {len(events)} 件の監査イベントを取得しました", file=sys.stderr)
    # AuditEvent のリストを返す
    return events


# ============================================================
# hash chain 検証ロジック
# ============================================================


def verify_chain(events: list[AuditEvent], tenant_id: str) -> ChainVerificationResult:
    """監査イベントの hash chain を検証する。

    各イベントの previous_hash が前のイベントの current_hash と一致することを確認する。
    また、Python 側で独立計算した期待ハッシュと current_hash を比較して
    データベース側の改ざんも検知する。

    Args:
        events: AuditEvent のリスト（時系列昇順）
        tenant_id: 検証対象のテナント識別子

    Returns:
        ChainVerificationResult: 検証結果のサマリー
    """
    # 検証結果を初期化する
    result = ChainVerificationResult(
        tenant_id=tenant_id,
        total_events=len(events),
        valid_count=0,
        broken_count=0,
        broken_events=[],
    )

    # イベントが 0 件の場合は検証をスキップする
    if not events:
        print("INFO: 検証対象のイベントが存在しません", file=sys.stderr)
        # 空の検証結果を返す
        return result

    # 前のイベントの current_hash を追跡する変数（最初は None）
    prev_current_hash: Optional[str] = None

    # 各イベントを順番に検証する
    for i, event in enumerate(events):
        # Python 側で期待ハッシュを独立計算する（DB 側の GENERATED ALWAYS AS と独立して検証）
        expected_current_hash = compute_expected_hash(
            previous_hash=event.previous_hash,
            event_id=event.event_id,
            event_kind=event.event_kind,
            payload=event.payload,
        )

        # 1. previous_hash と前のイベントの current_hash が一致するかを確認する
        chain_link_valid = (
            # チェーンの先頭（i=0）は previous_hash が None であることが正常
            (i == 0 and event.previous_hash is None)
            # 2 番目以降は previous_hash が前のイベントの current_hash と一致する
            or (i > 0 and event.previous_hash == prev_current_hash)
        )

        # 2. Python 側で計算した期待ハッシュと DB の current_hash が一致するかを確認する
        hash_integrity_valid = event.current_hash == expected_current_hash

        # 両方の検証が通過した場合は正常カウントをインクリメントする
        if chain_link_valid and hash_integrity_valid:
            result.valid_count += 1
        else:
            # 破断が検出された場合は詳細情報を記録する
            result.broken_count += 1
            # 破断イベントの詳細を broken_events に追加する
            result.broken_events.append(
                {
                    "event_id": event.event_id,
                    "event_at": event.event_at.isoformat(),
                    "event_kind": event.event_kind,
                    "chain_link_valid": chain_link_valid,
                    "hash_integrity_valid": hash_integrity_valid,
                    "expected_previous_hash": prev_current_hash,
                    "actual_previous_hash": event.previous_hash,
                    "expected_current_hash": expected_current_hash,
                    "actual_current_hash": event.current_hash,
                }
            )
            # 破断を警告ログとして出力する
            print(
                f"WARNING: hash chain 破断を検出しました: event_id={event.event_id}",
                file=sys.stderr,
            )

        # 次のイテレーションのために current_hash を更新する
        prev_current_hash = event.current_hash

    # 検証結果のサマリーをログに出力する
    print(
        f"INFO: 検証完了: total={result.total_events}, valid={result.valid_count}, broken={result.broken_count}",
        file=sys.stderr,
    )

    # 検証結果を返す
    return result


# ============================================================
# CLI エントリーポイント
# ============================================================


def parse_args() -> argparse.Namespace:
    """コマンドライン引数を解析して返す。"""
    # ArgumentParser を初期化してスクリプトの説明を設定する
    parser = argparse.ArgumentParser(
        description="audit_event テーブルの hash chain divergence を検証するスクリプト"
    )

    # データベース接続文字列の引数（環境変数 DATABASE_URL からのフォールバックを持つ）
    parser.add_argument(
        "--dsn",
        default=os.environ.get("DATABASE_URL", ""),
        help="PostgreSQL 接続文字列 (デフォルト: 環境変数 DATABASE_URL)",
    )

    # 検証対象のテナント ID（省略時は全テナントを対象とする）
    parser.add_argument(
        "--tenant-id",
        default=None,
        help="検証対象のテナント ID (省略時は全テナントを対象とする)",
    )

    # 取得する最大レコード数（大量データ時のメモリ保護）
    parser.add_argument(
        "--limit",
        type=int,
        default=10000,
        help="取得する最大イベント数 (デフォルト: 10000)",
    )

    # JSON 形式で結果を出力するフラグ（CI での機械的処理に使用する）
    parser.add_argument(
        "--json",
        action="store_true",
        help="結果を JSON 形式で出力する",
    )

    # 解析した引数を返す
    return parser.parse_args()


def main() -> int:
    """メイン処理を実行して exit code を返す。"""
    # コマンドライン引数を解析する
    args = parse_args()

    # DSN が空の場合はエラーを出力して終了する
    if not args.dsn:
        print("ERROR: --dsn または DATABASE_URL 環境変数が必要です", file=sys.stderr)
        # 引数不足のため exit code 2 で終了する
        return 2

    # PostgreSQL に接続する
    conn = get_db_connection(args.dsn)

    try:
        # テナント ID が指定された場合は単一テナントを検証する
        if args.tenant_id:
            # 指定テナントの監査イベントを取得する
            events = fetch_audit_events(conn, args.tenant_id, limit=args.limit)
            # hash chain を検証する
            result = verify_chain(events, args.tenant_id)
            # 検証結果のリストに追加する（後段の出力処理で統一的に扱う）
            results = [result]
        else:
            # 全テナントを対象とする場合はテナント一覧を取得する
            with conn.cursor() as cur:
                # audit_event に存在する全テナント ID を取得する
                cur.execute("SELECT DISTINCT tenant_id::text FROM k1s0.audit_event")
                # テナント ID の一覧を取得する
                tenant_ids = [row[0] for row in cur.fetchall()]

            # 全テナント ID の一覧をログに出力する
            print(f"INFO: {len(tenant_ids)} テナントを検証します", file=sys.stderr)

            # 各テナントの hash chain を順番に検証する
            results = []
            for tenant_id in tenant_ids:
                # テナントの監査イベントを取得する
                events = fetch_audit_events(conn, tenant_id, limit=args.limit)
                # hash chain を検証する
                result = verify_chain(events, tenant_id)
                # 検証結果をリストに追加する
                results.append(result)

        # 結果の出力（--json フラグに応じて形式を切り替える）
        if args.json:
            # JSON 形式で標準出力に出力する（CI での機械的処理に使用）
            output = [
                {
                    "tenant_id": r.tenant_id,
                    "total_events": r.total_events,
                    "valid_count": r.valid_count,
                    "broken_count": r.broken_count,
                    "broken_events": r.broken_events,
                    "verified_at": r.verified_at,
                }
                for r in results
            ]
            # JSON をインデント付きで出力する（人間が読みやすいフォーマット）
            print(json.dumps(output, indent=2, ensure_ascii=False))
        else:
            # テキスト形式でサマリーを出力する
            for r in results:
                print(f"テナント {r.tenant_id}: total={r.total_events}, valid={r.valid_count}, broken={r.broken_count}")

        # 全テナントで broken_count の合計を計算する
        total_broken = sum(r.broken_count for r in results)

        # 破断が検出された場合は exit code 1 で終了する（CI がエラーを検知できるようにする）
        if total_broken > 0:
            print(f"ERROR: 合計 {total_broken} 件の hash chain 破断を検出しました", file=sys.stderr)
            # 破断検出のため exit code 1 で終了する
            return 1

        # 全テナントで破断がなかった場合は正常終了する
        print("INFO: 全テナントの hash chain が正常です", file=sys.stderr)
        # 正常終了のため exit code 0 を返す
        return 0

    finally:
        # データベース接続を確実にクローズする（例外発生時も実行される）
        conn.close()
        # 接続クローズをログに出力する
        print("INFO: PostgreSQL 接続をクローズしました", file=sys.stderr)


# スクリプトとして直接実行された場合のエントリーポイント
if __name__ == "__main__":
    # main() の返り値を exit code として使用する
    sys.exit(main())
