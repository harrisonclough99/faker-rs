"""Benchmark faker-rs against Python's standard Faker library."""

from __future__ import annotations

import time
from collections.abc import Callable

from faker import Faker as PyFaker
from faker_rs import Faker as RsFaker

COUNT = 100_000


def bench(label: str, fn: Callable[[], object]) -> float:
    start = time.perf_counter()
    fn()
    elapsed = time.perf_counter() - start
    print(f"{label:<40} {elapsed:8.4f} s  ({COUNT / elapsed:>12,.0f} rec/s)")
    return elapsed


def main() -> None:
    py_fake = PyFaker()
    PyFaker.seed(42)

    rs_fake = RsFaker(seed=42)

    providers = [
        ("name", "names", py_fake.name, rs_fake.name, lambda: rs_fake.names(COUNT)),
        ("email", "emails", py_fake.email, rs_fake.email, lambda: rs_fake.emails(COUNT)),
        (
            "address",
            "addresses",
            py_fake.address,
            rs_fake.address,
            lambda: rs_fake.addresses(COUNT),
        ),
        (
            "phone_number",
            "phone_numbers",
            py_fake.phone_number,
            rs_fake.phone_number,
            lambda: rs_fake.phone_numbers(COUNT),
        ),
        (
            "company",
            "companies",
            py_fake.company,
            rs_fake.company,
            lambda: rs_fake.companies(COUNT),
        ),
        ("ipv4", "ipv4s", py_fake.ipv4, rs_fake.ipv4, lambda: rs_fake.ipv4s(COUNT)),
        ("text", "texts", py_fake.text, rs_fake.text, lambda: rs_fake.texts(COUNT)),
        ("date", "dates", py_fake.date, rs_fake.date, lambda: rs_fake.dates(COUNT)),
        ("uuid4", "uuid4s", py_fake.uuid4, rs_fake.uuid4, lambda: rs_fake.uuid4s(COUNT)),
        (
            "numerify",
            "numerifys",
            py_fake.numerify,
            rs_fake.numerify,
            lambda: rs_fake.numerifys(COUNT),
        ),
        (
            "pyfloat",
            "pyfloats",
            py_fake.pyfloat,
            rs_fake.pyfloat,
            lambda: rs_fake.pyfloats(COUNT),
        ),
        (
            "password",
            "passwords",
            py_fake.password,
            rs_fake.password,
            lambda: rs_fake.passwords(COUNT),
        ),
        (
            "paragraph",
            "paragraphs_batch",
            py_fake.paragraph,
            rs_fake.paragraph,
            lambda: rs_fake.paragraphs_batch(COUNT),
        ),
        (
            "credit_card",
            "credit_card_numbers",
            py_fake.credit_card_number,
            rs_fake.credit_card_number,
            lambda: rs_fake.credit_card_numbers(COUNT),
        ),
        (
            "ipv6",
            "ipv6s",
            py_fake.ipv6,
            rs_fake.ipv6,
            lambda: rs_fake.ipv6s(COUNT),
        ),
        (
            "date_object",
            "date_objects",
            py_fake.date_object,
            rs_fake.date_object,
            lambda: rs_fake.date_objects(COUNT),
        ),
        (
            "date_time",
            "date_times",
            py_fake.date_time,
            rs_fake.date_time,
            lambda: rs_fake.date_times(COUNT),
        ),
    ]

    print(f"Benchmarking {COUNT:,} records per provider...\n")
    print(f"{'Provider / Mode':<40} {'Time':>10}  {'Throughput':>18}")
    print("-" * 72)

    results: list[tuple[str, float, float, float]] = []
    for name, batch_name, py_fn, rs_single, rs_batch in providers:
        t_py = bench(
            f"Python Faker: {name}() x {COUNT:,}",
            lambda fn=py_fn: [fn() for _ in range(COUNT)],
        )
        t_rs_single = bench(
            f"faker-rs:     {name}() x {COUNT:,}",
            lambda fn=rs_single: [fn() for _ in range(COUNT)],
        )
        t_rs_batch = bench(f"faker-rs:     {batch_name}({COUNT:,})", rs_batch)
        results.append((name, t_py, t_rs_single, t_rs_batch))
        print("-" * 72)

    print("\nSummary Speedups vs Python Faker:")
    header = (
        f"{'Provider':<14} {'Python (s)':>12} {'rs single (s)':>14} "
        f"{'Single Speedup':>15} {'rs batch (s)':>13} {'Batch Speedup':>14}"
    )
    print(header)
    for name, t_py, t_rs_s, t_rs_b in results:
        row = (
            f"{name:<14} {t_py:12.4f} {t_rs_s:14.4f} "
            f"{t_py / t_rs_s:14.1f}x {t_rs_b:13.4f} {t_py / t_rs_b:13.1f}x"
        )
        print(row)


if __name__ == "__main__":
    main()
