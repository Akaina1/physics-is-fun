// Geodesic equations and photon state for Kerr spacetime

use std::f64::consts::PI;
use crate::coordinates::{sigma, delta};

// ============================================================================
// GEODESIC STATE (PHOTON/RAY TRACKING)
// ============================================================================

// State of a photon/ray during geodesic integration
// 
// Physics: In Kerr spacetime, a photon's path is completely determined by:
// - Current position (t, r, θ, φ) in Boyer-Lindquist-like coordinates
// - Momentum direction (encapsulated in conserved quantities)
// - Three constants of motion: E (energy), L_z (angular momentum), Q (Carter constant)
// 
// These conserved quantities stay constant along the geodesic and make
// integration much more efficient than tracking full 4-velocity.
#[derive(Debug, Clone, Copy)]
pub struct PhotonState {
    // Boyer-Lindquist-like coordinates (we'll use Kerr-Schild for actual integration)
    
    // Time coordinate (we don't track this explicitly for null geodesics)
    // pub t: f64,
    
    // Radial coordinate (distance from center, but NOT Euclidean distance!)
    // This is the "r" in the Kerr metric
    pub r: f64,
    
    // Polar angle θ ∈ [0, π]
    // θ=0: north pole, θ=π/2: equator, θ=π: south pole
    pub theta: f64,
    
    // Azimuthal angle φ ∈ [0, 2π]
    // Angle around the rotation axis
    pub phi: f64,
    
    // Conserved quantities (constants of motion)
    // These are THE KEY to efficient geodesic integration in Kerr spacetime
    
    // Energy (actually -p_t, the time component of momentum)
    // For photons from infinity, E ≈ 1
    pub energy: f64,
    
    // z-component of angular momentum (p_φ)
    // This is the "impact parameter" - how much the photon "misses" the BH
    pub angular_momentum: f64,
    
    // Carter's constant Q (separates the equations of motion)
    // Related to θ motion; Q=0 for equatorial orbits
    // This is what makes Kerr geodesics separable!
    pub carter_q: f64,
}

impl PhotonState {
    // Create a new photon state at a given position with conserved quantities
    pub fn new(r: f64, theta: f64, phi: f64, energy: f64, angular_momentum: f64, carter_q: f64) -> Self {
        assert!(r > 0.0, "Radius must be positive");
        assert!(theta >= 0.0 && theta <= PI, "Theta must be in [0, π]");
        Self {
            r,
            theta,
            phi,
            energy,
            angular_momentum,
            carter_q,
        }
    }
    
    // Check if photon has crossed the equatorial plane (z=0, or θ=π/2)
    // This is where the accretion disc lives
    #[inline]
    pub fn is_at_equator(&self) -> bool {
        (self.theta - PI / 2.0).abs() < 1e-6
    }
    
    // Get the "impact parameter" b = L_z / E
    // This is the classical notion of how far the photon would miss the BH
    // if gravity didn't bend its path
    #[inline]
    pub fn impact_parameter(&self) -> f64 {
        self.angular_momentum / self.energy
    }
}

// ============================================================================
// GEODESIC INTEGRATION RESULT
// ============================================================================

// Result of integrating a single geodesic (ray trace)
// 
// After we trace a light ray backward from the camera, we need to record:
// - Did it hit the disc? (or escape to infinity / fall into horizon)
// - If it hit the disc, where? (r, θ, φ coordinates)
// - What are the conserved quantities? (for Doppler shift calculation)
// - Which order is this? (primary, secondary, tertiary image)
// - Path statistics and validation metrics
#[derive(Debug, Clone, Copy)]
pub enum GeodesicResult {
    // Ray hit the accretion disc at radius r and angle φ
    DiscHit {
        r: f64,                   // Radial coordinate where disc was hit
        theta: f64,               // Polar angle at disc intersection
        phi: f64,                 // Azimuthal angle at hit point
        energy: f64,              // Conserved energy E = -k_t
        angular_momentum: f64,    // Conserved L_z = k_φ
        carter_q: f64,            // Carter constant Q
        impact_parameter: f64,    // b = L_z/E
        redshift_factor: f64,     // Gravitational redshift g-factor
        affine_parameter: f64,    // Affine parameter λ at hit
        phi_wraps: f64,           // Number of φ wraps around BH
        order: u8,                // 0=direct, 1=primary, 2=secondary, etc.
        null_invariant_error: f64, // |g_μν k^μ k^ν| for validation
    },
    
    // Ray fell into the black hole (crossed horizon)
    Captured,
    
    // Ray escaped to infinity (no disc intersection)
    Escaped,
}

impl GeodesicResult {
    // Check if this result represents a disc hit
    #[inline]
    pub fn is_hit(&self) -> bool {
        matches!(self, GeodesicResult::DiscHit { .. })
    }
    
    // Get the disc hit data, if any (returns None otherwise)
    pub fn disc_hit_data(&self) -> Option<(f64, f64, f64, f64, f64, f64, f64, f64, f64, f64, u8, f64)> {
        match self {
            GeodesicResult::DiscHit { 
                r, theta, phi, energy, angular_momentum, carter_q, 
                impact_parameter, redshift_factor, affine_parameter, phi_wraps,
                order, null_invariant_error 
            } => {
                Some((*r, *theta, *phi, *energy, *angular_momentum, *carter_q, 
                      *impact_parameter, *redshift_factor, *affine_parameter, *phi_wraps,
                      *order, *null_invariant_error))
            }
            _ => None,
        }
    }
}

// ============================================================================
// GEODESIC EQUATIONS OF MOTION
// ============================================================================

// Equations of motion for null geodesics in Kerr spacetime
// 
// Physics Background:
// Light follows null geodesics: paths where ds² = 0
// In Kerr spacetime, these are governed by the geodesic equation:
//   d²x^μ/dλ² + Γ^μ_νρ (dx^ν/dλ)(dx^ρ/dλ) = 0
// 
// However, we use CONSERVED QUANTITIES (E, L_z, Q) to simplify.
// This reduces the problem to three first-order ODEs:
//   dr/dλ, dθ/dλ, dφ/dλ
// 
// These are the "Carter equations" - separable thanks to Carter's Q constant!

// Compute dr/dλ (radial equation of motion)
// 
// Physics: R(r) = [(r² + a²)E - aL_z]² - Δ[Q + (L_z - aE)²]
//          dr/dλ = ±√R(r) / Σ
// 
// The sign determines inward (-) vs outward (+) motion
pub fn geodesic_dr_dlambda(
    r: f64, 
    theta: f64,
    energy: f64,      // E = -k_t
    angular_momentum: f64,  // L_z = k_φ  
    carter_q: f64,    // Q (Carter constant)
    m: f64,           // Black hole mass
    a: f64,           // Black hole spin
    sign: f64,        // ±1 for direction
) -> f64 {
    let r2 = r * r;
    let a2 = a * a;
    
    // Δ = r² - 2Mr + a²
    let delta_val = delta(r, m, a);
    
    // First term: [(r² + a²)E - aL_z]²
    let term1 = (r2 + a2) * energy - a * angular_momentum;
    let term1_sq = term1 * term1;
    
    // Second term: (L_z - aE)²
    let term2 = angular_momentum - a * energy;
    let term2_sq = term2 * term2;
    
    // R(r) = term1² - Δ[Q + term2²]
    let r_potential = term1_sq - delta_val * (carter_q + term2_sq);
    
    // Σ = r² + a²cos²θ
    let sigma_val = sigma(r, theta, a);
    
    // dr/dλ = sign × √R(r) / Σ
    // Handle negative R gracefully (shouldn't happen, but numerical errors)
    if r_potential < 0.0 {
        return 0.0;  // Turning point
    }
    
    sign * r_potential.sqrt() / sigma_val
}

// Compute dθ/dλ (polar equation of motion)
// 
// Physics: Θ(θ) = Q - cos²θ[a²(E² - 1) + L_z²/sin²θ]
//          dθ/dλ = ±√Θ(θ) / Σ
// 
// For photons (massless), we use E²-1 → E² (slight modification)
pub fn geodesic_dtheta_dlambda(
    r: f64,
    theta: f64,
    energy: f64,
    angular_momentum: f64,
    carter_q: f64,
    a: f64,
    sign: f64,        // ±1 for direction
) -> f64 {
    let a2 = a * a;
    let cos_theta = theta.cos();
    let sin_theta = theta.sin();
    let cos2_theta = cos_theta * cos_theta;
    let sin2_theta = sin_theta * sin_theta;
    
    // Avoid division by zero at poles (θ=0 or π)
    if sin2_theta < 1e-10 {
        return 0.0;
    }
    
    // Θ(θ) = Q - cos²θ × [a²E² + L_z²/sin²θ]
    // For photons from infinity, E ≈ 1, so a²(E²-1) ≈ 0, simplifies to:
    let term1 = a2 * energy * energy * cos2_theta;
    let term2 = (angular_momentum * angular_momentum / sin2_theta) * cos2_theta;
    
    let theta_potential = carter_q - term1 - term2;
    
    // Validation moved to validation.rs module
    
    // Σ = r² + a²cos²θ
    let sigma_val = sigma(r, theta, a);
    
    // dθ/dλ = sign × √Θ(θ) / Σ
    if theta_potential < 0.0 {
        return 0.0;  // Turning point
    }
    
    sign * theta_potential.sqrt() / sigma_val
}

// Compute dφ/dλ (azimuthal equation of motion)
// 
// Physics: dφ/dλ = [aE(r²+a² - Δ) + (1 - Δ/Σ)L_z/sin²θ] / Σ
// 
// This equation includes frame-dragging effects (the 'aE' term)
// Frame-dragging: rotation of spacetime itself near spinning BH
pub fn geodesic_dphi_dlambda(
    r: f64,
    theta: f64,
    energy: f64,
    angular_momentum: f64,
    m: f64,
    a: f64,
) -> f64 {
    let r2 = r * r;
    let a2 = a * a;
    let sin_theta = theta.sin();
    let sin2_theta = sin_theta * sin_theta;
    
    // Avoid division by zero at poles
    if sin2_theta < 1e-10 {
        return 0.0;
    }
    
    // Δ and Σ
    let delta_val = delta(r, m, a);
    let sigma_val = sigma(r, theta, a);
    
    // Frame-dragging term: aE(r² + a² - Δ)
    let frame_drag = a * energy * (r2 + a2 - delta_val);
    
    // Angular momentum term: L_z × (1 - Δ/Σ) / sin²θ
    let ang_mom_term = angular_momentum * (1.0 - delta_val / sigma_val) / sin2_theta;
    
    // dφ/dλ = [frame_drag + ang_mom_term] / Σ
    (frame_drag + ang_mom_term) / sigma_val
}

// Compute the null geodesic invariant g_μν k^μ k^ν
// 
// Physics: For null (lightlike) geodesics, this should be exactly 0
// We use this as a validation check for numerical accuracy
// 
// Returns: |g_μν k^μ k^ν| (absolute value of the invariant, should be ~0)
// 
// Note: This is derived from the first integral of geodesic motion:
// g_μν (dx^μ/dλ)(dx^ν/dλ) = constant = 0 for null geodesics
pub fn compute_null_invariant(
    r: f64,
    theta: f64,
    energy: f64,          // E = -k_t
    angular_momentum: f64, // L_z = k_φ
    carter_q: f64,        // Q (Carter constant)
    m: f64,
    a: f64,
) -> f64 {
    let r2 = r * r;
    let a2 = a * a;
    let sin2_theta = theta.sin().powi(2);
    
    let delta_val = delta(r, m, a);
    let sigma_val = sigma(r, theta, a);
    
    // From Carter's formulation, the null geodesic condition is:
    // Σ² g_μν k^μ k^ν = -[(r²+a²)E - aL_z]² + Δ[(L_z - aE)² + Q] + Q
    //
    // For a valid null geodesic with properly chosen E, L_z, Q, this equals 0
    
    let term1 = (r2 + a2) * energy - a * angular_momentum;
    let term2 = angular_momentum - a * energy;
    
    // The Carter null condition:
    // R(r) + Θ(θ) = 0 when expanded properly
    // R(r) = [(r²+a²)E - aL_z]² - Δ[Q + (L_z-aE)²]
    // Θ(θ) = Q - [a²E²cos²θ + L_z²cos²θ/sin²θ]
    
    // Simpler approach: use the first integral directly
    // For null geodesics: (dr/dλ)² + V_eff = 0 where V_eff is the effective potential
    // The invariant is: Σ⁻² [R(r) + Θ(θ)]
    
    let r_potential = term1 * term1 - delta_val * (carter_q + term2 * term2);
    
    let cos2_theta = theta.cos().powi(2);
    let theta_potential = carter_q - a2 * energy * energy * cos2_theta 
                          - (angular_momentum * angular_momentum / sin2_theta.max(1e-10)) * cos2_theta;
    
    // The null invariant (normalized by Σ²)
    let invariant = (r_potential + theta_potential) / (sigma_val * sigma_val);
    
    invariant.abs()
}

// Compute the redshift factor g at a point on the disc
// 
// Physics: g = -u_μ k^μ
// where u_μ is the 4-velocity of the disc material (in covariant form)
// and k^μ is the photon 4-momentum (contravariant)
// 
// This gives the ratio of emitted to observed frequency: ν_obs/ν_emit = g
// Includes both gravitational redshift (from metric) and Doppler shift (from disc motion)
pub fn compute_redshift_factor(
    r: f64,
    theta: f64,
    energy: f64,
    angular_momentum: f64,
    m: f64,
    a: f64,
    omega_disc: f64,  // Angular velocity of disc at this radius
) -> f64 {
    let r2 = r * r;
    let a2 = a * a;
    let sigma_val = sigma(r, theta, a);
    let sin2_theta = theta.sin().powi(2);
    
    // Kerr metric components in Boyer-Lindquist coordinates
    let g_tt = -(1.0 - 2.0 * m * r / sigma_val);
    let g_t_phi = -2.0 * m * r * a * sin2_theta / sigma_val;
    let g_phi_phi = (r2 + a2 + 2.0 * m * r * a2 * sin2_theta / sigma_val) * sin2_theta;
    
    // For a circular orbit in the equatorial plane with angular velocity Ω:
    // The 4-velocity must satisfy: g_μν u^μ u^ν = -1
    // With the constraint: u^φ = Ω u^t (corotating disc)
    
    // Substituting: g_tt (u^t)² + 2 g_tφ u^t u^φ + g_φφ (u^φ)² = -1
    //              g_tt (u^t)² + 2 g_tφ Ω (u^t)² + g_φφ Ω² (u^t)² = -1
    //              (u^t)² [g_tt + 2Ω g_tφ + Ω² g_φφ] = -1
    
    let norm_factor = g_tt + 2.0 * omega_disc * g_t_phi + omega_disc * omega_disc * g_phi_phi;
    
    // u^t (contravariant time component)
    let u_t = if norm_factor < 0.0 {
        1.0 / (-norm_factor).sqrt()
    } else {
        // Fallback for numerical edge cases
        1.0
    };
    
    // u^φ (contravariant phi component)
    let u_phi = omega_disc * u_t;
    
    // Now compute u_μ (covariant components) using g_μν:
    // u_t = g_tt u^t + g_tφ u^φ
    // u_φ = g_φt u^t + g_φφ u^φ  (note: g_φt = g_tφ by symmetry)
    
    let u_cov_t = g_tt * u_t + g_t_phi * u_phi;
    let u_cov_phi = g_t_phi * u_t + g_phi_phi * u_phi;
    
    // Photon 4-momentum (contravariant):
    // k^t = -E (energy, defined as -p_t)
    // k^φ = L_z (angular momentum)
    let k_t = -energy;
    let k_phi = angular_momentum;
    
    // Redshift factor: g = -u_μ k^μ = -(u_t k^t + u_φ k^φ)
    let redshift = -(u_cov_t * k_t + u_cov_phi * k_phi);
    
    redshift.abs()
}

