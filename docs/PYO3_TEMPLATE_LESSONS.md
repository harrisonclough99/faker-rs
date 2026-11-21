# PyO3 + Maturin Template Lessons Learned

A reusable engineering playbook for building high-performance, Rust-backed Python packages with PyO3 and Maturin, distilled from building `faker-rs`.

---

## 1. Build & Maturin Gotchas

### Never put `pyo3/extension-module` in default `Cargo.toml` features
- **Symptom**: `cargo test` fails to link (`undefined reference to PyExc_...` on Linux/macOS or missing Python symbols) when `extension-module` is enabled in `Cargo.toml` under `[dependencies]`.
- **Fix**: Keep `pyo3 = { version = "0.23", features = ["abi3-py39"] }` in `Cargo.toml` without `extension-module`, and enable it exclusively for wheel builds in `pyproject.toml`:
  ```toml
  [tool.maturin]
  features = ["pyo3/extension-module"]
  ```

### Use `abi3-py39` for Broad Python Compatibility
- Enabling `abi3-py39` builds a single Stable ABI wheel (`cp39-abi3`) per OS/arch instead of one wheel per Python minor version (3.9, 3.10, 3.11, 3.12, 3.13).
- This cuts CI build matrices by ~5x while remaining forward-compatible with newer CPython releases.

### Hybrid `python-source` + `_native` Extension Layout
- Set `python-source = "python"` and `module-name = "pkg_name._pkg_name"` in `pyproject.toml`.
- Re-export public symbols in `python/pkg_name/__init__.py` and ship `__init__.pyi` + `py.typed` alongside it so IDEs and `mypy`/`pyright` get full type signatures without needing `stubgen`.

### Windows Linker & Virtualenv Discovery
- On Windows, `pyo3-build-config` resolves `python3.lib` / `python3xx.lib` from the active `VIRTUAL_ENV` or `PATH`. Ensure `.venv\Scripts` is on `PATH` (or `VIRTUAL_ENV` is set) when running `cargo test` or `cargo clippy` outside `maturin`.

### Observed Build & Lint Gotchas in `faker-rs`
- **`README.md` required before `maturin develop`**: `maturin` parses `readme = "README.md"` from `Cargo.toml` / `pyproject.toml` up front and aborts if the file does not exist yet. Create a stub `README.md` as part of initial scaffolding before running `maturin`.
- **Newer Clippy lints (`clippy::manual_is_multiple_of`)**: On Rust `1.100.0`, `cargo clippy --all-targets -- -D warnings` rejected `year % 4 == 0` in favor of `year.is_multiple_of(4)`. Keep `cargo clippy -- -D warnings` in both local checks and CI so toolchain upgrades are caught immediately.

---

## 2. PyO3 / FFI Performance Patterns

### Batch APIs Eliminate Python-to-Rust FFI Overhead
- Calling a Rust `#[pymethods]` function 100,000 times from a Python `for` loop pays the Python-to-C-API call boundary, GIL/object dispatch, and mutex acquisition 100,000 times.
- Providing batch methods (`names(n)`, `emails(n)`, etc.) that acquire the RNG lock **once** and construct a `PyList` in a single Rust loop (`PyList::new(py, (0..n).map(|_| gen(rng)))`) avoids intermediate `Vec<String>` allocations and slashes per-record overhead.
- **Observed in `benchmarks/bench.py` (100,000 records, CPython 3.12.11)**:
  - `uuid4()` in a Python loop took `0.0172 s` (~172 ns/call), while batch `uuid4s(100_000)` took `0.0071 s` (~71 ns/record, 14.0M rec/s) — a **2.4x** speedup over single-call FFI and **44.2x** over Python `Faker`.
  - Across `name`, `email`, `address`, and `date`, batch mode cut runtime by **1.5x–2.0x** compared to single-call FFI (`0.0183 s -> 0.0096 s` for `name`, `0.0260 s -> 0.0131 s` for `email`, `0.0280 s -> 0.0181 s` for `address`, `0.0193 s -> 0.0106 s` for `date`), reaching **72.4x–910.0x** over Python `Faker`.
  - The remaining ~71–181 ns/record in batch mode is CPython `PyString` + `PyList` allocation inside `PyList::new`, confirming that single-lock `PyList::new` is the practical ceiling for `list[str]` returns.

### Pre-size Buffers and Avoid `format!` in Hot Loops
- Use `String::with_capacity(...)` + `push_str` / `std::fmt::Write` rather than `format!` when assembling strings in tight loops.
- For fixed-width outputs like UUIDv4 (36 bytes), format directly into a stack `[u8; 36]` buffer using a hex lookup table and convert via `String::from_utf8_unchecked`.

### Lock-Free / Low-Contention Seeding (`Sync` `#[pyclass]`)
- PyO3 0.23 requires `#[pyclass]` types to be `Send + Sync` (unless marked `unsendable`). Wrapping RNG state in `std::sync::Mutex` satisfies `Sync` with ~10–15 ns uncontended lock overhead.
- For global `Class.seed(n)` semantics without a global lock on every call, pair `AtomicU64` (`GLOBAL_SEED`) with a `SEED_EPOCH` counter (`AtomicU64::load(Ordering::Acquire)`). Unseeded instances only re-seed when `epoch != self.epoch`.

---

## 3. CI/CD & Wheel Publishing Traps

### `PyO3/maturin-action` + `manylinux: auto`
- Always pass `manylinux: auto` on Linux in `release.yml` so wheels are compatible with standard Linux distros (not just the runner's glibc).
- Build both `x86_64` and `aarch64` for Linux and macOS, plus `x64` for Windows, and a separate `sdist` job.

### Trusted Publishing (OIDC) Instead of Long-Lived PyPI Tokens
- Use `pypa/gh-action-pypi-publish@release/v1` in a dedicated `publish` job with `environment: name: pypi` and `permissions: id-token: write`. No `PYPI_API_TOKEN` secret is needed.

### Clippy + Ruff Enforcement & CI Traps
- Run `cargo clippy --all-targets -- -D warnings` and `ruff check .` in CI so Rust warnings and Python lint issues never drift.
- Use `Swatinem/rust-cache@v2` in GitHub Actions to avoid recompiling `pyo3` and `syn` from scratch on every PR.
- **Do not pass `--locked` to `cargo test` if `Cargo.lock` is in `.gitignore`**: Cargo will fail with `cannot create the lock file Cargo.lock` in CI.
- **Avoid `maturin develop` on bare `actions/setup-python` runners**: `maturin develop` requires an active virtualenv (`VIRTUAL_ENV` or `.venv`), which `setup-python` does not create. Use `maturin build --release --out dist` followed by `pip install --force-reinstall --no-deps dist/*.whl` for cross-platform CI (Linux/macOS/Windows).

---

## 4. Reusable Boilerplate Checklist

- [ ] `Cargo.toml`: `crate-type = ["cdylib"]`, `pyo3` with `abi3-py39` (no `extension-module`), `[profile.release]` with `lto = "thin"`, `codegen-units = 1`.
- [ ] `pyproject.toml`: `maturin>=1.7,<2.0` build backend, `features = ["pyo3/extension-module"]`, `python-source = "python"`, `module-name = "<pkg>._<pkg>"`, `ruff` + `pytest` config.
- [ ] `python/<pkg>/__init__.py`, `__init__.pyi`, `py.typed`.
- [ ] `src/lib.rs`: `#[pyclass(module = "<pkg>._<pkg>")]`, `#[pyo3(signature = (...))]` for optional args, single-call + batch `PyList::new` methods.
- [ ] Dual license files (`LICENSE-MIT`, `LICENSE-APACHE`) + `.gitignore` (Rust + Python).
- [ ] `.github/workflows/ci.yml` (`cargo test`, `cargo clippy`, `pytest`, `ruff`) & `release.yml` (`PyO3/maturin-action` + Trusted Publishing).
- [ ] `tests/` + `benchmarks/` verifying correctness, determinism, and FFI speedup.
