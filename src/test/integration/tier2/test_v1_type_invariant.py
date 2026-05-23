"""src/test/integration/tier2/test_v1_type_invariant.py

tier2 軸の v1_type_invariant 検証クラスに対するインテグレーションテスト。
テナント ID の UUID v4 フォーマット・クォータ制限の正整数型・
RLS ポリシー式の app.current_tenant 参照を検証する。

仕様: docs/04_詳細設計/01_適合仕様/19_検証規律適合仕様.md

ai_generator_id: claude-sonnet-4-6
model_version: claude-sonnet-4-6
prompt_hash: tier2_type_invariant_v1
"""

from __future__ import annotations

# 標準ライブラリをインポートする
import re
import uuid
# pytest フレームワークをインポートする
import pytest
# 型ヒントをインポートする
from typing import Generator, List, Optional
# dataclass デコレータをインポートする
from dataclasses import dataclass, field


# UUID v4 バリデーション用の正規表現パターンを定義する
UUID_V4_PATTERN = re.compile(
    r"^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$",
    re.IGNORECASE,
)

# RLS ポリシー式に必要なテナント参照パターンを定義する
RLS_TENANT_REF_PATTERN = re.compile(r"app\.current_tenant")


# テナント ID を表すバリュークラスを定義する
@dataclass(frozen=True)
class TenantId:
    """テナント ID バリュークラス。UUID v4 形式でなければならない。"""

    # UUID 文字列を保持するフィールドを定義する
    value: str

    # 初期化後にバリデーションを実行するポストイニットメソッドを定義する
    def __post_init__(self) -> None:
        """UUID v4 フォーマットを検証する。"""
        # UUID v4 パターンに一致しない場合はエラーを発生させる
        if not UUID_V4_PATTERN.match(self.value):
            # バリデーションエラーを発生させる
            raise ValueError(
                f"TenantId must be UUID v4 format, got: {self.value!r}"
            )

    # ファクトリメソッドで新しい TenantId を生成するクラスメソッドを定義する
    @classmethod
    def generate(cls) -> "TenantId":
        """新しい UUID v4 を生成して TenantId を作成する。"""
        # uuid4 を生成して文字列に変換する
        return cls(value=str(uuid.uuid4()))


# クォータ制限を表すバリュークラスを定義する
@dataclass(frozen=True)
class QuotaLimit:
    """クォータ制限バリュークラス。正の整数でなければならない。"""

    # クォータ値を保持するフィールドを定義する
    value: int

    # 初期化後にバリデーションを実行するポストイニットメソッドを定義する
    def __post_init__(self) -> None:
        """正の整数であることを検証する。"""
        # 整数型でない場合はエラーを発生させる
        if not isinstance(self.value, int):
            # 型エラーを発生させる
            raise TypeError(
                f"QuotaLimit must be int, got {type(self.value).__name__}"
            )
        # 正の整数でない場合はエラーを発生させる
        if self.value <= 0:
            # バリデーションエラーを発生させる
            raise ValueError(
                f"QuotaLimit must be positive, got {self.value}"
            )


# RLS ポリシーを表すデータクラスを定義する
@dataclass
class RLSPolicy:
    """Row Level Security ポリシーのデータ型。"""

    # ポリシー名を保持するフィールドを定義する
    policy_name: str
    # ポリシーが適用されるテーブル名を保持するフィールドを定義する
    table_name: str
    # ポリシーの USING 式を保持するフィールドを定義する
    using_expression: str

    # ポリシーの不変条件を検証するメソッドを定義する
    def validate(self) -> None:
        """RLS ポリシーの不変条件を検証する。"""
        # USING 式が空でないことを確認する
        if not self.using_expression.strip():
            # 空の式の場合はエラーを発生させる
            raise ValueError("using_expression must not be empty")
        # USING 式が app.current_tenant を参照していることを確認する
        if not RLS_TENANT_REF_PATTERN.search(self.using_expression):
            # テナント参照が欠落している場合はエラーを発生させる
            raise ValueError(
                f"RLS policy must reference app.current_tenant, "
                f"but expression is: {self.using_expression!r}"
            )


# テナント設定を表すデータクラスを定義する
@dataclass
class TenantConfiguration:
    """テナントの設定情報を保持するデータクラス。"""

    # テナント ID を保持するフィールドを定義する
    tenant_id: TenantId
    # API レート制限クォータを保持するフィールドを定義する
    api_rate_limit: QuotaLimit
    # ストレージクォータを保持するフィールドを定義する
    storage_quota_bytes: QuotaLimit
    # 最大接続数クォータを保持するフィールドを定義する
    max_connections: QuotaLimit
    # RLS ポリシーのリストを保持するフィールドを定義する
    rls_policies: list[RLSPolicy] = field(default_factory=list)

    # テナント設定の不変条件を検証するメソッドを定義する
    def validate(self) -> None:
        """テナント設定の不変条件を検証する。"""
        # 全 RLS ポリシーを検証する
        for policy in self.rls_policies:
            # 各ポリシーを検証する
            policy.validate()


# tier2 型不変条件テストスイートクラスを定義する
class TestTier2TypeInvariant:
    """tier2 軸 v1_type_invariant の検証テストスイート。"""

    # 有効な RLS ポリシーのフィクスチャを定義する
    @pytest.fixture
    def valid_rls_policy(self) -> Generator[RLSPolicy, None, None]:
        """有効な RLS ポリシーフィクスチャを生成する。"""
        # 有効な RLS ポリシーを生成する
        policy = RLSPolicy(
            policy_name="tenant_isolation",
            table_name="orders",
            using_expression="tenant_id = app.current_tenant",
        )
        # フィクスチャを返す
        yield policy

    # テナント ID が UUID v4 形式であることを検証するテストを定義する
    def test_tenant_id_must_be_uuid_v4(self) -> None:
        """テナント ID が UUID v4 形式でなければならないことを検証する。"""
        # 有効な UUID v4 でテナント ID を生成する
        valid_uuid = str(uuid.uuid4())
        # TenantId を生成する
        tenant_id = TenantId(value=valid_uuid)
        # 生成された TenantId の値が元の UUID と一致することを確認する
        assert tenant_id.value == valid_uuid
        # UUID v4 ファクトリメソッドで生成したテナント ID を確認する
        generated = TenantId.generate()
        # 生成された UUID が v4 パターンに一致することを確認する
        assert UUID_V4_PATTERN.match(generated.value), (
            f"Generated TenantId does not match UUID v4 pattern: {generated.value}"
        )

    # UUID v4 以外の文字列がテナント ID として拒否されることを検証するテストを定義する
    def test_invalid_tenant_id_formats_rejected(self) -> None:
        """UUID v4 以外の形式がテナント ID として拒否されることを検証する。"""
        # 無効なフォーマットのリストを定義する
        invalid_formats = [
            # 空文字列は拒否する
            "",
            # 単純な文字列は拒否する
            "not-a-uuid",
            # UUID v1 フォーマットは拒否する
            "550e8400-e29b-11d4-a716-446655440000",
            # バージョン情報が欠落している場合は拒否する
            "00000000-0000-0000-0000-000000000000",
            # 数字のみの文字列は拒否する
            "12345678",
        ]
        # 各無効フォーマットでエラーが発生することを確認する
        for invalid in invalid_formats:
            # 無効なフォーマットで TenantId を生成しエラーを確認する
            with pytest.raises(ValueError, match="UUID v4"):
                # 無効な値で TenantId を生成する
                TenantId(value=invalid)

    # クォータ制限が正の整数でなければならないことを検証するテストを定義する
    def test_quota_limit_must_be_positive_int(self) -> None:
        """クォータ制限が正の整数でなければならないことを検証する。"""
        # 有効な正の整数でクォータ制限を生成する
        quota = QuotaLimit(value=1000)
        # 値が正しく設定されていることを確認する
        assert quota.value == 1000
        # 1 は最小の正の整数として有効であることを確認する
        min_quota = QuotaLimit(value=1)
        # 最小クォータの値が 1 であることを確認する
        assert min_quota.value == 1
        # 大きな値も有効であることを確認する
        large_quota = QuotaLimit(value=10_000_000)
        # 大きなクォータの値が正しく設定されていることを確認する
        assert large_quota.value == 10_000_000

    # 0 以下のクォータ制限が拒否されることを検証するテストを定義する
    def test_non_positive_quota_limit_rejected(self) -> None:
        """0 以下の値がクォータ制限として拒否されることを検証する。"""
        # 0 は拒否されることを確認する
        with pytest.raises(ValueError, match="positive"):
            # 0 でクォータ制限を生成する
            QuotaLimit(value=0)
        # 負の値は拒否されることを確認する
        with pytest.raises(ValueError, match="positive"):
            # 負の値でクォータ制限を生成する
            QuotaLimit(value=-1)
        # 大きな負の値も拒否されることを確認する
        with pytest.raises(ValueError, match="positive"):
            # 大きな負の値でクォータ制限を生成する
            QuotaLimit(value=-9999)

    # 浮動小数点数がクォータ制限として拒否されることを検証するテストを定義する
    def test_float_quota_limit_rejected(self) -> None:
        """浮動小数点数がクォータ制限として拒否されることを検証する。"""
        # 浮動小数点数は型エラーが発生することを確認する
        with pytest.raises(TypeError, match="int"):
            # 浮動小数点数でクォータ制限を生成する
            QuotaLimit(value=100.5)  # type: ignore[arg-type]

    # RLS ポリシーが app.current_tenant を参照していることを検証するテストを定義する
    def test_rls_policy_must_reference_current_tenant(self) -> None:
        """RLS ポリシーの USING 式が app.current_tenant を参照していることを検証する。"""
        # 有効な RLS ポリシーを生成する
        valid_policy = RLSPolicy(
            policy_name="valid_policy",
            table_name="products",
            using_expression="tenant_id = app.current_tenant AND status = 'active'",
        )
        # 有効なポリシーの検証がパスすることを確認する
        valid_policy.validate()

    # テナント参照が欠落した RLS ポリシーが拒否されることを検証するテストを定義する
    def test_rls_policy_without_tenant_ref_rejected(self) -> None:
        """app.current_tenant 参照のない RLS ポリシーが拒否されることを検証する。"""
        # テナント参照が欠落したポリシーを生成する
        invalid_policy = RLSPolicy(
            policy_name="bad_policy",
            table_name="orders",
            using_expression="status = 'active'",
        )
        # validate() が ValueError を発生させることを確認する
        with pytest.raises(ValueError, match="app.current_tenant"):
            # バリデーションを実行する
            invalid_policy.validate()

    # 空の USING 式を持つ RLS ポリシーが拒否されることを検証するテストを定義する
    def test_rls_policy_empty_expression_rejected(self) -> None:
        """空の USING 式を持つ RLS ポリシーが拒否されることを検証する。"""
        # 空の USING 式を持つポリシーを生成する
        empty_policy = RLSPolicy(
            policy_name="empty_policy",
            table_name="records",
            using_expression="",
        )
        # validate() が ValueError を発生させることを確認する
        with pytest.raises(ValueError, match="empty"):
            # バリデーションを実行する
            empty_policy.validate()

    # テナント設定が複数の RLS ポリシーを保持できることを検証するテストを定義する
    def test_tenant_configuration_holds_multiple_rls_policies(self) -> None:
        """テナント設定が複数の有効な RLS ポリシーを保持できることを検証する。"""
        # テナント ID を生成する
        tenant_id = TenantId.generate()
        # 複数の RLS ポリシーを生成する
        policies = [
            RLSPolicy(
                policy_name=f"policy_{i}",
                table_name=f"table_{i}",
                using_expression=f"tenant_id = app.current_tenant AND col_{i} = {i}",
            )
            for i in range(5)
        ]
        # テナント設定を生成する
        config = TenantConfiguration(
            tenant_id=tenant_id,
            api_rate_limit=QuotaLimit(value=1000),
            storage_quota_bytes=QuotaLimit(value=10_737_418_240),
            max_connections=QuotaLimit(value=100),
            rls_policies=policies,
        )
        # テナント設定のバリデーションがパスすることを確認する
        config.validate()
        # RLS ポリシー数が 5 であることを確認する
        assert len(config.rls_policies) == 5

    # 無効な RLS ポリシーを含むテナント設定が拒否されることを検証するテストを定義する
    def test_tenant_configuration_rejects_invalid_rls_policy(self) -> None:
        """無効な RLS ポリシーを含むテナント設定のバリデーションが失敗することを検証する。"""
        # テナント ID を生成する
        tenant_id = TenantId.generate()
        # 無効な RLS ポリシーを生成する
        invalid_policy = RLSPolicy(
            policy_name="bad_policy",
            table_name="secret_table",
            using_expression="1=1",
        )
        # テナント設定を生成する
        config = TenantConfiguration(
            tenant_id=tenant_id,
            api_rate_limit=QuotaLimit(value=500),
            storage_quota_bytes=QuotaLimit(value=1_073_741_824),
            max_connections=QuotaLimit(value=50),
            rls_policies=[invalid_policy],
        )
        # バリデーションが失敗することを確認する
        with pytest.raises(ValueError, match="app.current_tenant"):
            # バリデーションを実行する
            config.validate()

    # TenantId の equality が正しく機能することを検証するテストを定義する
    def test_tenant_id_equality(self) -> None:
        """同じ UUID 文字列を持つ TenantId が等しいことを検証する。"""
        # UUID 文字列を生成する
        uuid_str = str(uuid.uuid4())
        # 同じ UUID 文字列で 2 つの TenantId を生成する
        id1 = TenantId(value=uuid_str)
        # 2 つ目の TenantId を生成する
        id2 = TenantId(value=uuid_str)
        # 2 つの TenantId が等しいことを確認する
        assert id1 == id2
        # 異なる UUID 文字列で生成した TenantId が等しくないことを確認する
        id3 = TenantId.generate()
        # id1 と id3 が異なることを確認する
        assert id1 != id3

    # QuotaLimit の equality が正しく機能することを検証するテストを定義する
    def test_quota_limit_equality(self) -> None:
        """同じ値を持つ QuotaLimit が等しいことを検証する。"""
        # 同じ値で 2 つの QuotaLimit を生成する
        q1 = QuotaLimit(value=500)
        # 2 つ目の QuotaLimit を生成する
        q2 = QuotaLimit(value=500)
        # 2 つの QuotaLimit が等しいことを確認する
        assert q1 == q2
        # 異なる値を持つ QuotaLimit が等しくないことを確認する
        q3 = QuotaLimit(value=501)
        # q1 と q3 が異なることを確認する
        assert q1 != q3


# tier2 型不変条件追加テストスイートクラスを定義する
class TestTier2TypeInvariantExtended:
    """tier2 軸 v1_type_invariant の追加検証テストスイート。

    テナント設定の大規模バリデーション・RLS 式の複合条件を検証する。
    """

    # 大量のテナント ID が全て UUID v4 形式を満たすことを検証するテストを定義する
    def test_bulk_tenant_id_generation_all_uuid_v4(self) -> None:
        """100 件のテナント ID 生成が全て UUID v4 形式を満たすことを検証する。"""
        # 100 件のテナント ID を生成する
        tenant_ids = [TenantId.generate() for _ in range(100)]
        # 全テナント ID が UUID v4 形式を満たすことを確認する
        for tenant_id in tenant_ids:
            # UUID v4 パターンに一致することを確認する
            assert UUID_V4_PATTERN.match(tenant_id.value), (
                f"Generated TenantId does not match UUID v4 pattern: {tenant_id.value}"
            )
        # 全テナント ID が一意であることを確認する
        unique_values = {t.value for t in tenant_ids}
        # 重複がないことを確認する
        assert len(unique_values) == 100, "Generated TenantIds must be unique"

    # RLS ポリシーが複合条件で app.current_tenant を参照できることを検証するテストを定義する
    def test_rls_policy_complex_expression_valid(self) -> None:
        """RLS ポリシーが複雑な条件式でも app.current_tenant 参照があれば有効であることを検証する。"""
        # 複合 RLS 式を持つポリシーを生成する
        complex_policy = RLSPolicy(
            policy_name="complex_rls",
            table_name="manufacturing_orders",
            using_expression=(
                "(tenant_id = app.current_tenant) AND "
                "(status IN ('active', 'pending')) AND "
                "(created_at > NOW() - INTERVAL '30 days')"
            ),
        )
        # バリデーションがパスすることを確認する
        complex_policy.validate()

    # 各クォータ値のデータ型が正しく設定されることを検証するテストを定義する
    @pytest.mark.parametrize(
        "value,expected",
        [
            # 1 の場合の期待値を定義する
            (1, 1),
            # 最大値の場合の期待値を定義する
            (2**31 - 1, 2**31 - 1),
            # 一般的な値の場合の期待値を定義する
            (1000, 1000),
            # 大きな値の場合の期待値を定義する
            (10_000_000, 10_000_000),
        ],
    )
    def test_quota_limit_parametrized_values(self, value: int, expected: int) -> None:
        """様々な正の整数でクォータ制限が正しく生成されることを検証する。"""
        # クォータを生成する
        quota = QuotaLimit(value=value)
        # 値が正しく設定されていることを確認する
        assert quota.value == expected

    # TenantId の hashability を検証するテストを定義する
    def test_tenant_id_is_hashable(self) -> None:
        """TenantId が辞書キーやセットの要素として使用できることを検証する。"""
        # テナント ID を生成する
        t1 = TenantId.generate()
        # 2 つ目のテナント ID を生成する
        t2 = TenantId.generate()
        # テナント ID をセットに追加する
        tenant_set = {t1, t2}
        # セットに 2 件格納されていることを確認する
        assert len(tenant_set) == 2
        # テナント ID を辞書キーとして使用する
        tenant_dict = {t1: "quota_a", t2: "quota_b"}
        # 辞書から値を取得できることを確認する
        assert tenant_dict[t1] == "quota_a"

    # QuotaLimit の hashability を検証するテストを定義する
    def test_quota_limit_is_hashable(self) -> None:
        """QuotaLimit が辞書キーやセットの要素として使用できることを検証する。"""
        # クォータを生成する
        q1 = QuotaLimit(value=100)
        # 2 つ目のクォータを生成する
        q2 = QuotaLimit(value=200)
        # クォータをセットに追加する
        quota_set = {q1, q2}
        # セットに 2 件格納されていることを確認する
        assert len(quota_set) == 2

    # TenantConfiguration が RLS ポリシーなしで生成できることを検証するテストを定義する
    def test_tenant_configuration_no_rls_policies_valid(self) -> None:
        """RLS ポリシーなしのテナント設定がバリデーションを通過することを検証する。"""
        # テナント ID を生成する
        tenant_id = TenantId.generate()
        # RLS ポリシーなしのテナント設定を生成する
        config = TenantConfiguration(
            tenant_id=tenant_id,
            api_rate_limit=QuotaLimit(value=500),
            storage_quota_bytes=QuotaLimit(value=1_073_741_824),
            max_connections=QuotaLimit(value=50),
            rls_policies=[],
        )
        # バリデーションがパスすることを確認する
        config.validate()
        # RLS ポリシー数が 0 であることを確認する
        assert len(config.rls_policies) == 0

    # 空白のみの USING 式を持つ RLS ポリシーが拒否されることを検証するテストを定義する
    def test_rls_policy_whitespace_only_expression_rejected(self) -> None:
        """空白のみの USING 式を持つ RLS ポリシーが拒否されることを検証する。"""
        # 空白のみの式を持つポリシーを生成する
        whitespace_policy = RLSPolicy(
            policy_name="whitespace_policy",
            table_name="orders",
            using_expression="   \t  \n  ",
        )
        # バリデーションが失敗することを確認する
        with pytest.raises(ValueError, match="empty"):
            # バリデーションを実行する
            whitespace_policy.validate()

    # UUID v4 の大文字版が TenantId として有効であることを検証するテストを定義する
    def test_tenant_id_accepts_uppercase_uuid(self) -> None:
        """UUID v4 の大文字版が TenantId として受け入れられることを検証する。"""
        # 大文字の UUID v4 を生成する
        lower_uuid = str(uuid.uuid4())
        # 大文字に変換する
        upper_uuid = lower_uuid.upper()
        # 大文字の UUID v4 で TenantId を生成する
        tenant_id = TenantId(value=upper_uuid)
        # 値が正しく設定されていることを確認する
        assert tenant_id.value == upper_uuid


# tier2 型不変条件拡張テストスイートクラスを定義する
class TestTier2TypeInvariantExtended2:
    """tier2 軸 v1_type_invariant の第 2 拡張テストスイート。さらに多くの型不変条件を検証する。"""

    # TenantId が UUID v4 バリデーションを厳密に行うことをテストする
    @pytest.mark.parametrize(
        "invalid_uuid",
        [
            # UUID v3（MD5 ベース）を無効として定義する
            "550e8400-e29b-31d4-a716-446655440000",
            # UUID v5（SHA-1 ベース）を無効として定義する
            "550e8400-e29b-51d4-a716-446655440000",
            # 無効な形式（短すぎる）を無効として定義する
            "550e8400-e29b-4",
            # 空文字列を無効として定義する
            "",
            # 数字のみを無効として定義する
            "12345678",
            # コロン区切りを無効として定義する
            "550e8400:e29b:41d4:a716:446655440000",
        ],
    )
    def test_tenant_id_rejects_non_v4_uuid(self, invalid_uuid: str) -> None:
        """UUID v4 でない文字列が TenantId として拒否されることを検証する。"""
        # 無効な UUID v4 で TenantId を生成しようとする
        with pytest.raises(ValueError):
            # バリデーションが失敗することを確認する
            TenantId(value=invalid_uuid)

    # QuotaLimit の最小境界値をテストする
    def test_quota_limit_minimum_value_one(self) -> None:
        """QuotaLimit の最小値が 1 であることを検証する。"""
        # 最小値 1 でクォータを生成する
        quota = QuotaLimit(value=1)
        # 値が 1 であることを確認する
        assert quota.value == 1

    # QuotaLimit が 0 を拒否することをテストする
    def test_quota_limit_rejects_zero(self) -> None:
        """QuotaLimit が 0 を拒否することを検証する。"""
        # 0 でクォータを生成しようとする
        with pytest.raises(ValueError):
            # バリデーションが失敗することを確認する
            QuotaLimit(value=0)

    # QuotaLimit が負の値を拒否することをテストする
    @pytest.mark.parametrize("negative_value", [-1, -100, -1000, -2**31])
    def test_quota_limit_rejects_negative_values(self, negative_value: int) -> None:
        """QuotaLimit が負の値を拒否することを検証する。"""
        # 負の値でクォータを生成しようとする
        with pytest.raises(ValueError):
            # バリデーションが失敗することを確認する
            QuotaLimit(value=negative_value)

    # RLS ポリシーが app.current_tenant を欠いた式を拒否することをテストする
    @pytest.mark.parametrize(
        "bad_expression",
        [
            # テナント参照なしの式を無効として定義する
            "tenant_id = 'hardcoded-uuid'",
            # 空文字列を無効として定義する
            "",
            # 異なる参照パターンを無効として定義する
            "current_tenant = tenant_id",
            # 部分一致（current_tenant だけでは不十分）を無効として定義する
            "current_tenant IS NOT NULL",
        ],
    )
    def test_rls_policy_rejects_missing_app_current_tenant(
        self, bad_expression: str
    ) -> None:
        """RLS ポリシーが app.current_tenant を含まない式を拒否することを検証する。"""
        # 無効な式を持つ RLS ポリシーを生成する
        policy = RLSPolicy(
            policy_name="invalid_rls",
            table_name="orders",
            using_expression=bad_expression,
        )
        # バリデーションが失敗することを確認する
        with pytest.raises(ValueError):
            # バリデーションを実行する
            policy.validate()

    # TenantConfiguration が RLS ポリシーを複数持てることをテストする
    def test_tenant_configuration_multiple_rls_policies(self) -> None:
        """TenantConfiguration が複数の RLS ポリシーを持てることを検証する。"""
        # テナント ID を生成する
        tenant_id = TenantId.generate()
        # 1 つ目の RLS ポリシーを生成する
        policy1 = RLSPolicy(
            policy_name="orders_policy",
            table_name="orders",
            using_expression="tenant_id = app.current_tenant",
        )
        # 2 つ目の RLS ポリシーを生成する
        policy2 = RLSPolicy(
            policy_name="inventory_policy",
            table_name="inventory",
            using_expression="owner_tenant = app.current_tenant AND active = TRUE",
        )
        # 3 つ目の RLS ポリシーを生成する
        policy3 = RLSPolicy(
            policy_name="audit_policy",
            table_name="audit_log",
            using_expression="app.current_tenant = audit_tenant_id",
        )
        # テナント設定を生成する
        config = TenantConfiguration(
            tenant_id=tenant_id,
            api_rate_limit=QuotaLimit(value=1000),
            storage_quota_bytes=QuotaLimit(value=10_737_418_240),
            max_connections=QuotaLimit(value=100),
            rls_policies=[policy1, policy2, policy3],
        )
        # バリデーションがパスすることを確認する
        config.validate()
        # RLS ポリシー数が 3 であることを確認する
        assert len(config.rls_policies) == 3

    # TenantId が等値比較を正しく行うことをテストする
    def test_tenant_id_equality(self) -> None:
        """同じ UUID 文字列から生成した TenantId が等値であることを検証する。"""
        # UUID v4 を生成する
        uuid_str = str(uuid.uuid4())
        # 同じ UUID で 2 つの TenantId を生成する
        t1 = TenantId(value=uuid_str)
        # 同じ UUID で 2 つ目の TenantId を生成する
        t2 = TenantId(value=uuid_str)
        # 等値であることを確認する
        assert t1 == t2

    # TenantId が異なる UUID で不等値であることをテストする
    def test_tenant_id_inequality(self) -> None:
        """異なる UUID から生成した TenantId が不等値であることを検証する。"""
        # 1 つ目の TenantId を生成する
        t1 = TenantId.generate()
        # 2 つ目の TenantId を生成する
        t2 = TenantId.generate()
        # 不等値であることを確認する
        assert t1 != t2

    # QuotaLimit が大きな値を受け入れることをテストする
    def test_quota_limit_accepts_large_values(self) -> None:
        """QuotaLimit が大きな正の整数を受け入れることを検証する。"""
        # 大きな値でクォータを生成する（1 TiB = 2^40 バイト）
        large_quota = QuotaLimit(value=2**40)
        # 値が正しく設定されていることを確認する
        assert large_quota.value == 2**40
        # 別の大きな値でクォータを生成する
        another = QuotaLimit(value=10**12)
        # 値が正しく設定されていることを確認する
        assert another.value == 10**12

    # TenantConfiguration の全フィールドが正しく設定されることをテストする
    def test_tenant_configuration_all_fields_set(self) -> None:
        """TenantConfiguration の全フィールドが正しく設定されることを検証する。"""
        # テナント ID を生成する
        tenant_id = TenantId.generate()
        # API レート制限を設定する
        api_rate = QuotaLimit(value=5000)
        # ストレージクォータを設定する
        storage = QuotaLimit(value=1_073_741_824)
        # 最大接続数を設定する
        max_conn = QuotaLimit(value=200)
        # RLS ポリシーを生成する
        policy = RLSPolicy(
            policy_name="all_fields_policy",
            table_name="products",
            using_expression="product_tenant = app.current_tenant",
        )
        # テナント設定を生成する
        config = TenantConfiguration(
            tenant_id=tenant_id,
            api_rate_limit=api_rate,
            storage_quota_bytes=storage,
            max_connections=max_conn,
            rls_policies=[policy],
        )
        # 各フィールドが正しく設定されていることを確認する
        assert config.tenant_id == tenant_id
        # API レート制限が正しいことを確認する
        assert config.api_rate_limit == api_rate
        # ストレージクォータが正しいことを確認する
        assert config.storage_quota_bytes == storage
        # 最大接続数が正しいことを確認する
        assert config.max_connections == max_conn
        # RLS ポリシーが 1 件であることを確認する
        assert len(config.rls_policies) == 1


# tier2 型不変条件の第四拡張テストクラスを定義する
class TestTier2TypeInvariantExtended4:
    """テナント ID・クォータ・RLS ポリシーの型不変条件の追加テスト群（第四拡張）。"""

    # TenantId が連続して生成した場合に全て一意であることをテストする
    def test_tenant_id_generated_are_unique(self) -> None:
        """連続して生成した TenantId が全て一意であることを検証する。"""
        # 20 件の TenantId を生成する
        ids = [TenantId.generate() for _ in range(20)]
        # 全て一意であることを確認する
        unique_values = set(t.value for t in ids)
        # 20 件全て一意であることを確認する
        assert len(unique_values) == 20

    # TenantId が等値比較で同じ値を正しく判定することをテストする
    def test_tenant_id_equality_same_value(self) -> None:
        """同じ UUID 値を持つ TenantId が等しいと判定されることを検証する。"""
        # 特定の UUID を生成する
        uuid_val = str(uuid.uuid4())
        # 同じ値で 2 つの TenantId を生成する
        t1 = TenantId(value=uuid_val)
        # 2 つ目の TenantId を生成する
        t2 = TenantId(value=uuid_val)
        # 等しいことを確認する
        assert t1 == t2

    # QuotaLimit が大きな値でも正常に動作することをテストする
    def test_quota_limit_very_large_value(self) -> None:
        """QuotaLimit が非常に大きな値でも正常に動作することを検証する。"""
        # 大きな値で QuotaLimit を生成する
        large_quota = QuotaLimit(value=2**62)
        # 値が正しいことを確認する
        assert large_quota.value == 2**62

    # RLS ポリシーが複数の app.current_tenant 参照を持つ式を受け入れることをテストする
    def test_rls_policy_multiple_tenant_refs_accepted(self) -> None:
        """RLS ポリシーが複数の app.current_tenant 参照を含む式を受け入れることを検証する。"""
        # 複数参照を持つ USING 式を定義する
        complex_expr = (
            "tenant_id = app.current_tenant AND "
            "org_id IN (SELECT id FROM orgs WHERE app.current_tenant = owner)"
        )
        # RLS ポリシーを生成する
        policy = RLSPolicy(
            policy_name="complex_rls",
            table_name="orders",
            using_expression=complex_expr,
        )
        # バリデーションが成功することを確認する（例外なし）
        policy.validate()

    # 空の RLS 式が拒否されることをテストする
    def test_rls_policy_empty_expression_rejected(self) -> None:
        """RLS ポリシーの空の USING 式が拒否されることを検証する。"""
        # 空の式を持つ RLS ポリシーを生成する
        with pytest.raises((ValueError, AssertionError)):
            # 空の USING 式でポリシーを生成する
            policy = RLSPolicy(
                policy_name="empty_rls",
                table_name="t",
                using_expression="",
            )
            # バリデーションを実行する
            policy.validate()

    # TenantId が不正な形式を全て拒否することをテストする（境界値）
    @pytest.mark.parametrize("invalid_id", [
        "not-a-uuid",
        "12345678-1234-1234-1234-12345678901g",
        "00000000-0000-0000-0000-000000000000",
        "",
        "   ",
    ])
    def test_tenant_id_invalid_formats_rejected(self, invalid_id: str) -> None:
        """TenantId が不正な UUID 形式を全て拒否することを検証する。"""
        # 不正な UUID で TenantId を生成する
        with pytest.raises(ValueError):
            # 不正な ID で TenantId を生成する
            TenantId(value=invalid_id)

    # QuotaLimit が 1 の場合に正常動作することをテストする
    def test_quota_limit_minimum_value_one(self) -> None:
        """QuotaLimit の最小有効値 1 が正常に動作することを検証する。"""
        # 最小値の QuotaLimit を生成する
        q = QuotaLimit(value=1)
        # 値が 1 であることを確認する
        assert q.value == 1

    # RLS ポリシーの app.current_tenant なし式が拒否されることをテストする
    def test_rls_policy_no_tenant_ref_rejected(self) -> None:
        """RLS ポリシーの app.current_tenant 参照なし式が拒否されることを検証する。"""
        # テナント参照のない式を定義する
        no_ref_expr = "user_id = current_user"
        # テナント参照のない式で RLS ポリシーを生成する
        with pytest.raises((ValueError, AssertionError)):
            # RLS ポリシーを生成する
            policy = RLSPolicy(
                policy_name="no_ref_rls",
                table_name="users",
                using_expression=no_ref_expr,
            )
            # バリデーションを実行する
            policy.validate()


# tier2 型不変条件の第五拡張テストクラスを定義する
class TestTier2TypeInvariantExtended5:
    """テナント ID・クォータ・RLS ポリシーの型不変条件の追加テスト群（第五拡張）。"""

    # QuotaLimit が異なる値で等しくないことをテストする
    def test_quota_limit_different_values_not_equal(self) -> None:
        """異なる値の QuotaLimit が等しくないことを検証する。"""
        # 値 10 の QuotaLimit を生成する
        q1 = QuotaLimit(value=10)
        # 値 20 の QuotaLimit を生成する
        q2 = QuotaLimit(value=20)
        # 等しくないことを確認する
        assert q1 != q2

    # TenantId が UUID v4 以外の UUID バージョンを拒否することをテストする
    def test_tenant_id_rejects_uuid_v1(self) -> None:
        """TenantId が UUID v1 形式を拒否することを検証する。"""
        # UUID v1 形式の文字列を定義する（4 が 1 に変わっている）
        uuid_v1_like = "6ba7b810-9dad-11d1-80b4-00c04fd430c8"
        # UUID v1 で TenantId を生成する
        with pytest.raises(ValueError):
            # UUID v1 で TenantId を生成する
            TenantId(value=uuid_v1_like)

    # RLS ポリシーに app.current_tenant を含む複雑な AND 式が受け入れられることをテストする
    def test_rls_policy_complex_and_expression_accepted(self) -> None:
        """RLS ポリシーに複雑な AND 式が受け入れられることを検証する。"""
        # 複雑な AND 式を定義する
        expr = "tenant_id = app.current_tenant AND deleted_at IS NULL AND status != 'archived'"
        # RLS ポリシーを生成する
        policy = RLSPolicy(
            policy_name="complex_and_rls",
            table_name="products",
            using_expression=expr,
        )
        # バリデーションが成功することを確認する
        policy.validate()

    # TenantId の生成が異なるプロセスで一意であることをテストする
    def test_tenant_id_generate_many_unique(self) -> None:
        """多数の TenantId 生成が全て一意であることを検証する。"""
        # 50 件の TenantId を生成する
        ids = [TenantId.generate() for _ in range(50)]
        # セットに変換して一意性を確認する
        unique_count = len(set(t.value for t in ids))
        # 全て一意であることを確認する
        assert unique_count == 50
