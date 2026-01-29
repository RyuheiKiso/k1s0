"""k1s0-config: YAML-based configuration management."""

from __future__ import annotations

from k1s0_config.config import K1s0Config
from k1s0_config.loader import load_config

__all__ = ["K1s0Config", "load_config"]
