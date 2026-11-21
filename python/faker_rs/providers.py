from __future__ import annotations

from typing import Any


class BaseProvider:
    """Minimal drop-in base class for custom ``faker-rs`` providers."""

    def __init__(self, generator: Any) -> None:
        self.generator = generator


__all__ = ["BaseProvider"]
