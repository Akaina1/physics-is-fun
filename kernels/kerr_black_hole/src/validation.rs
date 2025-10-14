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

