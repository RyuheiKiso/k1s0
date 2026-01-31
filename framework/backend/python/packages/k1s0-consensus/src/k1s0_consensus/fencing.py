"""Fence token validation for distributed lock safety."""

from __future__ import annotations

import threading


class FencingValidator:
    """Thread-safe monotonic fence token validator.

    A fence token is a monotonically increasing integer issued when a lock
    is acquired. Resources protected by fencing reject operations that
    present a token lower than the highest previously accepted token,
    preventing stale lock holders from corrupting data.

    Example::

        validator = FencingValidator()
        assert validator.validate(1) is True
        assert validator.validate(2) is True
        assert validator.validate(1) is False  # stale token rejected

    This class is thread-safe and can be shared across threads.
    """

    def __init__(self) -> None:
        self._lock = threading.Lock()
        self._highest_token: int = -1

    def validate(self, token: int) -> bool:
        """Validate a fence token against the highest seen token.

        The token is accepted only if it is strictly greater than the
        highest previously accepted token. On acceptance, the highest
        token is updated.

        Args:
            token: The fence token to validate.

        Returns:
            True if the token is valid (higher than any previously seen),
            False if stale.
        """
        with self._lock:
            if token > self._highest_token:
                self._highest_token = token
                return True
            return False

    @property
    def highest_token(self) -> int:
        """The highest fence token accepted so far.

        Returns -1 if no token has been validated yet.
        """
        with self._lock:
            return self._highest_token

    def reset(self) -> None:
        """Reset the validator to its initial state.

        This is primarily useful in testing. In production, resetting
        the validator can lead to stale token acceptance.
        """
        with self._lock:
            self._highest_token = -1
