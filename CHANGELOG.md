# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.6.0] - 2026-11-21

### Added
- **Multi-locale support**: Built-in localized datasets for `en_US`, `en_GB`, `de_DE`, `fr_FR`, `es_ES`, and `ja_JP` (`Faker("de_DE")`, `Faker(["en_US", "fr_FR"])`, `fake.locales`).
- **`fake.optional` proxy**: Returns `None` with probability `1 - p` (`fake.optional.email(p=0.5)`).
- **Custom providers**: Added `BaseProvider` (`faker_rs.providers.BaseProvider`) and `fake.add_provider(...)` for drop-in custom Python providers.

## [0.5.0] - 2026-11-21

### Added
- **Date/Time objects & ranges**: `date(pattern="%Y-%m-%d")`, `date_object()`, `date_time()`, `date_between(start_date="-30y", end_date="today")`, `date_of_birth(minimum_age=0, maximum_age=115)` (+ batch `dates`, `date_objects`, `date_times`, `date_betweens`, `date_of_births`).

## [0.4.0] - 2026-11-21

### Added
- **Finance providers**: Luhn-valid `credit_card_number()`, `credit_card_expire()`, `credit_card_security_code()`, `credit_card_provider()`, `iban()`, `currency_code()` (+ batch `*s(n)` methods).
- **Internet providers**: `ipv6()`, `mac_address()`, `domain_name()`, `user_agent()`, `slug()`, `free_email()`, `company_email()`, `safe_email()` (+ batch `*s(n)` methods).
- **Color, Geo & File providers**: `hex_color()`, `rgb_color()`, `latitude()`, `longitude()`, `file_name()`, `mime_type()` (+ batch `*s(n)` methods).
- **Unique proxy**: Extended `fake.unique` with all new finance, internet, color, and file providers.

## [0.3.0] - 2026-11-21

### Added
- **Pattern & primitive helpers**: `numerify()`, `lexify()`, `bothify()`, `pyint()`, `pyfloat()`, `pybool()`, `pystr()`, `password()`, `random_element()`, `random_elements()` (+ batch `numerifys`, `lexifys`, `bothifys`, `pyints`, `pyfloats`, `pybools`, `pystrs`, `passwords`).
- **Text providers**: `word()`, `words(nb=3)`, `paragraph(nb_sentences=3)`, `paragraphs(nb=3)` (+ batch `words_batch`, `paragraphs_batch`).
- **Unique proxy**: Added `word`, `numerify`, `lexify`, `bothify`, `pystr`, `pyint` to `fake.unique`.

## [0.2.1] - 2026-11-21

### Changed
- Upgraded `pyo3` to `0.30` for native Python 3.14 and 3.15 support (including source builds on Python 3.15.2).
- Added Python `3.14` and `3.15` PyPI classifiers and added Python `3.15` to the CI test matrix.

## [0.2.0] - 2026-11-21

### Added
- **Expanded core providers** with matching batch (`*s(n)`) methods:
  - Person: `first_name()`, `last_name()`, `user_name()`
  - Address: `city()`, `state()`, `postcode()`, `country()`
  - Company & Contact: `company()`, `job()`, `phone_number()`
  - Internet: `url()`, `ipv4()`
  - Text & Primitives: `sentence()`, `text(max_nb_chars=200)`, `boolean(chance_of_getting_true=50)`, `random_int(min=0, max=9999)`
  - Date & Time: `iso8601()`
- **`fake.unique` proxy**: Per-provider uniqueness tracking with `fake.unique.clear()` and `UniquenessException` (exported from `faker_rs` and `faker_rs.exceptions`).
- **Open-source governance & templates**: `CHANGELOG.md`, `CODE_OF_CONDUCT.md`, `SECURITY.md`, GitHub issue/PR templates, and `dependabot.yml`.

## [0.1.0] - 2026-11-21

### Added
- Initial Rust + PyO3 (`abi3-py39`) implementation of `Faker` with `name()`, `email()`, `address()`, `date()`, `uuid4()`, and batch equivalents (`names(n)`, `emails(n)`, `addresses(n)`, `dates(n)`, `uuid4s(n)`).
- Deterministic global (`Faker.seed(n)`) and instance (`Faker(seed=n)`, `seed_instance(n)`) seeding powered by `fastrand::Rng`.
- GitHub Actions CI (`cargo test`, `cargo clippy`, `pytest`, `ruff`) and Trusted Publishing release workflow (`PyO3/maturin-action`).
