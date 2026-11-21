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
