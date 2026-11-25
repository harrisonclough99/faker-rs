use std::sync::{Arc, Mutex};

use fastrand::Rng;
use pyo3::prelude::*;

use crate::locale::{pick_locale, LocaleData};
use crate::providers;
use crate::RngState;

/// Proxy returned by `Faker.optional` that returns `None` with probability `1 - p`.
#[pyclass(module = "faker_rs._faker_rs")]
pub struct OptionalProxy {
    pub(crate) state: Arc<Mutex<RngState>>,
    pub(crate) locales: Vec<&'static LocaleData>,
}

impl OptionalProxy {
    #[inline]
    fn maybe<T>(&self, p: f64, f: impl FnOnce(&mut Rng, &'static LocaleData) -> T) -> Option<T> {
        let mut guard = self.state.lock().expect("faker-rs RNG mutex poisoned");
        guard.sync_global_seed();
        if p <= 0.0 || (p < 1.0 && guard.rng.f64() >= p) {
            return None;
        }
        let loc = pick_locale(&mut guard.rng, &self.locales);
        Some(f(&mut guard.rng, loc))
    }
}

#[pymethods]
impl OptionalProxy {
    #[pyo3(signature = (p=0.5))]
    fn name(&self, p: f64) -> Option<String> {
        self.maybe(p, providers::gen_name_loc)
    }

    #[pyo3(signature = (p=0.5))]
    fn first_name(&self, p: f64) -> Option<String> {
        self.maybe(p, providers::gen_first_name_loc)
    }

    #[pyo3(signature = (p=0.5))]
    fn last_name(&self, p: f64) -> Option<String> {
        self.maybe(p, providers::gen_last_name_loc)
    }

    #[pyo3(signature = (p=0.5))]
    fn email(&self, p: f64) -> Option<String> {
        self.maybe(p, providers::gen_email_loc)
    }

    #[pyo3(signature = (p=0.5))]
    fn address(&self, p: f64) -> Option<String> {
        self.maybe(p, providers::gen_address_loc)
    }

    #[pyo3(signature = (p=0.5))]
    fn city(&self, p: f64) -> Option<String> {
        self.maybe(p, providers::gen_city_loc)
    }

    #[pyo3(signature = (p=0.5))]
    fn state(&self, p: f64) -> Option<String> {
        self.maybe(p, providers::gen_state_loc)
    }

    #[pyo3(signature = (p=0.5))]
    fn country(&self, p: f64) -> Option<String> {
        self.maybe(p, |_, loc| loc.country.to_owned())
    }

    #[pyo3(signature = (p=0.5))]
    fn postcode(&self, p: f64) -> Option<String> {
        self.maybe(p, providers::gen_postcode_loc)
    }

    #[pyo3(signature = (p=0.5))]
    fn phone_number(&self, p: f64) -> Option<String> {
        self.maybe(p, providers::gen_phone_number_loc)
    }

    #[pyo3(signature = (p=0.5))]
    fn company(&self, p: f64) -> Option<String> {
        self.maybe(p, |rng, _| providers::gen_company(rng))
    }

    #[pyo3(signature = (p=0.5))]
    fn ipv4(&self, p: f64) -> Option<String> {
        self.maybe(p, |rng, _| providers::gen_ipv4(rng))
    }

    #[pyo3(signature = (p=0.5))]
    fn date(&self, p: f64) -> Option<String> {
        self.maybe(p, |rng, _| providers::gen_date(rng))
    }

    #[pyo3(signature = (p=0.5))]
    fn uuid4(&self, p: f64) -> Option<String> {
        self.maybe(p, |rng, _| providers::gen_uuid4(rng))
    }

    #[pyo3(signature = (p=0.5))]
    fn word(&self, p: f64) -> Option<String> {
        self.maybe(p, |rng, _| providers::gen_word(rng))
    }

    #[pyo3(signature = (min=0, max=9999, p=0.5))]
    fn random_int(&self, min: i64, max: i64, p: f64) -> Option<i64> {
        self.maybe(p, |rng, _| providers::gen_random_int(rng, min, max))
    }

    #[pyo3(signature = (card_type=None, p=0.5))]
    fn credit_card_number(&self, card_type: Option<&str>, p: f64) -> Option<String> {
        self.maybe(p, |rng, _| providers::gen_credit_card_number(rng, card_type))
    }
}
