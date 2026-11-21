# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
