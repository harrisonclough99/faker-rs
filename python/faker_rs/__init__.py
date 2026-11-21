from __future__ import annotations

from typing import Any

from ._faker_rs import (
    Faker as _RustFaker,
)
from ._faker_rs import (
    OptionalProxy,
    UniquenessException,
    UniqueProxy,
    __version__,
)
from .providers import BaseProvider


class Faker(_RustFaker):
    """High-performance, drop-in replacement for Python's ``faker.Faker``."""

    def add_provider(self, provider: type[BaseProvider] | BaseProvider | Any) -> None:
        """Register a custom provider class or instance on this ``Faker`` object."""
        instance = provider(self) if isinstance(provider, type) else provider
        for attr in dir(instance):
            if attr.startswith("_") or attr == "generator":
                continue
            member = getattr(instance, attr)
            if callable(member):
                setattr(self, attr, member)


__all__ = [
    "BaseProvider",
    "Faker",
    "OptionalProxy",
    "UniqueProxy",
    "UniquenessException",
    "__version__",
]
