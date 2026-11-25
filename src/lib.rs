mod data;
mod locale;
mod optional;
mod providers;
mod unique;

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use fastrand::Rng;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyDate, PyDateTime, PyDict, PyList, PyTuple};

use crate::locale::{parse_locales, pick_locale, LocaleData};
use crate::optional::OptionalProxy;
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
#[pyclass(module = "faker_rs._faker_rs", subclass)]
pub struct Faker {
    state: Arc<Mutex<RngState>>,
    seen: Arc<Mutex<SeenMap>>,
    locales: Vec<&'static LocaleData>,
}

impl Faker {
    #[inline]
    fn with_rng<T>(&self, f: impl FnOnce(&mut Rng) -> T) -> T {
        let mut guard = self.state.lock().expect("faker-rs RNG mutex poisoned");
        guard.sync_global_seed();
        f(&mut guard.rng)
    }

    #[inline]
    fn with_rng_loc<T>(&self, f: impl FnOnce(&mut Rng, &'static LocaleData) -> T) -> T {
        let mut guard = self.state.lock().expect("faker-rs RNG mutex poisoned");
        guard.sync_global_seed();
        let loc = pick_locale(&mut guard.rng, &self.locales);
        f(&mut guard.rng, loc)
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

    #[inline]
    fn batch_values_loc<'py, T>(
        &self,
        py: Python<'py>,
        n: usize,
        mut gen: impl FnMut(&mut Rng, &'static LocaleData) -> T,
    ) -> PyResult<Bound<'py, PyList>>
    where
        T: for<'a> IntoPyObject<'a>,
    {
        let mut guard = self.state.lock().expect("faker-rs RNG mutex poisoned");
        guard.sync_global_seed();
        let rng = &mut guard.rng;
        let locales = &self.locales;
        PyList::new(
            py,
            (0..n).map(|_| {
                let loc = pick_locale(rng, locales);
                gen(rng, loc)
            }),
        )
    }
}

#[pymethods]
impl Faker {
    /// Create a new `Faker` instance.
    ///
    /// Accepts optional `locale` (`str` or `list[str]`) and `seed`.
    #[new]
    #[pyo3(signature = (locale=None, seed=None))]
    fn new(locale: Option<Bound<'_, PyAny>>, seed: Option<u64>) -> PyResult<Self> {
        let locales = parse_locales(locale)?;
        Ok(Self {
            state: Arc::new(Mutex::new(RngState::new(seed))),
            seen: Arc::new(Mutex::new(HashMap::new())),
            locales,
        })
    }

    /// Return the configured locale codes for this `Faker` instance.
    #[getter]
    fn locales(&self) -> Vec<&'static str> {
        self.locales.iter().map(|l| l.code).collect()
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
            locales: self.locales.clone(),
        }
    }

    /// Return a proxy that returns `None` with probability `1 - p`.
    #[getter]
    fn optional(&self) -> OptionalProxy {
        OptionalProxy {
            state: Arc::clone(&self.state),
            locales: self.locales.clone(),
        }
    }

    // --- Single-record providers ---

    fn name(&self) -> String {
        self.with_rng_loc(providers::gen_name_loc)
    }

    fn first_name(&self) -> String {
        self.with_rng_loc(providers::gen_first_name_loc)
    }

    fn last_name(&self) -> String {
        self.with_rng_loc(providers::gen_last_name_loc)
    }

    fn user_name(&self) -> String {
        self.with_rng_loc(providers::gen_user_name_loc)
    }

    #[pyo3(signature = (domain=None, name=None))]
    fn email(&self, domain: Option<&str>, name: Option<&str>) -> String {
        self.with_rng_loc(|rng, loc| providers::gen_email_custom(rng, loc, domain, name))
    }

    #[pyo3(signature = (name, domain=None, add_digits=false))]
    fn email_from_name(&self, name: &str, domain: Option<&str>, add_digits: bool) -> String {
        self.with_rng(|rng| providers::gen_email_from_name(rng, name, domain, add_digits))
    }

    fn address(&self) -> String {
        self.with_rng_loc(providers::gen_address_loc)
    }

    fn city(&self) -> String {
        self.with_rng_loc(providers::gen_city_loc)
    }

    fn state(&self) -> String {
        self.with_rng_loc(providers::gen_state_loc)
    }

    fn postcode(&self) -> String {
        self.with_rng_loc(providers::gen_postcode_loc)
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
        self.with_rng_loc(providers::gen_phone_number_loc)
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

    #[pyo3(signature = (start_date=None, end_date=None))]
    fn date_between<'py>(
        &self,
        py: Python<'py>,
        start_date: Option<Bound<'py, PyAny>>,
        end_date: Option<Bound<'py, PyAny>>,
    ) -> PyResult<Bound<'py, PyDate>> {
        let today = providers::current_utc_days();
        let s = extract_date_days(start_date, "-30y", today)?;
        let e = extract_date_days(end_date, "today", today)?;
        let (y, m, d) = self.with_rng(|rng| providers::gen_date_between_days(rng, s, e));
        PyDate::new(py, y, m, d)
    }

    #[pyo3(signature = (start_date=None, end_date=None))]
    fn date_time_between<'py>(
        &self,
        py: Python<'py>,
        start_date: Option<Bound<'py, PyAny>>,
        end_date: Option<Bound<'py, PyAny>>,
    ) -> PyResult<Bound<'py, PyDateTime>> {
        let now = providers::current_utc_secs();
        let s = extract_datetime_secs(start_date, "-30y", now)?;
        let e = extract_datetime_secs(end_date, "now", now)?;
        let (y, m, d, hh, mm, ss) =
            self.with_rng(|rng| providers::gen_datetime_between_secs(rng, s, e));
        PyDateTime::new(py, y, m, d, hh, mm, ss, 0, None)
    }

    #[pyo3(signature = (start_date=None, end_date=None))]
    fn iso8601_between<'py>(
        &self,
        start_date: Option<Bound<'py, PyAny>>,
        end_date: Option<Bound<'py, PyAny>>,
    ) -> PyResult<String> {
        let now = providers::current_utc_secs();
        let s = extract_datetime_secs(start_date, "-30y", now)?;
        let e = extract_datetime_secs(end_date, "now", now)?;
        let (y, m, d, hh, mm, ss) =
            self.with_rng(|rng| providers::gen_datetime_between_secs(rng, s, e));
        Ok(providers::format_iso8601(y, m, d, hh, mm, ss))
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
        let (items, weights) = extract_elements(py, elements)?;
        if items.is_empty() {
            return Err(PyValueError::new_err("Cannot choose from an empty sequence"));
        }
        let idx = match weights {
            Some(w) => {
                let cum = build_cumulative_weights(&w)?;
                self.with_rng(|rng| pick_weighted_idx(rng, &cum))
            }
            None => self.with_rng(|rng| rng.usize(..items.len())),
        };
        Ok(items[idx].clone_ref(py))
    }

    #[pyo3(signature = (elements=None, length=None, unique=false, use_weighting=true))]
    fn random_elements<'py>(
        &self,
        py: Python<'py>,
        elements: Option<Bound<'py, PyAny>>,
        length: Option<usize>,
        unique: bool,
        use_weighting: bool,
    ) -> PyResult<Bound<'py, PyList>> {
        let (mut items, weights) = extract_elements(py, elements)?;
        if items.is_empty() {
            return Ok(PyList::empty(py));
        }
        let target_len = length.unwrap_or_else(|| self.with_rng(|rng| rng.usize(1..=items.len())));
        if unique && target_len > items.len() {
            return Err(PyValueError::new_err(
                "Sample larger than population when unique=True",
            ));
        }

        let active_weights = if use_weighting { weights } else { None };
        let mut chosen = Vec::with_capacity(target_len);

        match active_weights {
            Some(mut w) => {
                if unique {
                    let positive_count = w.iter().filter(|&&x| x > 0.0).count();
                    if target_len > positive_count {
                        return Err(PyValueError::new_err(
                            "Sample larger than number of positive-weight items when unique=True",
                        ));
                    }
                    self.with_rng(|rng| {
                        for _ in 0..target_len {
                            let total: f64 = w.iter().sum();
                            let r = rng.f64() * total;
                            let mut acc = 0.0;
                            let mut picked = w.len() - 1;
                            for (i, &weight) in w.iter().enumerate() {
                                acc += weight;
                                if r < acc && weight > 0.0 {
                                    picked = i;
                                    break;
                                }
                            }
                            chosen.push(items[picked].clone_ref(py));
                            w[picked] = 0.0;
                        }
                    });
                } else {
                    let cum = build_cumulative_weights(&w)?;
                    self.with_rng(|rng| {
                        for _ in 0..target_len {
                            let idx = pick_weighted_idx(rng, &cum);
                            chosen.push(items[idx].clone_ref(py));
                        }
                    });
                }
            }
            None => {
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
            }
        }
        PyList::new(py, chosen)
    }

    #[pyo3(signature = (elements=None, length=None))]
    fn random_sample<'py>(
        &self,
        py: Python<'py>,
        elements: Option<Bound<'py, PyAny>>,
        length: Option<usize>,
    ) -> PyResult<Bound<'py, PyList>> {
        self.random_elements(py, elements, length, true, true)
    }

    #[pyo3(signature = (elements, weights=None, k=1))]
    fn random_choices<'py>(
        &self,
        py: Python<'py>,
        elements: Bound<'py, PyAny>,
        weights: Option<Vec<f64>>,
        k: usize,
    ) -> PyResult<Bound<'py, PyList>> {
        let (items, dict_weights) = extract_elements(py, Some(elements))?;
        if items.is_empty() {
            return Err(PyValueError::new_err("Cannot choose from an empty sequence"));
        }
        let eff_weights = match weights {
            Some(w) => {
                if w.len() != items.len() {
                    return Err(PyValueError::new_err(
                        "The number of weights does not match the population",
                    ));
                }
                Some(w)
            }
            None => dict_weights,
        };
        let mut chosen = Vec::with_capacity(k);
        match eff_weights {
            Some(w) => {
                let cum = build_cumulative_weights(&w)?;
                self.with_rng(|rng| {
                    for _ in 0..k {
                        let idx = pick_weighted_idx(rng, &cum);
                        chosen.push(items[idx].clone_ref(py));
                    }
                });
            }
            None => {
                self.with_rng(|rng| {
                    for _ in 0..k {
                        let idx = rng.usize(..items.len());
                        chosen.push(items[idx].clone_ref(py));
                    }
                });
            }
        }
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

    #[pyo3(signature = (domain=None, name=None))]
    fn company_email(&self, domain: Option<&str>, name: Option<&str>) -> String {
        self.with_rng_loc(|rng, loc| providers::gen_company_email_custom(rng, loc, domain, name))
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
        self.batch_values_loc(py, n, providers::gen_name_loc)
    }

    fn first_names<'py>(&self, py: Python<'py>, n: usize) -> PyResult<Bound<'py, PyList>> {
        self.batch_values_loc(py, n, providers::gen_first_name_loc)
    }

    fn last_names<'py>(&self, py: Python<'py>, n: usize) -> PyResult<Bound<'py, PyList>> {
        self.batch_values_loc(py, n, providers::gen_last_name_loc)
    }

    fn user_names<'py>(&self, py: Python<'py>, n: usize) -> PyResult<Bound<'py, PyList>> {
        self.batch_values_loc(py, n, providers::gen_user_name_loc)
    }

    #[pyo3(signature = (n, domain=None))]
    fn emails<'py>(
        &self,
        py: Python<'py>,
        n: usize,
        domain: Option<&str>,
    ) -> PyResult<Bound<'py, PyList>> {
        self.batch_values_loc(py, n, |rng, loc| {
            providers::gen_email_custom(rng, loc, domain, None)
        })
    }

    fn addresses<'py>(&self, py: Python<'py>, n: usize) -> PyResult<Bound<'py, PyList>> {
        self.batch_values_loc(py, n, providers::gen_address_loc)
    }

    fn cities<'py>(&self, py: Python<'py>, n: usize) -> PyResult<Bound<'py, PyList>> {
        self.batch_values_loc(py, n, providers::gen_city_loc)
    }

    fn states<'py>(&self, py: Python<'py>, n: usize) -> PyResult<Bound<'py, PyList>> {
        self.batch_values_loc(py, n, providers::gen_state_loc)
    }

    fn postcodes<'py>(&self, py: Python<'py>, n: usize) -> PyResult<Bound<'py, PyList>> {
        self.batch_values_loc(py, n, providers::gen_postcode_loc)
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
        self.batch_values_loc(py, n, providers::gen_phone_number_loc)
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

    #[pyo3(signature = (n, start_date=None, end_date=None))]
    fn date_betweens<'py>(
        &self,
        py: Python<'py>,
        n: usize,
        start_date: Option<Bound<'py, PyAny>>,
        end_date: Option<Bound<'py, PyAny>>,
    ) -> PyResult<Bound<'py, PyList>> {
        let today = providers::current_utc_days();
        let s = extract_date_days(start_date, "-30y", today)?;
        let e = extract_date_days(end_date, "today", today)?;
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

    #[pyo3(signature = (n, start_date=None, end_date=None))]
    fn date_time_betweens<'py>(
        &self,
        py: Python<'py>,
        n: usize,
        start_date: Option<Bound<'py, PyAny>>,
        end_date: Option<Bound<'py, PyAny>>,
    ) -> PyResult<Bound<'py, PyList>> {
        let now = providers::current_utc_secs();
        let s = extract_datetime_secs(start_date, "-30y", now)?;
        let e = extract_datetime_secs(end_date, "now", now)?;
        let mut guard = self.state.lock().expect("faker-rs RNG mutex poisoned");
        guard.sync_global_seed();
        let rng = &mut guard.rng;
        let items = (0..n)
            .map(|_| {
                let (y, m, d, hh, mm, ss) = providers::gen_datetime_between_secs(rng, s, e);
                PyDateTime::new(py, y, m, d, hh, mm, ss, 0, None)
            })
            .collect::<PyResult<Vec<_>>>()?;
        PyList::new(py, items)
    }

    #[pyo3(signature = (n, start_date=None, end_date=None))]
    fn iso8601_betweens<'py>(
        &self,
        py: Python<'py>,
        n: usize,
        start_date: Option<Bound<'py, PyAny>>,
        end_date: Option<Bound<'py, PyAny>>,
    ) -> PyResult<Bound<'py, PyList>> {
        let now = providers::current_utc_secs();
        let s = extract_datetime_secs(start_date, "-30y", now)?;
        let e = extract_datetime_secs(end_date, "now", now)?;
        self.batch_values(py, n, |rng| {
            let (y, m, d, hh, mm, ss) = providers::gen_datetime_between_secs(rng, s, e);
            providers::format_iso8601(y, m, d, hh, mm, ss)
        })
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

    #[pyo3(signature = (n, domain=None))]
    fn company_emails<'py>(
        &self,
        py: Python<'py>,
        n: usize,
        domain: Option<&str>,
    ) -> PyResult<Bound<'py, PyList>> {
        self.batch_values_loc(py, n, |rng, loc| {
            providers::gen_company_email_custom(rng, loc, domain, None)
        })
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

type WeightedItems = (Vec<Py<PyAny>>, Option<Vec<f64>>);

fn extract_elements<'py>(
    py: Python<'py>,
    elements: Option<Bound<'py, PyAny>>,
) -> PyResult<WeightedItems> {
    match elements {
        None => {
            let items = PyTuple::new(py, ["a", "b", "c"])?
                .iter()
                .map(|b| b.unbind())
                .collect();
            Ok((items, None))
        }
        Some(obj) => {
            if let Ok(dict) = obj.cast::<PyDict>() {
                let mut items = Vec::with_capacity(dict.len());
                let mut weights = Vec::with_capacity(dict.len());
                for (k, v) in dict.iter() {
                    let w: f64 = v.extract()?;
                    if w < 0.0 || !w.is_finite() {
                        return Err(PyValueError::new_err("Weights must be finite and non-negative"));
                    }
                    items.push(k.unbind());
                    weights.push(w);
                }
                Ok((items, Some(weights)))
            } else {
                let items = obj
                    .try_iter()?
                    .map(|item| item.map(|b| b.unbind()))
                    .collect::<PyResult<Vec<_>>>()?;
                Ok((items, None))
            }
        }
    }
}

fn build_cumulative_weights(weights: &[f64]) -> PyResult<Vec<f64>> {
    let mut cum = Vec::with_capacity(weights.len());
    let mut total = 0.0;
    for &w in weights {
        if w < 0.0 || !w.is_finite() {
            return Err(PyValueError::new_err("Weights must be finite and non-negative"));
        }
        total += w;
        cum.push(total);
    }
    if total <= 0.0 {
        return Err(PyValueError::new_err("Total of weights must be greater than zero"));
    }
    Ok(cum)
}

#[inline]
fn pick_weighted_idx(rng: &mut Rng, cum: &[f64]) -> usize {
    let total = *cum.last().unwrap_or(&1.0);
    let r = rng.f64() * total;
    let idx = cum.partition_point(|&c| c <= r);
    idx.min(cum.len().saturating_sub(1))
}

fn extract_date_days(
    obj: Option<Bound<'_, PyAny>>,
    default_spec: &str,
    today_days: i64,
) -> PyResult<i64> {
    match obj {
        None => Ok(providers::parse_date_spec(default_spec, today_days)),
        Some(val) => {
            if let Ok(s) = val.extract::<String>() {
                Ok(providers::parse_date_spec(&s, today_days))
            } else if val.hasattr("year")? && val.hasattr("month")? && val.hasattr("day")? {
                let y: i32 = val.getattr("year")?.extract()?;
                let m: u8 = val.getattr("month")?.extract()?;
                let d: u8 = val.getattr("day")?.extract()?;
                Ok(providers::ymd_to_days(y, m, d))
            } else {
                Err(PyValueError::new_err(
                    "Expected a relative date string, ISO date string, or datetime.date",
                ))
            }
        }
    }
}

fn extract_datetime_secs(
    obj: Option<Bound<'_, PyAny>>,
    default_spec: &str,
    now_secs: i64,
) -> PyResult<i64> {
    match obj {
        None => Ok(providers::parse_datetime_spec(default_spec, now_secs)),
        Some(val) => {
            if let Ok(s) = val.extract::<String>() {
                Ok(providers::parse_datetime_spec(&s, now_secs))
            } else if val.hasattr("year")? && val.hasattr("month")? && val.hasattr("day")? {
                let y: i32 = val.getattr("year")?.extract()?;
                let m: u8 = val.getattr("month")?.extract()?;
                let d: u8 = val.getattr("day")?.extract()?;
                let hh: u8 = val
                    .getattr("hour")
                    .and_then(|v| v.extract())
                    .unwrap_or(0);
                let mm: u8 = val
                    .getattr("minute")
                    .and_then(|v| v.extract())
                    .unwrap_or(0);
                let ss: u8 = val
                    .getattr("second")
                    .and_then(|v| v.extract())
                    .unwrap_or(0);
                Ok(providers::ymdhms_to_secs(y, m, d, hh, mm, ss))
            } else {
                Err(PyValueError::new_err(
                    "Expected a relative datetime string, ISO string, or datetime.date/datetime",
                ))
            }
        }
    }
}

#[pymodule]
fn _faker_rs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Faker>()?;
    m.add_class::<UniqueProxy>()?;
    m.add_class::<OptionalProxy>()?;
    m.add("UniquenessException", m.py().get_type::<UniquenessException>())?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
