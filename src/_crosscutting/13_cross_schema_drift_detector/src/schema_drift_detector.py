#!/usr/bin/env python3
"""
schema_drift_detector.py — k1s0 クロス軸スキーマドリフト検出器

製造業プラットフォーム k1s0 において、複数軸（tier1/tier2/tier3/data/security 等）が
使用する protobuf スキーマ（.proto）および Avro スキーマ（.avsc）のバージョン間変化を
検出し、後方互換性に関する問題を自動分類する。

検出する変化の分類:
  BREAKING   : フィールド削除、型変更、必須化など後方互換性を壊す変化
  ADDITIVE   : 新規オプションフィールドの追加、新規 enum 値など安全な拡張
  SAFE       : コメント変更、オプション変更などバイナリに影響しない変化

利用例:
  old_schema = load_schema(Path("v1/k1s0_transport.proto"))
  new_schema = load_schema(Path("v2/k1s0_transport.proto"))
  result = drift_between(old_schema, new_schema)
  print(result.summary())

対応スキーマ形式:
  - Protocol Buffers v3 (.proto): message/enum/field の変化を追跡
  - Apache Avro (.avsc, JSON 形式): record/field/type の変化を追跡
"""

from __future__ import annotations

# dataclasses: スキーマ表現と検出結果の値オブジェクトに使用する
import dataclasses
# enum: 変化の分類を列挙型で表現するために使用する
import enum
# json: Avro スキーマ（.avsc）の解析に使用する
import json
# logging: 解析過程のデバッグ情報を出力するために使用する
import logging
# pathlib: スキーマファイルパスをプラットフォーム非依存に扱うために使用する
from pathlib import Path
# re: .proto ファイルの簡易パースに正規表現を使用する
import re
# typing: 型ヒントに使用する
from typing import Dict, FrozenSet, List, Optional, Set, Tuple

# ロガーを初期化する（モジュール名をロガー名として使用する）
logger = logging.getLogger(__name__)

# protobuf フィールド型の互換性マトリクス（old_type -> 互換 new_types の集合）を定義する
# ワイヤ型が同一の型間のみ互換性を持つ（protobuf wire type compatibility）
_PROTO_COMPATIBLE_TYPES: Dict[str, FrozenSet[str]] = {
    "int32": frozenset({"int32", "sint32", "sfixed32"}),
    "int64": frozenset({"int64", "sint64", "sfixed64"}),
    "uint32": frozenset({"uint32", "fixed32"}),
    "uint64": frozenset({"uint64", "fixed64"}),
    "sint32": frozenset({"int32", "sint32", "sfixed32"}),
    "sint64": frozenset({"int64", "sint64", "sfixed64"}),
    "fixed32": frozenset({"uint32", "fixed32"}),
    "fixed64": frozenset({"uint64", "fixed64"}),
    "sfixed32": frozenset({"int32", "sint32", "sfixed32"}),
    "sfixed64": frozenset({"int64", "sint64", "sfixed64"}),
    "bool": frozenset({"bool"}),
    "string": frozenset({"string", "bytes"}),
    "bytes": frozenset({"string", "bytes"}),
    "float": frozenset({"float"}),
    "double": frozenset({"double"}),
}

# Avro 型の互換性マトリクスを定義する（reader の型が writer の型を読めるかどうか）
_AVRO_PROMOTABLE_TYPES: Dict[str, FrozenSet[str]] = {
    "int": frozenset({"int", "long", "float", "double"}),
    "long": frozenset({"long", "float", "double"}),
    "float": frozenset({"float", "double"}),
    "double": frozenset({"double"}),
    "string": frozenset({"string", "bytes"}),
    "bytes": frozenset({"bytes", "string"}),
    "boolean": frozenset({"boolean"}),
    "null": frozenset({"null"}),
}


class DriftSeverity(enum.Enum):
    """スキーマドリフトの深刻度を表す列挙型。"""

    # BREAKING: 後方互換性を壊す変化（デプロイには全クライアントのアップグレードが必要）
    BREAKING = "BREAKING"
    # ADDITIVE: 安全な拡張（古いクライアントは新フィールドを無視できる）
    ADDITIVE = "ADDITIVE"
    # SAFE: バイナリ互換性に影響しない変化（コメント、オプション等）
    SAFE = "SAFE"


class SchemaFormat(enum.Enum):
    """スキーマファイルの形式を表す列挙型。"""

    # PROTO: Protocol Buffers v3 形式（.proto ファイル）
    PROTO = "proto"
    # AVRO: Apache Avro 形式（.avsc ファイル、JSON）
    AVRO = "avro"
    # UNKNOWN: 不明な形式（解析スキップ）
    UNKNOWN = "unknown"


@dataclasses.dataclass(frozen=True)
class FieldDef:
    """スキーマフィールドの定義を表す不変データクラス。"""

    # name: フィールド名
    name: str
    # field_type: フィールドの型（protobuf 型名 or Avro 型名）
    field_type: str
    # field_number: protobuf のフィールド番号（Avro では 0 を使用する）
    field_number: int
    # is_repeated: repeated フィールド（配列）かどうか（protobuf 用）
    is_repeated: bool
    # is_optional: optional フィールドかどうか（Avro の nullable union 含む）
    is_optional: bool
    # default_value: デフォルト値の文字列表現（存在する場合）
    default_value: Optional[str]


@dataclasses.dataclass(frozen=True)
class MessageDef:
    """protobuf message または Avro record の定義を表す不変データクラス。"""

    # name: メッセージ/レコード名
    name: str
    # fields: フィールド定義の辞書（フィールド名 -> FieldDef）
    fields: Dict[str, FieldDef]
    # field_numbers: フィールド番号 -> フィールド名のマッピング（protobuf 用）
    field_numbers: Dict[int, str]


@dataclasses.dataclass(frozen=True)
class EnumDef:
    """protobuf enum または Avro enum の定義を表す不変データクラス。"""

    # name: enum 名
    name: str
    # values: enum 値の集合（値名のみ）
    values: FrozenSet[str]
    # value_numbers: 値名 -> 数値のマッピング（protobuf 用）
    value_numbers: Dict[str, int]


@dataclasses.dataclass
class SchemaVersion:
    """
    単一バージョンのスキーマを表すデータクラス。

    load_schema() または parse_proto_content() / parse_avro_content() で生成する。
    """

    # path: スキーマファイルのパス（生成元ファイルの追跡用）
    path: Optional[Path]
    # format: スキーマの形式（PROTO / AVRO / UNKNOWN）
    schema_format: SchemaFormat
    # messages: メッセージ/レコード定義の辞書（名前 -> MessageDef）
    messages: Dict[str, MessageDef]
    # enums: enum 定義の辞書（名前 -> EnumDef）
    enums: Dict[str, EnumDef]
    # raw_content: 元のファイル内容（デバッグ用）
    raw_content: str


@dataclasses.dataclass(frozen=True)
class DriftEntry:
    """スキーマ間の単一の差分エントリを表す不変データクラス。"""

    # severity: この差分の深刻度（BREAKING / ADDITIVE / SAFE）
    severity: DriftSeverity
    # location: 差分が発生した場所（"Message.field_name" 形式）
    location: str
    # change_type: 変化の種類（"field_removed", "type_changed" など）
    change_type: str
    # old_value: 変化前の値の文字列表現
    old_value: str
    # new_value: 変化後の値の文字列表現
    new_value: str
    # description: 人間が読める説明文
    description: str

    def __str__(self) -> str:
        """差分エントリを人間が読める文字列に変換する。"""
        # [深刻度] 場所: 変化の種類 (old -> new): 説明 の形式で返す
        return (
            f"[{self.severity.value}] {self.location}: "
            f"{self.change_type} ({self.old_value!r} -> {self.new_value!r}): "
            f"{self.description}"
        )


@dataclasses.dataclass
class DriftResult:
    """
    drift_between() の検出結果を格納するデータクラス。

    entries リストに全差分エントリが格納される。
    """

    # entries: 検出された全差分エントリのリスト
    entries: List[DriftEntry]
    # old_path: 旧スキーマのパス（追跡用）
    old_path: Optional[Path]
    # new_path: 新スキーマのパス（追跡用）
    new_path: Optional[Path]

    @property
    def has_breaking_changes(self) -> bool:
        """BREAKING 分類のエントリが存在する場合に True を返す。"""
        # entries に BREAKING エントリが存在するかどうかを確認する
        return any(e.severity == DriftSeverity.BREAKING for e in self.entries)

    @property
    def breaking_entries(self) -> List[DriftEntry]:
        """BREAKING 分類のエントリのみを抽出して返す。"""
        # BREAKING エントリのみをフィルタリングして返す
        return [e for e in self.entries if e.severity == DriftSeverity.BREAKING]

    @property
    def additive_entries(self) -> List[DriftEntry]:
        """ADDITIVE 分類のエントリのみを抽出して返す。"""
        # ADDITIVE エントリのみをフィルタリングして返す
        return [e for e in self.entries if e.severity == DriftSeverity.ADDITIVE]

    @property
    def safe_entries(self) -> List[DriftEntry]:
        """SAFE 分類のエントリのみを抽出して返す。"""
        # SAFE エントリのみをフィルタリングして返す
        return [e for e in self.entries if e.severity == DriftSeverity.SAFE]

    def summary(self) -> str:
        """検出結果のサマリ文字列を返す。"""
        # 総エントリ数と深刻度別の内訳を含むサマリを構築する
        lines = [
            f"DriftResult: {len(self.entries)} entries total",
            f"  BREAKING : {len(self.breaking_entries)}",
            f"  ADDITIVE : {len(self.additive_entries)}",
            f"  SAFE     : {len(self.safe_entries)}",
        ]
        # BREAKING エントリが存在する場合は詳細を追加する
        if self.breaking_entries:
            lines.append("  --- BREAKING changes ---")
            for entry in self.breaking_entries:
                lines.append(f"    {entry}")
        # サマリ文字列を改行で結合して返す
        return "\n".join(lines)


def _detect_schema_format(path: Path) -> SchemaFormat:
    """ファイルの拡張子からスキーマ形式を判定して返す。"""
    # .proto 拡張子の場合は PROTO を返す
    if path.suffix == ".proto":
        return SchemaFormat.PROTO
    # .avsc 拡張子の場合は AVRO を返す
    if path.suffix == ".avsc":
        return SchemaFormat.AVRO
    # .json 拡張子の場合は内容を確認して Avro かどうかを判定する
    if path.suffix == ".json":
        return SchemaFormat.AVRO
    # それ以外は UNKNOWN を返す
    return SchemaFormat.UNKNOWN


def _parse_proto_fields(message_body: str) -> Tuple[Dict[str, FieldDef], Dict[int, str]]:
    """
    protobuf message ボディ文字列からフィールド定義を抽出する。

    戻り値: (フィールド名 -> FieldDef の辞書, フィールド番号 -> フィールド名の辞書)
    """
    # フィールド定義を格納する辞書を初期化する
    fields: Dict[str, FieldDef] = {}
    # フィールド番号マッピングを初期化する
    field_numbers: Dict[int, str] = {}
    # コメントを除去した後のボディを処理する（// コメントを削除する）
    body_no_comments = re.sub(r"//[^\n]*", "", message_body)
    # フィールド定義を正規表現でマッチする
    # パターン: [repeated/optional] <type> <name> = <number> [default = <value>];
    field_pattern = re.compile(
        r"\b(repeated\s+|optional\s+)?"  # ラベル（省略可能）
        r"([\w.]+)\s+"                   # 型名
        r"(\w+)\s*=\s*(\d+)"             # フィールド名とフィールド番号
        r"(?:\s*\[.*?\])?"               # オプション（省略可能）
        r"\s*;",                          # セミコロン
        re.DOTALL,
    )
    # ボディを走査してフィールドを抽出する
    for match in field_pattern.finditer(body_no_comments):
        # ラベル部分を取得する（repeated/optional）
        label = (match.group(1) or "").strip()
        # 型名を取得する
        field_type = match.group(2)
        # フィールド名を取得する
        field_name = match.group(3)
        # フィールド番号を整数として取得する
        try:
            field_num = int(match.group(4))
        except ValueError:
            # フィールド番号のパースに失敗した場合はスキップする
            continue
        # reserved キーワードをスキップする
        if field_name in ("reserved", "option", "extensions", "to", "max"):
            continue
        # FieldDef を生成して辞書に追加する
        fd = FieldDef(
            name=field_name,
            field_type=field_type,
            field_number=field_num,
            is_repeated=(label == "repeated"),
            is_optional=(label == "optional"),
            default_value=None,
        )
        # フィールド名 -> FieldDef のマッピングに追加する
        fields[field_name] = fd
        # フィールド番号 -> フィールド名のマッピングに追加する
        field_numbers[field_num] = field_name
    # フィールド辞書とフィールド番号マッピングを返す
    return fields, field_numbers


def parse_proto_content(content: str, path: Optional[Path] = None) -> SchemaVersion:
    """
    protobuf (.proto) ファイルの内容を解析して SchemaVersion を返す。

    簡易パーサー（message/enum/field の抽出のみ）で完全な protobuf 文法は非対応。
    生成コードの検証用途に特化した実装。
    """
    # messages/enums の辞書を初期化する
    messages: Dict[str, MessageDef] = {}
    enums: Dict[str, EnumDef] = {}
    # コメントを除去したコンテンツを生成する（// 行コメントを削除する）
    content_no_comments = re.sub(r"//[^\n]*", "", content)
    # ブロックコメント（/* ... */）を削除する
    content_no_comments = re.sub(r"/\*.*?\*/", "", content_no_comments, flags=re.DOTALL)
    # message ブロックを正規表現で抽出する（ネストは考慮しない）
    message_pattern = re.compile(
        r"\bmessage\s+(\w+)\s*\{([^{}]*(?:\{[^{}]*\}[^{}]*)*)\}",
        re.DOTALL,
    )
    # 全 message ブロックを処理する
    for msg_match in message_pattern.finditer(content_no_comments):
        # message 名を取得する
        msg_name = msg_match.group(1)
        # message ボディを取得する
        msg_body = msg_match.group(2)
        # フィールドを解析する
        fields, field_numbers = _parse_proto_fields(msg_body)
        # MessageDef を生成して辞書に追加する
        messages[msg_name] = MessageDef(
            name=msg_name,
            fields=fields,
            field_numbers=field_numbers,
        )
    # enum ブロックを正規表現で抽出する
    enum_pattern = re.compile(
        r"\benum\s+(\w+)\s*\{([^{}]*)\}",
        re.DOTALL,
    )
    # 全 enum ブロックを処理する
    for enum_match in enum_pattern.finditer(content_no_comments):
        # enum 名を取得する
        enum_name = enum_match.group(1)
        # enum ボディを取得する
        enum_body = enum_match.group(2)
        # enum 値を抽出する
        value_nums: Dict[str, int] = {}
        # enum 値のパターン: <VALUE_NAME> = <number>;
        value_pattern = re.compile(r"\b(\w+)\s*=\s*(\d+)\s*;")
        # 全 enum 値を処理する
        for val_match in value_pattern.finditer(enum_body):
            # 値名を取得する
            val_name = val_match.group(1)
            # 値番号を取得する
            try:
                val_num = int(val_match.group(2))
            except ValueError:
                # 数値パースに失敗した場合はスキップする
                continue
            # option キーワードをスキップする
            if val_name == "option":
                continue
            # value_nums に追加する
            value_nums[val_name] = val_num
        # EnumDef を生成して辞書に追加する
        enums[enum_name] = EnumDef(
            name=enum_name,
            values=frozenset(value_nums.keys()),
            value_numbers=value_nums,
        )
    # SchemaVersion を生成して返す
    return SchemaVersion(
        path=path,
        schema_format=SchemaFormat.PROTO,
        messages=messages,
        enums=enums,
        raw_content=content,
    )


def _avro_type_str(avro_type: object) -> str:
    """Avro 型表現を文字列に変換する（null union を含む場合は nullable として返す）。"""
    # 文字列の場合はそのまま返す
    if isinstance(avro_type, str):
        return avro_type
    # リストの場合は union 型として処理する
    if isinstance(avro_type, list):
        # null を除いた型名を結合する
        non_null = [t for t in avro_type if t != "null"]
        # null が含まれていれば nullable とマークする
        if len(non_null) == 1:
            return f"nullable<{non_null[0]}>"
        return f"union<{', '.join(str(t) for t in avro_type)}>"
    # 辞書の場合は type フィールドを確認する
    if isinstance(avro_type, dict):
        t = avro_type.get("type", "unknown")
        # record 型の場合は名前を返す
        if t == "record":
            return f"record:{avro_type.get('name', 'anonymous')}"
        # enum 型の場合は名前を返す
        if t == "enum":
            return f"enum:{avro_type.get('name', 'anonymous')}"
        # array 型の場合は items の型を返す
        if t == "array":
            return f"array<{_avro_type_str(avro_type.get('items', 'unknown'))}>"
        # map 型の場合は values の型を返す
        if t == "map":
            return f"map<{_avro_type_str(avro_type.get('values', 'unknown'))}>"
        return str(t)
    # その他の場合は文字列に変換して返す
    return str(avro_type)


def parse_avro_content(content: str, path: Optional[Path] = None) -> SchemaVersion:
    """
    Avro (.avsc) スキーマファイルの JSON 内容を解析して SchemaVersion を返す。

    record/field/enum の抽出のみ対応（ネストした record は最上位のみ処理する）。
    """
    # messages/enums の辞書を初期化する
    messages: Dict[str, MessageDef] = {}
    enums: Dict[str, EnumDef] = {}
    # JSON をデコードする
    try:
        schema_obj = json.loads(content)
    except json.JSONDecodeError as exc:
        # JSON デコードエラーは空の SchemaVersion を返す
        logger.warning("Avro JSON デコードエラー: %s", exc)
        return SchemaVersion(
            path=path,
            schema_format=SchemaFormat.AVRO,
            messages=messages,
            enums=enums,
            raw_content=content,
        )
    # トップレベルが辞書の場合は単一スキーマとして処理する
    schema_list = schema_obj if isinstance(schema_obj, list) else [schema_obj]
    # 各スキーマを処理する
    for schema in schema_list:
        # 辞書でない場合はスキップする
        if not isinstance(schema, dict):
            continue
        # スキーマの type を取得する
        schema_type = schema.get("type", "")
        # record 型の場合は MessageDef として処理する
        if schema_type == "record":
            # record 名を取得する
            rec_name = schema.get("name", "anonymous")
            # フィールドリストを取得する
            avro_fields = schema.get("fields", [])
            # フィールド定義の辞書を構築する
            fields: Dict[str, FieldDef] = {}
            # フィールド番号は Avro では定義されないため、インデックスを使用する
            field_numbers: Dict[int, str] = {}
            # 各フィールドを処理する
            for idx, f in enumerate(avro_fields):
                # フィールドが辞書でない場合はスキップする
                if not isinstance(f, dict):
                    continue
                # フィールド名を取得する
                f_name = f.get("name", f"field_{idx}")
                # フィールド型を文字列に変換する
                f_type_raw = f.get("type", "null")
                f_type_str = _avro_type_str(f_type_raw)
                # nullable（null union）かどうかを判定する
                is_optional = (
                    isinstance(f_type_raw, list) and "null" in f_type_raw
                ) or f_type_raw == "null"
                # デフォルト値を取得する
                default_val = f.get("default", None)
                # FieldDef を生成して辞書に追加する
                fd = FieldDef(
                    name=f_name,
                    field_type=f_type_str,
                    field_number=idx,
                    is_repeated=False,
                    is_optional=is_optional,
                    default_value=str(default_val) if default_val is not None else None,
                )
                # フィールド名マッピングに追加する
                fields[f_name] = fd
                # インデックスマッピングに追加する
                field_numbers[idx] = f_name
            # MessageDef を生成して辞書に追加する
            messages[rec_name] = MessageDef(
                name=rec_name,
                fields=fields,
                field_numbers=field_numbers,
            )
        # enum 型の場合は EnumDef として処理する
        elif schema_type == "enum":
            # enum 名を取得する
            enum_name = schema.get("name", "anonymous")
            # シンボルリストを取得する
            symbols = schema.get("symbols", [])
            # 値番号マッピングを構築する（インデックスを番号として使用する）
            value_nums = {sym: i for i, sym in enumerate(symbols) if isinstance(sym, str)}
            # EnumDef を生成して辞書に追加する
            enums[enum_name] = EnumDef(
                name=enum_name,
                values=frozenset(value_nums.keys()),
                value_numbers=value_nums,
            )
    # SchemaVersion を生成して返す
    return SchemaVersion(
        path=path,
        schema_format=SchemaFormat.AVRO,
        messages=messages,
        enums=enums,
        raw_content=content,
    )


def load_schema(path: Path) -> SchemaVersion:
    """
    ファイルパスからスキーマを読み込んで SchemaVersion を返す。

    ファイルの拡張子から形式を自動判定する（.proto / .avsc / .json）。
    """
    # ファイルの存在を確認する
    if not path.exists():
        raise FileNotFoundError(f"スキーマファイルが存在しません: {path}")
    # ファイルを UTF-8 で読み込む
    content = path.read_text(encoding="utf-8")
    # 拡張子からスキーマ形式を判定する
    fmt = _detect_schema_format(path)
    # 形式に応じてパーサーを選択する
    if fmt == SchemaFormat.PROTO:
        # protobuf 形式として解析する
        return parse_proto_content(content, path=path)
    elif fmt == SchemaFormat.AVRO:
        # Avro 形式として解析する
        return parse_avro_content(content, path=path)
    else:
        # 不明な形式の場合は空の SchemaVersion を返す
        logger.warning("不明なスキーマ形式: %s", path)
        return SchemaVersion(
            path=path,
            schema_format=SchemaFormat.UNKNOWN,
            messages={},
            enums={},
            raw_content=content,
        )


def _compare_messages(
    old_msg: MessageDef,
    new_msg: MessageDef,
    schema_format: SchemaFormat,
) -> List[DriftEntry]:
    """
    2 つのメッセージ定義を比較してドリフトエントリのリストを返す。

    比較対象: フィールドの追加/削除/型変更/番号変更/ラベル変更
    """
    # ドリフトエントリを格納するリストを初期化する
    entries: List[DriftEntry] = []
    # 旧フィールド名の集合を取得する
    old_fields: Set[str] = set(old_msg.fields.keys())
    # 新フィールド名の集合を取得する
    new_fields: Set[str] = set(new_msg.fields.keys())
    # 削除されたフィールドを検出する（BREAKING）
    for removed in old_fields - new_fields:
        old_fd = old_msg.fields[removed]
        entry = DriftEntry(
            severity=DriftSeverity.BREAKING,
            location=f"{old_msg.name}.{removed}",
            change_type="field_removed",
            old_value=f"type={old_fd.field_type}, number={old_fd.field_number}",
            new_value="(deleted)",
            description=(
                f"フィールド '{removed}' が削除されました。"
                f"旧クライアントはデータを読めなくなります。"
            ),
        )
        entries.append(entry)
    # 追加されたフィールドを検出する（ADDITIVE: オプションなら安全）
    for added in new_fields - old_fields:
        new_fd = new_msg.fields[added]
        # required フィールドの追加は BREAKING、optional は ADDITIVE
        severity = DriftSeverity.ADDITIVE if new_fd.is_optional else DriftSeverity.BREAKING
        entry = DriftEntry(
            severity=severity,
            location=f"{new_msg.name}.{added}",
            change_type="field_added",
            old_value="(none)",
            new_value=f"type={new_fd.field_type}, number={new_fd.field_number}",
            description=(
                f"フィールド '{added}' が追加されました。"
                + (
                    "optional のため旧クライアントは無視できます。"
                    if new_fd.is_optional
                    else "required のため旧クライアントとの互換性が失われます。"
                )
            ),
        )
        entries.append(entry)
    # 共通フィールドの変化を検出する
    for field_name in old_fields & new_fields:
        old_fd = old_msg.fields[field_name]
        new_fd = new_msg.fields[field_name]
        # 型の変化を検出する
        if old_fd.field_type != new_fd.field_type:
            # protobuf の場合はワイヤ型互換性を確認する
            compatible = False
            if schema_format == SchemaFormat.PROTO:
                compatible_types = _PROTO_COMPATIBLE_TYPES.get(old_fd.field_type, frozenset())
                compatible = new_fd.field_type in compatible_types
            elif schema_format == SchemaFormat.AVRO:
                compatible_types = _AVRO_PROMOTABLE_TYPES.get(old_fd.field_type, frozenset())
                compatible = new_fd.field_type in compatible_types
            # 互換性のない型変更は BREAKING
            severity = DriftSeverity.SAFE if compatible else DriftSeverity.BREAKING
            entry = DriftEntry(
                severity=severity,
                location=f"{old_msg.name}.{field_name}",
                change_type="type_changed",
                old_value=old_fd.field_type,
                new_value=new_fd.field_type,
                description=(
                    f"フィールド '{field_name}' の型が変更されました。"
                    + (
                        "ワイヤ型互換性があります。"
                        if compatible
                        else "ワイヤ型互換性がなく、デシリアライズが失敗します。"
                    )
                ),
            )
            entries.append(entry)
        # protobuf のフィールド番号変更を検出する（常に BREAKING）
        if (
            schema_format == SchemaFormat.PROTO
            and old_fd.field_number != new_fd.field_number
        ):
            entry = DriftEntry(
                severity=DriftSeverity.BREAKING,
                location=f"{old_msg.name}.{field_name}",
                change_type="field_number_changed",
                old_value=str(old_fd.field_number),
                new_value=str(new_fd.field_number),
                description=(
                    f"フィールド '{field_name}' のフィールド番号が変更されました。"
                    f"protobuf のバイナリ互換性が失われます。"
                ),
            )
            entries.append(entry)
        # repeated -> optional または optional -> repeated の変更を検出する
        if old_fd.is_repeated != new_fd.is_repeated:
            entry = DriftEntry(
                severity=DriftSeverity.BREAKING,
                location=f"{old_msg.name}.{field_name}",
                change_type="repeated_changed",
                old_value=f"repeated={old_fd.is_repeated}",
                new_value=f"repeated={new_fd.is_repeated}",
                description=(
                    f"フィールド '{field_name}' の repeated/optional が変更されました。"
                    f"シリアライズ形式が変わりバイナリ互換性が失われます。"
                ),
            )
            entries.append(entry)
    # ドリフトエントリのリストを返す
    return entries


def _compare_enums(old_enum: EnumDef, new_enum: EnumDef) -> List[DriftEntry]:
    """
    2 つの enum 定義を比較してドリフトエントリのリストを返す。

    比較対象: 値の追加/削除/番号変更
    """
    # ドリフトエントリを格納するリストを初期化する
    entries: List[DriftEntry] = []
    # 削除された値を検出する（BREAKING: 旧クライアントがこの値を受信する可能性がある）
    for removed in old_enum.values - new_enum.values:
        entry = DriftEntry(
            severity=DriftSeverity.BREAKING,
            location=f"{old_enum.name}.{removed}",
            change_type="enum_value_removed",
            old_value=f"{removed}={old_enum.value_numbers.get(removed, '?')}",
            new_value="(deleted)",
            description=(
                f"enum 値 '{removed}' が削除されました。"
                f"この値を処理している旧クライアントでエラーが発生します。"
            ),
        )
        entries.append(entry)
    # 追加された値を検出する（ADDITIVE: 旧クライアントは unknown として処理できる）
    for added in new_enum.values - old_enum.values:
        entry = DriftEntry(
            severity=DriftSeverity.ADDITIVE,
            location=f"{new_enum.name}.{added}",
            change_type="enum_value_added",
            old_value="(none)",
            new_value=f"{added}={new_enum.value_numbers.get(added, '?')}",
            description=(
                f"enum 値 '{added}' が追加されました。"
                f"旧クライアントはこの値を unknown として処理します。"
            ),
        )
        entries.append(entry)
    # 共通値のフィールド番号変更を検出する（BREAKING）
    for val_name in old_enum.values & new_enum.values:
        old_num = old_enum.value_numbers.get(val_name)
        new_num = new_enum.value_numbers.get(val_name)
        # 番号が変更された場合は BREAKING として記録する
        if old_num is not None and new_num is not None and old_num != new_num:
            entry = DriftEntry(
                severity=DriftSeverity.BREAKING,
                location=f"{old_enum.name}.{val_name}",
                change_type="enum_value_number_changed",
                old_value=str(old_num),
                new_value=str(new_num),
                description=(
                    f"enum 値 '{val_name}' の番号が {old_num} から {new_num} に変更されました。"
                    f"バイナリ互換性が失われます。"
                ),
            )
            entries.append(entry)
    # ドリフトエントリのリストを返す
    return entries


def drift_between(old: SchemaVersion, new: SchemaVersion) -> DriftResult:
    """
    旧スキーマと新スキーマを比較してドリフト結果を返す。

    引数:
      old: 旧バージョンのスキーマ（load_schema() で生成）
      new: 新バージョンのスキーマ（load_schema() で生成）

    戻り値:
      DriftResult: 検出された全差分エントリを含む結果
    """
    # ドリフトエントリを格納するリストを初期化する
    all_entries: List[DriftEntry] = []
    # スキーマ形式を決定する（new の形式を優先する）
    schema_format = new.schema_format if new.schema_format != SchemaFormat.UNKNOWN else old.schema_format
    # 旧 message 名の集合を取得する
    old_messages: Set[str] = set(old.messages.keys())
    # 新 message 名の集合を取得する
    new_messages: Set[str] = set(new.messages.keys())
    # 削除された message を BREAKING として記録する
    for removed in old_messages - new_messages:
        entry = DriftEntry(
            severity=DriftSeverity.BREAKING,
            location=removed,
            change_type="message_removed",
            old_value=f"fields={list(old.messages[removed].fields.keys())}",
            new_value="(deleted)",
            description=(
                f"message '{removed}' がスキーマから削除されました。"
                f"このメッセージを使用している全クライアントとの互換性が失われます。"
            ),
        )
        all_entries.append(entry)
    # 追加された message を ADDITIVE として記録する
    for added in new_messages - old_messages:
        entry = DriftEntry(
            severity=DriftSeverity.ADDITIVE,
            location=added,
            change_type="message_added",
            old_value="(none)",
            new_value=f"fields={list(new.messages[added].fields.keys())}",
            description=f"message '{added}' が追加されました。旧クライアントはこの型を無視します。",
        )
        all_entries.append(entry)
    # 共通 message のフィールド変化を比較する
    for msg_name in old_messages & new_messages:
        # フィールドの変化を検出して entries に追加する
        msg_entries = _compare_messages(
            old.messages[msg_name],
            new.messages[msg_name],
            schema_format,
        )
        all_entries.extend(msg_entries)
    # 旧 enum 名の集合を取得する
    old_enums: Set[str] = set(old.enums.keys())
    # 新 enum 名の集合を取得する
    new_enums: Set[str] = set(new.enums.keys())
    # 削除された enum を BREAKING として記録する
    for removed in old_enums - new_enums:
        entry = DriftEntry(
            severity=DriftSeverity.BREAKING,
            location=removed,
            change_type="enum_removed",
            old_value=f"values={list(old.enums[removed].values)}",
            new_value="(deleted)",
            description=f"enum '{removed}' が削除されました。",
        )
        all_entries.append(entry)
    # 追加された enum を ADDITIVE として記録する
    for added in new_enums - old_enums:
        entry = DriftEntry(
            severity=DriftSeverity.ADDITIVE,
            location=added,
            change_type="enum_added",
            old_value="(none)",
            new_value=f"values={list(new.enums[added].values)}",
            description=f"enum '{added}' が追加されました。",
        )
        all_entries.append(entry)
    # 共通 enum の値変化を比較する
    for enum_name in old_enums & new_enums:
        # 値の変化を検出して entries に追加する
        enum_entries = _compare_enums(old.enums[enum_name], new.enums[enum_name])
        all_entries.extend(enum_entries)
    # DriftResult を生成して返す
    return DriftResult(
        entries=all_entries,
        old_path=old.path,
        new_path=new.path,
    )


def compare_schema_files(old_path: Path, new_path: Path) -> DriftResult:
    """
    2 つのスキーマファイルを読み込んで比較する便利関数。

    引数:
      old_path: 旧スキーマファイルのパス
      new_path: 新スキーマファイルのパス

    戻り値:
      DriftResult: 検出された全差分エントリを含む結果
    """
    # 旧スキーマを読み込む
    old_schema = load_schema(old_path)
    # 新スキーマを読み込む
    new_schema = load_schema(new_path)
    # 比較を実行して結果を返す
    return drift_between(old_schema, new_schema)
