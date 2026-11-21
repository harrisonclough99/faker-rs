pub const FIRST_NAMES: &[&str] = &[
    "James", "Mary", "Robert", "Patricia", "John", "Jennifer", "Michael", "Linda",
    "David", "Elizabeth", "William", "Barbara", "Richard", "Susan", "Joseph", "Jessica",
    "Thomas", "Sarah", "Christopher", "Karen", "Charles", "Lisa", "Daniel", "Nancy",
    "Matthew", "Betty", "Anthony", "Sandra", "Mark", "Margaret", "Donald", "Ashley",
    "Steven", "Kimberly", "Andrew", "Emily", "Paul", "Donna", "Joshua", "Michelle",
    "Kenneth", "Carol", "Kevin", "Amanda", "Brian", "Melissa", "George", "Deborah",
    "Timothy", "Stephanie", "Ronald", "Rebecca", "Jason", "Sharon", "Edward", "Laura",
    "Jeffrey", "Cynthia", "Ryan", "Dorothy", "Jacob", "Amy", "Gary", "Kathleen",
    "Nicholas", "Angela", "Eric", "Shirley", "Jonathan", "Emma", "Stephen", "Brenda",
    "Larry", "Pamela", "Justin", "Nicole", "Scott", "Samantha", "Brandon", "Katherine",
    "Benjamin", "Christine", "Samuel", "Debra", "Raymond", "Rachel", "Gregory", "Carolyn",
    "Frank", "Janet", "Alexander", "Maria", "Patrick", "Olivia", "Jack", "Heather",
];

pub const LAST_NAMES: &[&str] = &[
    "Smith", "Johnson", "Williams", "Brown", "Jones", "Garcia", "Miller", "Davis",
    "Rodriguez", "Martinez", "Hernandez", "Lopez", "Gonzalez", "Wilson", "Anderson",
    "Thomas", "Taylor", "Moore", "Jackson", "Martin", "Lee", "Perez", "Thompson",
    "White", "Harris", "Sanchez", "Clark", "Ramirez", "Lewis", "Robinson", "Walker",
    "Young", "Allen", "King", "Wright", "Scott", "Torres", "Nguyen", "Hill", "Flores",
    "Green", "Adams", "Nelson", "Baker", "Hall", "Rivera", "Campbell", "Mitchell",
    "Carter", "Roberts", "Gomez", "Phillips", "Evans", "Turner", "Diaz", "Parker",
    "Cruz", "Edwards", "Collins", "Reyes", "Stewart", "Morris", "Morales", "Murphy",
    "Cook", "Rogers", "Gutierrez", "Ortiz", "Morgan", "Cooper", "Peterson", "Bailey",
    "Reed", "Kelly", "Howard", "Ramos", "Kim", "Cox", "Ward", "Richardson", "Watson",
    "Brooks", "Chavez", "Wood", "James", "Bennett", "Gray", "Mendoza", "Ruiz",
    "Hughes", "Price", "Alvarez", "Castillo", "Sanders", "Patel", "Myers", "Long",
];

pub const EMAIL_DOMAINS: &[&str] = &[
    "example.com",
    "example.org",
    "example.net",
    "mail.example.com",
    "test.example.org",
];

pub const STREET_NAMES: &[&str] = &[
    "Main", "Oak", "Pine", "Maple", "Cedar", "Elm", "Washington", "Lake", "Hill",
    "Park", "View", "Sunset", "Ridge", "River", "Forest", "Meadow", "Spring", "Highland",
    "Valley", "Church", "Mill", "North", "South", "Lincoln", "Jefferson", "Madison",
    "Franklin", "Adams", "Jackson", "Monroe", "Center", "Willow", "Birch", "Chestnut",
    "Walnut", "Spruce", "Sycamore", "Poplar", "Hickory", "Aspen", "Cherry", "Dogwood",
];

pub const STREET_SUFFIXES: &[&str] = &[
    "St", "Ave", "Blvd", "Ln", "Rd", "Dr", "Way", "Ct", "Pl", "Ter", "Cir",
];

pub const CITIES: &[&str] = &[
    "Springfield", "Riverside", "Franklin", "Greenville", "Bristol", "Clinton",
    "Fairview", "Salem", "Madison", "Georgetown", "Arlington", "Centerville",
    "Oakland", "Burlington", "Manchester", "Milton", "Newport", "Ashland",
    "Dover", "Kingston", "Marion", "Oxford", "Troy", "Winchester", "Auburn",
    "Dayton", "Lexington", "Milford", "Mount Vernon", "Portsmouth", "Rochester",
    "Shelbyville", "Trenton", "Watertown", "Westfield", "Yorktown",
];

pub const STATES: &[&str] = &[
    "AL", "AK", "AZ", "AR", "CA", "CO", "CT", "DE", "FL", "GA", "HI", "ID", "IL",
    "IN", "IA", "KS", "KY", "LA", "ME", "MD", "MA", "MI", "MN", "MS", "MO", "MT",
    "NE", "NV", "NH", "NJ", "NM", "NY", "NC", "ND", "OH", "OK", "OR", "PA", "RI",
    "SC", "SD", "TN", "TX", "UT", "VT", "VA", "WA", "WV", "WI", "WY",
];

pub const SECONDARY_PREFIXES: &[&str] = &["Apt.", "Suite"];

pub const COUNTRIES: &[&str] = &[
    "United States", "Canada", "United Kingdom", "Germany", "France", "Australia",
    "Japan", "Brazil", "India", "Netherlands", "Sweden", "Norway", "Denmark",
    "Finland", "Switzerland", "Spain", "Italy", "Portugal", "Ireland", "New Zealand",
    "Singapore", "South Korea", "Mexico", "Argentina", "Poland", "Austria", "Belgium",
    "Czech Republic", "Greece", "Chile",
];

pub const COMPANY_WORDS: &[&str] = &[
    "Acme", "Apex", "Summit", "Vertex", "Horizon", "Vanguard", "Nexus", "Pinnacle",
    "Catalyst", "Meridian", "Stratos", "Quantum", "Aurora", "Beacon", "Crest", "Delta",
    "Echo", "Fusion", "Global", "Helix", "Ignite", "Keystone", "Lumen", "Matrix",
    "Nova", "Orbit", "Pulse", "Quasar", "Radiant", "Synergy", "Titan", "Vector",
    "Zenith", "Northstar", "Silverline", "Bluewave", "Ironclad", "Clearview",
];

pub const COMPANY_SUFFIXES: &[&str] = &[
    "Inc", "LLC", "Corp", "Group", "Solutions", "Systems", "Technologies",
    "Labs", "Partners", "Industries", "Dynamics", "Enterprises", "Ltd",
];

pub const JOBS: &[&str] = &[
    "Software Engineer", "Data Scientist", "Product Manager", "Systems Architect",
    "DevOps Engineer", "UX Designer", "Security Analyst", "Database Administrator",
    "Technical Writer", "Site Reliability Engineer", "Machine Learning Engineer",
    "Frontend Developer", "Backend Developer", "QA Engineer", "Cloud Engineer",
    "Business Analyst", "Project Manager", "Engineering Manager", "Solutions Architect",
    "Network Engineer", "Research Scientist", "Financial Analyst", "Marketing Manager",
    "Operations Manager", "Sales Engineer", "Customer Success Manager",
];

pub const TLDS: &[&str] = &["com", "org", "net", "io", "dev", "co"];

pub const LOREM_WORDS: &[&str] = &[
    "time", "way", "year", "work", "day", "man", "world", "life", "hand", "part",
    "child", "eye", "woman", "place", "week", "case", "point", "company", "number",
    "group", "problem", "fact", "data", "system", "network", "service", "model",
    "process", "result", "value", "design", "code", "build", "test", "deploy",
    "scale", "fast", "clean", "simple", "modern", "secure", "reliable", "open",
    "stream", "memory", "thread", "async", "cloud", "edge", "query", "index",
    "schema", "client", "server", "record", "batch", "event", "signal", "metric",
];

pub const CARD_PROVIDERS: &[&str] = &["Visa", "Mastercard", "American Express", "Discover"];

pub const CURRENCY_CODES: &[&str] = &[
    "USD", "EUR", "GBP", "JPY", "CAD", "AUD", "CHF", "CNY", "SEK", "NOK", "DKK", "NZD",
    "SGD", "HKD", "KRW", "INR", "BRL", "MXN", "PLN", "TRY",
];

pub const IBAN_PREFIXES: &[(&str, usize)] = &[
    ("DE", 22), ("FR", 27), ("GB", 22), ("ES", 24), ("IT", 27), ("NL", 18), ("BE", 16),
    ("CH", 21), ("SE", 24), ("NO", 15), ("PL", 28), ("PT", 25), ("AT", 20), ("IE", 22),
];

pub const FREE_EMAIL_DOMAINS: &[&str] = &["gmail.com", "yahoo.com", "hotmail.com", "outlook.com", "proton.me"];

pub const USER_AGENTS: &[&str] = &[
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 14_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.1 Safari/605.1.15",
    "Mozilla/5.0 (X11; Linux x86_64; rv:133.0) Gecko/20100101 Firefox/133.0",
    "Mozilla/5.0 (iPhone; CPU iPhone OS 18_1 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.1 Mobile/15E148 Safari/604.1",
    "Mozilla/5.0 (Linux; Android 15; Pixel 9) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Mobile Safari/537.36",
];

pub const FILE_EXTENSIONS: &[&str] = &[
    "txt", "csv", "json", "parquet", "pdf", "png", "jpg", "svg", "mp4", "mp3", "zip", "py", "rs",
];

pub const MIME_TYPES: &[&str] = &[
    "text/plain", "text/csv", "application/json", "application/pdf", "image/png",
    "image/jpeg", "image/svg+xml", "video/mp4", "audio/mpeg", "application/zip",
    "application/octet-stream", "text/html",
];
