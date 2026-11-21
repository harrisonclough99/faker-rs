use std::fmt::Write as _;

use fastrand::Rng;

use crate::data::{
    CITIES, COMPANY_SUFFIXES, COMPANY_WORDS, COUNTRIES, EMAIL_DOMAINS, FIRST_NAMES, JOBS,
    LAST_NAMES, LOREM_WORDS, SECONDARY_PREFIXES, STATES, STREET_NAMES, STREET_SUFFIXES, TLDS,
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
    let year = rng.u16(1970..=2026);
    let month = rng.u8(1..=12);
    let max_day = days_in_month(year, month);
    let day = rng.u8(1..=max_day);
    let mut out = String::with_capacity(10);
    let _ = write!(out, "{year:04}-{month:02}-{day:02}");
    out
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
}
