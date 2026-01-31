"""Tests for FencingValidator."""

from __future__ import annotations

import threading

import pytest

from k1s0_consensus.fencing import FencingValidator


class TestFencingValidator:
    """Tests for the FencingValidator class."""

    def test_initial_state(self, fencing_validator: FencingValidator) -> None:
        """Highest token should be -1 initially."""
        assert fencing_validator.highest_token == -1

    def test_accepts_first_token(self, fencing_validator: FencingValidator) -> None:
        """The first token should always be accepted if > -1."""
        assert fencing_validator.validate(0) is True
        assert fencing_validator.highest_token == 0

    def test_accepts_increasing_tokens(self, fencing_validator: FencingValidator) -> None:
        """Monotonically increasing tokens should all be accepted."""
        assert fencing_validator.validate(1) is True
        assert fencing_validator.validate(5) is True
        assert fencing_validator.validate(100) is True
        assert fencing_validator.highest_token == 100

    def test_rejects_stale_token(self, fencing_validator: FencingValidator) -> None:
        """A token lower than the highest seen should be rejected."""
        fencing_validator.validate(10)
        assert fencing_validator.validate(5) is False
        assert fencing_validator.highest_token == 10

    def test_rejects_equal_token(self, fencing_validator: FencingValidator) -> None:
        """A token equal to the highest seen should be rejected."""
        fencing_validator.validate(10)
        assert fencing_validator.validate(10) is False

    def test_reset(self, fencing_validator: FencingValidator) -> None:
        """After reset, the validator should accept any token again."""
        fencing_validator.validate(50)
        fencing_validator.reset()
        assert fencing_validator.highest_token == -1
        assert fencing_validator.validate(1) is True

    def test_thread_safety(self) -> None:
        """Concurrent validation should not lose updates."""
        validator = FencingValidator()
        results: list[bool] = []
        lock = threading.Lock()

        def validate_range(start: int, end: int) -> None:
            for i in range(start, end):
                result = validator.validate(i)
                with lock:
                    results.append(result)

        threads = [
            threading.Thread(target=validate_range, args=(0, 100)),
            threading.Thread(target=validate_range, args=(50, 150)),
        ]
        for t in threads:
            t.start()
        for t in threads:
            t.join()

        # Exactly one True per unique accepted value
        true_count = sum(1 for r in results if r)
        # The highest token should be 149
        assert validator.highest_token == 149
        # At least 150 tokens attempted (some overlap rejected)
        assert len(results) == 200
        # Each unique value can be accepted at most once
        assert true_count <= 150

    @pytest.mark.parametrize(
        ("tokens", "expected_results"),
        [
            ([1, 2, 3], [True, True, True]),
            ([3, 2, 1], [True, False, False]),
            ([1, 1, 2, 2, 3], [True, False, True, False, True]),
            ([0], [True]),
        ],
        ids=["ascending", "descending", "duplicates", "single-zero"],
    )
    def test_token_sequences(
        self,
        tokens: list[int],
        expected_results: list[bool],
    ) -> None:
        """Parameterized test for various token sequences."""
        validator = FencingValidator()
        results = [validator.validate(t) for t in tokens]
        assert results == expected_results
