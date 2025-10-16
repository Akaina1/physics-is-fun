// Geodesic validation and invariant checking

use crate::geodesic::PhotonState;
use crate::coordinates::{sigma, delta};

// ============================================================================
// NULL GEODESIC INVARIANT
// ============================================================================

// Check the null geodesic invariant: g_μν k^μ k^ν = 0
// 
// Physics: For photons (massless particles), the 4-momentum k must satisfy
// the null condition. This is a fundamental constraint that should be
// preserved throughout the integration.
// 
// In Boyer-Lindquist coordinates for Kerr:
// g_μν k^μ k^ν = g_tt (k^t)² + 2 g_tφ k^t k^φ + g_rr (k^r)² + g_θθ (k^θ)² + g_φφ (k^φ)²
// 
// For our conserved quantities (E, L_z, Q), we can check this indirectly
pub fn check_null_invariant(
    photon: &PhotonState,
    dr_dlambda: f64,
    dtheta_dlambda: f64,
    dphi_dlambda: f64,
    m: f64,
    a: f64,
) -> f64 {
    let r = photon.r;
    let theta = photon.theta;
    let energy = photon.energy;
    let _l_z = photon.angular_momentum;  // Reserved for future use
    
    // Metric components
    let sigma_val = sigma(r, theta, a);
    let delta_val = delta(r, m, a);
    
    let r2 = r * r;
    let a2 = a * a;
    let sin_theta = theta.sin();
    let sin2_theta = sin_theta * sin_theta;
    let _cos_theta = theta.cos();  // Reserved for future use
    
    // A² = (r² + a²)² - a²Δsin²θ
    let a_squared = (r2 + a2) * (r2 + a2) - a2 * delta_val * sin2_theta;
    
    // Metric components in Boyer-Lindquist
    // g_tt = -(1 - 2Mr/Σ)
    let g_tt = -(1.0 - 2.0 * m * r / sigma_val);
    
    // g_tφ = -2Mra sin²θ / Σ
    let g_tphi = -2.0 * m * r * a * sin2_theta / sigma_val;
    
    // g_rr = Σ/Δ
    let g_rr = sigma_val / delta_val;
    
    // g_θθ = Σ
    let g_thth = sigma_val;
    
    // g_φφ = A²sin²θ/Σ
    let g_phiphi = a_squared * sin2_theta / sigma_val;
    
    // 4-velocity components (using affine parameter λ)
    let k_t = energy;  // -k_t in our convention, but we use E directly
    let k_r = dr_dlambda;
    let k_theta = dtheta_dlambda;
    let k_phi = dphi_dlambda;
    
    // Compute g_μν k^μ k^ν
    // Note: Our E is actually -k_t, so we need to be careful with signs
    let invariant = g_tt * k_t * k_t
        + 2.0 * g_tphi * k_t * k_phi
        + g_rr * k_r * k_r
        + g_thth * k_theta * k_theta
        + g_phiphi * k_phi * k_phi;
    
    // Should be zero for null geodesics
    invariant.abs()
}

// Track statistics about invariant errors during integration
#[derive(Debug, Default)]
pub struct ValidationStats {
    pub max_invariant_error: f64,
    pub mean_invariant_error: f64,
    pub sample_count: usize,
}

impl ValidationStats {
    pub fn new() -> Self {
        Self {
            max_invariant_error: 0.0,
            mean_invariant_error: 0.0,
            sample_count: 0,
        }
    }
    
    pub fn update(&mut self, error: f64) {
        self.max_invariant_error = self.max_invariant_error.max(error);
        
        // Running mean calculation
        let n = self.sample_count as f64;
        self.mean_invariant_error = (self.mean_invariant_error * n + error) / (n + 1.0);
        
        self.sample_count += 1;
    }
    
    pub fn report(&self) -> String {
        format!(
            "Validation Stats: max_error={:.2e}, mean_error={:.2e}, samples={}",
            self.max_invariant_error,
            self.mean_invariant_error,
            self.sample_count
        )
    }
}

// ============================================================================
// GEODESIC POTENTIAL VALIDATION
// ============================================================================

// Check if θ-potential allows motion toward equator
// 
// Physics (null): For a photon to reach the equatorial plane (θ=π/2), the θ-potential
// Θ(θ) = Q + a²E² cos²θ - (L_z²/sin²θ) cos²θ must be positive at the starting θ.
// If Θ(θ) < 0, the photon is at a turning point and cannot change θ.
pub fn check_theta_potential(
    photon: &PhotonState,
    a: f64,
) -> (f64, bool) {
    let theta = photon.theta;
    let energy = photon.energy;
    let angular_momentum = photon.angular_momentum;
    let carter_q = photon.carter_q;
    
    let a2 = a * a;
    let cos_theta = theta.cos();
    let sin_theta = theta.sin();
    let cos2_theta = cos_theta * cos_theta;
    let sin2_theta = sin_theta * sin_theta;
    
    // Avoid division by zero at poles
    if sin2_theta < 1e-10 {
        return (0.0, false);
    }
    
    // Calculate θ-potential (null form)
    let term1 = a2 * energy * energy * cos2_theta;
    let term2 = (angular_momentum * angular_momentum / sin2_theta) * cos2_theta;
    let theta_potential = carter_q + term1 - term2;
    
    let can_reach_equator = theta_potential > 0.0;
    
    (theta_potential, can_reach_equator)
}

// Check if r-potential allows inward motion
// 
// Physics: For a photon to fall toward the black hole, the r-potential
// R(r) must be positive. If R(r) < 0, the photon is at a turning point.
pub fn check_r_potential(
    photon: &PhotonState,
    m: f64,
    a: f64,
) -> (f64, bool) {
    let r = photon.r;
    let energy = photon.energy;
    let angular_momentum = photon.angular_momentum;
    let carter_q = photon.carter_q;
    
    let r2 = r * r;
    let a2 = a * a;
    
    // Δ = r² - 2Mr + a²
    let delta_val = delta(r, m, a);
    
    // R(r) = [(r² + a²)E - aL_z]² - Δ[Q + (L_z - aE)²]
    let term1 = (r2 + a2) * energy - a * angular_momentum;
    let term1_sq = term1 * term1;
    
    let term2 = angular_momentum - a * energy;
    let term2_sq = term2 * term2;
    
    let r_potential = term1_sq - delta_val * (carter_q + term2_sq);
    
    let can_fall_inward = r_potential > 0.0;
    
    (r_potential, can_fall_inward)
}

// Comprehensive geodesic validation
pub fn validate_geodesic_state(
    photon: &PhotonState,
    m: f64,
    a: f64,
) -> GeodesicValidation {
    let (theta_pot, theta_ok) = check_theta_potential(photon, a);
    let (r_pot, r_ok) = check_r_potential(photon, m, a);
    
    GeodesicValidation {
        theta_potential: theta_pot,
        r_potential: r_pot,
        can_reach_equator: theta_ok,
        can_fall_inward: r_ok,
        is_valid: theta_ok && r_ok,
    }
}

#[derive(Debug)]
pub struct GeodesicValidation {
    pub theta_potential: f64,
    pub r_potential: f64,
    pub can_reach_equator: bool,
    pub can_fall_inward: bool,
    pub is_valid: bool,
}

impl GeodesicValidation {
    pub fn report(&self) -> String {
        format!(
            "Geodesic Validation: Θ={:.6} ({}), R={:.6} ({}), Valid={}",
            self.theta_potential,
            if self.can_reach_equator { "OK" } else { "BLOCKED" },
            self.r_potential,
            if self.can_fall_inward { "OK" } else { "BLOCKED" },
            self.is_valid
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validation_stats() {
        let mut stats = ValidationStats::new();
        
        stats.update(1e-10);
        stats.update(2e-10);
        stats.update(3e-10);
        
        assert_eq!(stats.sample_count, 3);
        assert!((stats.max_invariant_error - 3e-10).abs() < 1e-15);
        assert!((stats.mean_invariant_error - 2e-10).abs() < 1e-15);
    }
}

