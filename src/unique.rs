use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use fastrand::Rng;
use pyo3::create_exception;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;

use crate::locale::{pick_locale, LocaleData};
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
    pub(crate) locales: Vec<&'static LocaleData>,
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

    fn next_unique_loc(
        &self,
        key: &'static str,
        mut gen: impl FnMut(&mut Rng, &'static LocaleData) -> String,
    ) -> PyResult<String> {
        let mut rng_guard = self.state.lock().expect("faker-rs RNG mutex poisoned");
        rng_guard.sync_global_seed();
        let mut seen_guard = self.seen.lock().expect("faker-rs unique mutex poisoned");
        let set = seen_guard.entry(key).or_default();

        for _ in 0..MAX_UNIQUE_RETRIES {
            let loc = pick_locale(&mut rng_guard.rng, &self.locales);
            let val = gen(&mut rng_guard.rng, loc);
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
        self.next_unique_loc("name", providers::gen_name_loc)
    }

    fn first_name(&self) -> PyResult<String> {
        self.next_unique_loc("first_name", providers::gen_first_name_loc)
    }

    fn last_name(&self) -> PyResult<String> {
        self.next_unique_loc("last_name", providers::gen_last_name_loc)
    }

    fn user_name(&self) -> PyResult<String> {
        self.next_unique_loc("user_name", providers::gen_user_name_loc)
    }

    #[pyo3(signature = (domain=None, name=None))]
    fn email(&self, domain: Option<&str>, name: Option<&str>) -> PyResult<String> {
        let mut attempt = 0usize;
        self.next_unique_loc("email", |rng, loc| {
            let add_digits = attempt > 0;
            attempt += 1;
            match name {
                Some(n) => providers::gen_email_from_name(rng, n, domain, add_digits),
                None => providers::gen_email_custom(rng, loc, domain, None),
            }
        })
    }

    fn address(&self) -> PyResult<String> {
        self.next_unique_loc("address", providers::gen_address_loc)
    }

    fn city(&self) -> PyResult<String> {
        self.next_unique_loc("city", providers::gen_city_loc)
    }

    fn state(&self) -> PyResult<String> {
        self.next_unique_loc("state", providers::gen_state_loc)
    }

    fn postcode(&self) -> PyResult<String> {
        self.next_unique_loc("postcode", providers::gen_postcode_loc)
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
        self.next_unique_loc("phone_number", providers::gen_phone_number_loc)
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

    fn word(&self) -> PyResult<String> {
        self.next_unique("word", providers::gen_word)
    }

    #[pyo3(signature = (text="###"))]
    fn numerify(&self, text: &str) -> PyResult<String> {
        self.next_unique("numerify", |rng| providers::gen_numerify(rng, text))
    }

    #[pyo3(signature = (text="????"))]
    fn lexify(&self, text: &str) -> PyResult<String> {
        self.next_unique("lexify", |rng| providers::gen_lexify(rng, text))
    }

    #[pyo3(signature = (text="##??"))]
    fn bothify(&self, text: &str) -> PyResult<String> {
        self.next_unique("bothify", |rng| providers::gen_bothify(rng, text))
    }

    #[pyo3(signature = (min_chars=None, max_chars=20))]
    fn pystr(&self, min_chars: Option<usize>, max_chars: usize) -> PyResult<String> {
        self.next_unique("pystr", |rng| providers::gen_pystr(rng, min_chars, max_chars))
    }

    #[pyo3(signature = (min_value=0, max_value=9999))]
    fn pyint(&self, min_value: i64, max_value: i64) -> PyResult<i64> {
        self.random_int(min_value, max_value)
    }

    #[pyo3(signature = (card_type=None))]
    fn credit_card_number(&self, card_type: Option<&str>) -> PyResult<String> {
        self.next_unique("credit_card_number", |rng| {
            providers::gen_credit_card_number(rng, card_type)
        })
    }

    fn iban(&self) -> PyResult<String> {
        self.next_unique("iban", providers::gen_iban)
    }

    fn currency_code(&self) -> PyResult<String> {
        self.next_unique("currency_code", providers::gen_currency_code)
    }

    fn ipv6(&self) -> PyResult<String> {
        self.next_unique("ipv6", providers::gen_ipv6)
    }

    fn mac_address(&self) -> PyResult<String> {
        self.next_unique("mac_address", providers::gen_mac_address)
    }

    fn domain_name(&self) -> PyResult<String> {
        self.next_unique("domain_name", providers::gen_domain_name)
    }

    fn slug(&self) -> PyResult<String> {
        self.next_unique("slug", providers::gen_slug)
    }

    fn free_email(&self) -> PyResult<String> {
        self.next_unique("free_email", providers::gen_free_email)
    }

    #[pyo3(signature = (domain=None, name=None))]
    fn company_email(&self, domain: Option<&str>, name: Option<&str>) -> PyResult<String> {
        let mut attempt = 0usize;
        self.next_unique_loc("company_email", |rng, loc| {
            let add_digits = attempt > 0;
            attempt += 1;
            match name {
                Some(n) => {
                    let dom = domain.map(str::to_owned).unwrap_or_else(|| providers::gen_domain_name(rng));
                    providers::gen_email_from_name(rng, n, Some(&dom), add_digits)
                }
                None => providers::gen_company_email_custom(rng, loc, domain, None),
            }
        })
    }

    fn safe_email(&self) -> PyResult<String> {
        self.next_unique("safe_email", providers::gen_safe_email)
    }

    fn hex_color(&self) -> PyResult<String> {
        self.next_unique("hex_color", providers::gen_hex_color)
    }

    #[pyo3(signature = (extension=None))]
    fn file_name(&self, extension: Option<&str>) -> PyResult<String> {
        self.next_unique("file_name", |rng| providers::gen_file_name(rng, extension))
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
