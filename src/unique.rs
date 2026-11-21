use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use fastrand::Rng;
use pyo3::create_exception;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;

use crate::providers;
use crate::RngState;

const MAX_UNIQUE_RETRIES: usize = 10_000;

create_exception!(faker_rs, UniquenessException, PyException);

pub(crate) type SeenMap = HashMap<&'static str, HashSet<String>>;

/// Proxy returned by `Faker.unique` that guarantees unique values per provider.
#[pyclass(module = "faker_rs._faker_rs")]
pub struct UniqueProxy {
    pub(crate) state: Arc<Mutex<RngState>>,
    pub(crate) seen: Arc<Mutex<SeenMap>>,
}

impl UniqueProxy {
    fn next_unique(
        &self,
        key: &'static str,
        mut gen: impl FnMut(&mut Rng) -> String,
    ) -> PyResult<String> {
        let mut rng_guard = self.state.lock().expect("faker-rs RNG mutex poisoned");
        rng_guard.sync_global_seed();
        let mut seen_guard = self.seen.lock().expect("faker-rs unique mutex poisoned");
        let set = seen_guard.entry(key).or_default();

        for _ in 0..MAX_UNIQUE_RETRIES {
            let val = gen(&mut rng_guard.rng);
            if set.insert(val.clone()) {
                return Ok(val);
            }
        }

        Err(UniquenessException::new_err(format!(
            "Got duplicated values after {MAX_UNIQUE_RETRIES} iterations for '{key}'."
        )))
    }
}

#[pymethods]
impl UniqueProxy {
    /// Reset all seen unique values across every provider.
    fn clear(&self) {
        let mut seen_guard = self.seen.lock().expect("faker-rs unique mutex poisoned");
        seen_guard.clear();
    }

    fn name(&self) -> PyResult<String> {
        self.next_unique("name", providers::gen_name)
    }

    fn first_name(&self) -> PyResult<String> {
        self.next_unique("first_name", providers::gen_first_name)
    }

    fn last_name(&self) -> PyResult<String> {
        self.next_unique("last_name", providers::gen_last_name)
    }

    fn user_name(&self) -> PyResult<String> {
        self.next_unique("user_name", providers::gen_user_name)
    }

    fn email(&self) -> PyResult<String> {
        self.next_unique("email", providers::gen_email)
    }

    fn address(&self) -> PyResult<String> {
        self.next_unique("address", providers::gen_address)
    }

    fn city(&self) -> PyResult<String> {
        self.next_unique("city", providers::gen_city)
    }

    fn state(&self) -> PyResult<String> {
        self.next_unique("state", providers::gen_state)
    }

    fn postcode(&self) -> PyResult<String> {
        self.next_unique("postcode", providers::gen_postcode)
    }

    fn country(&self) -> PyResult<String> {
        self.next_unique("country", providers::gen_country)
    }

    fn company(&self) -> PyResult<String> {
        self.next_unique("company", providers::gen_company)
    }

    fn job(&self) -> PyResult<String> {
        self.next_unique("job", providers::gen_job)
    }

    fn phone_number(&self) -> PyResult<String> {
        self.next_unique("phone_number", providers::gen_phone_number)
    }

    fn url(&self) -> PyResult<String> {
        self.next_unique("url", providers::gen_url)
    }

    fn ipv4(&self) -> PyResult<String> {
        self.next_unique("ipv4", providers::gen_ipv4)
    }

    fn date(&self) -> PyResult<String> {
        self.next_unique("date", providers::gen_date)
    }

    fn iso8601(&self) -> PyResult<String> {
        self.next_unique("iso8601", providers::gen_iso8601)
    }

    fn uuid4(&self) -> PyResult<String> {
        self.next_unique("uuid4", providers::gen_uuid4)
    }

    #[pyo3(signature = (min=0, max=9999))]
    fn random_int(&self, min: i64, max: i64) -> PyResult<i64> {
        let mut rng_guard = self.state.lock().expect("faker-rs RNG mutex poisoned");
        rng_guard.sync_global_seed();
        let mut seen_guard = self.seen.lock().expect("faker-rs unique mutex poisoned");
        let set = seen_guard.entry("random_int").or_default();

        for _ in 0..MAX_UNIQUE_RETRIES {
            let val = providers::gen_random_int(&mut rng_guard.rng, min, max);
            if set.insert(val.to_string()) {
                return Ok(val);
            }
        }

        Err(UniquenessException::new_err(format!(
            "Got duplicated values after {MAX_UNIQUE_RETRIES} iterations for 'random_int'."
        )))
    }
}
