use fastrand::Rng;
use pyo3::prelude::*;

use crate::data::{CITIES, FIRST_NAMES, LAST_NAMES, STATES, STREET_NAMES, STREET_SUFFIXES};

#[derive(Clone, Copy)]
pub(crate) struct LocaleData {
    pub code: &'static str,
    pub first_names: &'static [&'static str],
    pub last_names: &'static [&'static str],
    pub cities: &'static [&'static str],
    pub streets: &'static [&'static str],
    pub street_suffixes: &'static [&'static str],
    pub states: &'static [&'static str],
    pub country: &'static str,
}

pub(crate) static EN_US: LocaleData = LocaleData {
    code: "en_US",
    first_names: FIRST_NAMES,
    last_names: LAST_NAMES,
    cities: CITIES,
    streets: STREET_NAMES,
    street_suffixes: STREET_SUFFIXES,
    states: STATES,
    country: "United States",
};

pub(crate) static EN_GB: LocaleData = LocaleData {
    code: "en_GB",
    first_names: &[
        "Oliver", "George", "Harry", "Noah", "Jack", "Leo", "Arthur", "Muhammad",
        "Olivia", "Amelia", "Isla", "Ava", "Mia", "Ivy", "Lily", "Sophia",
        "Charlie", "Oscar", "Jacob", "Alfie", "Emily", "Poppy", "Grace", "Ella",
    ],
    last_names: &[
        "Smith", "Jones", "Taylor", "Brown", "Williams", "Wilson", "Johnson", "Davies",
        "Robinson", "Wright", "Thompson", "Evans", "Walker", "White", "Roberts", "Green",
        "Hall", "Wood", "Jackson", "Clarke", "Patel", "Khan", "Lewis", "Hughes",
    ],
    cities: &[
        "London", "Manchester", "Birmingham", "Leeds", "Glasgow", "Liverpool",
        "Bristol", "Sheffield", "Edinburgh", "Cardiff", "Belfast", "Oxford",
        "Cambridge", "York", "Bath", "Brighton", "Norwich", "Exeter",
    ],
    streets: &[
        "High", "Station", "Church", "Victoria", "London", "Park", "Green",
        "King", "Queen", "New", "Mill", "Manor", "School", "Bridge",
    ],
    street_suffixes: &["Street", "Road", "Lane", "Avenue", "Close", "Drive", "Way"],
    states: &["ENG", "SCT", "WLS", "NIR"],
    country: "United Kingdom",
};

pub(crate) static DE_DE: LocaleData = LocaleData {
    code: "de_DE",
    first_names: &[
        "Lukas", "Leon", "Maximilian", "Felix", "Paul", "Jonas", "Tim", "Julian",
        "Mia", "Emma", "Hannah", "Sofia", "Anna", "Lea", "Lena", "Marie",
        "Moritz", "Niklas", "Jan", "Tobias", "Laura", "Sarah", "Julia", "Katharina",
    ],
    last_names: &[
        "Mueller", "Schmidt", "Schneider", "Fischer", "Weber", "Meyer", "Wagner", "Becker",
        "Schulz", "Hoffmann", "Schaefer", "Koch", "Bauer", "Richter", "Klein", "Wolf",
        "Schroeder", "Neumann", "Schwarz", "Zimmermann", "Braun", "Krueger", "Hofmann", "Hartmann",
    ],
    cities: &[
        "Berlin", "Hamburg", "Muenchen", "Koeln", "Frankfurt", "Stuttgart",
        "Duesseldorf", "Leipzig", "Dortmund", "Essen", "Bremen", "Dresden",
        "Hannover", "Nuernberg", "Duisburg", "Bochum", "Bonn", "Muenster",
    ],
    streets: &[
        "Haupt", "Bahnhof", "Schul", "Garten", "Dorf", "Kirch", "Wald",
        "Ring", "Berg", "Linden", "Goethe", "Schiller", "Berliner", "Mittel",
    ],
    street_suffixes: &["strasse", "weg", "allee", "platz", "gasse", "ring"],
    states: &["BY", "NW", "BW", "NI", "HE", "BE", "HH", "SN", "RP", "SH"],
    country: "Germany",
};

pub(crate) static FR_FR: LocaleData = LocaleData {
    code: "fr_FR",
    first_names: &[
        "Lucas", "Hugo", "Gabriel", "Leo", "Raphael", "Arthur", "Louis", "Jules",
        "Emma", "Jade", "Louise", "Alice", "Chloe", "Lina", "Lea", "Manon",
        "Antoine", "Thomas", "Camille", "Sarah", "Nicolas", "Julien", "Claire", "Marie",
    ],
    last_names: &[
        "Martin", "Bernard", "Thomas", "Petit", "Robert", "Richard", "Durand", "Dubois",
        "Moreau", "Laurent", "Simon", "Michel", "Lefebvre", "Leroy", "Roux", "David",
        "Bertrand", "Morel", "Fournier", "Girard", "Bonnet", "Dupont", "Lambert", "Fontaine",
    ],
    cities: &[
        "Paris", "Marseille", "Lyon", "Toulouse", "Nice", "Nantes",
        "Strasbourg", "Montpellier", "Bordeaux", "Lille", "Rennes", "Reims",
        "Toulon", "Grenoble", "Dijon", "Angers", "Nimes", "Aix-en-Provence",
    ],
    streets: &[
        "Victor Hugo", "Jean Jaures", "Pasteur", "Republique", "Liberte", "Gambetta",
        "Voltaire", "Moliere", "Carnot", "Jules Ferry", "General de Gaulle", "Verdun",
    ],
    street_suffixes: &["Rue", "Avenue", "Boulevard", "Place", "Allee", "Chemin"],
    states: &["IDF", "ARA", "NAQ", "OCC", "HDF", "PAC", "GES", "BRE"],
    country: "France",
};

pub(crate) static ES_ES: LocaleData = LocaleData {
    code: "es_ES",
    first_names: &[
        "Hugo", "Mateo", "Martin", "Lucas", "Leo", "Daniel", "Alejandro", "Pablo",
        "Lucia", "Sofia", "Martina", "Maria", "Paula", "Julia", "Valeria", "Alba",
        "Carlos", "Javier", "Diego", "Adrian", "Elena", "Carmen", "Laura", "Marta",
    ],
    last_names: &[
        "Garcia", "Rodriguez", "Gonzalez", "Fernandez", "Lopez", "Martinez", "Sanchez", "Perez",
        "Gomez", "Martin", "Jimenez", "Ruiz", "Hernandez", "Diaz", "Moreno", "Munoz",
        "Alvarez", "Romero", "Alonso", "Gutierrez", "Navarro", "Torres", "Dominguez", "Vazquez",
    ],
    cities: &[
        "Madrid", "Barcelona", "Valencia", "Sevilla", "Zaragoza", "Malaga",
        "Murcia", "Palma", "Bilbao", "Alicante", "Cordoba", "Valladolid",
        "Vigo", "Gijon", "Granada", "Oviedo", "Santander", "Toledo",
    ],
    streets: &[
        "Mayor", "Real", "Sol", "Constitucion", "Cervantes", "Velazquez",
        "Goya", "Alcala", "Castellana", "Gran Via", "Colon", "Paz",
    ],
    street_suffixes: &["Calle", "Avenida", "Plaza", "Paseo", "Camino"],
    states: &["MD", "CT", "AN", "VC", "PV", "GA", "CL", "AR"],
    country: "Spain",
};

pub(crate) static JA_JP: LocaleData = LocaleData {
    code: "ja_JP",
    first_names: &[
        "Haruto", "Minato", "Yuto", "Sota", "Ren", "Riku", "Kaito", "Asahi",
        "Himari", "Hina", "Yua", "Sakura", "Ichika", "Akari", "Mei", "Yui",
        "Takumi", "Kenji", "Satoshi", "Daiki", "Aoi", "Nanami", "Ayaka", "Rina",
    ],
    last_names: &[
        "Sato", "Suzuki", "Takahashi", "Tanaka", "Watanabe", "Ito", "Yamamoto", "Nakamura",
        "Kobayashi", "Kato", "Yoshida", "Yamada", "Sasaki", "Yamaguchi", "Matsumoto", "Inoue",
        "Kimura", "Hayashi", "Shimizu", "Yamazaki", "Mori", "Abe", "Ikeda", "Hashimoto",
    ],
    cities: &[
        "Tokyo", "Yokohama", "Osaka", "Nagoya", "Sapporo", "Fukuoka",
        "Kobe", "Kawasaki", "Kyoto", "Saitama", "Hiroshima", "Sendai",
        "Chiba", "Kitakyushu", "Sakai", "Niigata", "Hamamatsu", "Kumamoto",
    ],
    streets: &[
        "Chuo", "Sakura", "Ginza", "Shinjuku", "Shibuya", "Umeda",
        "Namba", "Tenjin", "Aoba", "Minato", "Midori", "Asahi",
    ],
    street_suffixes: &["Dori", "Chome", "Machi", "Ku"],
    states: &["13", "27", "23", "14", "01", "40", "28", "26"],
    country: "Japan",
};

pub(crate) fn lookup_locale(code: &str) -> &'static LocaleData {
    let normalized = code.replace('-', "_");
    match normalized.as_str() {
        "en_GB" | "en_UK" => &EN_GB,
        "de_DE" | "de" => &DE_DE,
        "fr_FR" | "fr" => &FR_FR,
        "es_ES" | "es" => &ES_ES,
        "ja_JP" | "ja" => &JA_JP,
        _ => &EN_US,
    }
}

pub(crate) fn parse_locales(locale: Option<Bound<'_, PyAny>>) -> PyResult<Vec<&'static LocaleData>> {
    match locale {
        None => Ok(vec![&EN_US]),
        Some(obj) => {
            if let Ok(s) = obj.extract::<String>() {
                Ok(vec![lookup_locale(&s)])
            } else if let Ok(list) = obj.extract::<Vec<String>>() {
                if list.is_empty() {
                    Ok(vec![&EN_US])
                } else {
                    Ok(list.iter().map(|s| lookup_locale(s)).collect())
                }
            } else {
                Ok(vec![&EN_US])
            }
        }
    }
}

#[inline]
pub(crate) fn pick_locale(rng: &mut Rng, locales: &[&'static LocaleData]) -> &'static LocaleData {
    if locales.len() == 1 {
        locales[0]
    } else {
        locales[rng.usize(..locales.len())]
    }
}
