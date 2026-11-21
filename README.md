# faker-rs

[![CI](https://github.com/harrisonclough99/faker-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/harrisonclough99/faker-rs/actions/workflows/ci.yml)
[![PyPI version](https://img.shields.io/pypi/v/faker-rs.svg)](https://pypi.org/project/faker-rs/)
[![Python versions](https://img.shields.io/pypi/pyversions/faker-rs.svg)](https://pypi.org/project/faker-rs/)
[![License](https://img.shields.io/badge/license-MIT%20%2F%20Apache--2.0-blue.svg)](LICENSE-MIT)

A high-performance, drop-in replacement for Python's `Faker` library, backed by Rust and [PyO3](https://pyo3.rs/).

## Why `faker-rs`?

- **12x–1,000x faster** than Python `Faker` for test fixtures, synthetic datasets, and database seeding.
- **Zero-loop batch APIs** (`names(n)`, `emails(n)`, `companies(n)`, etc.) generate an entire Python `list` in a single Rust loop.
- **Drop-in compatible**: `Faker()`, `Faker.seed(n)`, `fake.unique.*`, `UniquenessException`, and PEP 561 type stubs (`py.typed`).
- **Pre-built `abi3` wheels** for Linux (`manylinux`), macOS, and Windows (Python 3.9+) — no Rust compiler needed to `pip install`.

## 10-Second Quickstart

```bash
pip install faker-rs
```

```python
from faker_rs import Faker

fake = Faker()
print(fake.name())          # "Emily Johnson"
print(fake.email())         # "david.smith412@example.com"
print(fake.address())       # "4821 Oak Ave\nSpringfield, IL 62704"
print(fake.phone_number())  # "(415) 832-0914"
print(fake.company())       # "Apex Horizon Technologies"
print(fake.ipv4())          # "192.168.42.17"
print(fake.date())          # "2014-07-19"
print(fake.uuid4())         # "9b1deb4d-3b7d-4bad-9bdd-2b0d7b3dcb6d"
```

## Drop-In Migration

Replace your import and keep your existing code:

```python
# Before
# from faker import Faker
# from faker.exceptions import UniquenessException

# After
from faker_rs import Faker
from faker_rs.exceptions import UniquenessException

Faker.seed(42)
fake = Faker("en_US")
users = [
    {
        "name": fake.name(),
        "email": fake.unique.email(),
        "company": fake.company(),
        "ip": fake.ipv4(),
    }
    for _ in range(1000)
]
```

### Batch Mode (Zero-Loop FFI)

For bulk generation, use the batch methods to generate an entire Python `list` in a single Rust loop:

```python
fake = Faker(seed=42)
names = fake.names(100_000)
emails = fake.emails(100_000)
```

## Provider Compatibility

| Category | Single Methods | Batch Methods | `fake.unique` |
| --- | --- | --- | --- |
| **Person** | `name()`, `first_name()`, `last_name()`, `user_name()` | `names(n)`, `first_names(n)`, `last_names(n)`, `user_names(n)` | Yes |
| **Address** | `address()`, `city()`, `state()`, `postcode()`, `country()` | `addresses(n)`, `cities(n)`, `states(n)`, `postcodes(n)`, `countries(n)` | Yes |
| **Company & Contact** | `company()`, `job()`, `phone_number()` | `companies(n)`, `jobs(n)`, `phone_numbers(n)` | Yes |
| **Internet** | `email()`, `url()`, `ipv4()` | `emails(n)`, `urls(n)`, `ipv4s(n)` | Yes |
| **Date & Time** | `date()`, `iso8601()` | `dates(n)`, `iso8601s(n)` | Yes |
| **Identifiers** | `uuid4()` | `uuid4s(n)` | Yes |
| **Text & Primitives** | `sentence()`, `text(max_nb_chars=200)`, `boolean(chance_of_getting_true=50)`, `random_int(min=0, max=9999)` | `sentences(n)`, `texts(n, ...)`, `booleans(n, ...)`, `random_ints(n, ...)` | `random_int()` |

## Benchmarks (100,000 Records)

Measured on Windows (CPython 3.12.11, `faker==40.63.0`, `faker-rs==0.2.0` built with `maturin develop --release`) via `python benchmarks/bench.py`:

| Provider | Python `Faker` | `faker-rs` (single loop) | Single Speedup | `faker-rs` (batch) | Batch Speedup |
| --- | ---: | ---: | ---: | ---: | ---: |
| `name` | 8.3670 s | 0.0153 s | **546.4x** | 0.0084 s | **1,000.6x** |
| `company` | 11.6019 s | 0.0199 s | **583.6x** | 0.0130 s | **893.7x** |
| `email` | 8.8890 s | 0.0198 s | **449.0x** | 0.0132 s | **674.4x** |
| `address` | 12.1838 s | 0.0340 s | **358.7x** | 0.0237 s | **514.3x** |
| `ipv4` | 3.1658 s | 0.0289 s | **109.6x** | 0.0194 s | **163.3x** |
| `phone_number` | 1.0797 s | 0.0205 s | **52.7x** | 0.0135 s | **80.0x** |
| `text` | 4.7088 s | 0.0773 s | **60.9x** | 0.0671 s | **70.2x** |
| `date` | 0.6662 s | 0.0207 s | **32.2x** | 0.0148 s | **45.1x** |
| `uuid4` | 0.2524 s | 0.0199 s | **12.7x** | 0.0116 s | **21.7x** |

## Contributing

Contributions are welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for local `maturin develop` setup instructions and PR guidelines.

## License

Dual-licensed under either of:

- [MIT License](LICENSE-MIT)
- [Apache License, Version 2.0](LICENSE-APACHE)

at your option.
