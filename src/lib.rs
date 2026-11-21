mod data;
mod providers;
mod unique;

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use fastrand::Rng;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyDate, PyDateTime, PyList, PyTuple};

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

    #[pyo3(signature = (pattern="%Y-%m-%d"))]
    fn date(&self, pattern: &str) -> String {
        self.with_rng(|rng| providers::gen_date_pattern(rng, pattern))
    }

    fn date_object<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDate>> {
        let (y, m, d) = self.with_rng(providers::gen_ymd);
        PyDate::new(py, y, m, d)
    }

    fn date_time<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDateTime>> {
        let (y, m, d, hh, mm, ss) = self.with_rng(providers::gen_ymdhms);
        PyDateTime::new(py, y, m, d, hh, mm, ss, 0, None)
    }

    #[pyo3(signature = (start_date="-30y", end_date="today"))]
    fn date_between<'py>(
        &self,
        py: Python<'py>,
        start_date: &str,
        end_date: &str,
    ) -> PyResult<Bound<'py, PyDate>> {
        let today = providers::current_utc_days();
        let s = providers::parse_date_spec(start_date, today);
        let e = providers::parse_date_spec(end_date, today);
        let (y, m, d) = self.with_rng(|rng| providers::gen_date_between_days(rng, s, e));
        PyDate::new(py, y, m, d)
    }

    #[pyo3(signature = (minimum_age=0, maximum_age=115))]
    fn date_of_birth<'py>(
        &self,
        py: Python<'py>,
        minimum_age: u32,
        maximum_age: u32,
    ) -> PyResult<Bound<'py, PyDate>> {
        let today = providers::current_utc_days();
        let (y, m, d) =
            self.with_rng(|rng| providers::gen_date_of_birth(rng, today, minimum_age, maximum_age));
        PyDate::new(py, y, m, d)
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

    #[pyo3(signature = (text="###"))]
    fn numerify(&self, text: &str) -> String {
        self.with_rng(|rng| providers::gen_numerify(rng, text))
    }

    #[pyo3(signature = (text="????"))]
    fn lexify(&self, text: &str) -> String {
        self.with_rng(|rng| providers::gen_lexify(rng, text))
    }

    #[pyo3(signature = (text="##??"))]
    fn bothify(&self, text: &str) -> String {
        self.with_rng(|rng| providers::gen_bothify(rng, text))
    }

    #[pyo3(signature = (min_value=0, max_value=9999))]
    fn pyint(&self, min_value: i64, max_value: i64) -> i64 {
        self.with_rng(|rng| providers::gen_random_int(rng, min_value, max_value))
    }

    #[pyo3(signature = (left_digits=None, right_digits=None, positive=false, min_value=None, max_value=None))]
    fn pyfloat(
        &self,
        left_digits: Option<u32>,
        right_digits: Option<u32>,
        positive: bool,
        min_value: Option<f64>,
        max_value: Option<f64>,
    ) -> f64 {
        self.with_rng(|rng| {
            providers::gen_pyfloat(rng, left_digits, right_digits, positive, min_value, max_value)
        })
    }

    #[pyo3(signature = (truth_probability=50))]
    fn pybool(&self, truth_probability: u8) -> bool {
        self.with_rng(|rng| providers::gen_boolean(rng, truth_probability))
    }

    #[pyo3(signature = (min_chars=None, max_chars=20))]
    fn pystr(&self, min_chars: Option<usize>, max_chars: usize) -> String {
        self.with_rng(|rng| providers::gen_pystr(rng, min_chars, max_chars))
    }

    #[pyo3(signature = (length=10, special_chars=true, digits=true, upper_case=true, lower_case=true))]
    fn password(
        &self,
        length: usize,
        special_chars: bool,
        digits: bool,
        upper_case: bool,
        lower_case: bool,
    ) -> String {
        self.with_rng(|rng| {
            providers::gen_password(rng, length, special_chars, digits, upper_case, lower_case)
        })
    }

    #[pyo3(signature = (elements=None))]
    fn random_element<'py>(
        &self,
        py: Python<'py>,
        elements: Option<Bound<'py, PyAny>>,
    ) -> PyResult<Py<PyAny>> {
        let items: Vec<Py<PyAny>> = match elements {
            Some(obj) => obj
                .try_iter()?
                .map(|item| item.map(|b| b.unbind()))
                .collect::<PyResult<Vec<_>>>()?,
            None => PyTuple::new(py, ["a", "b", "c"])?
                .iter()
                .map(|b| b.unbind())
                .collect(),
        };
        if items.is_empty() {
            return Err(PyValueError::new_err("Cannot choose from an empty sequence"));
        }
        let idx = self.with_rng(|rng| rng.usize(..items.len()));
        Ok(items[idx].clone_ref(py))
    }

    #[pyo3(signature = (elements=None, length=None, unique=false))]
    fn random_elements<'py>(
        &self,
        py: Python<'py>,
        elements: Option<Bound<'py, PyAny>>,
        length: Option<usize>,
        unique: bool,
    ) -> PyResult<Bound<'py, PyList>> {
        let mut items: Vec<Py<PyAny>> = match elements {
            Some(obj) => obj
                .try_iter()?
                .map(|item| item.map(|b| b.unbind()))
                .collect::<PyResult<Vec<_>>>()?,
            None => PyTuple::new(py, ["a", "b", "c"])?
                .iter()
                .map(|b| b.unbind())
                .collect(),
        };
        if items.is_empty() {
            return Ok(PyList::empty(py));
        }
        let target_len = length.unwrap_or_else(|| self.with_rng(|rng| rng.usize(1..=items.len())));
        if unique && target_len > items.len() {
            return Err(PyValueError::new_err(
                "Sample larger than population when unique=True",
            ));
        }

        let mut chosen = Vec::with_capacity(target_len);
        self.with_rng(|rng| {
            if unique {
                for i in 0..target_len {
                    let j = rng.usize(i..items.len());
                    items.swap(i, j);
                    chosen.push(items[i].clone_ref(py));
                }
            } else {
                for _ in 0..target_len {
                    let idx = rng.usize(..items.len());
                    chosen.push(items[idx].clone_ref(py));
                }
            }
        });
        PyList::new(py, chosen)
    }

    fn word(&self) -> String {
        self.with_rng(providers::gen_word)
    }

    #[pyo3(signature = (nb=3))]
    fn words(&self, nb: usize) -> Vec<String> {
        self.with_rng(|rng| providers::gen_words(rng, nb))
    }

    #[pyo3(signature = (nb_sentences=3))]
    fn paragraph(&self, nb_sentences: usize) -> String {
        self.with_rng(|rng| providers::gen_paragraph(rng, nb_sentences))
    }

    #[pyo3(signature = (nb=3))]
    fn paragraphs(&self, nb: usize) -> Vec<String> {
        self.with_rng(|rng| providers::gen_paragraphs(rng, nb))
    }

    #[pyo3(signature = (card_type=None))]
    fn credit_card_number(&self, card_type: Option<&str>) -> String {
        self.with_rng(|rng| providers::gen_credit_card_number(rng, card_type))
    }

    fn credit_card_expire(&self) -> String {
        self.with_rng(providers::gen_credit_card_expire)
    }

    #[pyo3(signature = (card_type=None))]
    fn credit_card_security_code(&self, card_type: Option<&str>) -> String {
        self.with_rng(|rng| providers::gen_credit_card_security_code(rng, card_type))
    }

    fn credit_card_provider(&self) -> String {
        self.with_rng(providers::gen_credit_card_provider)
    }

    fn iban(&self) -> String {
        self.with_rng(providers::gen_iban)
    }

    fn currency_code(&self) -> String {
        self.with_rng(providers::gen_currency_code)
    }

    fn ipv6(&self) -> String {
        self.with_rng(providers::gen_ipv6)
    }

    fn mac_address(&self) -> String {
        self.with_rng(providers::gen_mac_address)
    }

    fn domain_name(&self) -> String {
        self.with_rng(providers::gen_domain_name)
    }

    fn user_agent(&self) -> String {
        self.with_rng(providers::gen_user_agent)
    }

    fn slug(&self) -> String {
        self.with_rng(providers::gen_slug)
    }

    fn free_email(&self) -> String {
        self.with_rng(providers::gen_free_email)
    }

    fn company_email(&self) -> String {
        self.with_rng(providers::gen_company_email)
    }

    fn safe_email(&self) -> String {
        self.with_rng(providers::gen_safe_email)
    }

    fn hex_color(&self) -> String {
        self.with_rng(providers::gen_hex_color)
    }

    fn rgb_color(&self) -> String {
        self.with_rng(providers::gen_rgb_color)
    }

    fn latitude(&self) -> f64 {
        self.with_rng(providers::gen_latitude)
    }

    fn longitude(&self) -> f64 {
        self.with_rng(providers::gen_longitude)
    }

    #[pyo3(signature = (category=None, extension=None))]
    fn file_name(&self, category: Option<&str>, extension: Option<&str>) -> String {
        let _ = category;
        self.with_rng(|rng| providers::gen_file_name(rng, extension))
    }

    fn mime_type(&self) -> String {
        self.with_rng(providers::gen_mime_type)
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

    #[pyo3(signature = (n, pattern="%Y-%m-%d"))]
    fn dates<'py>(
        &self,
        py: Python<'py>,
        n: usize,
        pattern: &str,
    ) -> PyResult<Bound<'py, PyList>> {
        self.batch_values(py, n, |rng| providers::gen_date_pattern(rng, pattern))
    }

    fn date_objects<'py>(&self, py: Python<'py>, n: usize) -> PyResult<Bound<'py, PyList>> {
        let mut guard = self.state.lock().expect("faker-rs RNG mutex poisoned");
        guard.sync_global_seed();
        let rng = &mut guard.rng;
        let items = (0..n)
            .map(|_| {
                let (y, m, d) = providers::gen_ymd(rng);
                PyDate::new(py, y, m, d)
            })
            .collect::<PyResult<Vec<_>>>()?;
        PyList::new(py, items)
    }

    fn date_times<'py>(&self, py: Python<'py>, n: usize) -> PyResult<Bound<'py, PyList>> {
        let mut guard = self.state.lock().expect("faker-rs RNG mutex poisoned");
        guard.sync_global_seed();
        let rng = &mut guard.rng;
        let items = (0..n)
            .map(|_| {
                let (y, m, d, hh, mm, ss) = providers::gen_ymdhms(rng);
                PyDateTime::new(py, y, m, d, hh, mm, ss, 0, None)
            })
            .collect::<PyResult<Vec<_>>>()?;
        PyList::new(py, items)
    }

    #[pyo3(signature = (n, start_date="-30y", end_date="today"))]
    fn date_betweens<'py>(
        &self,
        py: Python<'py>,
        n: usize,
        start_date: &str,
        end_date: &str,
    ) -> PyResult<Bound<'py, PyList>> {
        let today = providers::current_utc_days();
        let s = providers::parse_date_spec(start_date, today);
        let e = providers::parse_date_spec(end_date, today);
        let mut guard = self.state.lock().expect("faker-rs RNG mutex poisoned");
        guard.sync_global_seed();
        let rng = &mut guard.rng;
        let items = (0..n)
            .map(|_| {
                let (y, m, d) = providers::gen_date_between_days(rng, s, e);
                PyDate::new(py, y, m, d)
            })
            .collect::<PyResult<Vec<_>>>()?;
        PyList::new(py, items)
    }

    #[pyo3(signature = (n, minimum_age=0, maximum_age=115))]
    fn date_of_births<'py>(
        &self,
        py: Python<'py>,
        n: usize,
        minimum_age: u32,
        maximum_age: u32,
    ) -> PyResult<Bound<'py, PyList>> {
        let today = providers::current_utc_days();
        let mut guard = self.state.lock().expect("faker-rs RNG mutex poisoned");
        guard.sync_global_seed();
        let rng = &mut guard.rng;
        let items = (0..n)
            .map(|_| {
                let (y, m, d) = providers::gen_date_of_birth(rng, today, minimum_age, maximum_age);
                PyDate::new(py, y, m, d)
            })
            .collect::<PyResult<Vec<_>>>()?;
        PyList::new(py, items)
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

    #[pyo3(signature = (n, text="###"))]
    fn numerifys<'py>(
        &self,
        py: Python<'py>,
        n: usize,
        text: &str,
    ) -> PyResult<Bound<'py, PyList>> {
        self.batch_values(py, n, |rng| providers::gen_numerify(rng, text))
    }

    #[pyo3(signature = (n, text="????"))]
    fn lexifys<'py>(
        &self,
        py: Python<'py>,
        n: usize,
        text: &str,
    ) -> PyResult<Bound<'py, PyList>> {
        self.batch_values(py, n, |rng| providers::gen_lexify(rng, text))
    }

    #[pyo3(signature = (n, text="##??"))]
    fn bothifys<'py>(
        &self,
        py: Python<'py>,
        n: usize,
        text: &str,
    ) -> PyResult<Bound<'py, PyList>> {
        self.batch_values(py, n, |rng| providers::gen_bothify(rng, text))
    }

    #[pyo3(signature = (n, min_value=0, max_value=9999))]
    fn pyints<'py>(
        &self,
        py: Python<'py>,
        n: usize,
        min_value: i64,
        max_value: i64,
    ) -> PyResult<Bound<'py, PyList>> {
        self.batch_values(py, n, |rng| providers::gen_random_int(rng, min_value, max_value))
    }

    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (n, left_digits=None, right_digits=None, positive=false, min_value=None, max_value=None))]
    fn pyfloats<'py>(
        &self,
        py: Python<'py>,
        n: usize,
        left_digits: Option<u32>,
        right_digits: Option<u32>,
        positive: bool,
        min_value: Option<f64>,
        max_value: Option<f64>,
    ) -> PyResult<Bound<'py, PyList>> {
        self.batch_values(py, n, |rng| {
            providers::gen_pyfloat(rng, left_digits, right_digits, positive, min_value, max_value)
        })
    }

    #[pyo3(signature = (n, truth_probability=50))]
    fn pybools<'py>(
        &self,
        py: Python<'py>,
        n: usize,
        truth_probability: u8,
    ) -> PyResult<Bound<'py, PyList>> {
        self.batch_values(py, n, |rng| providers::gen_boolean(rng, truth_probability))
    }

    #[pyo3(signature = (n, min_chars=None, max_chars=20))]
    fn pystrs<'py>(
        &self,
        py: Python<'py>,
        n: usize,
        min_chars: Option<usize>,
        max_chars: usize,
    ) -> PyResult<Bound<'py, PyList>> {
        self.batch_values(py, n, |rng| providers::gen_pystr(rng, min_chars, max_chars))
    }

    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (n, length=10, special_chars=true, digits=true, upper_case=true, lower_case=true))]
    fn passwords<'py>(
        &self,
        py: Python<'py>,
        n: usize,
        length: usize,
        special_chars: bool,
        digits: bool,
        upper_case: bool,
        lower_case: bool,
    ) -> PyResult<Bound<'py, PyList>> {
        self.batch_values(py, n, |rng| {
            providers::gen_password(rng, length, special_chars, digits, upper_case, lower_case)
        })
    }

    #[pyo3(signature = (n, nb=3))]
    fn words_batch<'py>(
        &self,
        py: Python<'py>,
        n: usize,
        nb: usize,
    ) -> PyResult<Bound<'py, PyList>> {
        self.batch_values(py, n, |rng| providers::gen_words(rng, nb))
    }

    #[pyo3(signature = (n, nb=3))]
    fn paragraphs_batch<'py>(
        &self,
        py: Python<'py>,
        n: usize,
        nb: usize,
    ) -> PyResult<Bound<'py, PyList>> {
        self.batch_values(py, n, |rng| providers::gen_paragraphs(rng, nb))
    }

    #[pyo3(signature = (n, card_type=None))]
    fn credit_card_numbers<'py>(
        &self,
        py: Python<'py>,
        n: usize,
        card_type: Option<&str>,
    ) -> PyResult<Bound<'py, PyList>> {
        self.batch_values(py, n, |rng| providers::gen_credit_card_number(rng, card_type))
    }

    fn credit_card_expires<'py>(&self, py: Python<'py>, n: usize) -> PyResult<Bound<'py, PyList>> {
        self.batch_values(py, n, providers::gen_credit_card_expire)
    }

    #[pyo3(signature = (n, card_type=None))]
    fn credit_card_security_codes<'py>(
        &self,
        py: Python<'py>,
        n: usize,
        card_type: Option<&str>,
    ) -> PyResult<Bound<'py, PyList>> {
        self.batch_values(py, n, |rng| {
            providers::gen_credit_card_security_code(rng, card_type)
        })
    }

    fn credit_card_providers<'py>(
        &self,
        py: Python<'py>,
        n: usize,
    ) -> PyResult<Bound<'py, PyList>> {
        self.batch_values(py, n, providers::gen_credit_card_provider)
    }

    fn ibans<'py>(&self, py: Python<'py>, n: usize) -> PyResult<Bound<'py, PyList>> {
        self.batch_values(py, n, providers::gen_iban)
    }

    fn currency_codes<'py>(&self, py: Python<'py>, n: usize) -> PyResult<Bound<'py, PyList>> {
        self.batch_values(py, n, providers::gen_currency_code)
    }

    fn ipv6s<'py>(&self, py: Python<'py>, n: usize) -> PyResult<Bound<'py, PyList>> {
        self.batch_values(py, n, providers::gen_ipv6)
    }

    fn mac_addresses<'py>(&self, py: Python<'py>, n: usize) -> PyResult<Bound<'py, PyList>> {
        self.batch_values(py, n, providers::gen_mac_address)
    }

    fn domain_names<'py>(&self, py: Python<'py>, n: usize) -> PyResult<Bound<'py, PyList>> {
        self.batch_values(py, n, providers::gen_domain_name)
    }

    fn user_agents<'py>(&self, py: Python<'py>, n: usize) -> PyResult<Bound<'py, PyList>> {
        self.batch_values(py, n, providers::gen_user_agent)
    }

    fn slugs<'py>(&self, py: Python<'py>, n: usize) -> PyResult<Bound<'py, PyList>> {
        self.batch_values(py, n, providers::gen_slug)
    }

    fn free_emails<'py>(&self, py: Python<'py>, n: usize) -> PyResult<Bound<'py, PyList>> {
        self.batch_values(py, n, providers::gen_free_email)
    }

    fn company_emails<'py>(&self, py: Python<'py>, n: usize) -> PyResult<Bound<'py, PyList>> {
        self.batch_values(py, n, providers::gen_company_email)
    }

    fn safe_emails<'py>(&self, py: Python<'py>, n: usize) -> PyResult<Bound<'py, PyList>> {
        self.batch_values(py, n, providers::gen_safe_email)
    }

    fn hex_colors<'py>(&self, py: Python<'py>, n: usize) -> PyResult<Bound<'py, PyList>> {
        self.batch_values(py, n, providers::gen_hex_color)
    }

    fn rgb_colors<'py>(&self, py: Python<'py>, n: usize) -> PyResult<Bound<'py, PyList>> {
        self.batch_values(py, n, providers::gen_rgb_color)
    }

    fn latitudes<'py>(&self, py: Python<'py>, n: usize) -> PyResult<Bound<'py, PyList>> {
        self.batch_values(py, n, providers::gen_latitude)
    }

    fn longitudes<'py>(&self, py: Python<'py>, n: usize) -> PyResult<Bound<'py, PyList>> {
        self.batch_values(py, n, providers::gen_longitude)
    }

    #[pyo3(signature = (n, category=None, extension=None))]
    fn file_names<'py>(
        &self,
        py: Python<'py>,
        n: usize,
        category: Option<&str>,
        extension: Option<&str>,
    ) -> PyResult<Bound<'py, PyList>> {
        let _ = category;
        self.batch_values(py, n, |rng| providers::gen_file_name(rng, extension))
    }

    fn mime_types<'py>(&self, py: Python<'py>, n: usize) -> PyResult<Bound<'py, PyList>> {
        self.batch_values(py, n, providers::gen_mime_type)
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
