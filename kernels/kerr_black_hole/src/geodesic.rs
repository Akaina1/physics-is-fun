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
// - If it hit the disc, where? (r, φ coordinates)
// - What are the conserved quantities? (for Doppler shift calculation)
// - Which order is this? (primary, secondary, tertiary image)
#[derive(Debug, Clone, Copy)]
pub enum GeodesicResult {
    // Ray hit the accretion disc at radius r and angle φ
    DiscHit {
        r: f64,              // Radial coordinate where disc was hit
        phi: f64,            // Azimuthal angle at hit point
        energy: f64,         // Conserved energy E = -k_t
        angular_momentum: f64, // Conserved L_z = k_φ
        order: u8,           // 0=direct, 1=primary, 2=secondary, etc.
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
    pub fn disc_hit_data(&self) -> Option<(f64, f64, f64, f64, u8)> {
        match self {
            GeodesicResult::DiscHit { r, phi, energy, angular_momentum, order } => {
                Some((*r, *phi, *energy, *angular_momentum, *order))
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

