mod data;
mod providers;
mod unique;

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use fastrand::Rng;
use pyo3::prelude::*;
use pyo3::types::PyList;

use crate::unique::{SeenMap, UniqueProxy, UniquenessException};

static GLOBAL_SEED: AtomicU64 = AtomicU64::new(0);
static SEED_EPOCH: AtomicU64 = AtomicU64::new(0);

pub(crate) struct RngState {
    pub(crate) rng: Rng,
    epoch: u64,
    instance_seeded: bool,
}

impl RngState {
    fn new(seed: Option<u64>) -> Self {
        match seed {
            Some(s) => Self {
                rng: Rng::with_seed(s),
                epoch: SEED_EPOCH.load(Ordering::Relaxed),
                instance_seeded: true,
            },
            None => {
                let epoch = SEED_EPOCH.load(Ordering::Acquire);
                let rng = if epoch > 0 {
                    Rng::with_seed(GLOBAL_SEED.load(Ordering::Relaxed))
                } else {
                    Rng::new()
                };
                Self {
                    rng,
                    epoch,
                    instance_seeded: false,
                }
            }
        }
    }

    #[inline]
    pub(crate) fn sync_global_seed(&mut self) {
        if !self.instance_seeded {
            let epoch = SEED_EPOCH.load(Ordering::Acquire);
            if epoch != self.epoch {
                self.rng.seed(GLOBAL_SEED.load(Ordering::Relaxed));
                self.epoch = epoch;
            }
        }
    }
}

/// High-performance drop-in replacement for Python's `faker.Faker`.
#[pyclass(module = "faker_rs._faker_rs")]
pub struct Faker {
    state: Arc<Mutex<RngState>>,
    seen: Arc<Mutex<SeenMap>>,
}

impl Faker {
    #[inline]
    fn with_rng<T>(&self, f: impl FnOnce(&mut Rng) -> T) -> T {
        let mut guard = self.state.lock().expect("faker-rs RNG mutex poisoned");
        guard.sync_global_seed();
        f(&mut guard.rng)
    }

    #[inline]
    fn batch_values<'py, T>(
        &self,
        py: Python<'py>,
        n: usize,
        mut gen: impl FnMut(&mut Rng) -> T,
    ) -> PyResult<Bound<'py, PyList>>
    where
        T: for<'a> IntoPyObject<'a>,
    {
        let mut guard = self.state.lock().expect("faker-rs RNG mutex poisoned");
        guard.sync_global_seed();
        let rng = &mut guard.rng;
        PyList::new(py, (0..n).map(|_| gen(rng)))
    }
}

#[pymethods]
impl Faker {
    /// Create a new `Faker` instance.
    ///
    /// Accepts optional `locale` (for drop-in compatibility) and `seed`.
    #[new]
    #[pyo3(signature = (locale=None, seed=None))]
    fn new(locale: Option<&str>, seed: Option<u64>) -> Self {
        let _ = locale;
        Self {
            state: Arc::new(Mutex::new(RngState::new(seed))),
            seen: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Seed all unseeded `Faker` instances globally for deterministic output.
    #[staticmethod]
    fn seed(seed: u64) {
        GLOBAL_SEED.store(seed, Ordering::Relaxed);
        SEED_EPOCH.fetch_add(1, Ordering::Release);
    }

    /// Seed this specific `Faker` instance independently of the global seed.
    fn seed_instance(&self, seed: u64) {
        let mut guard = self.state.lock().expect("faker-rs RNG mutex poisoned");
        guard.rng.seed(seed);
        guard.instance_seeded = true;
    }

    /// Return a proxy that guarantees unique values per provider.
    #[getter]
    fn unique(&self) -> UniqueProxy {
        UniqueProxy {
            state: Arc::clone(&self.state),
            seen: Arc::clone(&self.seen),
        }
    }

    // --- Single-record providers ---

    fn name(&self) -> String {
        self.with_rng(providers::gen_name)
    }

    fn first_name(&self) -> String {
        self.with_rng(providers::gen_first_name)
    }

    fn last_name(&self) -> String {
        self.with_rng(providers::gen_last_name)
    }

    fn user_name(&self) -> String {
        self.with_rng(providers::gen_user_name)
    }

    fn email(&self) -> String {
        self.with_rng(providers::gen_email)
    }

    fn address(&self) -> String {
        self.with_rng(providers::gen_address)
    }

    fn city(&self) -> String {
        self.with_rng(providers::gen_city)
    }

    fn state(&self) -> String {
        self.with_rng(providers::gen_state)
    }

    fn postcode(&self) -> String {
        self.with_rng(providers::gen_postcode)
    }

    fn country(&self) -> String {
        self.with_rng(providers::gen_country)
    }

    fn company(&self) -> String {
        self.with_rng(providers::gen_company)
    }

    fn job(&self) -> String {
        self.with_rng(providers::gen_job)
    }

    fn phone_number(&self) -> String {
        self.with_rng(providers::gen_phone_number)
    }

    fn url(&self) -> String {
        self.with_rng(providers::gen_url)
    }

    fn ipv4(&self) -> String {
        self.with_rng(providers::gen_ipv4)
    }

    fn date(&self) -> String {
        self.with_rng(providers::gen_date)
    }

    fn iso8601(&self) -> String {
        self.with_rng(providers::gen_iso8601)
    }

    fn uuid4(&self) -> String {
        self.with_rng(providers::gen_uuid4)
    }

    fn sentence(&self) -> String {
        self.with_rng(providers::gen_sentence)
    }

    #[pyo3(signature = (max_nb_chars=200))]
    fn text(&self, max_nb_chars: usize) -> String {
        self.with_rng(|rng| providers::gen_text(rng, max_nb_chars))
    }

    #[pyo3(signature = (chance_of_getting_true=50))]
    fn boolean(&self, chance_of_getting_true: u8) -> bool {
        self.with_rng(|rng| providers::gen_boolean(rng, chance_of_getting_true))
    }

    #[pyo3(signature = (min=0, max=9999))]
    fn random_int(&self, min: i64, max: i64) -> i64 {
        self.with_rng(|rng| providers::gen_random_int(rng, min, max))
    }

    // --- Batch providers (single Rust loop, minimal FFI overhead) ---

    fn names<'py>(&self, py: Python<'py>, n: usize) -> PyResult<Bound<'py, PyList>> {
        self.batch_values(py, n, providers::gen_name)
    }

    fn first_names<'py>(&self, py: Python<'py>, n: usize) -> PyResult<Bound<'py, PyList>> {
        self.batch_values(py, n, providers::gen_first_name)
    }

    fn last_names<'py>(&self, py: Python<'py>, n: usize) -> PyResult<Bound<'py, PyList>> {
        self.batch_values(py, n, providers::gen_last_name)
    }

    fn user_names<'py>(&self, py: Python<'py>, n: usize) -> PyResult<Bound<'py, PyList>> {
        self.batch_values(py, n, providers::gen_user_name)
    }

    fn emails<'py>(&self, py: Python<'py>, n: usize) -> PyResult<Bound<'py, PyList>> {
        self.batch_values(py, n, providers::gen_email)
    }

    fn addresses<'py>(&self, py: Python<'py>, n: usize) -> PyResult<Bound<'py, PyList>> {
        self.batch_values(py, n, providers::gen_address)
    }

    fn cities<'py>(&self, py: Python<'py>, n: usize) -> PyResult<Bound<'py, PyList>> {
        self.batch_values(py, n, providers::gen_city)
    }

    fn states<'py>(&self, py: Python<'py>, n: usize) -> PyResult<Bound<'py, PyList>> {
        self.batch_values(py, n, providers::gen_state)
    }

    fn postcodes<'py>(&self, py: Python<'py>, n: usize) -> PyResult<Bound<'py, PyList>> {
        self.batch_values(py, n, providers::gen_postcode)
    }

    fn countries<'py>(&self, py: Python<'py>, n: usize) -> PyResult<Bound<'py, PyList>> {
        self.batch_values(py, n, providers::gen_country)
    }

    fn companies<'py>(&self, py: Python<'py>, n: usize) -> PyResult<Bound<'py, PyList>> {
        self.batch_values(py, n, providers::gen_company)
    }

    fn jobs<'py>(&self, py: Python<'py>, n: usize) -> PyResult<Bound<'py, PyList>> {
        self.batch_values(py, n, providers::gen_job)
    }

    fn phone_numbers<'py>(&self, py: Python<'py>, n: usize) -> PyResult<Bound<'py, PyList>> {
        self.batch_values(py, n, providers::gen_phone_number)
    }

    fn urls<'py>(&self, py: Python<'py>, n: usize) -> PyResult<Bound<'py, PyList>> {
        self.batch_values(py, n, providers::gen_url)
    }

    fn ipv4s<'py>(&self, py: Python<'py>, n: usize) -> PyResult<Bound<'py, PyList>> {
        self.batch_values(py, n, providers::gen_ipv4)
    }

    fn dates<'py>(&self, py: Python<'py>, n: usize) -> PyResult<Bound<'py, PyList>> {
        self.batch_values(py, n, providers::gen_date)
    }

    fn iso8601s<'py>(&self, py: Python<'py>, n: usize) -> PyResult<Bound<'py, PyList>> {
        self.batch_values(py, n, providers::gen_iso8601)
    }

    fn uuid4s<'py>(&self, py: Python<'py>, n: usize) -> PyResult<Bound<'py, PyList>> {
        self.batch_values(py, n, providers::gen_uuid4)
    }

    fn sentences<'py>(&self, py: Python<'py>, n: usize) -> PyResult<Bound<'py, PyList>> {
        self.batch_values(py, n, providers::gen_sentence)
    }

    #[pyo3(signature = (n, max_nb_chars=200))]
    fn texts<'py>(
        &self,
        py: Python<'py>,
        n: usize,
        max_nb_chars: usize,
    ) -> PyResult<Bound<'py, PyList>> {
        self.batch_values(py, n, |rng| providers::gen_text(rng, max_nb_chars))
    }

    #[pyo3(signature = (n, chance_of_getting_true=50))]
    fn booleans<'py>(
        &self,
        py: Python<'py>,
        n: usize,
        chance_of_getting_true: u8,
    ) -> PyResult<Bound<'py, PyList>> {
        self.batch_values(py, n, |rng| providers::gen_boolean(rng, chance_of_getting_true))
    }

    #[pyo3(signature = (n, min=0, max=9999))]
    fn random_ints<'py>(
        &self,
        py: Python<'py>,
        n: usize,
        min: i64,
        max: i64,
    ) -> PyResult<Bound<'py, PyList>> {
        self.batch_values(py, n, |rng| providers::gen_random_int(rng, min, max))
    }
}

#[pymodule]
fn _faker_rs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Faker>()?;
    m.add_class::<UniqueProxy>()?;
    m.add("UniquenessException", m.py().get_type::<UniquenessException>())?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
