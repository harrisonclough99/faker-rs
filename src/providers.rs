use std::fmt::Write as _;

use fastrand::Rng;

use crate::data::{
    CARD_PROVIDERS, CITIES, COMPANY_SUFFIXES, COMPANY_WORDS, COUNTRIES, CURRENCY_CODES,
    EMAIL_DOMAINS, FILE_EXTENSIONS, FIRST_NAMES, FREE_EMAIL_DOMAINS, IBAN_PREFIXES, JOBS,
    LAST_NAMES, LOREM_WORDS, MIME_TYPES, SECONDARY_PREFIXES, STATES, STREET_NAMES,
    STREET_SUFFIXES, TLDS, USER_AGENTS,
};

#[inline]
fn pick<'a>(rng: &mut Rng, slice: &'a [&'a str]) -> &'a str {
    slice[rng.usize(..slice.len())]
}

#[inline]
pub fn gen_name(rng: &mut Rng) -> String {
    let first = pick(rng, FIRST_NAMES);
    let last = pick(rng, LAST_NAMES);
    let mut out = String::with_capacity(first.len() + 1 + last.len());
    out.push_str(first);
    out.push(' ');
    out.push_str(last);
    out
}

#[inline]
pub fn gen_email(rng: &mut Rng) -> String {
    let first = pick(rng, FIRST_NAMES);
    let last = pick(rng, LAST_NAMES);
    let domain = pick(rng, EMAIL_DOMAINS);
    let num = rng.u16(1..1000);
    let mut out = String::with_capacity(first.len() + last.len() + domain.len() + 6);
    for b in first.bytes() {
        out.push(b.to_ascii_lowercase() as char);
    }
    out.push('.');
    for b in last.bytes() {
        out.push(b.to_ascii_lowercase() as char);
    }
    let _ = write!(out, "{num}@{domain}");
    out
}

#[inline]
pub fn gen_address(rng: &mut Rng) -> String {
    let number = rng.u16(100..10000);
    let street = pick(rng, STREET_NAMES);
    let suffix = pick(rng, STREET_SUFFIXES);
    let city = pick(rng, CITIES);
    let state = pick(rng, STATES);
    let zip = rng.u32(10000..99999);

    let mut out = String::with_capacity(64);
    if rng.u8(0..4) == 0 {
        let sec = pick(rng, SECONDARY_PREFIXES);
        let unit = rng.u16(1..999);
        let _ = write!(out, "{number} {street} {suffix} {sec} {unit}\n{city}, {state} {zip:05}");
    } else {
        let _ = write!(out, "{number} {street} {suffix}\n{city}, {state} {zip:05}");
    }
    out
}

#[inline]
fn is_leap_year(year: u16) -> bool {
    (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
}

#[inline]
fn days_in_month(year: u16, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => unreachable!(),
    }
}

#[inline]
pub fn gen_date(rng: &mut Rng) -> String {
    gen_date_pattern(rng, "%Y-%m-%d")
}

const MONTH_ABBR: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];
const MONTH_FULL: [&str; 12] = [
    "January", "February", "March", "April", "May", "June", "July", "August", "September",
    "October", "November", "December",
];

#[inline]
pub fn gen_date_pattern(rng: &mut Rng, pattern: &str) -> String {
    let year = rng.u16(1970..=2026);
    let month = rng.u8(1..=12);
    let max_day = days_in_month(year, month);
    let day = rng.u8(1..=max_day);
    if pattern == "%Y-%m-%d" {
        let mut out = String::with_capacity(10);
        let _ = write!(out, "{year:04}-{month:02}-{day:02}");
        return out;
    }
    let mut out = String::with_capacity(pattern.len() + 8);
    let mut chars = pattern.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '%' {
            match chars.next() {
                Some('Y') => {
                    let _ = write!(out, "{year:04}");
                }
                Some('y') => {
                    let _ = write!(out, "{:02}", year % 100);
                }
                Some('m') => {
                    let _ = write!(out, "{month:02}");
                }
                Some('d') => {
                    let _ = write!(out, "{day:02}");
                }
                Some('b') => out.push_str(MONTH_ABBR[(month - 1) as usize]),
                Some('B') => out.push_str(MONTH_FULL[(month - 1) as usize]),
                Some(other) => {
                    out.push('%');
                    out.push(other);
                }
                None => out.push('%'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// Convert (year, month, day) to days since 1970-01-01 (Howard Hinnant's algorithm).
#[inline]
pub fn ymd_to_days(y: i32, m: u8, d: u8) -> i64 {
    let y = y as i64 - if m <= 2 { 1 } else { 0 };
    let era = y.div_euclid(400);
    let yoe = y.rem_euclid(400);
    let m = m as i64;
    let doy = (153 * (m + if m > 2 { -3 } else { 9 }) + 2) / 5 + d as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// Convert days since 1970-01-01 to (year, month, day).
#[inline]
pub fn days_to_ymd(days: i64) -> (i32, u8, u8) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u8;
    let m = (mp + if mp < 10 { 3 } else { -9 }) as u8;
    let year = (y + if m <= 2 { 1 } else { 0 }) as i32;
    (year, m, d)
}

#[inline]
pub fn current_utc_days() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    secs / 86_400
}

pub fn parse_date_spec(spec: &str, today_days: i64) -> i64 {
    let s = spec.trim();
    if s.eq_ignore_ascii_case("today") || s.eq_ignore_ascii_case("now") {
        return today_days;
    }
    if let Some((y, m, d)) = parse_iso_ymd(s) {
        return ymd_to_days(y, m, d);
    }
    if s.len() >= 2 {
        let unit = s.as_bytes()[s.len() - 1];
        if let Ok(val) = s[..s.len() - 1].parse::<i64>() {
            return match unit {
                b'd' => today_days + val,
                b'w' => today_days + val * 7,
                b'm' => today_days + val * 30,
                b'y' => today_days + val * 365,
                _ => today_days,
            };
        }
    }
    today_days
}

fn parse_iso_ymd(s: &str) -> Option<(i32, u8, u8)> {
    let mut parts = s.split('-');
    let y: i32 = parts.next()?.parse().ok()?;
    let m: u8 = parts.next()?.parse().ok()?;
    let d: u8 = parts.next()?.parse().ok()?;
    if parts.next().is_some() || !(1..=12).contains(&m) || !(1..=31).contains(&d) {
        return None;
    }
    Some((y, m, d))
}

#[inline]
pub fn gen_ymd(rng: &mut Rng) -> (i32, u8, u8) {
    let year = rng.i32(1970..=2026);
    let month = rng.u8(1..=12);
    let day = rng.u8(1..=days_in_month(year as u16, month));
    (year, month, day)
}

#[inline]
pub fn gen_ymdhms(rng: &mut Rng) -> (i32, u8, u8, u8, u8, u8) {
    let (y, m, d) = gen_ymd(rng);
    let hh = rng.u8(0..=23);
    let mm = rng.u8(0..=59);
    let ss = rng.u8(0..=59);
    (y, m, d, hh, mm, ss)
}

#[inline]
pub fn gen_date_between_days(rng: &mut Rng, start_days: i64, end_days: i64) -> (i32, u8, u8) {
    if start_days >= end_days {
        return days_to_ymd(start_days);
    }
    let picked = rng.i64(start_days..=end_days);
    days_to_ymd(picked)
}

#[inline]
pub fn gen_date_of_birth(
    rng: &mut Rng,
    today_days: i64,
    minimum_age: u32,
    maximum_age: u32,
) -> (i32, u8, u8) {
    let (min_a, max_a) = if minimum_age > maximum_age {
        (maximum_age, minimum_age)
    } else {
        (minimum_age, maximum_age)
    };
    let end_days = today_days - (min_a as i64) * 365;
    let start_days = today_days - ((max_a as i64) + 1) * 365 + 1;
    gen_date_between_days(rng, start_days, end_days)
}

const HEX_LOWER: &[u8; 16] = b"0123456789abcdef";

#[inline]
pub fn gen_uuid4(rng: &mut Rng) -> String {
    let mut bytes = [0u8; 16];
    let hi = rng.u64(..);
    let lo = rng.u64(..);
    bytes[..8].copy_from_slice(&hi.to_le_bytes());
    bytes[8..].copy_from_slice(&lo.to_le_bytes());

    // Set version (4) and variant (RFC 4122).
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;

    let mut buf = [0u8; 36];
    let mut pos = 0;
    for (i, &b) in bytes.iter().enumerate() {
        if i == 4 || i == 6 || i == 8 || i == 10 {
            buf[pos] = b'-';
            pos += 1;
        }
        buf[pos] = HEX_LOWER[(b >> 4) as usize];
        buf[pos + 1] = HEX_LOWER[(b & 0x0f) as usize];
        pos += 2;
    }

    // SAFETY: `buf` contains only ASCII hex digits and '-'.
    unsafe { String::from_utf8_unchecked(buf.to_vec()) }
}

#[inline]
pub fn gen_first_name(rng: &mut Rng) -> String {
    pick(rng, FIRST_NAMES).to_owned()
}

#[inline]
pub fn gen_last_name(rng: &mut Rng) -> String {
    pick(rng, LAST_NAMES).to_owned()
}

#[inline]
pub fn gen_user_name(rng: &mut Rng) -> String {
    let first = pick(rng, FIRST_NAMES);
    let last = pick(rng, LAST_NAMES);
    let num = rng.u16(1..1000);
    let mut out = String::with_capacity(first.len() + last.len() + 5);
    for b in first.bytes() {
        out.push(b.to_ascii_lowercase() as char);
    }
    out.push('.');
    for b in last.bytes() {
        out.push(b.to_ascii_lowercase() as char);
    }
    let _ = write!(out, "{num}");
    out
}

#[inline]
pub fn gen_city(rng: &mut Rng) -> String {
    pick(rng, CITIES).to_owned()
}

#[inline]
pub fn gen_state(rng: &mut Rng) -> String {
    pick(rng, STATES).to_owned()
}

#[inline]
pub fn gen_postcode(rng: &mut Rng) -> String {
    let zip = rng.u32(10000..=99999);
    let mut out = String::with_capacity(5);
    let _ = write!(out, "{zip:05}");
    out
}

#[inline]
pub fn gen_country(rng: &mut Rng) -> String {
    pick(rng, COUNTRIES).to_owned()
}

#[inline]
pub fn gen_company(rng: &mut Rng) -> String {
    let w1 = pick(rng, COMPANY_WORDS);
    let suffix = pick(rng, COMPANY_SUFFIXES);
    if rng.bool() {
        let w2 = pick(rng, COMPANY_WORDS);
        let mut out = String::with_capacity(w1.len() + w2.len() + suffix.len() + 2);
        let _ = write!(out, "{w1} {w2} {suffix}");
        out
    } else {
        let mut out = String::with_capacity(w1.len() + suffix.len() + 1);
        let _ = write!(out, "{w1} {suffix}");
        out
    }
}

#[inline]
pub fn gen_job(rng: &mut Rng) -> String {
    pick(rng, JOBS).to_owned()
}

#[inline]
pub fn gen_phone_number(rng: &mut Rng) -> String {
    let area = rng.u16(200..=999);
    let exchange = rng.u16(200..=999);
    let line = rng.u16(0..=9999);
    let mut out = String::with_capacity(14);
    let _ = write!(out, "({area:03}) {exchange:03}-{line:04}");
    out
}

#[inline]
pub fn gen_url(rng: &mut Rng) -> String {
    let word = pick(rng, COMPANY_WORDS);
    let tld = pick(rng, TLDS);
    let mut out = String::with_capacity(16 + word.len() + tld.len());
    out.push_str("https://www.");
    for b in word.bytes() {
        out.push(b.to_ascii_lowercase() as char);
    }
    out.push('.');
    out.push_str(tld);
    out.push('/');
    out
}

#[inline]
pub fn gen_ipv4(rng: &mut Rng) -> String {
    let a = rng.u8(1..=254);
    let b = rng.u8(0..=255);
    let c = rng.u8(0..=255);
    let d = rng.u8(1..=254);
    let mut out = String::with_capacity(15);
    let _ = write!(out, "{a}.{b}.{c}.{d}");
    out
}

#[inline]
pub fn gen_sentence(rng: &mut Rng) -> String {
    let word_count = rng.usize(5..=12);
    let mut out = String::with_capacity(word_count * 7);
    for i in 0..word_count {
        let w = pick(rng, LOREM_WORDS);
        if i == 0 {
            let mut bytes = w.bytes();
            if let Some(first) = bytes.next() {
                out.push(first.to_ascii_uppercase() as char);
                for b in bytes {
                    out.push(b as char);
                }
            }
        } else {
            out.push(' ');
            out.push_str(w);
        }
    }
    out.push('.');
    out
}

#[inline]
pub fn gen_text(rng: &mut Rng, max_nb_chars: usize) -> String {
    if max_nb_chars < 5 {
        return String::new();
    }
    let mut out = String::with_capacity(max_nb_chars.min(256));
    loop {
        let sentence = gen_sentence(rng);
        let needed = if out.is_empty() {
            sentence.len()
        } else {
            1 + sentence.len()
        };
        if out.len() + needed > max_nb_chars {
            break;
        }
        if !out.is_empty() {
            out.push(' ');
        }
        out.push_str(&sentence);
        if out.len() >= (max_nb_chars * 4) / 5 {
            break;
        }
    }
    if out.is_empty() {
        let fallback = gen_sentence(rng);
        let take = fallback.len().min(max_nb_chars.saturating_sub(1));
        out.push_str(&fallback[..take]);
        out.push('.');
    }
    out
}

#[inline]
pub fn gen_boolean(rng: &mut Rng, chance_of_getting_true: u8) -> bool {
    if chance_of_getting_true == 0 {
        return false;
    }
    if chance_of_getting_true >= 100 {
        return true;
    }
    rng.u8(0..100) < chance_of_getting_true
}

#[inline]
pub fn gen_random_int(rng: &mut Rng, min: i64, max: i64) -> i64 {
    if min >= max {
        return min;
    }
    rng.i64(min..=max)
}

#[inline]
pub fn gen_iso8601(rng: &mut Rng) -> String {
    let year = rng.u16(1970..=2026);
    let month = rng.u8(1..=12);
    let day = rng.u8(1..=days_in_month(year, month));
    let hour = rng.u8(0..=23);
    let min = rng.u8(0..=59);
    let sec = rng.u8(0..=59);
    let mut out = String::with_capacity(19);
    let _ = write!(out, "{year:04}-{month:02}-{day:02}T{hour:02}:{min:02}:{sec:02}");
    out
}

const ASCII_LETTERS: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
const PASSWORD_SPECIAL: &[u8] = b"!@#$%^&*()_+-=";
const DIGITS: &[u8] = b"0123456789";

#[inline]
fn replace_pattern(rng: &mut Rng, pattern: &str, digits: bool, letters: bool) -> String {
    let mut out = String::with_capacity(pattern.len());
    for ch in pattern.chars() {
        if digits && ch == '#' {
            out.push((b'0' + rng.u8(0..10)) as char);
        } else if letters && ch == '?' {
            out.push(ASCII_LETTERS[rng.usize(..ASCII_LETTERS.len())] as char);
        } else {
            out.push(ch);
        }
    }
    out
}

#[inline]
pub fn gen_numerify(rng: &mut Rng, text: &str) -> String {
    replace_pattern(rng, text, true, false)
}

#[inline]
pub fn gen_lexify(rng: &mut Rng, text: &str) -> String {
    replace_pattern(rng, text, false, true)
}

#[inline]
pub fn gen_bothify(rng: &mut Rng, text: &str) -> String {
    replace_pattern(rng, text, true, true)
}

#[inline]
pub fn gen_pyfloat(
    rng: &mut Rng,
    left_digits: Option<u32>,
    right_digits: Option<u32>,
    positive: bool,
    min_value: Option<f64>,
    max_value: Option<f64>,
) -> f64 {
    let right = right_digits.unwrap_or_else(|| rng.u32(1..=5)).min(10);
    let scale = 10f64.powi(right as i32);

    if min_value.is_some() || max_value.is_some() {
        let lo = min_value.unwrap_or(if positive { 0.0 } else { -10_000.0 });
        let hi = max_value.unwrap_or(lo + 10_000.0);
        if lo >= hi {
            return (lo * scale).round() / scale;
        }
        let val = lo + rng.f64() * (hi - lo);
        return (val * scale).round() / scale;
    }

    let left = left_digits.unwrap_or_else(|| rng.u32(1..=5)).min(12);
    let max_int = if left == 0 {
        0
    } else {
        10i64.saturating_pow(left).saturating_sub(1)
    };
    let int_part = if max_int == 0 {
        0.0
    } else {
        rng.i64(0..=max_int) as f64
    };
    let frac_part = (rng.f64() * scale).floor() / scale;
    let mut val = int_part + frac_part;
    if !positive && rng.bool() {
        val = -val;
    }
    val
}

#[inline]
pub fn gen_pystr(rng: &mut Rng, min_chars: Option<usize>, max_chars: usize) -> String {
    let lo = min_chars.unwrap_or(max_chars).min(max_chars);
    let len = if lo == max_chars {
        lo
    } else {
        rng.usize(lo..=max_chars)
    };
    let mut out = String::with_capacity(len);
    for _ in 0..len {
        out.push(ASCII_LETTERS[rng.usize(..ASCII_LETTERS.len())] as char);
    }
    out
}

#[inline]
pub fn gen_password(
    rng: &mut Rng,
    length: usize,
    special_chars: bool,
    digits: bool,
    upper_case: bool,
    lower_case: bool,
) -> String {
    if length == 0 {
        return String::new();
    }
    let mut pool = Vec::with_capacity(80);
    if lower_case {
        pool.extend_from_slice(&ASCII_LETTERS[..26]);
    }
    if upper_case {
        pool.extend_from_slice(&ASCII_LETTERS[26..]);
    }
    if digits {
        pool.extend_from_slice(DIGITS);
    }
    if special_chars {
        pool.extend_from_slice(PASSWORD_SPECIAL);
    }
    if pool.is_empty() {
        pool.extend_from_slice(ASCII_LETTERS);
    }

    let mut out = String::with_capacity(length);
    for _ in 0..length {
        out.push(pool[rng.usize(..pool.len())] as char);
    }
    out
}

#[inline]
pub fn gen_word(rng: &mut Rng) -> String {
    pick(rng, LOREM_WORDS).to_owned()
}

#[inline]
pub fn gen_words(rng: &mut Rng, nb: usize) -> Vec<String> {
    (0..nb).map(|_| gen_word(rng)).collect()
}

#[inline]
pub fn gen_paragraph(rng: &mut Rng, nb_sentences: usize) -> String {
    if nb_sentences == 0 {
        return String::new();
    }
    let mut out = String::with_capacity(nb_sentences * 48);
    for i in 0..nb_sentences {
        if i > 0 {
            out.push(' ');
        }
        out.push_str(&gen_sentence(rng));
    }
    out
}

#[inline]
pub fn gen_paragraphs(rng: &mut Rng, nb: usize) -> Vec<String> {
    (0..nb).map(|_| gen_paragraph(rng, 3)).collect()
}

// --- Finance providers ---

#[inline]
pub fn gen_credit_card_number(rng: &mut Rng, card_type: Option<&str>) -> String {
    let (prefix, total_len) = match card_type {
        Some("amex") => (if rng.bool() { "34" } else { "37" }, 15usize),
        Some("mastercard") => {
            let p = match rng.u8(0..5) {
                0 => "51",
                1 => "52",
                2 => "53",
                3 => "54",
                _ => "55",
            };
            (p, 16)
        }
        Some("discover") => ("6011", 16),
        Some("visa") => ("4", 16),
        _ => match rng.u8(0..3) {
            0 => ("4", 16),
            1 => ("52", 16),
            _ => ("34", 15),
        },
    };

    let mut digits = [0u8; 16];
    let mut len = 0;
    for b in prefix.bytes() {
        digits[len] = b - b'0';
        len += 1;
    }
    while len + 1 < total_len {
        digits[len] = rng.u8(0..10);
        len += 1;
    }

    // Compute Luhn check digit.
    let mut sum = 0u32;
    let mut double = true;
    for i in (0..len).rev() {
        let mut d = digits[i] as u32;
        if double {
            d *= 2;
            if d > 9 {
                d -= 9;
            }
        }
        sum += d;
        double = !double;
    }
    digits[len] = ((10 - (sum % 10)) % 10) as u8;
    len += 1;

    let mut out = String::with_capacity(len);
    for &d in &digits[..len] {
        out.push((b'0' + d) as char);
    }
    out
}

#[inline]
pub fn gen_credit_card_expire(rng: &mut Rng) -> String {
    let month = rng.u8(1..=12);
    let year = rng.u8(27..=34);
    let mut out = String::with_capacity(5);
    let _ = write!(out, "{month:02}/{year:02}");
    out
}

#[inline]
pub fn gen_credit_card_security_code(rng: &mut Rng, card_type: Option<&str>) -> String {
    if card_type == Some("amex") {
        let code = rng.u16(0..=9999);
        let mut out = String::with_capacity(4);
        let _ = write!(out, "{code:04}");
        out
    } else {
        let code = rng.u16(0..=999);
        let mut out = String::with_capacity(3);
        let _ = write!(out, "{code:03}");
        out
    }
}

#[inline]
pub fn gen_credit_card_provider(rng: &mut Rng) -> String {
    pick(rng, CARD_PROVIDERS).to_owned()
}

#[inline]
pub fn gen_iban(rng: &mut Rng) -> String {
    let (country, total_len) = IBAN_PREFIXES[rng.usize(..IBAN_PREFIXES.len())];
    let check = rng.u8(10..=98);
    let mut out = String::with_capacity(total_len);
    let _ = write!(out, "{country}{check:02}");
    while out.len() < total_len {
        out.push((b'0' + rng.u8(0..10)) as char);
    }
    out
}

#[inline]
pub fn gen_currency_code(rng: &mut Rng) -> String {
    pick(rng, CURRENCY_CODES).to_owned()
}

// --- Internet providers ---

#[inline]
pub fn gen_ipv6(rng: &mut Rng) -> String {
    let mut out = String::with_capacity(39);
    for i in 0..8 {
        if i > 0 {
            out.push(':');
        }
        let seg = rng.u16(..);
        let _ = write!(out, "{seg:04x}");
    }
    out
}

#[inline]
pub fn gen_mac_address(rng: &mut Rng) -> String {
    let mut out = String::with_capacity(17);
    for i in 0..6 {
        if i > 0 {
            out.push(':');
        }
        let b = rng.u8(..);
        let _ = write!(out, "{b:02x}");
    }
    out
}

#[inline]
pub fn gen_domain_name(rng: &mut Rng) -> String {
    let word = pick(rng, COMPANY_WORDS);
    let tld = pick(rng, TLDS);
    let mut out = String::with_capacity(word.len() + tld.len() + 1);
    for b in word.bytes() {
        out.push(b.to_ascii_lowercase() as char);
    }
    out.push('.');
    out.push_str(tld);
    out
}

#[inline]
pub fn gen_user_agent(rng: &mut Rng) -> String {
    pick(rng, USER_AGENTS).to_owned()
}

#[inline]
pub fn gen_slug(rng: &mut Rng) -> String {
    let w1 = pick(rng, LOREM_WORDS);
    let w2 = pick(rng, LOREM_WORDS);
    let w3 = pick(rng, LOREM_WORDS);
    let mut out = String::with_capacity(w1.len() + w2.len() + w3.len() + 2);
    let _ = write!(out, "{w1}-{w2}-{w3}");
    out
}

#[inline]
fn email_with_domain(rng: &mut Rng, domain: &str) -> String {
    let first = pick(rng, FIRST_NAMES);
    let last = pick(rng, LAST_NAMES);
    let num = rng.u16(1..1000);
    let mut out = String::with_capacity(first.len() + last.len() + domain.len() + 6);
    for b in first.bytes() {
        out.push(b.to_ascii_lowercase() as char);
    }
    out.push('.');
    for b in last.bytes() {
        out.push(b.to_ascii_lowercase() as char);
    }
    let _ = write!(out, "{num}@{domain}");
    out
}

#[inline]
pub fn gen_free_email(rng: &mut Rng) -> String {
    let domain = pick(rng, FREE_EMAIL_DOMAINS);
    email_with_domain(rng, domain)
}

#[inline]
pub fn gen_company_email(rng: &mut Rng) -> String {
    let domain = gen_domain_name(rng);
    email_with_domain(rng, &domain)
}

#[inline]
pub fn gen_safe_email(rng: &mut Rng) -> String {
    let domain = pick(rng, &EMAIL_DOMAINS[..3]);
    email_with_domain(rng, domain)
}

// --- Color, Geo & File providers ---

#[inline]
pub fn gen_hex_color(rng: &mut Rng) -> String {
    let r = rng.u8(..);
    let g = rng.u8(..);
    let b = rng.u8(..);
    let mut out = String::with_capacity(7);
    let _ = write!(out, "#{r:02x}{g:02x}{b:02x}");
    out
}

#[inline]
pub fn gen_rgb_color(rng: &mut Rng) -> String {
    let r = rng.u8(..);
    let g = rng.u8(..);
    let b = rng.u8(..);
    let mut out = String::with_capacity(11);
    let _ = write!(out, "{r},{g},{b}");
    out
}

#[inline]
pub fn gen_latitude(rng: &mut Rng) -> f64 {
    let raw = -90.0 + rng.f64() * 180.0;
    (raw * 1_000_000.0).round() / 1_000_000.0
}

#[inline]
pub fn gen_longitude(rng: &mut Rng) -> f64 {
    let raw = -180.0 + rng.f64() * 360.0;
    (raw * 1_000_000.0).round() / 1_000_000.0
}

#[inline]
pub fn gen_file_name(rng: &mut Rng, extension: Option<&str>) -> String {
    let word = pick(rng, LOREM_WORDS);
    let ext = extension.unwrap_or_else(|| pick(rng, FILE_EXTENSIONS));
    let mut out = String::with_capacity(word.len() + ext.len() + 1);
    let _ = write!(out, "{word}.{ext}");
    out
}

#[inline]
pub fn gen_mime_type(rng: &mut Rng) -> String {
    pick(rng, MIME_TYPES).to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_generation() {
        let mut rng1 = Rng::with_seed(42);
        let mut rng2 = Rng::with_seed(42);

        assert_eq!(gen_name(&mut rng1), gen_name(&mut rng2));
        assert_eq!(gen_email(&mut rng1), gen_email(&mut rng2));
        assert_eq!(gen_address(&mut rng1), gen_address(&mut rng2));
        assert_eq!(gen_date(&mut rng1), gen_date(&mut rng2));
        assert_eq!(gen_uuid4(&mut rng1), gen_uuid4(&mut rng2));
    }

    #[test]
    fn email_format_valid() {
        let mut rng = Rng::with_seed(123);
        for _ in 0..100 {
            let email = gen_email(&mut rng);
            assert!(email.contains('@'));
            assert!(email.is_ascii());
            assert_eq!(email, email.to_lowercase());
        }
    }

    #[test]
    fn address_format_valid() {
        let mut rng = Rng::with_seed(456);
        for _ in 0..100 {
            let addr = gen_address(&mut rng);
            let lines: Vec<&str> = addr.lines().collect();
            assert_eq!(lines.len(), 2);
            assert!(lines[1].contains(", "));
        }
    }

    #[test]
    fn date_format_and_leap_years_valid() {
        let mut rng = Rng::with_seed(789);
        for _ in 0..500 {
            let date = gen_date(&mut rng);
            assert_eq!(date.len(), 10);
            let parts: Vec<&str> = date.split('-').collect();
            assert_eq!(parts.len(), 3);
            let y: u16 = parts[0].parse().unwrap();
            let m: u8 = parts[1].parse().unwrap();
            let d: u8 = parts[2].parse().unwrap();
            assert!((1970..=2026).contains(&y));
            assert!((1..=12).contains(&m));
            assert!((1..=days_in_month(y, m)).contains(&d));
        }
    }

    #[test]
    fn uuid4_format_valid() {
        let mut rng = Rng::with_seed(999);
        for _ in 0..100 {
            let u = gen_uuid4(&mut rng);
            assert_eq!(u.len(), 36);
            let bytes = u.as_bytes();
            assert_eq!(bytes[8], b'-');
            assert_eq!(bytes[13], b'-');
            assert_eq!(bytes[14], b'4');
            assert_eq!(bytes[18], b'-');
            assert!(matches!(bytes[19], b'8' | b'9' | b'a' | b'b'));
            assert_eq!(bytes[23], b'-');
        }
    }

    #[test]
    fn extended_providers_valid() {
        let mut rng = Rng::with_seed(2026);
        assert!(!gen_first_name(&mut rng).is_empty());
        assert!(!gen_last_name(&mut rng).is_empty());
        assert!(gen_user_name(&mut rng).contains('.'));
        assert!(!gen_city(&mut rng).is_empty());
        assert_eq!(gen_state(&mut rng).len(), 2);
        assert_eq!(gen_postcode(&mut rng).len(), 5);
        assert!(!gen_country(&mut rng).is_empty());
        assert!(gen_company(&mut rng).contains(' '));
        assert!(!gen_job(&mut rng).is_empty());

        let phone = gen_phone_number(&mut rng);
        assert_eq!(phone.len(), 14);
        assert!(phone.starts_with('('));

        let url = gen_url(&mut rng);
        assert!(url.starts_with("https://www."));

        let ip = gen_ipv4(&mut rng);
        assert_eq!(ip.split('.').count(), 4);

        let sent = gen_sentence(&mut rng);
        assert!(sent.ends_with('.'));

        let txt = gen_text(&mut rng, 120);
        assert!(!txt.is_empty() && txt.len() <= 120);

        assert!(!gen_boolean(&mut rng, 0));
        assert!(gen_boolean(&mut rng, 100));

        for _ in 0..50 {
            let v = gen_random_int(&mut rng, 10, 20);
            assert!((10..=20).contains(&v));
        }

        let iso = gen_iso8601(&mut rng);
        assert_eq!(iso.len(), 19);
        assert_eq!(&iso[10..11], "T");
    }

    #[test]
    fn pattern_primitive_and_text_providers_valid() {
        let mut rng = Rng::with_seed(3030);

        let num = gen_numerify(&mut rng, "###-##-####");
        assert_eq!(num.len(), 11);
        assert!(num.chars().filter(|c| *c != '-').all(|c| c.is_ascii_digit()));

        let lex = gen_lexify(&mut rng, "????");
        assert_eq!(lex.len(), 4);
        assert!(lex.chars().all(|c| c.is_ascii_alphabetic()));

        let both = gen_bothify(&mut rng, "##??");
        assert_eq!(both.len(), 4);
        assert!(both[..2].chars().all(|c| c.is_ascii_digit()));
        assert!(both[2..].chars().all(|c| c.is_ascii_alphabetic()));

        for _ in 0..50 {
            let f = gen_pyfloat(&mut rng, None, Some(2), true, Some(1.5), Some(9.5));
            assert!((1.5..=9.5).contains(&f));
        }

        let s = gen_pystr(&mut rng, Some(5), 10);
        assert!((5..=10).contains(&s.len()));

        let pw = gen_password(&mut rng, 16, true, true, true, true);
        assert_eq!(pw.len(), 16);

        assert!(!gen_word(&mut rng).is_empty());
        assert_eq!(gen_words(&mut rng, 5).len(), 5);
        assert!(gen_paragraph(&mut rng, 3).ends_with('.'));
        assert_eq!(gen_paragraphs(&mut rng, 4).len(), 4);
    }

    fn luhn_valid(num: &str) -> bool {
        let mut sum = 0u32;
        let mut double = false;
        for b in num.bytes().rev() {
            let mut d = (b - b'0') as u32;
            if double {
                d *= 2;
                if d > 9 {
                    d -= 9;
                }
            }
            sum += d;
            double = !double;
        }
        sum.is_multiple_of(10)
    }

    #[test]
    fn finance_internet_geo_color_file_valid() {
        let mut rng = Rng::with_seed(4040);

        for _ in 0..30 {
            let cc = gen_credit_card_number(&mut rng, None);
            assert!(cc.len() == 15 || cc.len() == 16);
            assert!(luhn_valid(&cc));
        }
        assert!(luhn_valid(&gen_credit_card_number(&mut rng, Some("visa"))));
        assert!(luhn_valid(&gen_credit_card_number(&mut rng, Some("amex"))));
        assert_eq!(gen_credit_card_expire(&mut rng).len(), 5);
        assert_eq!(gen_credit_card_security_code(&mut rng, Some("amex")).len(), 4);
        assert!(!gen_credit_card_provider(&mut rng).is_empty());
        assert!(gen_iban(&mut rng).len() >= 15);
        assert_eq!(gen_currency_code(&mut rng).len(), 3);

        assert_eq!(gen_ipv6(&mut rng).split(':').count(), 8);
        assert_eq!(gen_mac_address(&mut rng).split(':').count(), 6);
        assert!(gen_domain_name(&mut rng).contains('.'));
        assert!(gen_user_agent(&mut rng).starts_with("Mozilla/"));
        assert_eq!(gen_slug(&mut rng).split('-').count(), 3);
        assert!(gen_free_email(&mut rng).contains('@'));
        assert!(gen_company_email(&mut rng).contains('@'));
        assert!(gen_safe_email(&mut rng).contains("@example."));

        let hex = gen_hex_color(&mut rng);
        assert_eq!(hex.len(), 7);
        assert!(hex.starts_with('#'));
        assert_eq!(gen_rgb_color(&mut rng).split(',').count(), 3);

        let lat = gen_latitude(&mut rng);
        let lon = gen_longitude(&mut rng);
        assert!((-90.0..=90.0).contains(&lat));
        assert!((-180.0..=180.0).contains(&lon));

        assert!(gen_file_name(&mut rng, None).contains('.'));
        assert!(gen_mime_type(&mut rng).contains('/'));
    }

    #[test]
    fn date_patterns_and_calendar_math_valid() {
        let mut rng = Rng::with_seed(5050);
        let d = gen_date_pattern(&mut rng, "%m/%d/%Y");
        assert_eq!(d.len(), 10);
        assert_eq!(&d[2..3], "/");
        assert_eq!(&d[5..6], "/");

        assert_eq!(days_to_ymd(0), (1970, 1, 1));
        assert_eq!(ymd_to_days(1970, 1, 1), 0);
        assert_eq!(days_to_ymd(ymd_to_days(2024, 2, 29)), (2024, 2, 29));

        let today = ymd_to_days(2026, 11, 21);
        let dob = gen_date_of_birth(&mut rng, today, 18, 30);
        assert!((1995..=2009).contains(&dob.0));
    }
}
