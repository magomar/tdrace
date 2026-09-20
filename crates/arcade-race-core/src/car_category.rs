use serde::{Deserialize, Serialize};

/// High-level vehicle and circuit category aligning with the core motorsport disciplines.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CarCategory {
    /// Gran Turismo, Endurance, and Sports/Drift GT racing.
    Gt,
    /// Stock car superspeedway pack racing.
    Nascar,
    /// Mixed-surface rallycross and all-terrain rally.
    Rally,
    /// Agile sprint and shifter go-karts.
    Kart,
    /// Sand rail, dune buggy, and extreme off-road stunt racing.
    #[serde(rename = "off_road", alias = "offroad", alias = "off-road", alias = "extreme_offroad")]
    OffRoad,
}

impl CarCategory {
    pub const ALL: [Self; 5] = [
        Self::Gt,
        Self::Nascar,
        Self::Rally,
        Self::Kart,
        Self::OffRoad,
    ];

    /// Canonical identifier string (e.g. "gt", "nascar", "rally", "kart", "off_road").
    pub fn id(&self) -> &'static str {
        match self {
            Self::Gt => "gt",
            Self::Nascar => "nascar",
            Self::Rally => "rally",
            Self::Kart => "kart",
            Self::OffRoad => "off_road",
        }
    }

    /// Short uppercase title (e.g. "GT", "NASCAR", "RALLY", "KART", "OFF-ROAD").
    pub fn title(&self) -> &'static str {
        match self {
            Self::Gt => "GT",
            Self::Nascar => "NASCAR",
            Self::Rally => "RALLY",
            Self::Kart => "KART",
            Self::OffRoad => "OFF-ROAD",
        }
    }

    /// Full display title.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Gt => "GT World Challenge",
            Self::Nascar => "NASCAR Stock Car",
            Self::Rally => "Rallycross",
            Self::Kart => "Karting",
            Self::OffRoad => "Extreme Off-Road",
        }
    }

    /// Resolves category from a module ID or vehicle string.
    pub fn from_id(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "gt" | "gt_challenge" | "f1" | "classic_gt" | "sports_car" | "drift_car" | "gt3_car"
            | "gt4_clubsport" | "gt3_evo" | "gt2_biturbo" | "gt1_legend" | "hypercar_prototype" => {
                Some(Self::Gt)
            }
            "nascar" | "classic_nascar" | "stock_car" => Some(Self::Nascar),
            "rally" | "classic_rally" | "rally_car" => Some(Self::Rally),
            "kart" | "classic_kart" | "shifter_kart_125" => Some(Self::Kart),
            "off_road" | "offroad" | "off-road" | "extreme_offroad" | "classic_offroad"
            | "sand_rail_buggy" | "sand_rail" => Some(Self::OffRoad),
            _ => None,
        }
    }
}

impl Default for CarCategory {
    fn default() -> Self {
        Self::Gt
    }
}

impl std::fmt::Display for CarCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.title())
    }
}
