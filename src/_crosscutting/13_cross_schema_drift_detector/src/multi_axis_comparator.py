#!/usr/bin/env python3
"""
multi_axis_comparator.py — k1s0 マルチ軸スキーマバージョン比較器

製造業プラットフォーム k1s0 の複数軸（tier1/tier2/tier3/data/security 等）が
それぞれ独立して進化するスキーマを、軸横断で一括比較してクロス軸のドリフトを検出する。

単一ペア比較（drift_between）ではなく、「軸 A の v2 スキーマ」と「軸 B の v2 スキーマ」の
共有メッセージ定義が一致しているかを確認するクロス軸整合性検証を提供する。

用途:
  - gRPC サービスを跨ぐ共有 Protobuf メッセージの定義一貫性確認
  - Kafka トピックに書き込む全軸のイベントスキーマ互換性確認
  - tier1/tier2/tier3 間でのリクエスト/レスポンス型の一致確認

利用例:
  comparator = MultiAxisComparator()
  comparator.add_axis("tier1", Path("src/tier1/proto/k1s0_transport.proto"))
  comparator.add_axis("tier2", Path("src/tier2/proto/k1s0_transport.proto"))
  report = comparator.compare_shared_messages()
  print(report.as_text())
"""

from __future__ import annotations

# dataclasses: 比較結果の値オブジェクトに使用する
import dataclasses
# logging: 比較過程のデバッグ情報を出力するために使用する
import logging
# pathlib: スキーマファイルパスに使用する
from pathlib import Path
# typing: 型ヒントに使用する
from typing import Dict, FrozenSet, List, Optional, Set, Tuple

# 同一パッケージから必要なクラスをインポートする
from .schema_drift_detector import (
    DriftEntry,
    DriftResult,
    DriftSeverity,
    FieldDef,
    MessageDef,
    SchemaFormat,
    SchemaVersion,
    drift_between,
    load_schema,
)

# ロガーを初期化する（モジュール名をロガー名として使用する）
logger = logging.getLogger(__name__)


@dataclasses.dataclass
class AxisSchema:
    """軸名とスキーマバージョンを組み合わせた保持クラス。"""

    # axis_name: 軸の名前（例: "tier1", "tier2", "data"）
    axis_name: str
    # schema: ロード済みのスキーマバージョン
    schema: SchemaVersion
    # version_tag: このスキーマのバージョンタグ（例: "v1", "v2.3"）
    version_tag: str


@dataclasses.dataclass(frozen=True)
class CrossAxisDrift:
    """2 軸間のスキーマドリフトを記録する不変データクラス。"""

    # axis_a: 比較元の軸名
    axis_a: str
    # axis_b: 比較先の軸名
    axis_b: str
    # message_name: ドリフトが検出された共有メッセージ名
    message_name: str
    # entries: 検出されたドリフトエントリのリスト
    entries: Tuple[DriftEntry, ...]

    @property
    def has_breaking(self) -> bool:
        """BREAKING エントリが存在する場合に True を返す。"""
        # entries に BREAKING エントリが存在するかどうかを確認する
        return any(e.severity == DriftSeverity.BREAKING for e in self.entries)


@dataclasses.dataclass
class CrossAxisReport:
    """MultiAxisComparator の比較結果を格納するデータクラス。"""

    # axis_names: 比較に参加した軸名のリスト
    axis_names: List[str]
    # shared_messages: 複数軸で共有されたメッセージ名の集合
    shared_messages: List[str]
    # cross_drifts: 検出されたクロス軸ドリフトのリスト
    cross_drifts: List[CrossAxisDrift]
    # consistent_messages: 全軸で定義が一致したメッセージ名の集合
    consistent_messages: List[str]

    @property
    def has_cross_breaking(self) -> bool:
        """BREAKING なクロス軸ドリフトが存在する場合に True を返す。"""
        # cross_drifts に BREAKING エントリが存在するかどうかを確認する
        return any(d.has_breaking for d in self.cross_drifts)

    def summary(self) -> str:
        """比較結果のサマリ文字列を返す。"""
        # サマリ行を格納するリストを初期化する
        lines = [
            f"CrossAxisReport: {len(self.axis_names)} axes",
            f"  shared messages  : {len(self.shared_messages)}",
            f"  cross drifts     : {len(self.cross_drifts)}",
            f"  consistent       : {len(self.consistent_messages)}",
            f"  has_cross_breaking: {self.has_cross_breaking}",
        ]
        # BREAKING ドリフトが存在する場合は詳細を追加する
        breaking_drifts = [d for d in self.cross_drifts if d.has_breaking]
        if breaking_drifts:
            lines.append("  --- BREAKING cross-axis drifts ---")
            for drift in breaking_drifts:
                lines.append(
                    f"    {drift.axis_a} <-> {drift.axis_b}: {drift.message_name} "
                    f"({len([e for e in drift.entries if e.severity == DriftSeverity.BREAKING])} breaking)"
                )
        # サマリ文字列を改行で結合して返す
        return "\n".join(lines)

    def as_text(self) -> str:
        """人間が読みやすいテキスト形式でレポートを返す。"""
        # テキスト行を格納するリストを初期化する
        lines = ["=" * 80, "K1s0 クロス軸スキーマドリフトレポート", "=" * 80]
        # 参加軸を追加する
        lines.append(f"参加軸: {', '.join(self.axis_names)}")
        # 共有メッセージ数を追加する
        lines.append(f"共有メッセージ数: {len(self.shared_messages)}")
        # 整合メッセージ数を追加する
        lines.append(f"整合メッセージ数: {len(self.consistent_messages)}")
        # クロスドリフト数を追加する
        lines.append(f"クロスドリフト数: {len(self.cross_drifts)}")
        lines.append("")
        # 各クロスドリフトを出力する
        for drift in self.cross_drifts:
            lines.append(f"  [{drift.axis_a} <-> {drift.axis_b}] {drift.message_name}")
            for entry in drift.entries:
                lines.append(f"    [{entry.severity.value}] {entry.location}: {entry.change_type}")
                lines.append(f"      {entry.description}")
        lines.append("=" * 80)
        # 全行を改行で結合して返す
        return "\n".join(lines)


def _fields_equal(a: FieldDef, b: FieldDef) -> bool:
    """2 つのフィールド定義が等価かどうかを返す。"""
    # フィールド名、型、番号が同一かどうかを確認する
    return (
        a.name == b.name
        and a.field_type == b.field_type
        and a.field_number == b.field_number
        and a.is_repeated == b.is_repeated
    )


def _messages_equal(a: MessageDef, b: MessageDef) -> bool:
    """2 つのメッセージ定義が等価かどうかを返す。"""
    # フィールド名の集合が一致するかを確認する
    if set(a.fields.keys()) != set(b.fields.keys()):
        return False
    # 各フィールドの定義が一致するかを確認する
    for fname in a.fields:
        if not _fields_equal(a.fields[fname], b.fields[fname]):
            return False
    # 全フィールドが一致した場合は True を返す
    return True


class MultiAxisComparator:
    """
    複数軸のスキーマを管理して共有メッセージのクロス軸ドリフトを検出するクラス。

    使用例:
      comparator = MultiAxisComparator()
      comparator.add_axis("tier1", Path("tier1/transport.proto"), "v2")
      comparator.add_axis("tier2", Path("tier2/transport.proto"), "v2")
      report = comparator.compare_shared_messages()
    """

    def __init__(self) -> None:
        """空の MultiAxisComparator を初期化する。"""
        # _axes: 軸名 -> AxisSchema のマッピング辞書を初期化する
        self._axes: Dict[str, AxisSchema] = {}

    def add_axis(
        self,
        axis_name: str,
        schema_path: Path,
        version_tag: str = "latest",
    ) -> None:
        """軸名とスキーマファイルパスを登録する。"""
        # スキーマを読み込む
        schema = load_schema(schema_path)
        # AxisSchema を生成する
        axis_schema = AxisSchema(
            axis_name=axis_name,
            schema=schema,
            version_tag=version_tag,
        )
        # 辞書に追加する
        self._axes[axis_name] = axis_schema
        # 登録をデバッグログに記録する
        logger.debug(
            "軸 '%s' を登録しました: path=%s, messages=%d, enums=%d",
            axis_name,
            schema_path,
            len(schema.messages),
            len(schema.enums),
        )

    def add_axis_from_content(
        self,
        axis_name: str,
        content: str,
        schema_format: SchemaFormat,
        version_tag: str = "latest",
    ) -> None:
        """文字列コンテンツからスキーマを直接登録する（テスト用途向け）。"""
        # スキーマを解析する
        from .schema_drift_detector import parse_proto_content, parse_avro_content
        if schema_format == SchemaFormat.PROTO:
            schema = parse_proto_content(content)
        elif schema_format == SchemaFormat.AVRO:
            schema = parse_avro_content(content)
        else:
            raise ValueError(f"非対応のスキーマ形式: {schema_format}")
        # AxisSchema を生成して登録する
        self._axes[axis_name] = AxisSchema(
            axis_name=axis_name,
            schema=schema,
            version_tag=version_tag,
        )

    def _find_shared_messages(self) -> Set[str]:
        """複数軸で定義されているメッセージ名の集合を返す。"""
        # 軸が 2 つ未満の場合は空集合を返す
        if len(self._axes) < 2:
            return set()
        # 全軸のメッセージ名集合を収集する
        axes_list = list(self._axes.values())
        # 最初の軸のメッセージ名集合を基準とする
        shared = set(axes_list[0].schema.messages.keys())
        # 全軸のメッセージ名の共通部分を計算する
        for axis_schema in axes_list[1:]:
            shared &= set(axis_schema.schema.messages.keys())
        # 共有メッセージ名の集合を返す
        return shared

    def compare_shared_messages(self) -> CrossAxisReport:
        """
        全登録軸の共有メッセージを軸間で比較してクロスドリフトレポートを返す。

        共有メッセージとは、2 つ以上の軸で同名のメッセージが定義されているもの。
        各軸ペアで drift_between を実行してドリフトエントリを収集する。
        """
        # 軸名のリストを取得する
        axis_names = list(self._axes.keys())
        # 共有メッセージ名を取得する
        shared_msg_names = self._find_shared_messages()
        # クロスドリフトを格納するリストを初期化する
        cross_drifts: List[CrossAxisDrift] = []
        # 整合メッセージを格納するリストを初期化する
        consistent_messages: List[str] = []
        # 共有メッセージが存在しない場合は空のレポートを返す
        if not shared_msg_names:
            return CrossAxisReport(
                axis_names=axis_names,
                shared_messages=[],
                cross_drifts=[],
                consistent_messages=[],
            )
        # 各軸ペアについて比較する
        axes_list = list(self._axes.values())
        # 共有メッセージごとに一致確認フラグを管理する辞書を初期化する
        msg_consistent: Dict[str, bool] = {m: True for m in shared_msg_names}
        # 全ペアの組み合わせを処理する
        for i in range(len(axes_list)):
            for j in range(i + 1, len(axes_list)):
                # 比較する 2 軸を取得する
                axis_a = axes_list[i]
                axis_b = axes_list[j]
                # 各共有メッセージについてドリフトを検出する
                for msg_name in shared_msg_names:
                    # 両軸のメッセージ定義が等価かどうかを確認する
                    msg_a = axis_a.schema.messages[msg_name]
                    msg_b = axis_b.schema.messages[msg_name]
                    # メッセージが等価な場合はスキップする
                    if _messages_equal(msg_a, msg_b):
                        continue
                    # 等価でない場合は drift_between で詳細を取得する
                    # SchemaVersion を一時的に構築して drift_between を呼び出す
                    schema_a = SchemaVersion(
                        path=axis_a.schema.path,
                        schema_format=axis_a.schema.schema_format,
                        messages={msg_name: msg_a},
                        enums={},
                        raw_content="",
                    )
                    schema_b = SchemaVersion(
                        path=axis_b.schema.path,
                        schema_format=axis_b.schema.schema_format,
                        messages={msg_name: msg_b},
                        enums={},
                        raw_content="",
                    )
                    # drift を計算する
                    drift_result = drift_between(schema_a, schema_b)
                    # エントリが存在する場合はクロスドリフトとして記録する
                    if drift_result.entries:
                        cross_drift = CrossAxisDrift(
                            axis_a=axis_a.axis_name,
                            axis_b=axis_b.axis_name,
                            message_name=msg_name,
                            entries=tuple(drift_result.entries),
                        )
                        cross_drifts.append(cross_drift)
                        # 整合フラグを False に設定する
                        msg_consistent[msg_name] = False
                    # ドリフト検出をデバッグログに記録する
                    logger.debug(
                        "クロス軸比較: %s <-> %s / %s: %d エントリ",
                        axis_a.axis_name,
                        axis_b.axis_name,
                        msg_name,
                        len(drift_result.entries),
                    )
        # 整合メッセージのリストを構築する
        for msg_name, is_consistent in msg_consistent.items():
            if is_consistent:
                consistent_messages.append(msg_name)
        # CrossAxisReport を生成して返す
        return CrossAxisReport(
            axis_names=axis_names,
            shared_messages=sorted(shared_msg_names),
            cross_drifts=cross_drifts,
            consistent_messages=sorted(consistent_messages),
        )

    def axis_count(self) -> int:
        """登録済みの軸数を返す。"""
        # 辞書のキー数を返す
        return len(self._axes)

    def axis_names(self) -> List[str]:
        """登録済みの軸名のリストを返す。"""
        # 辞書のキーをリストで返す
        return list(self._axes.keys())
