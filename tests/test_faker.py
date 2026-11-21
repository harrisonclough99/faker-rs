import datetime
import ipaddress
import re
import uuid

import pytest
from faker_rs import Faker, UniquenessException, __version__
from faker_rs.exceptions import UniquenessException as ExcFromSubmodule


def test_version_exposed() -> None:
    assert isinstance(__version__, str)
    assert len(__version__) > 0


def test_instantiation_drop_in() -> None:
    f1 = Faker()
    f2 = Faker("en_US")
    f3 = Faker(seed=12345)
    assert isinstance(f1.name(), str)
    assert isinstance(f2.name(), str)
    assert isinstance(f3.name(), str)


def test_single_providers_format() -> None:
    fake = Faker(seed=42)

    name = fake.name()
    assert isinstance(name, str)
    assert " " in name

    assert isinstance(fake.first_name(), str) and len(fake.first_name()) > 0
    assert isinstance(fake.last_name(), str) and len(fake.last_name()) > 0
    assert re.match(r"^[a-z]+\.[a-z]+\d+$", fake.user_name())

    email = fake.email()
    assert re.match(r"^[a-z]+\.[a-z]+\d+@[a-z.]+$", email)

    address = fake.address()
    lines = address.splitlines()
    assert len(lines) == 2
    assert re.search(r",\s[A-Z]{2}\s\d{5}$", lines[1])

    assert len(fake.city()) > 0
    assert len(fake.state()) == 2
    assert re.match(r"^\d{5}$", fake.postcode())
    assert len(fake.country()) > 0

    assert " " in fake.company()
    assert len(fake.job()) > 0
    assert re.match(r"^\(\d{3}\) \d{3}-\d{4}$", fake.phone_number())
    assert fake.url().startswith("https://www.")
    ipaddress.IPv4Address(fake.ipv4())

    date_str = fake.date()
    parsed_date = datetime.date.fromisoformat(date_str)
    assert 1970 <= parsed_date.year <= 2026

    iso_str = fake.iso8601()
    datetime.datetime.fromisoformat(iso_str)

    uuid_str = fake.uuid4()
    assert uuid.UUID(uuid_str).version == 4

    assert fake.sentence().endswith(".")
    text = fake.text(max_nb_chars=100)
    assert 0 < len(text) <= 100

    assert isinstance(fake.boolean(), bool)
    assert fake.boolean(chance_of_getting_true=0) is False
    assert fake.boolean(chance_of_getting_true=100) is True

    for _ in range(20):
        val = fake.random_int(min=5, max=15)
        assert 5 <= val <= 15


def test_batch_providers() -> None:
    fake = Faker(seed=99)

    assert fake.names(0) == []
    assert len(fake.names(50)) == 50
    assert len(fake.first_names(25)) == 25
    assert len(fake.last_names(25)) == 25
    assert len(fake.user_names(25)) == 25
    assert len(fake.emails(50)) == 50
    assert len(fake.addresses(50)) == 50
    assert len(fake.cities(25)) == 25
    assert len(fake.states(25)) == 25
    assert len(fake.postcodes(25)) == 25
    assert len(fake.countries(25)) == 25
    assert len(fake.companies(25)) == 25
    assert len(fake.jobs(25)) == 25
    assert len(fake.phone_numbers(25)) == 25
    assert len(fake.urls(25)) == 25
    assert len(fake.ipv4s(25)) == 25
    assert len(fake.sentences(25)) == 25
    assert all(len(t) <= 80 for t in fake.texts(25, max_nb_chars=80))
    assert all(isinstance(b, bool) for b in fake.booleans(25))
    assert all(1 <= v <= 10 for v in fake.random_ints(25, min=1, max=10))

    dates = fake.dates(100)
    assert len(dates) == 100
    for d in dates:
        datetime.date.fromisoformat(d)

    isos = fake.iso8601s(25)
    assert len(isos) == 25
    for iso in isos:
        datetime.datetime.fromisoformat(iso)

    uuids = fake.uuid4s(50)
    assert len(uuids) == 50
    for u in uuids:
        assert uuid.UUID(u).version == 4


def test_unique_proxy() -> None:
    assert UniquenessException is ExcFromSubmodule

    fake = Faker(seed=123)
    emails = [fake.unique.email() for _ in range(500)]
    assert len(set(emails)) == 500

    # Small domain exhaustion should raise UniquenessException.
    vals = {fake.unique.random_int(min=1, max=3) for _ in range(3)}
    assert vals == {1, 2, 3}
    with pytest.raises(UniquenessException):
        fake.unique.random_int(min=1, max=3)

    # Clearing resets seen values.
    fake.unique.clear()
    assert fake.unique.random_int(min=1, max=3) in {1, 2, 3}


def test_global_seeding_determinism() -> None:
    Faker.seed(42)
    f1 = Faker()
    seq1 = [
        f1.name(),
        f1.email(),
        f1.address(),
        f1.phone_number(),
        f1.company(),
        f1.ipv4(),
        f1.date(),
        f1.iso8601(),
        f1.uuid4(),
    ]

    Faker.seed(42)
    f2 = Faker()
    seq2 = [
        f2.name(),
        f2.email(),
        f2.address(),
        f2.phone_number(),
        f2.company(),
        f2.ipv4(),
        f2.date(),
        f2.iso8601(),
        f2.uuid4(),
    ]

    assert seq1 == seq2


def test_global_seeding_resets_existing_unseeded_instance() -> None:
    f1 = Faker()
    Faker.seed(777)
    a = (f1.name(), f1.email())

    Faker.seed(777)
    b = (f1.name(), f1.email())

    assert a == b


def test_instance_seeding_isolated_from_global() -> None:
    Faker.seed(1)
    f_inst = Faker(seed=555)
    first_run = f_inst.names(10)

    Faker.seed(99999)
    f_inst.seed_instance(555)
    second_run = f_inst.names(10)

    assert first_run == second_run


def test_patterns_primitives_and_text() -> None:
    fake = Faker(seed=303)

    assert re.match(r"^\d{3}-\d{2}-\d{4}$", fake.numerify("###-##-####"))
    assert re.match(r"^[a-zA-Z]{4}$", fake.lexify("????"))
    assert re.match(r"^\d{2}[a-zA-Z]{2}$", fake.bothify("##??"))

    assert 10 <= fake.pyint(min_value=10, max_value=20) <= 20
    assert isinstance(fake.pybool(), bool)
    assert fake.pybool(truth_probability=0) is False
    assert fake.pybool(truth_probability=100) is True

    f = fake.pyfloat(right_digits=2, positive=True, min_value=1.0, max_value=5.0)
    assert 1.0 <= f <= 5.0

    s = fake.pystr(min_chars=5, max_chars=8)
    assert 5 <= len(s) <= 8

    pw = fake.password(length=14)
    assert len(pw) == 14

    assert fake.random_element(["x", "y", "z"]) in {"x", "y", "z"}
    sample = fake.random_elements([1, 2, 3, 4], length=3, unique=True)
    assert len(sample) == 3 and len(set(sample)) == 3

    assert isinstance(fake.word(), str)
    assert len(fake.words(nb=5)) == 5
    assert fake.paragraph(nb_sentences=2).endswith(".")
    assert len(fake.paragraphs(nb=3)) == 3

    # Batch methods
    assert len(fake.numerifys(10, "###")) == 10
    assert len(fake.lexifys(10, "???")) == 10
    assert len(fake.bothifys(10, "#?")) == 10
    assert len(fake.pyints(10)) == 10
    assert len(fake.pyfloats(10)) == 10
    assert len(fake.pybools(10)) == 10
    assert len(fake.pystrs(10)) == 10
    assert len(fake.passwords(10)) == 10
    assert len(fake.words_batch(10, nb=4)) == 10
    assert len(fake.paragraphs_batch(10, nb=2)) == 10

    # Unique support
    assert len({fake.unique.numerify("##") for _ in range(20)}) == 20
    assert len({fake.unique.pystr(min_chars=8, max_chars=8) for _ in range(20)}) == 20


def _luhn_ok(num: str) -> bool:
    total = 0
    for idx, ch in enumerate(reversed(num)):
        d = int(ch)
        if idx % 2 == 1:
            d *= 2
            if d > 9:
                d -= 9
        total += d
    return total % 10 == 0


def test_finance_internet_geo_color_file() -> None:
    fake = Faker(seed=404)

    for card_type in (None, "visa", "mastercard", "amex", "discover"):
        cc = fake.credit_card_number(card_type=card_type)
        assert _luhn_ok(cc)

    assert re.match(r"^\d{2}/\d{2}$", fake.credit_card_expire())
    assert len(fake.credit_card_security_code("amex")) == 4
    assert len(fake.credit_card_security_code()) == 3
    assert len(fake.credit_card_provider()) > 0
    assert len(fake.iban()) >= 15
    assert len(fake.currency_code()) == 3

    ipaddress.IPv6Address(fake.ipv6())
    assert re.match(r"^([0-9a-f]{2}:){5}[0-9a-f]{2}$", fake.mac_address())
    assert "." in fake.domain_name()
    assert fake.user_agent().startswith("Mozilla/")
    assert len(fake.slug().split("-")) == 3
    assert "@" in fake.free_email()
    assert "@" in fake.company_email()
    assert "@example." in fake.safe_email()

    assert re.match(r"^#[0-9a-f]{6}$", fake.hex_color())
    assert len(fake.rgb_color().split(",")) == 3
    assert -90.0 <= fake.latitude() <= 90.0
    assert -180.0 <= fake.longitude() <= 180.0
    assert "." in fake.file_name()
    assert "/" in fake.mime_type()

    # Batch methods
    assert len(fake.credit_card_numbers(10)) == 10
    assert len(fake.credit_card_expires(10)) == 10
    assert len(fake.credit_card_security_codes(10)) == 10
    assert len(fake.credit_card_providers(10)) == 10
    assert len(fake.ibans(10)) == 10
    assert len(fake.currency_codes(10)) == 10
    assert len(fake.ipv6s(10)) == 10
    assert len(fake.mac_addresses(10)) == 10
    assert len(fake.domain_names(10)) == 10
    assert len(fake.user_agents(10)) == 10
    assert len(fake.slugs(10)) == 10
    assert len(fake.free_emails(10)) == 10
    assert len(fake.company_emails(10)) == 10
    assert len(fake.safe_emails(10)) == 10
    assert len(fake.hex_colors(10)) == 10
    assert len(fake.rgb_colors(10)) == 10
    assert len(fake.latitudes(10)) == 10
    assert len(fake.longitudes(10)) == 10
    assert len(fake.file_names(10)) == 10
    assert len(fake.mime_types(10)) == 10

    # Unique support
    assert len({fake.unique.credit_card_number() for _ in range(25)}) == 25
    assert len({fake.unique.iban() for _ in range(25)}) == 25
    assert len({fake.unique.ipv6() for _ in range(25)}) == 25
