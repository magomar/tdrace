use std::collections::{HashMap, HashSet};
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

/// Minor scalar offsets applied to the composite BotProfile so that drivers sharing
/// the same style feel subtly unique without baking in a skill tier.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DriverPersonalityOffsets {
    pub delta_lookahead: f32,
    pub delta_steering_kp: f32,
    pub delta_steering_kd: f32,
    pub delta_brake_margin: f32,
    pub delta_aggression: f32,
    pub delta_avoidance: f32,
    pub delta_speed_factor: f32,
}

impl DriverPersonalityOffsets {
    pub const ZERO: Self = Self {
        delta_lookahead: 0.0,
        delta_steering_kp: 0.0,
        delta_steering_kd: 0.0,
        delta_brake_margin: 0.0,
        delta_aggression: 0.0,
        delta_avoidance: 0.0,
        delta_speed_factor: 0.0,
    };

    pub const fn new(
        delta_lookahead: f32,
        delta_steering_kp: f32,
        delta_steering_kd: f32,
        delta_brake_margin: f32,
        delta_aggression: f32,
        delta_avoidance: f32,
        delta_speed_factor: f32,
    ) -> Self {
        Self {
            delta_lookahead,
            delta_steering_kp,
            delta_steering_kd,
            delta_brake_margin,
            delta_aggression,
            delta_avoidance,
            delta_speed_factor,
        }
    }
}

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
    pub offsets: DriverPersonalityOffsets,
    pub favorite_cars: &'static [DriverFavoriteCar],
}

impl DriverCharacter {
    /// Dynamically constructs the physics BotProfile for this character at the specified tier.
    pub fn resolve_profile(&self, tier: DriverTier) -> BotProfile {
        let quality = DriverQuality::for_tier(tier);
        let mut profile = BotProfile::from_style_and_quality(self.style, &quality);
        profile.name = self.name;
        profile.lookahead_time = (profile.lookahead_time + self.offsets.delta_lookahead).clamp(0.20, 0.55);
        profile.steering_kp = (profile.steering_kp + self.offsets.delta_steering_kp).clamp(1.5, 3.5);
        profile.steering_kd = (profile.steering_kd + self.offsets.delta_steering_kd).clamp(0.03, 0.12);
        profile.brake_margin = (profile.brake_margin + self.offsets.delta_brake_margin).clamp(0.80, 1.45);
        profile.aggression = (profile.aggression + self.offsets.delta_aggression).clamp(0.20, 1.00);
        profile.avoidance_distance = (profile.avoidance_distance + self.offsets.delta_avoidance).clamp(3.5, 12.0);
        profile.speed_factor = (profile.speed_factor + self.offsets.delta_speed_factor).clamp(0.80, 1.15);
        profile
    }

    /// Dynamically constructs the UI DriverStats for this character at the specified tier.
    pub fn resolve_stats(&self, tier: DriverTier) -> DriverStats {
        let quality = DriverQuality::for_tier(tier);
        let mut stats = DriverStats::from_style_and_quality(self.style, &quality);
        stats.speed = (stats.speed + self.offsets.delta_speed_factor * 1.5).clamp(0.60, 0.99);
        stats.aggression = (stats.aggression + self.offsets.delta_aggression).clamp(0.40, 0.99);
        stats.precision = (stats.precision + self.offsets.delta_steering_kd * 2.0).clamp(0.50, 0.99);
        stats.defense = (stats.defense - self.offsets.delta_avoidance * 0.05).clamp(0.50, 0.99);
        stats
    }

    /// Default profile for previewing in dossiers.
    pub fn default_profile(&self) -> BotProfile {
        self.resolve_profile(DriverTier::Pro)
    }

    /// Default stats for previewing in dossiers.
    pub fn default_stats(&self) -> DriverStats {
        self.resolve_stats(DriverTier::Pro)
    }

    /// 1. Silvia "Apex" Tanaka — The Precision Master (Smooth)
    pub const SILVIA_TANAKA: Self = Self {
        id: "silvia_tanaka",
        name: "Silvia Tanaka",
        alias: "Apex Tanaka",
        bio: "Former open-wheel champion whose surgical precision and textbook racing lines carve through chicanes like a scalpel.",
        style: DrivingStyle::Smooth,
        preferred_car: CarChoice::SportsCar,
        color_scheme: CarColorScheme::from_index(1), // Electric Blue
        offsets: DriverPersonalityOffsets::new(0.00, 0.1, -0.01, 0.00, 0.05, 0.0, 0.01),
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
        offsets: DriverPersonalityOffsets::new(0.00, 0.1, 0.00, -0.02, 0.00, 0.0, 0.03),
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
        offsets: DriverPersonalityOffsets::new(0.03, -0.2, 0.00, 0.04, -0.04, 0.6, 0.02),
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
        offsets: DriverPersonalityOffsets::new(0.01, -0.1, 0.01, 0.05, -0.05, 0.5, 0.01),
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
        offsets: DriverPersonalityOffsets::new(-0.01, 0.2, -0.01, 0.03, 0.01, -0.2, 0.02),
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
        offsets: DriverPersonalityOffsets::new(0.00, 0.1, 0.00, -0.01, 0.08, -0.5, 0.02),
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
        offsets: DriverPersonalityOffsets::new(0.01, 0.0, 0.00, -0.01, 0.06, 0.0, 0.01),
        favorite_cars: VIKTOR_FAVORITE_CARS,
    };

    /// 8. Maya "Phoenix" Lin — Telemetry Prodigy (Smooth)
    pub const MAYA_LIN: Self = Self {
        id: "maya_lin",
        name: "Maya Lin",
        alias: "Phoenix Lin",
        bio: "An engineering-minded racer who calculates optimal slip angles in real time, delivering blistering straight-line exits.",
        style: DrivingStyle::Smooth,
        preferred_car: CarChoice::SportsCar,
        color_scheme: CarColorScheme::from_index(8), // Cyber Magenta & Neon Cyan
        offsets: DriverPersonalityOffsets::new(-0.03, 0.0, -0.02, -0.02, 0.12, -0.3, 0.00),
        favorite_cars: MAYA_FAVORITE_CARS,
    };

    /// 9. Damon "The Ghost" Clark — Tactical Endurance Master (Tenacious)
    pub const DAMON_CLARK: Self = Self {
        id: "damon_clark",
        name: "Damon Clark",
        alias: "The Ghost",
        bio: "Quiet and hyper-calculating endurance specialist who runs relentless, identical lap times until his opponents crack.",
        style: DrivingStyle::Tenacious,
        preferred_car: CarChoice::SportsCar,
        color_scheme: CarColorScheme::new(
            Color::new(0.50, 0.55, 0.60, 1.0),
            Color::new(0.12, 0.14, 0.18, 1.0),
            Color::new(0.95, 0.95, 0.98, 1.0),
        ), // Slate Gray, Anthracite & Ghost Silver
        offsets: DriverPersonalityOffsets::new(0.00, 0.1, -0.01, -0.02, 0.10, 0.6, 0.02),
        favorite_cars: DAMON_FAVORITE_CARS,
    };

    /// 10. Chloe "The Dynamo" Laurent — Hillclimb Phenom (Calculating)
    pub const CHLOE_LAURENT: Self = Self {
        id: "chloe_laurent",
        name: "Chloe Laurent",
        alias: "The Dynamo",
        bio: "A fearless hybrid-era racer blending European hillclimb reflexes with blistering apex aggression in all conditions.",
        style: DrivingStyle::Calculating,
        preferred_car: CarChoice::RallyCar,
        color_scheme: CarColorScheme::new(
            Color::new(0.12, 0.78, 0.70, 1.0),
            Color::new(0.95, 0.85, 0.20, 1.0),
            Color::new(0.10, 0.10, 0.12, 1.0),
        ), // Bright Teal, Neon Gold & Jet Black
        offsets: DriverPersonalityOffsets::new(-0.04, 0.1, -0.01, -0.04, 0.11, -0.8, 0.01),
        favorite_cars: CHLOE_FAVORITE_CARS,
    };

    /// 11. Hiroshi "Tarmac Samurai" Takahashi — Tire Conservation Virtuoso (Balanced)
    pub const HIROSHI_TAKAHASHI: Self = Self {
        id: "hiroshi_takahashi",
        name: "Hiroshi Takahashi",
        alias: "Tarmac Samurai",
        bio: "Super GT veteran whose millimeter-perfect tire preservation and late-braking maneuvers dominate high-grip circuits.",
        style: DrivingStyle::Balanced,
        preferred_car: CarChoice::SportsCar,
        color_scheme: CarColorScheme::new(
            Color::new(0.55, 0.08, 0.12, 1.0),
            Color::new(0.85, 0.75, 0.35, 1.0),
            Color::new(0.98, 0.98, 0.98, 1.0),
        ), // Deep Maroon, Warm Gold & Pure White
        offsets: DriverPersonalityOffsets::new(0.01, 0.3, -0.01, -0.07, 0.15, -1.0, 0.04),
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
        offsets: DriverPersonalityOffsets::new(0.02, -0.1, 0.01, 0.00, 0.02, -0.1, 0.03),
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

        let mut rng = LcgRng::new(seed);
        rng.shuffle(&mut available);

        available.truncate(count);
        available
    }

    /// Selects `n` distinct opponents pseudo-randomly from the classic roster given a seed.
    pub fn sample_opponents(n: usize, seed: u64) -> Vec<Self> {
        Self::sample_from_slice(&Self::ROSTER, n, seed)
    }

    /// Samples `n` distinct opponents from a pool (or all 72 drivers across modules if pool is empty)
    /// enforcing a balanced, discrete uniform distribution across the 6 driving styles (P = 1/6).
    pub fn sample_casual_race_roster(pool: &[Self], n: usize, seed: u64) -> Vec<Self> {
        let global_pool;
        let effective_pool = if pool.is_empty() {
            global_pool = Self::all_across_modules();
            &global_pool[..]
        } else {
            pool
        };

        let count = n.min(effective_pool.len());
        if count == 0 {
            return Vec::new();
        }

        let mut rng = LcgRng::new(seed);

        // Partition count into balanced quotas across the 6 driving styles
        let base_quota = count / 6;
        let remainder = count % 6;

        let mut styles = DrivingStyle::ALL;
        rng.shuffle(&mut styles);

        let mut quotas = HashMap::new();
        for (i, &style) in styles.iter().enumerate() {
            let q = base_quota + if i < remainder { 1 } else { 0 };
            quotas.insert(style, q);
        }

        let mut selected = Vec::with_capacity(count);
        let mut selected_ids = HashSet::new();

        // Sample drivers for each style quota
        for &style in &DrivingStyle::ALL {
            let needed = quotas.get(&style).copied().unwrap_or(0);
            if needed == 0 {
                continue;
            }

            let mut candidates: Vec<Self> = effective_pool
                .iter()
                .filter(|d| d.style == style && !selected_ids.contains(d.id))
                .cloned()
                .collect();

            rng.shuffle(&mut candidates);

            for driver in candidates.into_iter().take(needed) {
                selected_ids.insert(driver.id);
                selected.push(driver);
            }
        }

        // Fallback: If any quota could not be fulfilled due to pool constraints, fill from remaining
        if selected.len() < count {
            let mut remaining: Vec<Self> = effective_pool
                .iter()
                .filter(|d| !selected_ids.contains(d.id))
                .cloned()
                .collect();
            rng.shuffle(&mut remaining);

            for driver in remaining.into_iter().take(count - selected.len()) {
                selected_ids.insert(driver.id);
                selected.push(driver);
            }
        }

        // Final shuffle so grid order mixes driving styles evenly
        rng.shuffle(&mut selected);
        selected
    }

    /// Selects `n` distinct casual race opponents across all 72 drivers from all modules
    /// with uniform driving style distribution.
    pub fn sample_casual_race_opponents(n: usize, seed: u64) -> Vec<Self> {
        let all = Self::all_across_modules();
        Self::sample_casual_race_roster(&all, n, seed)
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

/// Deterministic linear congruential generator for reproducible roster sampling.
#[derive(Debug, Clone)]
pub struct LcgRng(pub u64);

impl LcgRng {
    pub fn new(seed: u64) -> Self {
        Self(seed.wrapping_add(1442695040888963407))
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1);
        self.0
    }

    pub fn next_u32(&mut self) -> u32 {
        (self.next_u64() >> 32) as u32
    }

    pub fn next_f32(&mut self) -> f32 {
        (self.next_u32() as f32) / (u32::MAX as f32)
    }

    pub fn shuffle<T>(&mut self, slice: &mut [T]) {
        for i in (1..slice.len()).rev() {
            let j = (self.next_u64() >> 33) as usize % (i + 1);
            slice.swap(i, j);
        }
    }
}

