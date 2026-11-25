import csv
import datetime
import ipaddress
import re
import uuid
from collections import OrderedDict
from pathlib import Path

import pytest
from faker_rs import (
    BaseProvider,
    Dataset,
    Derived,
    Faker,
    ForeignKey,
    PrimaryKey,
    UniquenessException,
    Weighted,
    __version__,
)
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


def test_date_objects_and_ranges() -> None:
    fake = Faker(seed=505)

    slash_date = fake.date(pattern="%m/%d/%Y")
    assert re.match(r"^\d{2}/\d{2}/\d{4}$", slash_date)

    d_obj = fake.date_object()
    assert isinstance(d_obj, datetime.date) and not isinstance(d_obj, datetime.datetime)

    dt_obj = fake.date_time()
    assert isinstance(dt_obj, datetime.datetime)

    between = fake.date_between(start_date="2020-01-01", end_date="2020-12-31")
    assert datetime.date(2020, 1, 1) <= between <= datetime.date(2020, 12, 31)

    dob = fake.date_of_birth(minimum_age=20, maximum_age=30)
    assert isinstance(dob, datetime.date)
    today = datetime.date.today()
    age_days = (today - dob).days
    assert 20 * 365 <= age_days <= 31 * 365

    assert len(fake.dates(10, pattern="%Y/%m/%d")) == 10
    assert all(isinstance(x, datetime.date) for x in fake.date_objects(10))
    assert all(isinstance(x, datetime.datetime) for x in fake.date_times(10))
    assert len(fake.date_betweens(10, start_date="-5y", end_date="today")) == 10
    assert len(fake.date_of_births(10, minimum_age=18, maximum_age=65)) == 10


def test_locales_optional_and_add_provider() -> None:
    fr = Faker("fr_FR", seed=606)
    assert fr.locales == ["fr_FR"]
    assert isinstance(fr.name(), str)
    assert isinstance(fr.city(), str)

    multi = Faker(["en_GB", "ja_JP"], seed=607)
    assert multi.locales == ["en_GB", "ja_JP"]
    assert len(multi.names(20)) == 20

    # Optional proxy
    assert fr.optional.email(p=1.0) is not None
    assert fr.optional.email(p=0.0) is None
    results = [fr.optional.random_int(1, 10, p=0.5) for _ in range(100)]
    assert any(r is None for r in results) and any(r is not None for r in results)

    # Custom provider via add_provider
    class CustomProvider(BaseProvider):
        def custom_token(self) -> str:
            return f"tok-{self.generator.numerify('####')}"

    fr.add_provider(CustomProvider)
    tok = fr.custom_token()  # type: ignore[attr-defined]
    assert re.match(r"^tok-\d{4}$", tok)


def test_en_gb_and_locale_postcode_phone_address() -> None:
    uk_pc_re = re.compile(r"^[A-Z]{1,2}\d{1,2}[A-Z]?\s\d[A-Z]{2}$")
    uk_phone_re = re.compile(r"^(\+44 7\d{3} \d{6}|07\d{3} \d{6}|020 \d{4} \d{4}|01\d{3} \d{6})$")

    gb = Faker("en_GB", seed=701)
    for _ in range(50):
        pc = gb.postcode()
        assert uk_pc_re.match(pc), f"Invalid UK postcode: {pc}"
        ph = gb.phone_number()
        assert uk_phone_re.match(ph), f"Invalid UK phone: {ph}"
        addr = gb.address()
        assert "\n" in addr
        assert uk_pc_re.match(addr.split(", ")[-1]), f"UK address missing UK postcode: {addr}"

    assert all(uk_pc_re.match(p) for p in gb.postcodes(25))
    assert all(uk_phone_re.match(p) for p in gb.phone_numbers(25))
    assert uk_pc_re.match(gb.unique.postcode())
    assert uk_phone_re.match(gb.unique.phone_number())

    de = Faker("de_DE", seed=702)
    assert de.phone_number().startswith("+49 ")
    assert re.match(r"^\d{5}$", de.postcode())

    jp = Faker("ja_JP", seed=703)
    assert jp.phone_number().startswith("+81 ")
    assert re.match(r"^\d{3}-\d{4}$", jp.postcode())


def test_weighted_and_unique_sampling() -> None:
    fake = Faker(seed=704)
    weights = OrderedDict([("Operational", 0.66), ("Maintenance", 0.28), ("Offline_Failed", 0.06)])

    draws = [fake.random_element(weights) for _ in range(1000)]
    op_rate = draws.count("Operational") / len(draws)
    fail_rate = draws.count("Offline_Failed") / len(draws)
    assert 0.58 <= op_rate <= 0.74
    assert 0.02 <= fail_rate <= 0.12

    batch = fake.random_choices(
        ["Operational", "Maintenance", "Offline_Failed"], [66, 28, 6], k=500
    )
    assert len(batch) == 500
    assert batch.count("Operational") > batch.count("Offline_Failed") * 4

    # Weighted sampling without replacement
    uniq_weighted = fake.random_elements(weights, length=3, unique=True)
    assert set(uniq_weighted) == {"Operational", "Maintenance", "Offline_Failed"}

    # random_sample without replacement
    sample = fake.random_sample(list(range(20)), length=5)
    assert len(sample) == 5 and len(set(sample)) == 5


def test_bounded_datetime_and_iso8601_between() -> None:
    fake = Faker(seed=705)

    start = datetime.datetime(2026, 1, 1, 0, 0, 0)
    end = datetime.datetime(2026, 2, 15, 23, 59, 59)

    for _ in range(30):
        dt = fake.date_time_between(start_date=start, end_date=end)
        assert isinstance(dt, datetime.datetime)
        assert start <= dt <= end

        iso = fake.iso8601_between(start_date="2026-01-01T00:00:00", end_date="2026-02-15T23:59:59")
        parsed = datetime.datetime.fromisoformat(iso)
        assert start <= parsed <= end

    # Relative windows (last 45 days and next 14 days)
    past = fake.date_time_betweens(20, start_date="-45d", end_date="now")
    future = fake.iso8601_betweens(20, start_date="now", end_date="+14d")
    assert len(past) == 20 and all(isinstance(x, datetime.datetime) for x in past)
    assert len(future) == 20
    for s in future:
        datetime.datetime.fromisoformat(s)

    # date_between with datetime.date arguments
    d_start = datetime.date(2026, 3, 1)
    d_end = datetime.date(2026, 3, 31)
    d = fake.date_between(start_date=d_start, end_date=d_end)
    assert d_start <= d <= d_end


def test_custom_domain_and_name_derived_emails() -> None:
    fake = Faker("en_GB", seed=706)

    assert fake.email(domain="nhs.net").endswith("@nhs.net")
    assert fake.email(name="Dr. Sarah Connor", domain="nhs.net") == "sarah.connor@nhs.net"
    assert fake.company_email(domain="nhs.net", name="Prof. Alan Turing") == "alan.turing@nhs.net"
    assert re.match(
        r"^jane\.doe\d+@nhs\.net$",
        fake.email_from_name("Jane Doe", domain="nhs.net", add_digits=True),
    )
    assert all(e.endswith("@nhs.net") for e in fake.emails(20, domain="nhs.net"))
    assert all(e.endswith("@nhs.net") for e in fake.company_emails(20, domain="nhs.net"))

    # Unique email with duplicate name should disambiguate cleanly
    e1 = fake.unique.email(name="Jane Doe", domain="nhs.net")
    e2 = fake.unique.email(name="Jane Doe", domain="nhs.net")
    assert e1 == "jane.doe@nhs.net"
    assert e2 != e1 and e2.endswith("@nhs.net")


def test_relational_dataset_and_csv_export(tmp_path: Path) -> None:
    fake = Faker("en_GB", seed=707)
    ds = Dataset(fake)

    ds.add_table(
        "zones",
        rows=10,
        columns={
            "zone_id": PrimaryKey(prefix="ZN-", pad=3),
            "zone_type": Weighted({"Ward": 0.7, "Clinic": 0.2, "ICU_HDU": 0.1}),
            "lead_clinician_name": "name",
            "lead_clinician_email": Derived(
                lambda row, _: fake.email(name=row["lead_clinician_name"], domain="nhs.net")
            ),
        },
    )
    ds.add_table(
        "assets",
        rows=15,
        columns={
            "asset_id": PrimaryKey(prefix="AST-", pad=4),
            "status": Weighted({"Operational": 0.66, "Maintenance": 0.28, "Offline_Failed": 0.06}),
            "last_checked": lambda: fake.iso8601_between("-45d", "now"),
        },
    )
    links = ds.add_join_table(
        "link_asset_serves_zone",
        left_table="assets",
        left_pk="asset_id",
        right_table="zones",
        right_pk="zone_id",
        min_links=1,
        max_links=3,
    )
    ds.add_table(
        "work_orders",
        rows=20,
        columns={
            "wo_id": PrimaryKey(prefix="WO-", pad=4),
            "asset_id": ForeignKey("assets", "asset_id"),
            "created_at": lambda: fake.iso8601_between("-45d", "now"),
            "clinical_risk_score": Derived(
                lambda row, ctx: 10
                if next(a for a in ctx["assets"] if a["asset_id"] == row["asset_id"])["status"]
                == "Offline_Failed"
                else 2
            ),
        },
    )

    assert len(ds.tables["zones"]) == 10
    assert all(r["lead_clinician_email"].endswith("@nhs.net") for r in ds.tables["zones"])
    pairs = [(r["asset_id"], r["zone_id"]) for r in links]
    assert len(pairs) == len(set(pairs)) and len(pairs) >= 15
    assert all(r["clinical_risk_score"] in {2, 10} for r in ds.tables["work_orders"])

    written = ds.to_csv(tmp_path)
    assert set(written.keys()) == {"zones", "assets", "link_asset_serves_zone", "work_orders"}
    with (tmp_path / "zones.csv").open(encoding="utf-8") as fh:
        csv_rows = list(csv.DictReader(fh))
    assert len(csv_rows) == 10
    assert csv_rows[0]["zone_id"] == "ZN-001"
