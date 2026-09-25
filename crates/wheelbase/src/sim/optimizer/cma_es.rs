//! Pure-Rust, allocation-minimized Bounded Covariance Matrix Adaptation Evolution Strategy (CMA-ES).
//!
//! Provides deterministic non-convex optimization over bounded continuous parameter spaces
//! without external linear algebra dependencies.

/// Deterministic Pseudo-Random Number Generator (XorShift64*).
#[derive(Debug, Clone)]
pub struct DeterministicRng {
    state: u64,
    has_spare: bool,
    spare: f32,
}

impl DeterministicRng {
    pub fn new(seed: u64) -> Self {
        let non_zero_seed = if seed == 0 { 0x853c49e6748fea9b } else { seed };
        Self {
            state: non_zero_seed,
            has_spare: false,
            spare: 0.0,
        }
    }

    #[inline]
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        x.wrapping_mul(0x2545F4914F6CDD1D)
    }

    #[inline]
    pub fn next_f32(&mut self) -> f32 {
        let r = self.next_u64() >> 40;
        (r as f32) / ((1u64 << 24) as f32)
    }

    /// Box-Muller transform for standard normal random variable N(0, 1).
    pub fn next_gaussian(&mut self) -> f32 {
        if self.has_spare {
            self.has_spare = false;
            return self.spare;
        }

        let mut u1 = self.next_f32();
        while u1 <= 1e-7 {
            u1 = self.next_f32();
        }
        let u2 = self.next_f32();

        let radius = (-2.0 * u1.ln()).sqrt();
        let theta = 2.0 * std::f32::consts::PI * u2;

        self.spare = radius * theta.sin();
        self.has_spare = true;

        radius * theta.cos()
    }
}

/// Defines an individual parameter's tuning bounds and scaling.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterBound {
    pub name: String,
    pub min: f32,
    pub max: f32,
    pub default: f32,
}

impl ParameterBound {
    pub fn new(name: impl Into<String>, min: f32, max: f32, default: f32) -> Self {
        Self {
            name: name.into(),
            min,
            max,
            default: default.clamp(min, max),
        }
    }

    #[inline]
    pub fn to_normalized(&self, val: f32) -> f32 {
        let span = self.max - self.min;
        if span > 1e-6 {
            ((val - self.min) / span).clamp(0.0, 1.0)
        } else {
            0.5
        }
    }

    #[inline]
    pub fn from_normalized(&self, norm: f32) -> f32 {
        let clamped = norm.clamp(0.0, 1.0);
        self.min + clamped * (self.max - self.min)
    }
}

use serde::{Deserialize, Serialize};

/// Bounded CMA-ES Optimizer instance for dimension D.
#[derive(Debug, Clone)]
pub struct CmaEsOptimizer {
    pub dim: usize,
    pub lambda: usize,
    pub mu: usize,
    pub weights: Vec<f32>,
    pub mu_eff: f32,

    pub mean: Vec<f32>,
    pub sigma: f32,

    pub p_sigma: Vec<f32>,
    pub p_c: Vec<f32>,
    pub diag_c: Vec<f32>,

    pub c_sigma: f32,
    pub d_sigma: f32,
    pub c_c: f32,
    pub c_1: f32,
    pub c_mu: f32,
    pub chi_n: f32,

    pub rng: DeterministicRng,
    pub generation: usize,
    pub best_solution: Vec<f32>,
    pub best_cost: f32,
}

impl CmaEsOptimizer {
    pub fn new(bounds: &[ParameterBound], custom_population: Option<usize>, seed: u64) -> Self {
        let dim = bounds.len();
        let lambda = custom_population.unwrap_or_else(|| 4 + (3.0 * (dim as f32).ln()).floor() as usize);
        let mu = (lambda / 2).max(1);

        // Raw weights
        let mut raw_weights = Vec::with_capacity(mu);
        let mut sum_w = 0.0f32;
        let mut sum_sq_w = 0.0f32;
        for i in 0..mu {
            let w = ((mu as f32) + 0.5).ln() - ((i + 1) as f32).ln();
            raw_weights.push(w);
            sum_w += w;
        }

        // Normalized weights
        let weights: Vec<f32> = raw_weights
            .into_iter()
            .map(|w| {
                let norm_w = w / sum_w;
                sum_sq_w += norm_w * norm_w;
                norm_w
            })
            .collect();

        let mu_eff = 1.0 / sum_sq_w.max(1e-6);

        // Adaptation parameters
        let c_sigma = (mu_eff + 2.0) / ((dim as f32) + mu_eff + 5.0);
        let d_sigma = 1.0 + 2.0 * ((mu_eff - 1.0) / ((dim as f32) + 1.0)).max(0.0).sqrt() + c_sigma;
        let c_c = (4.0 + mu_eff / (dim as f32)) / ((dim as f32) + 4.0 + 2.0 * mu_eff / (dim as f32));
        let c_1 = 2.0 / (((dim as f32) + 1.3).powi(2) + mu_eff);
        let c_mu = (2.0 * (mu_eff - 2.0 + 1.0 / mu_eff))
            / (((dim as f32) + 2.0).powi(2) + mu_eff);
        let chi_n = (dim as f32).sqrt() * (1.0 - 1.0 / (4.0 * (dim as f32)) + 1.0 / (21.0 * (dim as f32).powi(2)));

        // Initial mean in normalized [0, 1] coordinates
        let mean: Vec<f32> = bounds.iter().map(|b| b.to_normalized(b.default)).collect();

        Self {
            dim,
            lambda,
            mu,
            weights,
            mu_eff,
            best_solution: mean.clone(),
            best_cost: f32::INFINITY,
            mean,
            sigma: 0.25, // Initial standard deviation in [0, 1] unit hypercube
            p_sigma: vec![0.0; dim],
            p_c: vec![0.0; dim],
            diag_c: vec![1.0; dim],
            c_sigma,
            d_sigma,
            c_c,
            c_1,
            c_mu,
            chi_n,
            rng: DeterministicRng::new(seed),
            generation: 0,
        }
    }

    /// Generates candidate parameter vectors (in normalized [0, 1] space) for evaluation.
    pub fn ask(&mut self) -> Vec<Vec<f32>> {
        let mut population = Vec::with_capacity(self.lambda);

        for _ in 0..self.lambda {
            let mut candidate = Vec::with_capacity(self.dim);
            for d in 0..self.dim {
                let z = self.rng.next_gaussian();
                let std_d = self.diag_c[d].sqrt();
                let val = self.mean[d] + self.sigma * std_d * z;
                // Reflective box boundary clamping to ensure valid parameters
                let clamped = if val < 0.0 {
                    (-val).min(1.0)
                } else if val > 1.0 {
                    (2.0 - val).max(0.0)
                } else {
                    val
                };
                candidate.push(clamped);
            }
            population.push(candidate);
        }

        population
    }

    /// Updates evolution paths, step size sigma, and covariance vector from evaluated candidates.
    pub fn tell(&mut self, mut evaluated_population: Vec<(Vec<f32>, f32)>) {
        if evaluated_population.is_empty() {
            return;
        }

        // Sort ascending by cost (minimization)
        evaluated_population.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        // Update overall best solution
        if evaluated_population[0].1 < self.best_cost {
            self.best_cost = evaluated_population[0].1;
            self.best_solution = evaluated_population[0].0.clone();
        }

        // Recombination: compute new mean
        let old_mean = self.mean.clone();
        let mut new_mean = vec![0.0f32; self.dim];
        for (rank, (cand, _cost)) in evaluated_population.iter().take(self.mu).enumerate() {
            let w = self.weights[rank];
            for d in 0..self.dim {
                new_mean[d] += w * cand[d];
            }
        }

        // Evolution path p_sigma
        let mut p_sigma_norm_sq = 0.0f32;
        for d in 0..self.dim {
            let z_mean = (new_mean[d] - old_mean[d]) / (self.sigma * self.diag_c[d].sqrt().max(1e-6));
            self.p_sigma[d] = (1.0 - self.c_sigma) * self.p_sigma[d]
                + (self.c_sigma * (2.0 - self.c_sigma) * self.mu_eff).sqrt() * z_mean;
            p_sigma_norm_sq += self.p_sigma[d] * self.p_sigma[d];
        }
        let p_sigma_len = p_sigma_norm_sq.sqrt();

        // Evolution path p_c
        for d in 0..self.dim {
            let delta = (new_mean[d] - old_mean[d]) / self.sigma.max(1e-6);
            self.p_c[d] = (1.0 - self.c_c) * self.p_c[d]
                + (self.c_c * (2.0 - self.c_c) * self.mu_eff).sqrt() * delta;
        }

        // Covariance adaptation (diagonal model for numerical robustness and zero allocation)
        for d in 0..self.dim {
            let rank_one = self.p_c[d] * self.p_c[d];
            let mut rank_mu = 0.0f32;
            for (rank, (cand, _cost)) in evaluated_population.iter().take(self.mu).enumerate() {
                let diff = (cand[d] - old_mean[d]) / self.sigma.max(1e-6);
                rank_mu += self.weights[rank] * diff * diff;
            }

            self.diag_c[d] = (1.0 - self.c_1 - self.c_mu) * self.diag_c[d]
                + self.c_1 * rank_one
                + self.c_mu * rank_mu;

            // Prevent variance collapse
            self.diag_c[d] = self.diag_c[d].clamp(1e-4, 25.0);
        }

        // Step-size adaptation (Cumulative Step-size Adaptation)
        let sigma_update = (self.c_sigma / self.d_sigma) * ((p_sigma_len / self.chi_n) - 1.0);
        self.sigma *= sigma_update.clamp(-0.5, 0.5).exp();
        self.sigma = self.sigma.clamp(1e-5, 2.0);

        self.mean = new_mean;
        self.generation += 1;
    }
}
