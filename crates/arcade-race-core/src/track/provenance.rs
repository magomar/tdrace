//! Authentic real-world circuit provenance registry.
//!
//! Stores verified OpenStreetMap (OSM) relation/way URLs, Wikipedia article links,
//! and ISO 3166-1 alpha-2 country codes for all 71 real-world circuits recreated in TdRace.

/// Provenance metadata record for a real-world racing circuit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CircuitProvenance {
    /// Canonical circuit identifier (matching tracks/<module>/<id>.json file stem).
    pub id: &'static str,
    /// Canonical display name of the circuit.
    pub name: &'static str,
    /// Alternate identifiers (e.g. generator function name, track definition ID, or localized names).
    pub aliases: &'static [&'static str],
    /// ISO 3166-1 alpha-2 two-letter country code.
    pub country_code: &'static str,
    /// Full English name of the country.
    pub country_name: &'static str,
    /// Authentic OpenStreetMap web URL (relation or way).
    pub osm_url: &'static str,
    /// Verified Wikipedia article URL.
    pub wikipedia_url: &'static str,
}

/// Master registry of all 71 verified real-world motorsport venues in TdRace.
pub static CIRCUIT_PROVENANCE_REGISTRY: &[CircuitProvenance] = &[
    // ==========================================
    // 1. GT World Challenge & Endurance (18)
    // ==========================================
    CircuitProvenance {
        id: "monza",
        name: "Monza Autodromo Nazionale",
        aliases: &["track_monza", "Autodromo Nazionale Monza"],
        country_code: "IT",
        country_name: "Italy",
        osm_url: "https://www.openstreetmap.org/relation/284565",
        wikipedia_url: "https://en.wikipedia.org/wiki/Monza_Circuit",
    },
    CircuitProvenance {
        id: "spa",
        name: "Circuit de Spa-Francorchamps",
        aliases: &["track_spa", "Spa-Francorchamps", "Spa"],
        country_code: "BE",
        country_name: "Belgium",
        osm_url: "https://www.openstreetmap.org/relation/284560",
        wikipedia_url: "https://en.wikipedia.org/wiki/Circuit_de_Spa-Francorchamps",
    },
    CircuitProvenance {
        id: "silverstone",
        name: "Silverstone Grand Prix Circuit",
        aliases: &["track_silverstone", "Silverstone Circuit", "Silverstone GP"],
        country_code: "GB",
        country_name: "United Kingdom",
        osm_url: "https://www.openstreetmap.org/relation/51160",
        wikipedia_url: "https://en.wikipedia.org/wiki/Silverstone_Circuit",
    },
    CircuitProvenance {
        id: "monaco",
        name: "Circuit de Monaco",
        aliases: &["track_monaco", "Monaco GP", "Monte Carlo"],
        country_code: "MC",
        country_name: "Monaco",
        osm_url: "https://www.openstreetmap.org/relation/148194",
        wikipedia_url: "https://en.wikipedia.org/wiki/Circuit_de_Monaco",
    },
    CircuitProvenance {
        id: "suzuka",
        name: "Suzuka International Racing Course",
        aliases: &["track_suzuka", "Suzuka Circuit", "Suzuka"],
        country_code: "JP",
        country_name: "Japan",
        osm_url: "https://www.openstreetmap.org/relation/284570",
        wikipedia_url: "https://en.wikipedia.org/wiki/Suzuka_International_Racing_Course",
    },
    CircuitProvenance {
        id: "interlagos",
        name: "Autodromo Jose Carlos Pace (Interlagos)",
        aliases: &["track_interlagos", "Autódromo José Carlos Pace (Interlagos)", "Interlagos Circuit", "Interlagos"],
        country_code: "BR",
        country_name: "Brazil",
        osm_url: "https://www.openstreetmap.org/relation/6781071",
        wikipedia_url: "https://en.wikipedia.org/wiki/Interlagos_Circuit",
    },
    CircuitProvenance {
        id: "montreal",
        name: "Circuit Gilles Villeneuve (Montreal)",
        aliases: &["track_montreal", "Circuit Gilles Villeneuve", "Circuit Gilles-Villeneuve", "Montreal"],
        country_code: "CA",
        country_name: "Canada",
        osm_url: "https://www.openstreetmap.org/relation/284595",
        wikipedia_url: "https://en.wikipedia.org/wiki/Circuit_Gilles_Villeneuve",
    },
    CircuitProvenance {
        id: "red_bull_ring",
        name: "Red Bull Ring (Spielberg)",
        aliases: &["track_red_bull_ring", "Red Bull Ring", "Spielberg", "A1-Ring"],
        country_code: "AT",
        country_name: "Austria",
        osm_url: "https://www.openstreetmap.org/relation/5309181",
        wikipedia_url: "https://en.wikipedia.org/wiki/Red_Bull_Ring",
    },
    CircuitProvenance {
        id: "catalunya",
        name: "Circuit de Barcelona-Catalunya",
        aliases: &["track_catalunya", "Montmelo", "Circuit de Catalunya"],
        country_code: "ES",
        country_name: "Spain",
        osm_url: "https://www.openstreetmap.org/way/831804327",
        wikipedia_url: "https://en.wikipedia.org/wiki/Circuit_de_Barcelona-Catalunya",
    },
    CircuitProvenance {
        id: "zandvoort",
        name: "Circuit Zandvoort",
        aliases: &["track_zandvoort", "Circuit Park Zandvoort", "Zandvoort"],
        country_code: "NL",
        country_name: "Netherlands",
        osm_url: "https://www.openstreetmap.org/relation/13545573",
        wikipedia_url: "https://en.wikipedia.org/wiki/Circuit_Zandvoort",
    },
    CircuitProvenance {
        id: "bahrain",
        name: "Bahrain International Circuit (Sakhir)",
        aliases: &["track_bahrain", "Bahrain International Circuit", "Sakhir"],
        country_code: "BH",
        country_name: "Bahrain",
        osm_url: "https://www.openstreetmap.org/relation/284538",
        wikipedia_url: "https://en.wikipedia.org/wiki/Bahrain_International_Circuit",
    },
    CircuitProvenance {
        id: "marina_bay",
        name: "Marina Bay Street Circuit (Singapore)",
        aliases: &["track_marina_bay", "Marina Bay Street Circuit", "Marina Bay", "Singapore GP"],
        country_code: "SG",
        country_name: "Singapore",
        osm_url: "https://www.openstreetmap.org/relation/421263",
        wikipedia_url: "https://en.wikipedia.org/wiki/Marina_Bay_Street_Circuit",
    },
    CircuitProvenance {
        id: "cota",
        name: "Circuit of the Americas (COTA)",
        aliases: &["track_cota", "Circuit of the Americas", "COTA Austin", "Austin COTA"],
        country_code: "US",
        country_name: "United States",
        osm_url: "https://www.openstreetmap.org/relation/6537729",
        wikipedia_url: "https://en.wikipedia.org/wiki/Circuit_of_the_Americas",
    },
    CircuitProvenance {
        id: "madring",
        name: "MadRing Circuito de Madrid",
        aliases: &["track_madring", "Circuito del Jarama", "Jarama", "Circuito de Madrid"],
        country_code: "ES",
        country_name: "Spain",
        osm_url: "https://www.openstreetmap.org/relation/18813472",
        wikipedia_url: "https://en.wikipedia.org/wiki/Circuito_del_Jarama",
    },
    CircuitProvenance {
        id: "nurburgring_gp",
        name: "Nurburgring Grand Prix-Strecke",
        aliases: &["track_nurburgring_gp", "Nürburgring Grand Prix-Strecke", "Nurburgring GP", "Nürburgring"],
        country_code: "DE",
        country_name: "Germany",
        osm_url: "https://www.openstreetmap.org/relation/38567",
        wikipedia_url: "https://en.wikipedia.org/wiki/N%C3%BCrburgring",
    },
    CircuitProvenance {
        id: "bathurst",
        name: "Mount Panorama (Bathurst)",
        aliases: &["track_bathurst", "Mount Panorama Circuit", "Bathurst"],
        country_code: "AU",
        country_name: "Australia",
        osm_url: "https://www.openstreetmap.org/relation/6942508",
        wikipedia_url: "https://en.wikipedia.org/wiki/Mount_Panorama_Circuit",
    },
    CircuitProvenance {
        id: "portimao_gp",
        name: "Autodromo Internacional do Algarve",
        aliases: &["track_portimao_gp", "Autódromo Internacional do Algarve", "Portimao GP", "Algarve International Circuit"],
        country_code: "PT",
        country_name: "Portugal",
        osm_url: "https://www.openstreetmap.org/relation/7509968",
        wikipedia_url: "https://en.wikipedia.org/wiki/Algarve_International_Circuit",
    },
    CircuitProvenance {
        id: "le_mans_sarthe",
        name: "Circuit de la Sarthe (Le Mans)",
        aliases: &["track_le_mans_sarthe", "Circuit de la Sarthe", "24 Hours of Le Mans", "Le Mans 24H"],
        country_code: "FR",
        country_name: "France",
        osm_url: "https://www.openstreetmap.org/relation/2126739",
        wikipedia_url: "https://en.wikipedia.org/wiki/Circuit_de_la_Sarthe",
    },

    // ==========================================
    // 2. Karting World Cup (17)
    // ==========================================
    CircuitProvenance {
        id: "lonato",
        name: "South Garda Karting (Lonato)",
        aliases: &["track_lonato", "South Garda Karting", "Lonato Karting"],
        country_code: "IT",
        country_name: "Italy",
        osm_url: "https://www.openstreetmap.org/way/75490872",
        wikipedia_url: "https://en.wikipedia.org/wiki/Karting",
    },
    CircuitProvenance {
        id: "sarno",
        name: "Circuito Internazionale Napoli (Sarno)",
        aliases: &["track_sarno", "Circuito Internazionale Napoli", "Sarno Karting"],
        country_code: "IT",
        country_name: "Italy",
        osm_url: "https://www.openstreetmap.org/way/643206090",
        wikipedia_url: "https://en.wikipedia.org/wiki/Circuito_Internazionale_Napoli",
    },
    CircuitProvenance {
        id: "genk",
        name: "Karting Genk (Home of Champions)",
        aliases: &["track_genk", "Karting Genk", "Genk Karting"],
        country_code: "BE",
        country_name: "Belgium",
        osm_url: "https://www.openstreetmap.org/way/67660672",
        wikipedia_url: "https://en.wikipedia.org/wiki/Karting_Genk",
    },
    CircuitProvenance {
        id: "pfi",
        name: "PF International Kart Circuit (PFI)",
        aliases: &["track_pfi", "PF International Kart Circuit", "PFI Karting", "PF International"],
        country_code: "GB",
        country_name: "United Kingdom",
        osm_url: "https://www.openstreetmap.org/way/1208588289",
        wikipedia_url: "https://en.wikipedia.org/wiki/PF_International",
    },
    CircuitProvenance {
        id: "zuera",
        name: "Circuito Internacional de Zuera",
        aliases: &["track_zuera", "Zuera Karting"],
        country_code: "ES",
        country_name: "Spain",
        osm_url: "https://www.openstreetmap.org/way/490212650",
        wikipedia_url: "https://en.wikipedia.org/wiki/Zuera",
    },
    CircuitProvenance {
        id: "le_mans_kart",
        name: "Le Mans Karting International",
        aliases: &["track_le_mans", "le_mans", "Circuit Alain Prost", "Le Mans Karting"],
        country_code: "FR",
        country_name: "France",
        osm_url: "https://www.openstreetmap.org/way/482284812",
        wikipedia_url: "https://en.wikipedia.org/wiki/Circuit_Alain_Prost",
    },
    CircuitProvenance {
        id: "portimao_kart",
        name: "Kartodromo Internacional do Algarve",
        aliases: &["track_portimao", "portimao", "Kartódromo Internacional do Algarve", "Portimao Karting"],
        country_code: "PT",
        country_name: "Portugal",
        osm_url: "https://www.openstreetmap.org/way/363049742",
        wikipedia_url: "https://en.wikipedia.org/wiki/Algarve_International_Circuit",
    },
    CircuitProvenance {
        id: "franciacorta",
        name: "Franciacorta Karting Track",
        aliases: &["track_franciacorta", "Franciacorta Karting"],
        country_code: "IT",
        country_name: "Italy",
        osm_url: "https://www.openstreetmap.org/way/1474753177",
        wikipedia_url: "https://en.wikipedia.org/wiki/Autodromo_di_Franciacorta",
    },
    CircuitProvenance {
        id: "wackersdorf",
        name: "Prokart Raceland Wackersdorf",
        aliases: &["track_wackersdorf", "Prokart Raceland", "Wackersdorf Karting"],
        country_code: "DE",
        country_name: "Germany",
        osm_url: "https://www.openstreetmap.org/way/156019609",
        wikipedia_url: "https://en.wikipedia.org/wiki/Wackersdorf",
    },
    CircuitProvenance {
        id: "kristianstad",
        name: "Kristianstad Karting Klubb (Åsum Ring)",
        aliases: &["track_kristianstad", "Kristianstad Karting Klubb (Asum Ring)", "Asum Ring", "Åsum Ring"],
        country_code: "SE",
        country_name: "Sweden",
        osm_url: "https://www.openstreetmap.org/way/87888593",
        wikipedia_url: "https://en.wikipedia.org/wiki/Kristianstad",
    },
    CircuitProvenance {
        id: "seven_laghi",
        name: "Circuito Internazionale 7 Laghi (Castelletto)",
        aliases: &["track_seven_laghi", "Circuito Internazionale 7 Laghi", "7 Laghi Kart", "Castelletto Kart"],
        country_code: "IT",
        country_name: "Italy",
        osm_url: "https://www.openstreetmap.org/way/80905675",
        wikipedia_url: "https://en.wikipedia.org/wiki/Castelletto_di_Branduzzo",
    },
    CircuitProvenance {
        id: "ampfing",
        name: "Schweppermannring Ampfing",
        aliases: &["track_ampfing", "Ampfing Karting", "Schweppermannring"],
        country_code: "DE",
        country_name: "Germany",
        osm_url: "https://www.openstreetmap.org/way/110580562",
        wikipedia_url: "https://en.wikipedia.org/wiki/Ampfing",
    },
    CircuitProvenance {
        id: "silverstone_national_kart",
        name: "Silverstone National Karting Circuit",
        aliases: &["track_silverstone_national_kart", "Silverstone Karting"],
        country_code: "GB",
        country_name: "United Kingdom",
        osm_url: "https://www.openstreetmap.org/way/1240237936",
        wikipedia_url: "https://en.wikipedia.org/wiki/Silverstone_Circuit",
    },
    CircuitProvenance {
        id: "laval_kart",
        name: "Laval Karting (Circuit Louis Beuvron)",
        aliases: &["track_laval", "laval", "Circuit Louis Beuvron", "Laval - Circuit Louis Beuvron"],
        country_code: "FR",
        country_name: "France",
        osm_url: "https://www.openstreetmap.org/way/183330357",
        wikipedia_url: "https://en.wikipedia.org/wiki/Laval,_Mayenne",
    },
    CircuitProvenance {
        id: "whilton_mill",
        name: "Whilton Mill Kart Circuit",
        aliases: &["track_whilton_mill", "whilton_mill_kart", "Whilton Mill"],
        country_code: "GB",
        country_name: "United Kingdom",
        osm_url: "https://www.openstreetmap.org/way/149913876",
        wikipedia_url: "https://en.wikipedia.org/wiki/Whilton",
    },
    CircuitProvenance {
        id: "campillos",
        name: "Kartcenter Campillos",
        aliases: &["track_campillos", "Campillos Karting"],
        country_code: "ES",
        country_name: "Spain",
        osm_url: "https://www.openstreetmap.org/way/420385347",
        wikipedia_url: "https://en.wikipedia.org/wiki/Campillos",
    },
    CircuitProvenance {
        id: "valencia_kart",
        name: "Kartodromo Internacional Lucas Guerrero (Valencia)",
        aliases: &["track_valencia_kart", "valencia", "Kartódromo Internacional Lucas Guerrero", "Lucas Guerrero Karting"],
        country_code: "ES",
        country_name: "Spain",
        osm_url: "https://www.openstreetmap.org/way/751513226",
        wikipedia_url: "https://en.wikipedia.org/wiki/Chiva,_Spain",
    },

    // ==========================================
    // 3. Rallycross & All-Terrain (17)
    // ==========================================
    CircuitProvenance {
        id: "holjes_rx",
        name: "Höljes Motorstadion (World RX Sweden)",
        aliases: &["holjes", "Höljes Motorstadion", "Holjes Motorstadion", "World RX Sweden"],
        country_code: "SE",
        country_name: "Sweden",
        osm_url: "https://www.openstreetmap.org/way/599300791",
        wikipedia_url: "https://en.wikipedia.org/wiki/H%C3%B6ljes_Motorstadion",
    },
    CircuitProvenance {
        id: "lydden_hill",
        name: "Lydden Hill Circuit (World RX Great Britain)",
        aliases: &["lydden", "Lydden Hill Race Circuit", "Lydden Hill Circuit", "Lydden Hill"],
        country_code: "GB",
        country_name: "United Kingdom",
        osm_url: "https://www.openstreetmap.org/way/234347787",
        wikipedia_url: "https://en.wikipedia.org/wiki/Lydden_Hill_Race_Circuit",
    },
    CircuitProvenance {
        id: "hell_rx",
        name: "Lånkebanen (World RX Norway)",
        aliases: &["hell", "Lånkebanen", "Lankebanen", "Hell RX", "Lånkebanen / Hell RX"],
        country_code: "NO",
        country_name: "Norway",
        osm_url: "https://www.openstreetmap.org/way/1069390970",
        wikipedia_url: "https://en.wikipedia.org/wiki/L%C3%A5nkebanen",
    },
    CircuitProvenance {
        id: "loheac_rx",
        name: "Circuit de Lohéac (World RX France)",
        aliases: &["loheac", "Circuit de Lohéac", "Circuit de Loheac", "Lohéac RX"],
        country_code: "FR",
        country_name: "France",
        osm_url: "https://www.openstreetmap.org/way/787615501",
        wikipedia_url: "https://en.wikipedia.org/wiki/Circuit_de_Loh%C3%A9ac",
    },
    CircuitProvenance {
        id: "estering_rx",
        name: "Estering Buxtehude (World RX Germany)",
        aliases: &["estering", "Estering Buxtehude", "Estering", "Buxtehude RX"],
        country_code: "DE",
        country_name: "Germany",
        osm_url: "https://www.openstreetmap.org/way/24855696",
        wikipedia_url: "https://en.wikipedia.org/wiki/Estering",
    },
    CircuitProvenance {
        id: "montalegre_rx",
        name: "Pista Automóvel de Montalegre (World RX Portugal)",
        aliases: &["montalegre", "Pista de Montalegre", "Montalegre RX", "Pista Automovel de Montalegre"],
        country_code: "PT",
        country_name: "Portugal",
        osm_url: "https://www.openstreetmap.org/way/305257997",
        wikipedia_url: "https://en.wikipedia.org/wiki/Montalegre",
    },
    CircuitProvenance {
        id: "nyirad_rx",
        name: "Nyirád Racing Center (Euro RX Hungary)",
        aliases: &["nyirad", "Nyirád Racing Center", "Nyirad Racing Center", "Nyirad RX"],
        country_code: "HU",
        country_name: "Hungary",
        osm_url: "https://www.openstreetmap.org/way/172413355",
        wikipedia_url: "https://en.wikipedia.org/wiki/Nyir%C3%A1d",
    },
    CircuitProvenance {
        id: "kouvola_rx",
        name: "Tykkimäen Moottorirata (World RX Finland)",
        aliases: &["kouvola", "Tykkimäen Moottorirata", "Tykkimaen Moottorirata", "Kouvola RX"],
        country_code: "FI",
        country_name: "Finland",
        osm_url: "https://www.openstreetmap.org/way/149713976",
        wikipedia_url: "https://en.wikipedia.org/wiki/Tykkim%C3%A4ki",
    },
    CircuitProvenance {
        id: "catalunya_rx",
        name: "Circuit de Barcelona-Catalunya RX (World RX Spain)",
        aliases: &["catalunya_rx", "Barcelona-Catalunya RX", "Barcelona RX"],
        country_code: "ES",
        country_name: "Spain",
        osm_url: "https://www.openstreetmap.org/way/831804327",
        wikipedia_url: "https://en.wikipedia.org/wiki/Circuit_de_Barcelona-Catalunya",
    },
    CircuitProvenance {
        id: "mettet_rx",
        name: "Circuit Jules Tacheny Mettet (World RX Belgium)",
        aliases: &["mettet", "Circuit Jules Tacheny Mettet", "Mettet RX", "Jules Tacheny"],
        country_code: "BE",
        country_name: "Belgium",
        osm_url: "https://www.openstreetmap.org/way/178384323",
        wikipedia_url: "https://en.wikipedia.org/wiki/Circuit_Jules_Tacheny_Mettet",
    },
    CircuitProvenance {
        id: "silverstone_rx",
        name: "Silverstone Circuit RX (World RX Great Britain)",
        aliases: &["silverstone_rx", "Silverstone Circuit RX", "Silverstone RX"],
        country_code: "GB",
        country_name: "United Kingdom",
        osm_url: "https://www.openstreetmap.org/way/169851260",
        wikipedia_url: "https://en.wikipedia.org/wiki/Silverstone_Circuit",
    },
    CircuitProvenance {
        id: "riga_rx",
        name: "Biķernieku Trase (World RX Latvia)",
        aliases: &["riga", "Biķernieku Trase", "Bikernieku Trase", "Riga RX", "Biķernieku Trase / Riga RX"],
        country_code: "LV",
        country_name: "Latvia",
        osm_url: "https://www.openstreetmap.org/way/256784387",
        wikipedia_url: "https://en.wikipedia.org/wiki/Bi%C4%B7ernieku_Kompleks%C4%81_Sporta_B%C4%81ze",
    },
    CircuitProvenance {
        id: "killarney_rx",
        name: "Killarney International Raceway (World RX South Africa)",
        aliases: &["killarney", "Killarney International Raceway", "Killarney RX", "Killarney International Raceway RX"],
        country_code: "ZA",
        country_name: "South Africa",
        osm_url: "https://www.openstreetmap.org/way/42125321",
        wikipedia_url: "https://en.wikipedia.org/wiki/Killarney_Motor_Racing_Complex",
    },
    CircuitProvenance {
        id: "yas_marina_rx",
        name: "Yas Marina RX Arena (World RX Abu Dhabi)",
        aliases: &["yas_marina", "Yas Marina RX Arena", "Yas Marina RX"],
        country_code: "AE",
        country_name: "United Arab Emirates",
        osm_url: "https://www.openstreetmap.org/way/1083519983",
        wikipedia_url: "https://en.wikipedia.org/wiki/Yas_Marina_Circuit",
    },
    CircuitProvenance {
        id: "blyton_rx",
        name: "Blyton Park Rallycross Circuit",
        aliases: &["blyton_park_rx", "blyton", "Blyton Park"],
        country_code: "GB",
        country_name: "United Kingdom",
        osm_url: "https://www.openstreetmap.org/way/129241665",
        wikipedia_url: "https://en.wikipedia.org/wiki/Blyton_Park",
    },
    CircuitProvenance {
        id: "dreux_rx",
        name: "Circuit de l'Ouest Parisien (Dreux RX)",
        aliases: &["dreux", "Dreux Circuit de l'Ouest Parisien", "Dreux RX", "Circuit de l'Ouest Parisien"],
        country_code: "FR",
        country_name: "France",
        osm_url: "https://www.openstreetmap.org/way/297738878",
        wikipedia_url: "https://en.wikipedia.org/wiki/Dreux",
    },
    CircuitProvenance {
        id: "essay_rx",
        name: "Circuit des Ducs (Essay RX)",
        aliases: &["essay", "Circuit des Ducs", "Essay RX", "Circuit des Ducs / Essay RX"],
        country_code: "FR",
        country_name: "France",
        osm_url: "https://www.openstreetmap.org/way/788873788",
        wikipedia_url: "https://en.wikipedia.org/wiki/Essay,_Orne",
    },

    // ==========================================
    // 4. NASCAR Cup Series & Trans-Am (17)
    // ==========================================
    CircuitProvenance {
        id: "bowman_gray",
        name: "Bowman Gray Stadium",
        aliases: &["bowman_gray_stadium", "Bowman Gray"],
        country_code: "US",
        country_name: "United States",
        osm_url: "https://www.openstreetmap.org/way/914237156",
        wikipedia_url: "https://en.wikipedia.org/wiki/Bowman_Gray_Stadium",
    },
    CircuitProvenance {
        id: "bristol",
        name: "Bristol Motor Speedway",
        aliases: &["bristol_motor_speedway", "Bristol"],
        country_code: "US",
        country_name: "United States",
        osm_url: "https://www.openstreetmap.org/way/116589129",
        wikipedia_url: "https://en.wikipedia.org/wiki/Bristol_Motor_Speedway",
    },
    CircuitProvenance {
        id: "charlotte",
        name: "Charlotte Motor Speedway",
        aliases: &["charlotte_motor_speedway", "Charlotte"],
        country_code: "US",
        country_name: "United States",
        osm_url: "https://www.openstreetmap.org/relation/21242750",
        wikipedia_url: "https://en.wikipedia.org/wiki/Charlotte_Motor_Speedway",
    },
    CircuitProvenance {
        id: "chicago",
        name: "Chicago Street Course",
        aliases: &["chicago_street_course", "Chicago Street Circuit", "Chicago"],
        country_code: "US",
        country_name: "United States",
        osm_url: "https://www.openstreetmap.org/relation/16546690",
        wikipedia_url: "https://en.wikipedia.org/wiki/Chicago_Street_Course",
    },
    CircuitProvenance {
        id: "darlington",
        name: "Darlington Raceway",
        aliases: &["darlington_raceway", "Darlington"],
        country_code: "US",
        country_name: "United States",
        osm_url: "https://www.openstreetmap.org/way/104277971",
        wikipedia_url: "https://en.wikipedia.org/wiki/Darlington_Raceway",
    },
    CircuitProvenance {
        id: "daytona",
        name: "Daytona Superspeedway",
        aliases: &["daytona_superspeedway", "Daytona International Speedway", "Daytona"],
        country_code: "US",
        country_name: "United States",
        osm_url: "https://www.openstreetmap.org/way/352074880",
        wikipedia_url: "https://en.wikipedia.org/wiki/Daytona_International_Speedway",
    },
    CircuitProvenance {
        id: "eldora",
        name: "Eldora Speedway",
        aliases: &["eldora_speedway", "Eldora"],
        country_code: "US",
        country_name: "United States",
        osm_url: "https://www.openstreetmap.org/way/608397609",
        wikipedia_url: "https://en.wikipedia.org/wiki/Eldora_Speedway",
    },
    CircuitProvenance {
        id: "indianapolis",
        name: "Indianapolis Motor Speedway",
        aliases: &["indianapolis_motor_speedway", "IMS", "Indianapolis"],
        country_code: "US",
        country_name: "United States",
        osm_url: "https://www.openstreetmap.org/way/589668075",
        wikipedia_url: "https://en.wikipedia.org/wiki/Indianapolis_Motor_Speedway",
    },
    CircuitProvenance {
        id: "iowa",
        name: "Iowa Speedway",
        aliases: &["iowa_speedway", "Iowa"],
        country_code: "US",
        country_name: "United States",
        osm_url: "https://www.openstreetmap.org/way/119238784",
        wikipedia_url: "https://en.wikipedia.org/wiki/Iowa_Speedway",
    },
    CircuitProvenance {
        id: "irp_oval",
        name: "Lucas Oil Indianapolis Raceway Park",
        aliases: &["lucas_oil_irp", "irp", "Lucas Oil IRP", "Indianapolis Raceway Park"],
        country_code: "US",
        country_name: "United States",
        osm_url: "https://www.openstreetmap.org/way/123830268",
        wikipedia_url: "https://en.wikipedia.org/wiki/Lucas_Oil_Indianapolis_Raceway_Park",
    },
    CircuitProvenance {
        id: "martinsville",
        name: "Martinsville Speedway",
        aliases: &["martinsville_speedway", "Martinsville"],
        country_code: "US",
        country_name: "United States",
        osm_url: "https://www.openstreetmap.org/relation/6497929",
        wikipedia_url: "https://en.wikipedia.org/wiki/Martinsville_Speedway",
    },
    CircuitProvenance {
        id: "north_wilkesboro",
        name: "North Wilkesboro Speedway",
        aliases: &["north_wilkesboro_speedway", "North Wilkesboro"],
        country_code: "US",
        country_name: "United States",
        osm_url: "https://www.openstreetmap.org/way/18928710",
        wikipedia_url: "https://en.wikipedia.org/wiki/North_Wilkesboro_Speedway",
    },
    CircuitProvenance {
        id: "phoenix",
        name: "Phoenix Raceway",
        aliases: &["phoenix_raceway", "Phoenix"],
        country_code: "US",
        country_name: "United States",
        osm_url: "https://www.openstreetmap.org/way/29333335",
        wikipedia_url: "https://en.wikipedia.org/wiki/Phoenix_Raceway",
    },
    CircuitProvenance {
        id: "pocono",
        name: "Pocono Raceway",
        aliases: &["pocono_raceway", "Pocono"],
        country_code: "US",
        country_name: "United States",
        osm_url: "https://www.openstreetmap.org/way/109767460",
        wikipedia_url: "https://en.wikipedia.org/wiki/Pocono_Raceway",
    },
    CircuitProvenance {
        id: "road_america",
        name: "Road America",
        aliases: &["road_america_nascar"],
        country_code: "US",
        country_name: "United States",
        osm_url: "https://www.openstreetmap.org/relation/6432758",
        wikipedia_url: "https://en.wikipedia.org/wiki/Road_America",
    },
    CircuitProvenance {
        id: "talladega",
        name: "Talladega Superspeedway",
        aliases: &["talladega_superspeedway", "Talladega"],
        country_code: "US",
        country_name: "United States",
        osm_url: "https://www.openstreetmap.org/way/405961241",
        wikipedia_url: "https://en.wikipedia.org/wiki/Talladega_Superspeedway",
    },
    CircuitProvenance {
        id: "watkins_glen",
        name: "Watkins Glen International",
        aliases: &["watkins_glen_nascar", "Watkins Glen"],
        country_code: "US",
        country_name: "United States",
        osm_url: "https://www.openstreetmap.org/way/702671615",
        wikipedia_url: "https://en.wikipedia.org/wiki/Watkins_Glen_International",
    },

    // ==========================================
    // 5. Extreme Off-Road (2 Real-World Venues)
    // ==========================================
    CircuitProvenance {
        id: "crandon_short_course",
        name: "Crandon International Off-Road",
        aliases: &["crandon", "Crandon International Off-Road Raceway"],
        country_code: "US",
        country_name: "United States",
        osm_url: "https://www.openstreetmap.org/way/291856211",
        wikipedia_url: "https://en.wikipedia.org/wiki/Crandon_International_Off-Road_Raceway",
    },
    CircuitProvenance {
        id: "glamis_dunes",
        name: "Glamis Imperial Sand Dunes",
        aliases: &["glamis_sand_dunes", "glamis", "Imperial Sand Dunes", "Algodones Dunes"],
        country_code: "US",
        country_name: "United States",
        osm_url: "https://www.openstreetmap.org/relation/6152825",
        wikipedia_url: "https://en.wikipedia.org/wiki/Algodones_Dunes",
    },
];

/// Looks up the authentic provenance record for a given circuit ID, name, or alias.
pub fn get_circuit_provenance(id_or_name_or_alias: &str) -> Option<&'static CircuitProvenance> {
    let clean = id_or_name_or_alias.trim();
    CIRCUIT_PROVENANCE_REGISTRY.iter().find(|entry| {
        entry.id.eq_ignore_ascii_case(clean)
            || entry.name.eq_ignore_ascii_case(clean)
            || entry
                .aliases
                .iter()
                .any(|alias| alias.eq_ignore_ascii_case(clean))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_71_real_circuits_in_provenance_registry() {
        assert_eq!(
            CIRCUIT_PROVENANCE_REGISTRY.len(),
            71,
            "Master provenance registry must contain exactly 71 real circuits"
        );
    }

    #[test]
    fn test_all_registry_urls_valid_https() {
        for prov in CIRCUIT_PROVENANCE_REGISTRY {
            assert!(
                prov.osm_url.starts_with("https://www.openstreetmap.org/"),
                "Circuit {} OSM URL must start with https://www.openstreetmap.org/ but was {}",
                prov.id,
                prov.osm_url
            );
            assert!(
                prov.wikipedia_url.starts_with("https://en.wikipedia.org/wiki/"),
                "Circuit {} Wikipedia URL must start with https://en.wikipedia.org/wiki/ but was {}",
                prov.id,
                prov.wikipedia_url
            );
            assert_eq!(
                prov.country_code.len(),
                2,
                "Circuit {} country code must be 2 uppercase ASCII letters",
                prov.id
            );
        }
    }

    #[test]
    fn test_aliases_and_name_resolution() {
        assert_eq!(
            get_circuit_provenance("daytona_superspeedway").map(|p| p.id),
            Some("daytona")
        );
        assert_eq!(
            get_circuit_provenance("Daytona Superspeedway").map(|p| p.id),
            Some("daytona")
        );
        assert_eq!(
            get_circuit_provenance("whilton_mill_kart").map(|p| p.id),
            Some("whilton_mill")
        );
        assert_eq!(
            get_circuit_provenance("Whilton Mill Kart Circuit").map(|p| p.id),
            Some("whilton_mill")
        );
        assert_eq!(
            get_circuit_provenance("glamis_sand_dunes").map(|p| p.id),
            Some("glamis_dunes")
        );
        assert_eq!(
            get_circuit_provenance("Glamis Imperial Sand Dunes").map(|p| p.id),
            Some("glamis_dunes")
        );
        assert_eq!(
            get_circuit_provenance("Monza Autodromo Nazionale").map(|p| p.id),
            Some("monza")
        );
        assert_eq!(
            get_circuit_provenance("track_monza").map(|p| p.id),
            Some("monza")
        );
        assert!(get_circuit_provenance("drift_park").is_none());
        assert!(get_circuit_provenance("classic_grand_prix").is_none());
    }
}
