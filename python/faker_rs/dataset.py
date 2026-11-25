"""Lightweight relational synthetic dataset builder and CSV exporter for faker-rs."""

from __future__ import annotations

import csv
from collections.abc import Callable, Mapping, Sequence
from dataclasses import dataclass
from pathlib import Path
from typing import Any


@dataclass(frozen=True)
class PrimaryKey:
    """Sequential primary key column specification."""

    prefix: str = ""
    start: int = 1
    pad: int = 0

    def generate(self, count: int) -> list[int | str]:
        if not self.prefix and self.pad == 0:
            return list(range(self.start, self.start + count))
        if self.pad > 0:
            return [f"{self.prefix}{i:0{self.pad}d}" for i in range(self.start, self.start + count)]
        return [f"{self.prefix}{i}" for i in range(self.start, self.start + count)]


@dataclass(frozen=True)
class ForeignKey:
    """Foreign key referencing a column in an already-generated parent table."""

    table: str
    column: str
    weights: Sequence[float] | Callable[[dict[str, Any]], float] | None = None


@dataclass(frozen=True)
class Weighted:
    """Weighted categorical column specification (uses faker-rs batch weighted sampling)."""

    choices: Mapping[Any, float]


@dataclass(frozen=True)
class Derived:
    """Column derived from the current row and previously generated tables."""

    fn: Callable[[dict[str, Any], dict[str, list[dict[str, Any]]]], Any]


def export_csv(rows: Sequence[Mapping[str, Any]], path: str | Path) -> Path:
    """Write a list of row dicts to a CSV file with UTF-8 encoding."""
    out_path = Path(path)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    fieldnames = list(rows[0].keys()) if rows else []
    with out_path.open("w", newline="", encoding="utf-8") as fh:
        writer = csv.DictWriter(fh, fieldnames=fieldnames)
        if fieldnames:
            writer.writeheader()
            writer.writerows(rows)
    return out_path


class Dataset:
    """Orchestrates multi-table relational synthetic data using a single seeded ``Faker``."""

    def __init__(self, faker: Any) -> None:
        self.faker = faker
        self.tables: dict[str, list[dict[str, Any]]] = {}

    def add_table(
        self,
        name: str,
        rows: int,
        columns: Mapping[str, Any],
    ) -> list[dict[str, Any]]:
        """Generate ``rows`` records for ``name`` and store them in ``self.tables[name]``.

        Each value in ``columns`` may be:
        - ``PrimaryKey(...)`` for sequential IDs
        - ``ForeignKey(table, column, weights=...)`` for parent references
        - ``Weighted({val: weight, ...})`` for skewed categorical columns
        - ``Derived(lambda row, tables: ...)`` for cross-column / cross-table values
        - a ``str`` naming a ``Faker`` method (uses batch ``*s(rows)`` when available)
        - a zero-arg callable ``() -> value``
        """
        precomputed: dict[str, list[Any]] = {}
        derived_cols: list[tuple[str, Derived]] = []

        for col_name, spec in columns.items():
            if isinstance(spec, PrimaryKey):
                precomputed[col_name] = spec.generate(rows)
            elif isinstance(spec, Weighted):
                precomputed[col_name] = self.faker.random_choices(spec.choices, k=rows)
            elif isinstance(spec, ForeignKey):
                parent_rows = self.tables[spec.table]
                parent_vals = [r[spec.column] for r in parent_rows]
                if spec.weights is None:
                    precomputed[col_name] = self.faker.random_choices(parent_vals, k=rows)
                elif callable(spec.weights):
                    w = [float(spec.weights(r)) for r in parent_rows]
                    precomputed[col_name] = self.faker.random_choices(
                        parent_vals, weights=w, k=rows
                    )
                else:
                    precomputed[col_name] = self.faker.random_choices(
                        parent_vals, weights=list(spec.weights), k=rows
                    )
            elif isinstance(spec, Derived):
                derived_cols.append((col_name, spec))
            elif isinstance(spec, str):
                batch_name = f"{spec}s"
                if hasattr(self.faker, batch_name):
                    precomputed[col_name] = getattr(self.faker, batch_name)(rows)
                else:
                    fn = getattr(self.faker, spec)
                    precomputed[col_name] = [fn() for _ in range(rows)]
            elif callable(spec):
                precomputed[col_name] = [spec() for _ in range(rows)]
            else:
                precomputed[col_name] = [spec] * rows

        col_order = list(columns.keys())
        records: list[dict[str, Any]] = []
        for idx in range(rows):
            row: dict[str, Any] = {}
            for col_name in col_order:
                if col_name in precomputed:
                    row[col_name] = precomputed[col_name][idx]
            for col_name, derived in derived_cols:
                row[col_name] = derived.fn(row, self.tables)
            records.append(row)

        self.tables[name] = records
        return records

    def add_join_table(
        self,
        name: str,
        left_table: str,
        left_pk: str,
        right_table: str,
        right_pk: str,
        *,
        left_col: str | None = None,
        right_col: str | None = None,
        min_links: int = 1,
        max_links: int = 3,
        extra_columns: Mapping[str, Any] | None = None,
    ) -> list[dict[str, Any]]:
        """Generate a many-to-many join table with unique ``(left_id, right_id)`` pairs."""
        left_ids = [r[left_pk] for r in self.tables[left_table]]
        right_ids = [r[right_pk] for r in self.tables[right_table]]
        l_col = left_col or left_pk
        r_col = right_col or right_pk

        max_possible = len(right_ids)
        lo = max(0, min(min_links, max_possible))
        hi = max(lo, min(max_links, max_possible))

        records: list[dict[str, Any]] = []
        for lid in left_ids:
            k = self.faker.random_int(min=lo, max=hi) if hi > lo else lo
            if k == 0:
                continue
            picked = self.faker.random_sample(right_ids, length=k)
            for rid in picked:
                row: dict[str, Any] = {l_col: lid, r_col: rid}
                if extra_columns:
                    for col_name, spec in extra_columns.items():
                        if isinstance(spec, Derived):
                            row[col_name] = spec.fn(row, self.tables)
                        elif isinstance(spec, Weighted):
                            row[col_name] = self.faker.random_element(spec.choices)
                        elif callable(spec):
                            row[col_name] = spec()
                        else:
                            row[col_name] = spec
                records.append(row)

        self.tables[name] = records
        return records

    def to_csv(self, output_dir: str | Path) -> dict[str, Path]:
        """Export every table in the dataset to ``<output_dir>/<table>.csv``."""
        out_dir = Path(output_dir)
        out_dir.mkdir(parents=True, exist_ok=True)
        written: dict[str, Path] = {}
        for table_name, rows in self.tables.items():
            written[table_name] = export_csv(rows, out_dir / f"{table_name}.csv")
        return written


__all__ = ["Dataset", "Derived", "ForeignKey", "PrimaryKey", "Weighted", "export_csv"]
