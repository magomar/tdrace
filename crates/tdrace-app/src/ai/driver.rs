use macroquad::color::Color;
use crate::ai::{BotProfile, DriverQuality, DriverTier, DrivingStyle};
use crate::catalog::RealCarModel;
use crate::module::GameModule;
use crate::render::color::CarColorScheme;
use crate::ui::menu::CarChoice;

/// High-level personality and skill stats for a driver character [0.0..1.0].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DriverStats {
    pub speed: f32,
    pub aggression: f32,
    pub precision: f32,
    pub defense: f32,
}

impl DriverStats {
    pub const fn new(speed: f32, aggression: f32, precision: f32, defense: f32) -> Self {
        Self {
            speed,
            aggression,
            precision,
            defense,
        }
    }

    pub fn from_style_and_quality(style: DrivingStyle, quality: &DriverQuality) -> Self {
        let (base_speed_mult, base_agg, base_prec, base_def) = match style {
            DrivingStyle::Smooth => (1.00, 0.65, 0.98, 0.85),
            DrivingStyle::Aggressive => (1.02, 0.96, 0.80, 0.88),
            DrivingStyle::Tenacious => (0.96, 0.82, 0.88, 0.98),
            DrivingStyle::Calculating => (1.00, 0.72, 0.96, 0.90),
            DrivingStyle::Bold => (1.01, 0.92, 0.78, 0.80),
            DrivingStyle::Balanced => (0.97, 0.70, 0.85, 0.85),
        };

        let speed = (0.70 + (quality.pace_limit - 0.85) * 1.5 * base_speed_mult).clamp(0.65, 0.99);
        let aggression = (base_agg * (0.90 + 0.10 * (1.0 - quality.composure))).clamp(0.40, 0.99);
        let precision = (base_prec * (0.80 + 0.20 * quality.consistency)).clamp(0.50, 0.99);
        let defense = (base_def * (0.80 + 0.20 * quality.composure)).clamp(0.50, 0.99);

        Self {
            speed,
            aggression,
            precision,
            defense,
        }
    }

    pub fn from_style(style: DrivingStyle) -> Self {
        match style {
            DrivingStyle::Smooth => Self::new(0.95, 0.70, 0.98, 0.88),
            DrivingStyle::Aggressive => Self::new(0.96, 0.96, 0.80, 0.86),
            DrivingStyle::Tenacious => Self::new(0.93, 0.80, 0.90, 0.98),
            DrivingStyle::Calculating => Self::new(0.94, 0.72, 0.96, 0.92),
            DrivingStyle::Bold => Self::new(0.95, 0.92, 0.82, 0.80),
            DrivingStyle::Balanced => Self::new(0.93, 0.72, 0.90, 0.88),
        }
    }

    pub fn from_archetype(name: &str) -> Self {
        let norm = name.to_ascii_lowercase();
        if let Some((s_str, t_str)) = norm.split_once('_') {
            let style = DrivingStyle::from_str_lossy(s_str);
            let tier = DriverTier::from_str_lossy(t_str);
            return Self::from_style_and_quality(style, &DriverQuality::for_tier(tier));
        }
        match norm.as_str() {
            "rookie" | "cautious" => Self::from_style_and_quality(DrivingStyle::Balanced, &DriverQuality::for_tier(DriverTier::Rookie)),
            "fast" | "pro" | "hotlap" => Self::from_style_and_quality(DrivingStyle::Smooth, &DriverQuality::for_tier(DriverTier::Legend)),
            "strategic" | "draft" => Self::from_style_and_quality(DrivingStyle::Calculating, &DriverQuality::for_tier(DriverTier::Pro)),
            "brawler" => Self::from_style_and_quality(DrivingStyle::Aggressive, &DriverQuality::for_tier(DriverTier::Pro)),
            "defender" => Self::from_style_and_quality(DrivingStyle::Tenacious, &DriverQuality::for_tier(DriverTier::Pro)),
            "drift" => Self::from_style_and_quality(DrivingStyle::Bold, &DriverQuality::for_tier(DriverTier::Pro)),
            "club" => Self::from_style_and_quality(DrivingStyle::Balanced, &DriverQuality::for_tier(DriverTier::Contender)),
            _ => Self::from_style(DrivingStyle::from_str_lossy(&norm)),
        }
    }

    pub fn archetype_for_index(idx: usize) -> Self {
        Self::from_style(DrivingStyle::ALL[idx % DrivingStyle::ALL.len()])
    }
}

/// Association between a motorsport discipline, performance tier (1..=5), and authentic car model ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DriverFavoriteCar {
    pub discipline: &'static str,
    pub tier: u8,
    pub model_id: &'static str,
}

impl DriverFavoriteCar {
    pub const fn new(discipline: &'static str, tier: u8, model_id: &'static str) -> Self {
        Self {
            discipline,
            tier,
            model_id,
        }
    }
}

// Favorite car mapping slices for all 12 core drivers across 6 disciplines (26 cars each)

const SILVIA_FAVORITE_CARS: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("classic", 1, "classic_gt"),
    DriverFavoriteCar::new("gt", 1, "gt_porsche_718_gt4"),
    DriverFavoriteCar::new("gt", 2, "gt_porsche_911_gt3r"),
    DriverFavoriteCar::new("gt", 3, "gt_maserati_mc20_gt2"),
    DriverFavoriteCar::new("gt", 4, "gt_porsche_911_gt1_98"),
    DriverFavoriteCar::new("gt", 5, "gt_porsche_963"),
    DriverFavoriteCar::new("nascar", 1, "nascar_dodge_dart_street_stock"),
    DriverFavoriteCar::new("nascar", 2, "nascar_late_model_stock_car"),
    DriverFavoriteCar::new("nascar", 3, "nascar_toyota_camry_arca"),
    DriverFavoriteCar::new("nascar", 4, "nascar_tundra_truck"),
    DriverFavoriteCar::new("nascar", 5, "nascar_challenger_ta1"),
    DriverFavoriteCar::new("rally", 1, "rally_clio_rally4"),
    DriverFavoriteCar::new("rally", 2, "rally_polo_rx"),
    DriverFavoriteCar::new("rally", 3, "rally_audi_sport_quattro_s1"),
    DriverFavoriteCar::new("rally", 4, "rally_audi_rs_q_etron"),
    DriverFavoriteCar::new("rally", 5, "rally_sst_super_truck"),
    DriverFavoriteCar::new("kart", 1, "kart_tony_kart_neos"),
    DriverFavoriteCar::new("kart", 2, "kart_tony_kart_racer_ok"),
    DriverFavoriteCar::new("kart", 3, "kart_tony_kart_racer_kz"),
    DriverFavoriteCar::new("kart", 4, "kart_honda_mean_mower"),
    DriverFavoriteCar::new("kart", 5, "kart_anderson_cs250"),
    DriverFavoriteCar::new("extreme_offroad", 1, "offroad_vw_sand_rail"),
    DriverFavoriteCar::new("extreme_offroad", 2, "offroad_mason_awd_truck"),
    DriverFavoriteCar::new("extreme_offroad", 3, "offroad_audi_quattro_ice"),
    DriverFavoriteCar::new("extreme_offroad", 4, "offroad_ford_f250_high_riser"),
    DriverFavoriteCar::new("extreme_offroad", 5, "offroad_bigfoot_crusher"),
];

const MARCO_FAVORITE_CARS: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("classic", 1, "classic_rally"),
    DriverFavoriteCar::new("gt", 1, "gt_bmw_m4_gt4"),
    DriverFavoriteCar::new("gt", 2, "gt_ferrari_296_gt3"),
    DriverFavoriteCar::new("gt", 3, "gt_porsche_911_gt2_rs"),
    DriverFavoriteCar::new("gt", 4, "gt_mercedes_clk_gtr"),
    DriverFavoriteCar::new("gt", 5, "gt_ferrari_499p"),
    DriverFavoriteCar::new("nascar", 1, "nascar_monte_carlo_ss"),
    DriverFavoriteCar::new("nascar", 2, "nascar_super_late_model"),
    DriverFavoriteCar::new("nascar", 3, "nascar_arca_chevy_ss"),
    DriverFavoriteCar::new("nascar", 4, "nascar_silverado_truck"),
    DriverFavoriteCar::new("nascar", 5, "nascar_corvette_ta1"),
    DriverFavoriteCar::new("rally", 1, "rally_fiesta_rally4"),
    DriverFavoriteCar::new("rally", 2, "rally_hyundai_i20_rx"),
    DriverFavoriteCar::new("rally", 3, "rally_peugeot_205_t16"),
    DriverFavoriteCar::new("rally", 4, "rally_toyota_hilux_t1_plus"),
    DriverFavoriteCar::new("rally", 5, "rally_sst_robby_gordon"),
    DriverFavoriteCar::new("kart", 1, "kart_crg_hero_60"),
    DriverFavoriteCar::new("kart", 2, "kart_crg_kt2_ok"),
    DriverFavoriteCar::new("kart", 3, "kart_crg_road_rebel_kz"),
    DriverFavoriteCar::new("kart", 4, "kart_john_deere_racing_mower"),
    DriverFavoriteCar::new("kart", 5, "kart_ms_superkart_250"),
    DriverFavoriteCar::new("extreme_offroad", 1, "offroad_sand_rail_buggy"),
    DriverFavoriteCar::new("extreme_offroad", 2, "offroad_baja_trophy_truck"),
    DriverFavoriteCar::new("extreme_offroad", 3, "offroad_subaru_ice_racer"),
    DriverFavoriteCar::new("extreme_offroad", 4, "offroad_mega_mud_truck"),
    DriverFavoriteCar::new("extreme_offroad", 5, "offroad_grave_crusher"),
];

const KENJI_FAVORITE_CARS: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("classic", 1, "classic_gt"),
    DriverFavoriteCar::new("gt", 1, "gt_toyota_supra_gt4"),
    DriverFavoriteCar::new("gt", 2, "gt_amg_gt3_evo"),
    DriverFavoriteCar::new("gt", 3, "gt_brabham_bt62_gt2"),
    DriverFavoriteCar::new("gt", 4, "gt_nissan_r390_gt1"),
    DriverFavoriteCar::new("gt", 5, "gt_toyota_gr010"),
    DriverFavoriteCar::new("nascar", 1, "nascar_dodge_dart_street_stock"),
    DriverFavoriteCar::new("nascar", 2, "nascar_super_late_model"),
    DriverFavoriteCar::new("nascar", 3, "nascar_toyota_camry_arca"),
    DriverFavoriteCar::new("nascar", 4, "nascar_tundra_truck"),
    DriverFavoriteCar::new("nascar", 5, "nascar_mustang_ta1"),
    DriverFavoriteCar::new("rally", 1, "rally_peugeot_208_rally4"),
    DriverFavoriteCar::new("rally", 2, "rally_audi_s1_rx"),
    DriverFavoriteCar::new("rally", 3, "rally_lancia_delta_s4"),
    DriverFavoriteCar::new("rally", 4, "rally_prodrive_hunter_t1"),
    DriverFavoriteCar::new("rally", 5, "rally_sst_traxxas_edition"),
    DriverFavoriteCar::new("kart", 1, "kart_birel_c28"),
    DriverFavoriteCar::new("kart", 2, "kart_birel_ry30_ok"),
    DriverFavoriteCar::new("kart", 3, "kart_birel_art_kz2"),
    DriverFavoriteCar::new("kart", 4, "kart_viking_t6_tractor"),
    DriverFavoriteCar::new("kart", 5, "kart_viper_250_twin"),
    DriverFavoriteCar::new("extreme_offroad", 1, "offroad_polaris_rzr_pro_r"),
    DriverFavoriteCar::new("extreme_offroad", 2, "offroad_bettantown_trophy_truck"),
    DriverFavoriteCar::new("extreme_offroad", 3, "offroad_lancer_evo_ice"),
    DriverFavoriteCar::new("extreme_offroad", 4, "offroad_chevy_k30_mud_bogger"),
    DriverFavoriteCar::new("extreme_offroad", 5, "offroad_max_d_monster"),
];

const ELENA_FAVORITE_CARS: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("classic", 1, "classic_gt"),
    DriverFavoriteCar::new("gt", 1, "gt_aston_vantage_gt4"),
    DriverFavoriteCar::new("gt", 2, "gt_audi_r8_gt3_evo2"),
    DriverFavoriteCar::new("gt", 3, "gt_audi_r8_gt2"),
    DriverFavoriteCar::new("gt", 4, "gt_mclaren_f1_gtr_lt"),
    DriverFavoriteCar::new("gt", 5, "gt_porsche_963"),
    DriverFavoriteCar::new("nascar", 1, "nascar_mustang_street_stock"),
    DriverFavoriteCar::new("nascar", 2, "nascar_mustang_super_late_model"),
    DriverFavoriteCar::new("nascar", 3, "nascar_ford_fusion_arca"),
    DriverFavoriteCar::new("nascar", 4, "nascar_f150_truck"),
    DriverFavoriteCar::new("nascar", 5, "nascar_corvette_ta1"),
    DriverFavoriteCar::new("rally", 1, "rally_clio_rally4"),
    DriverFavoriteCar::new("rally", 2, "rally_audi_s1_rx"),
    DriverFavoriteCar::new("rally", 3, "rally_audi_sport_quattro_s1"),
    DriverFavoriteCar::new("rally", 4, "rally_audi_rs_q_etron"),
    DriverFavoriteCar::new("rally", 5, "rally_sst_robby_gordon"),
    DriverFavoriteCar::new("kart", 1, "kart_tony_kart_neos"),
    DriverFavoriteCar::new("kart", 2, "kart_crg_kt2_ok"),
    DriverFavoriteCar::new("kart", 3, "kart_tony_kart_racer_kz"),
    DriverFavoriteCar::new("kart", 4, "kart_honda_mean_mower"),
    DriverFavoriteCar::new("kart", 5, "kart_anderson_cs250"),
    DriverFavoriteCar::new("extreme_offroad", 1, "offroad_vw_sand_rail"),
    DriverFavoriteCar::new("extreme_offroad", 2, "offroad_mason_awd_truck"),
    DriverFavoriteCar::new("extreme_offroad", 3, "offroad_audi_quattro_ice"),
    DriverFavoriteCar::new("extreme_offroad", 4, "offroad_ford_f250_high_riser"),
    DriverFavoriteCar::new("extreme_offroad", 5, "offroad_bigfoot_crusher"),
];

const JAX_FAVORITE_CARS: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("classic", 1, "classic_offroad"),
    DriverFavoriteCar::new("gt", 1, "gt_bmw_m4_gt4"),
    DriverFavoriteCar::new("gt", 2, "gt_amg_gt3_evo"),
    DriverFavoriteCar::new("gt", 3, "gt_porsche_911_gt2_rs"),
    DriverFavoriteCar::new("gt", 4, "gt_porsche_911_gt1_98"),
    DriverFavoriteCar::new("gt", 5, "gt_cadillac_v_series_r"),
    DriverFavoriteCar::new("nascar", 1, "nascar_mustang_street_stock"),
    DriverFavoriteCar::new("nascar", 2, "nascar_super_late_model"),
    DriverFavoriteCar::new("nascar", 3, "nascar_arca_chevy_ss"),
    DriverFavoriteCar::new("nascar", 4, "nascar_silverado_truck"),
    DriverFavoriteCar::new("nascar", 5, "nascar_challenger_ta1"),
    DriverFavoriteCar::new("rally", 1, "rally_fiesta_rally4"),
    DriverFavoriteCar::new("rally", 2, "rally_hyundai_i20_rx"),
    DriverFavoriteCar::new("rally", 3, "rally_peugeot_205_t16"),
    DriverFavoriteCar::new("rally", 4, "rally_prodrive_hunter_t1"),
    DriverFavoriteCar::new("rally", 5, "rally_sst_traxxas_edition"),
    DriverFavoriteCar::new("kart", 1, "kart_crg_hero_60"),
    DriverFavoriteCar::new("kart", 2, "kart_birel_ry30_ok"),
    DriverFavoriteCar::new("kart", 3, "kart_crg_road_rebel_kz"),
    DriverFavoriteCar::new("kart", 4, "kart_john_deere_racing_mower"),
    DriverFavoriteCar::new("kart", 5, "kart_ms_superkart_250"),
    DriverFavoriteCar::new("extreme_offroad", 1, "offroad_sand_rail_buggy"),
    DriverFavoriteCar::new("extreme_offroad", 2, "offroad_baja_trophy_truck"),
    DriverFavoriteCar::new("extreme_offroad", 3, "offroad_subaru_ice_racer"),
    DriverFavoriteCar::new("extreme_offroad", 4, "offroad_mega_mud_truck"),
    DriverFavoriteCar::new("extreme_offroad", 5, "offroad_grave_crusher"),
];

const LEO_FAVORITE_CARS: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("classic", 1, "classic_kart"),
    DriverFavoriteCar::new("gt", 1, "gt_porsche_718_gt4"),
    DriverFavoriteCar::new("gt", 2, "gt_ferrari_296_gt3"),
    DriverFavoriteCar::new("gt", 3, "gt_maserati_mc20_gt2"),
    DriverFavoriteCar::new("gt", 4, "gt_mclaren_f1_gtr_lt"),
    DriverFavoriteCar::new("gt", 5, "gt_ferrari_499p"),
    DriverFavoriteCar::new("nascar", 1, "nascar_dodge_dart_street_stock"),
    DriverFavoriteCar::new("nascar", 2, "nascar_late_model_stock_car"),
    DriverFavoriteCar::new("nascar", 3, "nascar_ford_fusion_arca"),
    DriverFavoriteCar::new("nascar", 4, "nascar_f150_truck"),
    DriverFavoriteCar::new("nascar", 5, "nascar_mustang_ta1"),
    DriverFavoriteCar::new("rally", 1, "rally_peugeot_208_rally4"),
    DriverFavoriteCar::new("rally", 2, "rally_polo_rx"),
    DriverFavoriteCar::new("rally", 3, "rally_lancia_delta_s4"),
    DriverFavoriteCar::new("rally", 4, "rally_toyota_hilux_t1_plus"),
    DriverFavoriteCar::new("rally", 5, "rally_sst_super_truck"),
    DriverFavoriteCar::new("kart", 1, "kart_birel_c28"),
    DriverFavoriteCar::new("kart", 2, "kart_tony_kart_racer_ok"),
    DriverFavoriteCar::new("kart", 3, "kart_birel_art_kz2"),
    DriverFavoriteCar::new("kart", 4, "kart_viking_t6_tractor"),
    DriverFavoriteCar::new("kart", 5, "kart_viper_250_twin"),
    DriverFavoriteCar::new("extreme_offroad", 1, "offroad_polaris_rzr_pro_r"),
    DriverFavoriteCar::new("extreme_offroad", 2, "offroad_bettantown_trophy_truck"),
    DriverFavoriteCar::new("extreme_offroad", 3, "offroad_lancer_evo_ice"),
    DriverFavoriteCar::new("extreme_offroad", 4, "offroad_chevy_k30_mud_bogger"),
    DriverFavoriteCar::new("extreme_offroad", 5, "offroad_max_d_monster"),
];

const VIKTOR_FAVORITE_CARS: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("classic", 1, "classic_nascar"),
    DriverFavoriteCar::new("gt", 1, "gt_aston_vantage_gt4"),
    DriverFavoriteCar::new("gt", 2, "gt_audi_r8_gt3_evo2"),
    DriverFavoriteCar::new("gt", 3, "gt_brabham_bt62_gt2"),
    DriverFavoriteCar::new("gt", 4, "gt_mercedes_clk_gtr"),
    DriverFavoriteCar::new("gt", 5, "gt_cadillac_v_series_r"),
    DriverFavoriteCar::new("nascar", 1, "nascar_dodge_dart_street_stock"),
    DriverFavoriteCar::new("nascar", 2, "nascar_late_model_stock_car"),
    DriverFavoriteCar::new("nascar", 3, "nascar_arca_chevy_ss"),
    DriverFavoriteCar::new("nascar", 4, "nascar_silverado_truck"),
    DriverFavoriteCar::new("nascar", 5, "nascar_challenger_ta1"),
    DriverFavoriteCar::new("rally", 1, "rally_clio_rally4"),
    DriverFavoriteCar::new("rally", 2, "rally_audi_s1_rx"),
    DriverFavoriteCar::new("rally", 3, "rally_audi_sport_quattro_s1"),
    DriverFavoriteCar::new("rally", 4, "rally_audi_rs_q_etron"),
    DriverFavoriteCar::new("rally", 5, "rally_sst_super_truck"),
    DriverFavoriteCar::new("kart", 1, "kart_tony_kart_neos"),
    DriverFavoriteCar::new("kart", 2, "kart_crg_kt2_ok"),
    DriverFavoriteCar::new("kart", 3, "kart_tony_kart_racer_kz"),
    DriverFavoriteCar::new("kart", 4, "kart_honda_mean_mower"),
    DriverFavoriteCar::new("kart", 5, "kart_anderson_cs250"),
    DriverFavoriteCar::new("extreme_offroad", 1, "offroad_vw_sand_rail"),
    DriverFavoriteCar::new("extreme_offroad", 2, "offroad_mason_awd_truck"),
    DriverFavoriteCar::new("extreme_offroad", 3, "offroad_audi_quattro_ice"),
    DriverFavoriteCar::new("extreme_offroad", 4, "offroad_ford_f250_high_riser"),
    DriverFavoriteCar::new("extreme_offroad", 5, "offroad_bigfoot_crusher"),
];

const MAYA_FAVORITE_CARS: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("classic", 1, "classic_gt"),
    DriverFavoriteCar::new("gt", 1, "gt_toyota_supra_gt4"),
    DriverFavoriteCar::new("gt", 2, "gt_porsche_911_gt3r"),
    DriverFavoriteCar::new("gt", 3, "gt_porsche_911_gt2_rs"),
    DriverFavoriteCar::new("gt", 4, "gt_nissan_r390_gt1"),
    DriverFavoriteCar::new("gt", 5, "gt_toyota_gr010"),
    DriverFavoriteCar::new("nascar", 1, "nascar_monte_carlo_ss"),
    DriverFavoriteCar::new("nascar", 2, "nascar_super_late_model"),
    DriverFavoriteCar::new("nascar", 3, "nascar_toyota_camry_arca"),
    DriverFavoriteCar::new("nascar", 4, "nascar_tundra_truck"),
    DriverFavoriteCar::new("nascar", 5, "nascar_corvette_ta1"),
    DriverFavoriteCar::new("rally", 1, "rally_fiesta_rally4"),
    DriverFavoriteCar::new("rally", 2, "rally_polo_rx"),
    DriverFavoriteCar::new("rally", 3, "rally_peugeot_205_t16"),
    DriverFavoriteCar::new("rally", 4, "rally_toyota_hilux_t1_plus"),
    DriverFavoriteCar::new("rally", 5, "rally_sst_robby_gordon"),
    DriverFavoriteCar::new("kart", 1, "kart_crg_hero_60"),
    DriverFavoriteCar::new("kart", 2, "kart_tony_kart_racer_ok"),
    DriverFavoriteCar::new("kart", 3, "kart_crg_road_rebel_kz"),
    DriverFavoriteCar::new("kart", 4, "kart_john_deere_racing_mower"),
    DriverFavoriteCar::new("kart", 5, "kart_ms_superkart_250"),
    DriverFavoriteCar::new("extreme_offroad", 1, "offroad_sand_rail_buggy"),
    DriverFavoriteCar::new("extreme_offroad", 2, "offroad_bettantown_trophy_truck"),
    DriverFavoriteCar::new("extreme_offroad", 3, "offroad_subaru_ice_racer"),
    DriverFavoriteCar::new("extreme_offroad", 4, "offroad_mega_mud_truck"),
    DriverFavoriteCar::new("extreme_offroad", 5, "offroad_grave_crusher"),
];

const DAMON_FAVORITE_CARS: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("classic", 1, "classic_gt"),
    DriverFavoriteCar::new("gt", 1, "gt_porsche_718_gt4"),
    DriverFavoriteCar::new("gt", 2, "gt_amg_gt3_evo"),
    DriverFavoriteCar::new("gt", 3, "gt_maserati_mc20_gt2"),
    DriverFavoriteCar::new("gt", 4, "gt_mercedes_clk_gtr"),
    DriverFavoriteCar::new("gt", 5, "gt_porsche_963"),
    DriverFavoriteCar::new("nascar", 1, "nascar_mustang_street_stock"),
    DriverFavoriteCar::new("nascar", 2, "nascar_mustang_super_late_model"),
    DriverFavoriteCar::new("nascar", 3, "nascar_ford_fusion_arca"),
    DriverFavoriteCar::new("nascar", 4, "nascar_f150_truck"),
    DriverFavoriteCar::new("nascar", 5, "nascar_mustang_ta1"),
    DriverFavoriteCar::new("rally", 1, "rally_peugeot_208_rally4"),
    DriverFavoriteCar::new("rally", 2, "rally_audi_s1_rx"),
    DriverFavoriteCar::new("rally", 3, "rally_lancia_delta_s4"),
    DriverFavoriteCar::new("rally", 4, "rally_prodrive_hunter_t1"),
    DriverFavoriteCar::new("rally", 5, "rally_sst_traxxas_edition"),
    DriverFavoriteCar::new("kart", 1, "kart_birel_c28"),
    DriverFavoriteCar::new("kart", 2, "kart_birel_ry30_ok"),
    DriverFavoriteCar::new("kart", 3, "kart_birel_art_kz2"),
    DriverFavoriteCar::new("kart", 4, "kart_viking_t6_tractor"),
    DriverFavoriteCar::new("kart", 5, "kart_viper_250_twin"),
    DriverFavoriteCar::new("extreme_offroad", 1, "offroad_polaris_rzr_pro_r"),
    DriverFavoriteCar::new("extreme_offroad", 2, "offroad_mason_awd_truck"),
    DriverFavoriteCar::new("extreme_offroad", 3, "offroad_audi_quattro_ice"),
    DriverFavoriteCar::new("extreme_offroad", 4, "offroad_ford_f250_high_riser"),
    DriverFavoriteCar::new("extreme_offroad", 5, "offroad_max_d_monster"),
];

const CHLOE_FAVORITE_CARS: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("classic", 1, "classic_rally"),
    DriverFavoriteCar::new("gt", 1, "gt_aston_vantage_gt4"),
    DriverFavoriteCar::new("gt", 2, "gt_ferrari_296_gt3"),
    DriverFavoriteCar::new("gt", 3, "gt_porsche_911_gt2_rs"),
    DriverFavoriteCar::new("gt", 4, "gt_mclaren_f1_gtr_lt"),
    DriverFavoriteCar::new("gt", 5, "gt_ferrari_499p"),
    DriverFavoriteCar::new("nascar", 1, "nascar_dodge_dart_street_stock"),
    DriverFavoriteCar::new("nascar", 2, "nascar_super_late_model"),
    DriverFavoriteCar::new("nascar", 3, "nascar_arca_chevy_ss"),
    DriverFavoriteCar::new("nascar", 4, "nascar_silverado_truck"),
    DriverFavoriteCar::new("nascar", 5, "nascar_corvette_ta1"),
    DriverFavoriteCar::new("rally", 1, "rally_clio_rally4"),
    DriverFavoriteCar::new("rally", 2, "rally_polo_rx"),
    DriverFavoriteCar::new("rally", 3, "rally_audi_sport_quattro_s1"),
    DriverFavoriteCar::new("rally", 4, "rally_audi_rs_q_etron"),
    DriverFavoriteCar::new("rally", 5, "rally_sst_super_truck"),
    DriverFavoriteCar::new("kart", 1, "kart_tony_kart_neos"),
    DriverFavoriteCar::new("kart", 2, "kart_tony_kart_racer_ok"),
    DriverFavoriteCar::new("kart", 3, "kart_tony_kart_racer_kz"),
    DriverFavoriteCar::new("kart", 4, "kart_honda_mean_mower"),
    DriverFavoriteCar::new("kart", 5, "kart_anderson_cs250"),
    DriverFavoriteCar::new("extreme_offroad", 1, "offroad_vw_sand_rail"),
    DriverFavoriteCar::new("extreme_offroad", 2, "offroad_baja_trophy_truck"),
    DriverFavoriteCar::new("extreme_offroad", 3, "offroad_lancer_evo_ice"),
    DriverFavoriteCar::new("extreme_offroad", 4, "offroad_mega_mud_truck"),
    DriverFavoriteCar::new("extreme_offroad", 5, "offroad_bigfoot_crusher"),
];

const HIROSHI_FAVORITE_CARS: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("classic", 1, "classic_gt"),
    DriverFavoriteCar::new("gt", 1, "gt_toyota_supra_gt4"),
    DriverFavoriteCar::new("gt", 2, "gt_audi_r8_gt3_evo2"),
    DriverFavoriteCar::new("gt", 3, "gt_brabham_bt62_gt2"),
    DriverFavoriteCar::new("gt", 4, "gt_nissan_r390_gt1"),
    DriverFavoriteCar::new("gt", 5, "gt_toyota_gr010"),
    DriverFavoriteCar::new("nascar", 1, "nascar_dodge_dart_street_stock"),
    DriverFavoriteCar::new("nascar", 2, "nascar_late_model_stock_car"),
    DriverFavoriteCar::new("nascar", 3, "nascar_toyota_camry_arca"),
    DriverFavoriteCar::new("nascar", 4, "nascar_tundra_truck"),
    DriverFavoriteCar::new("nascar", 5, "nascar_challenger_ta1"),
    DriverFavoriteCar::new("rally", 1, "rally_fiesta_rally4"),
    DriverFavoriteCar::new("rally", 2, "rally_hyundai_i20_rx"),
    DriverFavoriteCar::new("rally", 3, "rally_peugeot_205_t16"),
    DriverFavoriteCar::new("rally", 4, "rally_toyota_hilux_t1_plus"),
    DriverFavoriteCar::new("rally", 5, "rally_sst_robby_gordon"),
    DriverFavoriteCar::new("kart", 1, "kart_crg_hero_60"),
    DriverFavoriteCar::new("kart", 2, "kart_crg_kt2_ok"),
    DriverFavoriteCar::new("kart", 3, "kart_crg_road_rebel_kz"),
    DriverFavoriteCar::new("kart", 4, "kart_john_deere_racing_mower"),
    DriverFavoriteCar::new("kart", 5, "kart_ms_superkart_250"),
    DriverFavoriteCar::new("extreme_offroad", 1, "offroad_sand_rail_buggy"),
    DriverFavoriteCar::new("extreme_offroad", 2, "offroad_bettantown_trophy_truck"),
    DriverFavoriteCar::new("extreme_offroad", 3, "offroad_subaru_ice_racer"),
    DriverFavoriteCar::new("extreme_offroad", 4, "offroad_chevy_k30_mud_bogger"),
    DriverFavoriteCar::new("extreme_offroad", 5, "offroad_grave_crusher"),
];

const ZANE_FAVORITE_CARS: &[DriverFavoriteCar] = &[
    DriverFavoriteCar::new("classic", 1, "classic_offroad"),
    DriverFavoriteCar::new("gt", 1, "gt_bmw_m4_gt4"),
    DriverFavoriteCar::new("gt", 2, "gt_porsche_911_gt3r"),
    DriverFavoriteCar::new("gt", 3, "gt_audi_r8_gt2"),
    DriverFavoriteCar::new("gt", 4, "gt_porsche_911_gt1_98"),
    DriverFavoriteCar::new("gt", 5, "gt_cadillac_v_series_r"),
    DriverFavoriteCar::new("nascar", 1, "nascar_monte_carlo_ss"),
    DriverFavoriteCar::new("nascar", 2, "nascar_super_late_model"),
    DriverFavoriteCar::new("nascar", 3, "nascar_arca_chevy_ss"),
    DriverFavoriteCar::new("nascar", 4, "nascar_silverado_truck"),
    DriverFavoriteCar::new("nascar", 5, "nascar_challenger_ta1"),
    DriverFavoriteCar::new("rally", 1, "rally_peugeot_208_rally4"),
    DriverFavoriteCar::new("rally", 2, "rally_audi_s1_rx"),
    DriverFavoriteCar::new("rally", 3, "rally_lancia_delta_s4"),
    DriverFavoriteCar::new("rally", 4, "rally_prodrive_hunter_t1"),
    DriverFavoriteCar::new("rally", 5, "rally_sst_traxxas_edition"),
    DriverFavoriteCar::new("kart", 1, "kart_birel_c28"),
    DriverFavoriteCar::new("kart", 2, "kart_tony_kart_racer_ok"),
    DriverFavoriteCar::new("kart", 3, "kart_birel_art_kz2"),
    DriverFavoriteCar::new("kart", 4, "kart_viking_t6_tractor"),
    DriverFavoriteCar::new("kart", 5, "kart_viper_250_twin"),
    DriverFavoriteCar::new("extreme_offroad", 1, "offroad_vw_sand_rail"),
    DriverFavoriteCar::new("extreme_offroad", 2, "offroad_mason_awd_truck"),
    DriverFavoriteCar::new("extreme_offroad", 3, "offroad_subaru_ice_racer"),
    DriverFavoriteCar::new("extreme_offroad", 4, "offroad_mega_mud_truck"),
    DriverFavoriteCar::new("extreme_offroad", 5, "offroad_bigfoot_crusher"),
];

/// Predefined motorsport driver character with unique personality, backstory, preferred car, and AI style.
#[derive(Debug, Clone, PartialEq)]
pub struct DriverCharacter {
    pub id: &'static str,
    pub name: &'static str,
    pub alias: &'static str,
    pub bio: &'static str,
    pub style: DrivingStyle,
    pub preferred_car: CarChoice,
    pub color_scheme: CarColorScheme,
    pub profile: BotProfile,
    pub stats: DriverStats,
    pub favorite_cars: &'static [DriverFavoriteCar],
}

impl DriverCharacter {
    /// 1. Silvia "Apex" Tanaka — The Precision Master (Smooth)
    pub const SILVIA_TANAKA: Self = Self {
        id: "silvia_tanaka",
        name: "Silvia Tanaka",
        alias: "Apex Tanaka",
        bio: "Former open-wheel champion whose surgical precision and textbook racing lines carve through chicanes like a scalpel.",
        style: DrivingStyle::Smooth,
        preferred_car: CarChoice::SportsCar,
        color_scheme: CarColorScheme::from_index(1), // Electric Blue
        profile: BotProfile {
            name: "Silvia Tanaka",
            lookahead_time: 0.40,
            speed_factor: 1.02,
            steering_kp: 2.4,
            steering_kd: 0.07,
            brake_margin: 1.02,
            aggression: 0.75,
            avoidance_distance: 6.5,
        },
        stats: DriverStats {
            speed: 0.95,
            aggression: 0.70,
            precision: 0.98,
            defense: 0.88,
        },
        favorite_cars: SILVIA_FAVORITE_CARS,
    };

    /// 2. Marco "Thunder" Rossi — High-Speed Brawler (Aggressive)
    pub const MARCO_ROSSI: Self = Self {
        id: "marco_rossi",
        name: "Marco Rossi",
        alias: "Thunder Rossi",
        bio: "Fearless and aggressive, Marco thrives in wheel-to-wheel combat, braking at the absolute last millisecond into hairpins.",
        style: DrivingStyle::Aggressive,
        preferred_car: CarChoice::RallyCar,
        color_scheme: CarColorScheme::from_index(4), // Sunset Orange
        profile: BotProfile {
            name: "Marco Rossi",
            lookahead_time: 0.32,
            speed_factor: 1.05,
            steering_kp: 2.6,
            steering_kd: 0.05,
            brake_margin: 0.88,
            aggression: 0.95,
            avoidance_distance: 5.0,
        },
        stats: DriverStats {
            speed: 0.96,
            aggression: 0.96,
            precision: 0.80,
            defense: 0.86,
        },
        favorite_cars: MARCO_FAVORITE_CARS,
    };

    /// 3. Kenji "Drift King" Sato — Touge Slide Maestro (Bold)
    pub const KENJI_SATO: Self = Self {
        id: "kenji_sato",
        name: "Kenji Sato",
        alias: "Drift King Kenji",
        bio: "Honed on mountain passes under neon city lights, Kenji turns every apex into a controlled, high-speed sideways drift.",
        style: DrivingStyle::Bold,
        preferred_car: CarChoice::DriftCar,
        color_scheme: CarColorScheme::from_index(5), // Synthwave Purple
        profile: BotProfile {
            name: "Kenji Sato",
            lookahead_time: 0.34,
            speed_factor: 1.03,
            steering_kp: 2.5,
            steering_kd: 0.04,
            brake_margin: 0.92,
            aggression: 0.88,
            avoidance_distance: 5.8,
        },
        stats: DriverStats {
            speed: 0.94,
            aggression: 0.90,
            precision: 0.85,
            defense: 0.80,
        },
        favorite_cars: KENJI_FAVORITE_CARS,
    };

    /// 4. Elena "Viper" Frost — The Iceman of the Circuit (Calculating)
    pub const ELENA_FROST: Self = Self {
        id: "elena_frost",
        name: "Elena Frost",
        alias: "Viper Frost",
        bio: "Unflappable under pressure, Elena never misses a braking mark and capitalizes ruthlessly on opponents' mistakes.",
        style: DrivingStyle::Calculating,
        preferred_car: CarChoice::SportsCar,
        color_scheme: CarColorScheme::from_index(7), // Glacier White & Cyan
        profile: BotProfile {
            name: "Elena Frost",
            lookahead_time: 0.38,
            speed_factor: 0.99,
            steering_kp: 2.2,
            steering_kd: 0.06,
            brake_margin: 1.05,
            aggression: 0.70,
            avoidance_distance: 7.2,
        },
        stats: DriverStats {
            speed: 0.94,
            aggression: 0.68,
            precision: 0.97,
            defense: 0.94,
        },
        favorite_cars: ELENA_FAVORITE_CARS,
    };

    /// 5. Jax "Oversteer" Reed — The Wildcard Renegade (Aggressive)
    pub const JAX_REED: Self = Self {
        id: "jax_reed",
        name: "Jax Reed",
        alias: "Oversteer Reed",
        bio: "A rallycross veteran with lightning reflexes who uses curbs and sand transitions to slingshot past opponents.",
        style: DrivingStyle::Aggressive,
        preferred_car: CarChoice::RallyCar,
        color_scheme: CarColorScheme::from_index(3), // Sunburst Yellow & Crimson
        profile: BotProfile {
            name: "Jax Reed",
            lookahead_time: 0.30,
            speed_factor: 1.04,
            steering_kp: 2.7,
            steering_kd: 0.05,
            brake_margin: 0.86,
            aggression: 0.92,
            avoidance_distance: 5.2,
        },
        stats: DriverStats {
            speed: 0.95,
            aggression: 0.94,
            precision: 0.80,
            defense: 0.82,
        },
        favorite_cars: JAX_FAVORITE_CARS,
    };

    /// 6. Leo "Pocket Rocket" Bianchi — Agile Shifter Prodigy (Balanced)
    pub const LEO_BIANCHI: Self = Self {
        id: "leo_bianchi",
        name: "Leo Bianchi",
        alias: "Pocket Rocket Leo",
        bio: "A prodigy straight from shifter kart leagues, Leo carries ridiculous corner speed through tight 90-degree switchbacks.",
        style: DrivingStyle::Balanced,
        preferred_car: CarChoice::Kart,
        color_scheme: CarColorScheme::from_index(2), // Viper Green
        profile: BotProfile {
            name: "Leo Bianchi",
            lookahead_time: 0.36,
            speed_factor: 0.97,
            steering_kp: 2.3,
            steering_kd: 0.07,
            brake_margin: 1.08,
            aggression: 0.65,
            avoidance_distance: 6.8,
        },
        stats: DriverStats {
            speed: 0.93,
            aggression: 0.72,
            precision: 0.92,
            defense: 0.86,
        },
        favorite_cars: LEO_FAVORITE_CARS,
    };

    /// 7. Viktor "The Wall" Sterling — Ironclad Veteran (Tenacious)
    pub const VIKTOR_STERLING: Self = Self {
        id: "viktor_sterling",
        name: "Viktor Sterling",
        alias: "The Wall Sterling",
        bio: "With three decades of motorsport experience, Viktor makes his car as wide as the track, frustrating any pass attempt.",
        style: DrivingStyle::Tenacious,
        preferred_car: CarChoice::SportsCar,
        color_scheme: CarColorScheme::from_index(6), // Stealth Carbon Black
        profile: BotProfile {
            name: "Viktor Sterling",
            lookahead_time: 0.44,
            speed_factor: 0.95,
            steering_kp: 2.0,
            steering_kd: 0.08,
            brake_margin: 1.14,
            aggression: 0.78,
            avoidance_distance: 7.8,
        },
        stats: DriverStats {
            speed: 0.92,
            aggression: 0.80,
            precision: 0.90,
            defense: 0.98,
        },
        favorite_cars: VIKTOR_FAVORITE_CARS,
    };

    /// 8. Maya "Phoenix" Lin — Telemetry Prodigy (Calculating)
    pub const MAYA_LIN: Self = Self {
        id: "maya_lin",
        name: "Maya Lin",
        alias: "Phoenix Lin",
        bio: "An engineering-minded racer who calculates optimal slip angles in real time, delivering blistering straight-line exits.",
        style: DrivingStyle::Calculating,
        preferred_car: CarChoice::SportsCar,
        color_scheme: CarColorScheme::from_index(8), // Cyber Magenta & Neon Cyan
        profile: BotProfile {
            name: "Maya Lin",
            lookahead_time: 0.37,
            speed_factor: 1.01,
            steering_kp: 2.3,
            steering_kd: 0.06,
            brake_margin: 1.00,
            aggression: 0.82,
            avoidance_distance: 6.2,
        },
        stats: DriverStats {
            speed: 0.95,
            aggression: 0.78,
            precision: 0.96,
            defense: 0.88,
        },
        favorite_cars: MAYA_FAVORITE_CARS,
    };

    /// 9. Damon "The Ghost" Clark — Tactical Endurance Master (Balanced)
    pub const DAMON_CLARK: Self = Self {
        id: "damon_clark",
        name: "Damon Clark",
        alias: "The Ghost",
        bio: "Quiet and hyper-calculating endurance specialist who runs relentless, identical lap times until his opponents crack.",
        style: DrivingStyle::Balanced,
        preferred_car: CarChoice::SportsCar,
        color_scheme: CarColorScheme::new(
            Color::new(0.50, 0.55, 0.60, 1.0),
            Color::new(0.12, 0.14, 0.18, 1.0),
            Color::new(0.95, 0.95, 0.98, 1.0),
        ), // Slate Gray, Anthracite & Ghost Silver
        profile: BotProfile {
            name: "Damon Clark",
            lookahead_time: 0.42,
            speed_factor: 1.01,
            steering_kp: 2.3,
            steering_kd: 0.07,
            brake_margin: 1.03,
            aggression: 0.72,
            avoidance_distance: 6.6,
        },
        stats: DriverStats {
            speed: 0.94,
            aggression: 0.74,
            precision: 0.93,
            defense: 0.89,
        },
        favorite_cars: DAMON_FAVORITE_CARS,
    };

    /// 10. Chloe "The Dynamo" Laurent — Hillclimb Phenom (Smooth)
    pub const CHLOE_LAURENT: Self = Self {
        id: "chloe_laurent",
        name: "Chloe Laurent",
        alias: "The Dynamo",
        bio: "A fearless hybrid-era racer blending European hillclimb reflexes with blistering apex aggression in all conditions.",
        style: DrivingStyle::Smooth,
        preferred_car: CarChoice::RallyCar,
        color_scheme: CarColorScheme::new(
            Color::new(0.12, 0.78, 0.70, 1.0),
            Color::new(0.95, 0.85, 0.20, 1.0),
            Color::new(0.10, 0.10, 0.12, 1.0),
        ), // Bright Teal, Neon Gold & Jet Black
        profile: BotProfile {
            name: "Chloe Laurent",
            lookahead_time: 0.35,
            speed_factor: 1.03,
            steering_kp: 2.5,
            steering_kd: 0.05,
            brake_margin: 0.90,
            aggression: 0.89,
            avoidance_distance: 5.5,
        },
        stats: DriverStats {
            speed: 0.94,
            aggression: 0.72,
            precision: 0.97,
            defense: 0.88,
        },
        favorite_cars: CHLOE_FAVORITE_CARS,
    };

    /// 11. Hiroshi "Tarmac Samurai" Takahashi — Tire Conservation Virtuoso (Tenacious)
    pub const HIROSHI_TAKAHASHI: Self = Self {
        id: "hiroshi_takahashi",
        name: "Hiroshi Takahashi",
        alias: "Tarmac Samurai",
        bio: "Super GT veteran whose millimeter-perfect tire preservation and late-braking maneuvers dominate high-grip circuits.",
        style: DrivingStyle::Tenacious,
        preferred_car: CarChoice::SportsCar,
        color_scheme: CarColorScheme::new(
            Color::new(0.55, 0.08, 0.12, 1.0),
            Color::new(0.85, 0.75, 0.35, 1.0),
            Color::new(0.98, 0.98, 0.98, 1.0),
        ), // Deep Maroon, Warm Gold & Pure White
        profile: BotProfile {
            name: "Hiroshi Takahashi",
            lookahead_time: 0.39,
            speed_factor: 1.02,
            steering_kp: 2.4,
            steering_kd: 0.06,
            brake_margin: 0.98,
            aggression: 0.80,
            avoidance_distance: 6.0,
        },
        stats: DriverStats {
            speed: 0.93,
            aggression: 0.82,
            precision: 0.95,
            defense: 0.94,
        },
        favorite_cars: HIROSHI_FAVORITE_CARS,
    };

    /// 12. Zane "Thunderbolt" Holland — Low-Traction Acrobat (Bold)
    pub const ZANE_HOLLAND: Self = Self {
        id: "zane_holland",
        name: "Zane Holland",
        alias: "Thunderbolt",
        bio: "Cross-discipline daredevil known for audacious divebombs and supernatural recovery saves in low-traction ruts.",
        style: DrivingStyle::Bold,
        preferred_car: CarChoice::SandRail,
        color_scheme: CarColorScheme::new(
            Color::new(0.10, 0.35, 0.85, 1.0),
            Color::new(0.98, 0.75, 0.08, 1.0),
            Color::new(0.98, 0.98, 0.98, 1.0),
        ), // Cobalt Blue, Lightning Yellow & Pure White
        profile: BotProfile {
            name: "Zane Holland",
            lookahead_time: 0.33,
            speed_factor: 1.04,
            steering_kp: 2.6,
            steering_kd: 0.05,
            brake_margin: 0.88,
            aggression: 0.94,
            avoidance_distance: 5.1,
        },
        stats: DriverStats {
            speed: 0.95,
            aggression: 0.93,
            precision: 0.82,
            defense: 0.82,
        },
        favorite_cars: ZANE_FAVORITE_CARS,
    };

    /// Complete registry of all 12 predefined driver characters.
    pub const ROSTER: [Self; 12] = [
        Self::SILVIA_TANAKA,
        Self::MARCO_ROSSI,
        Self::KENJI_SATO,
        Self::ELENA_FROST,
        Self::JAX_REED,
        Self::LEO_BIANCHI,
        Self::VIKTOR_STERLING,
        Self::MAYA_LIN,
        Self::DAMON_CLARK,
        Self::CHLOE_LAURENT,
        Self::HIROSHI_TAKAHASHI,
        Self::ZANE_HOLLAND,
    ];

    /// Returns a slice of all 12 predefined driver characters for the classic module.
    pub fn all() -> &'static [Self; 12] {
        &Self::ROSTER
    }

    /// Finds a driver by their unique identifier string in the classic roster.
    pub fn find_by_id(id: &str) -> Option<&'static Self> {
        Self::ROSTER.iter().find(|d| d.id == id)
    }

    /// Returns all 72 predefined driver characters across all 6 motorsport modules.
    pub fn all_across_modules() -> Vec<Self> {
        let mut all = Self::ROSTER.to_vec();
        all.extend(crate::module::gt::GtWorldChallengeModule::new().drivers());
        all.extend(crate::module::nascar::NascarGameModule::new().drivers());
        all.extend(crate::module::rally::RallyGameModule::new().drivers());
        all.extend(crate::module::kart::KartGameModule::new().drivers());
        all.extend(crate::module::extreme_offroad::ExtremeOffRoadModule::new().drivers());
        all
    }

    /// Finds a driver by ID across all 72 predefined drivers in all motorsport modules.
    pub fn find_global(id: &str) -> Option<Self> {
        if let Some(d) = Self::ROSTER.iter().find(|d| d.id == id) {
            return Some(d.clone());
        }
        Self::all_across_modules().into_iter().find(|d| d.id == id)
    }

    /// Selects `n` distinct opponents pseudo-randomly from an arbitrary pool of driver characters given a seed.
    pub fn sample_from_slice(pool: &[Self], n: usize, seed: u64) -> Vec<Self> {
        if pool.is_empty() {
            return Vec::new();
        }
        let count = n.min(pool.len());
        let mut available: Vec<Self> = pool.to_vec();

        // Simple deterministic LCG shuffle using seed
        let mut s = seed.wrapping_add(1442695040888963407);
        for i in (1..available.len()).rev() {
            s = s.wrapping_mul(6364136223846793005).wrapping_add(1);
            let j = (s >> 33) as usize % (i + 1);
            available.swap(i, j);
        }

        available.truncate(count);
        available
    }

    /// Selects `n` distinct opponents pseudo-randomly from the classic roster given a seed.
    pub fn sample_opponents(n: usize, seed: u64) -> Vec<Self> {
        Self::sample_from_slice(&Self::ROSTER, n, seed)
    }

    /// Normalizes raw discipline strings (e.g. "gt_challenge", "rx", "off_road") to canonical discipline keys.
    pub fn normalize_discipline(discipline: &str) -> &'static str {
        match discipline.to_ascii_lowercase().as_str() {
            "classic" => "classic",
            "gt" | "gt_challenge" | "gt_world_challenge" => "gt",
            "nascar" | "stock_car" | "trans_am" => "nascar",
            "rally" | "rallycross" | "rx" => "rally",
            "kart" | "karting" => "kart",
            "extreme_offroad" | "off_road" | "offroad" => "extreme_offroad",
            _ => "classic",
        }
    }

    /// Returns the signature vehicle model ID for a specific discipline and tier.
    /// If discipline is "classic", tier is strictly normalized to Tier 1.
    pub fn favorite_car_for_discipline_and_tier(&self, discipline: &str, tier: u8) -> Option<&'static str> {
        let norm_disc = Self::normalize_discipline(discipline);
        let effective_tier = if norm_disc == "classic" { 1 } else { tier.clamp(1, 5) };

        // 1. Exact match for discipline and tier
        if let Some(fav) = self.favorite_cars.iter().find(|f| f.discipline == norm_disc && f.tier == effective_tier) {
            return Some(fav.model_id);
        }
        // 2. Fallback to any car in the same discipline
        if let Some(fav) = self.favorite_cars.iter().find(|f| f.discipline == norm_disc) {
            return Some(fav.model_id);
        }
        // 3. Fallback for classic discipline based on preferred_car
        if norm_disc == "classic" {
            return match self.preferred_car {
                CarChoice::SportsCar
                | CarChoice::DriftCar
                | CarChoice::GT4Clubsport
                | CarChoice::GT3Car
                | CarChoice::GT2Biturbo
                | CarChoice::GT1Legend
                | CarChoice::HypercarPrototype => Some("classic_gt"),
                CarChoice::StockCar => Some("classic_nascar"),
                CarChoice::RallyCar => Some("classic_rally"),
                CarChoice::Kart => Some("classic_kart"),
                CarChoice::SandRail => Some("classic_offroad"),
            };
        }
        None
    }

    /// Resolves the full RealCarModel definition from the authentic vehicle catalog.
    pub fn favorite_model_for_discipline_and_tier(&self, discipline: &str, tier: u8) -> Option<&'static RealCarModel> {
        let car_id = self.favorite_car_for_discipline_and_tier(discipline, tier)?;
        crate::catalog::find_model_by_id(car_id)
    }

    /// Returns the appropriate CarChoice archetype enum matching the favorite vehicle.
    pub fn effective_car_choice_for_discipline_and_tier(&self, discipline: &str, tier: u8) -> CarChoice {
        if let Some(model) = self.favorite_model_for_discipline_and_tier(discipline, tier) {
            model.base_car_choice
        } else {
            let norm_disc = Self::normalize_discipline(discipline);
            match norm_disc {
                "gt" => CarChoice::GT4Clubsport,
                "nascar" => CarChoice::StockCar,
                "rally" => CarChoice::RallyCar,
                "kart" => CarChoice::Kart,
                "extreme_offroad" => CarChoice::SandRail,
                _ => self.preferred_car,
            }
        }
    }
}
