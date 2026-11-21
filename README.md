# faker-rs

[![CI](https://github.com/harrisonclough99/faker-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/harrisonclough99/faker-rs/actions/workflows/ci.yml)
[![PyPI version](https://img.shields.io/pypi/v/faker-rs.svg)](https://pypi.org/project/faker-rs/)
[![Python versions](https://img.shields.io/pypi/pyversions/faker-rs.svg)](https://pypi.org/project/faker-rs/)
[![License](https://img.shields.io/badge/license-MIT%20%2F%20Apache--2.0-blue.svg)](LICENSE-MIT)

A high-performance, drop-in replacement for Python's `Faker` library, backed by Rust and [PyO3](https://pyo3.rs/).

## Why `faker-rs`?

- **14x–988x faster** than Python `Faker` for test fixtures, synthetic datasets, and database seeding.
- **Zero-loop batch APIs** (`names(n)`, `emails(n)`, `credit_card_numbers(n)`, `date_objects(n)`, etc.) generate an entire Python `list` in a single Rust loop.
- **Drop-in compatible**: `Faker(locale)`, `Faker.seed(n)`, `fake.unique.*`, `fake.optional.*`, `fake.add_provider(...)`, `UniquenessException`, and PEP 561 type stubs (`py.typed`).
- **Pre-built `abi3` wheels** for Linux (`manylinux`), macOS, and Windows (Python 3.9–3.15) — no Rust compiler needed to `pip install`.

## 10-Second Quickstart

```bash
pip install faker-rs
```

```python
from faker_rs import Faker

fake = Faker()
print(fake.name())                # "Emily Johnson"
print(fake.email())               # "david.smith412@example.com"
print(fake.address())             # "4821 Oak Ave\nSpringfield, IL 62704"
print(fake.credit_card_number())  # Luhn-valid Visa / Mastercard / Amex
print(fake.date_of_birth(21, 65)) # datetime.date(...)
print(fake.uuid4())               # "9b1deb4d-3b7d-4bad-9bdd-2b0d7b3dcb6d"
```

## Drop-In Migration

Replace your import and keep your existing code:

```python
# Before
# from faker import Faker
# from faker.exceptions import UniquenessException
# from faker.providers import BaseProvider

# After
from faker_rs import BaseProvider, Faker
from faker_rs.exceptions import UniquenessException

Faker.seed(42)
fake = Faker(["en_US", "fr_FR"])
users = [
    {
        "name": fake.name(),
        "email": fake.unique.email(),
        "phone": fake.optional.phone_number(p=0.8),
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
floats = fake.pyfloats(100_000)
```

## Provider Compatibility

| Category | Single Methods | Batch Methods |
| --- | --- | --- |
| **Person** | `name()`, `first_name()`, `last_name()`, `user_name()` | `names(n)`, `first_names(n)`, `last_names(n)`, `user_names(n)` |
| **Address** | `address()`, `city()`, `state()`, `postcode()`, `country()` | `addresses(n)`, `cities(n)`, `states(n)`, `postcodes(n)`, `countries(n)` |
| **Company & Contact** | `company()`, `job()`, `phone_number()` | `companies(n)`, `jobs(n)`, `phone_numbers(n)` |
| **Internet** | `email()`, `free_email()`, `company_email()`, `safe_email()`, `url()`, `domain_name()`, `ipv4()`, `ipv6()`, `mac_address()`, `user_agent()`, `slug()` | `emails(n)`, `free_emails(n)`, `company_emails(n)`, `safe_emails(n)`, `urls(n)`, `domain_names(n)`, `ipv4s(n)`, `ipv6s(n)`, `mac_addresses(n)`, `user_agents(n)`, `slugs(n)` |
| **Finance** | `credit_card_number()`, `credit_card_expire()`, `credit_card_security_code()`, `credit_card_provider()`, `iban()`, `currency_code()` | `credit_card_numbers(n)`, `credit_card_expires(n)`, `credit_card_security_codes(n)`, `credit_card_providers(n)`, `ibans(n)`, `currency_codes(n)` |
| **Date & Time** | `date(pattern)`, `iso8601()`, `date_object()`, `date_time()`, `date_between()`, `date_of_birth()` | `dates(n, pattern)`, `iso8601s(n)`, `date_objects(n)`, `date_times(n)`, `date_betweens(n)`, `date_of_births(n)` |
| **Patterns & Primitives** | `numerify()`, `lexify()`, `bothify()`, `pyint()`, `pyfloat()`, `pybool()`, `pystr()`, `password()`, `random_int()`, `random_element()`, `random_elements()`, `boolean()` | `numerifys(n)`, `lexifys(n)`, `bothifys(n)`, `pyints(n)`, `pyfloats(n)`, `pybools(n)`, `pystrs(n)`, `passwords(n)`, `random_ints(n)`, `booleans(n)` |
| **Text, Color, Geo, File** | `word()`, `words()`, `sentence()`, `paragraph()`, `paragraphs()`, `text()`, `hex_color()`, `rgb_color()`, `latitude()`, `longitude()`, `file_name()`, `mime_type()`, `uuid4()` | `words_batch(n)`, `sentences(n)`, `paragraphs_batch(n)`, `texts(n)`, `hex_colors(n)`, `rgb_colors(n)`, `latitudes(n)`, `longitudes(n)`, `file_names(n)`, `mime_types(n)`, `uuid4s(n)` |
| **Framework** | `Faker(locale)`, `Faker.seed(n)`, `seed_instance(n)`, `fake.unique.*`, `fake.optional.*`, `add_provider()`, `BaseProvider` | Locales: `en_US`, `en_GB`, `de_DE`, `fr_FR`, `es_ES`, `ja_JP` |

## Benchmarks (100,000 Records)

Measured on Windows (CPython 3.12.11, `faker==40.63.0`, built with `maturin develop --release`) via `python benchmarks/bench.py`:

| Provider | Python `Faker` | `faker-rs` (single loop) | Single Speedup | `faker-rs` (batch) | Batch Speedup |
| --- | ---: | ---: | ---: | ---: | ---: |
| `name` | 8.3959 s | 0.0130 s | **647.1x** | 0.0085 s | **988.8x** |
| `company` | 11.8047 s | 0.0173 s | **681.7x** | 0.0126 s | **939.5x** |
| `address` | 11.8129 s | 0.0243 s | **485.9x** | 0.0188 s | **627.1x** |
| `email` | 8.3588 s | 0.0194 s | **430.0x** | 0.0136 s | **614.5x** |
| `ipv4` | 3.1587 s | 0.0194 s | **162.5x** | 0.0147 s | **214.2x** |
| `credit_card_number` | 1.4828 s | 0.0170 s | **87.3x** | 0.0104 s | **142.0x** |
| `pyfloat` | 0.5233 s | 0.0105 s | **49.8x** | 0.0052 s | **99.7x** |
| `phone_number` | 1.0438 s | 0.0154 s | **67.9x** | 0.0110 s | **95.2x** |
| `text` | 4.7037 s | 0.0771 s | **61.0x** | 0.0696 s | **67.6x** |
| `password` | 0.6328 s | 0.0189 s | **33.4x** | 0.0116 s | **54.7x** |
| `date` | 0.6379 s | 0.0196 s | **32.6x** | 0.0120 s | **53.0x** |
| `numerify` | 0.3167 s | 0.0142 s | **22.4x** | 0.0077 s | **41.3x** |
| `date_object` | 0.3751 s | 0.0167 s | **22.4x** | 0.0122 s | **30.7x** |
| `ipv6` | 0.4850 s | 0.0229 s | **21.2x** | 0.0187 s | **25.9x** |
| `uuid4` | 0.1971 s | 0.0129 s | **15.3x** | 0.0085 s | **23.3x** |
| `date_time` | 0.3783 s | 0.0271 s | **14.0x** | 0.0221 s | **17.1x** |

## Contributing

Contributions are welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for local `maturin develop` setup instructions and PR guidelines.

## License

Dual-licensed under either of:

- [MIT License](LICENSE-MIT)
- [Apache License, Version 2.0](LICENSE-APACHE)

at your option.
