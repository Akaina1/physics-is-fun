// Geodesic integration using RK4 method

use std::f64::consts::PI;
use crate::geodesic::{PhotonState, GeodesicResult, geodesic_dr_dlambda, geodesic_dtheta_dlambda, geodesic_dphi_dlambda, compute_null_invariant, compute_redshift_factor};
use crate::types::{BlackHole};

// ============================================================================
// GEODESIC INTEGRATION STEPPER
// ============================================================================

// Integration state for a single step
// Holds the current position and direction signs for integration
#[derive(Debug, Clone, Copy)]
struct IntegrationState {
    r: f64,
    theta: f64,
    phi: f64,
    lambda: f64,      // Affine parameter (integrated step size)
    initial_phi: f64, // Starting phi value (to track wraps)
    sign_r: f64,      // ±1: direction of radial motion
    sign_theta: f64,  // ±1: direction of polar motion
}

impl IntegrationState {
    fn new(photon: &PhotonState) -> Self {
        Self {
            r: photon.r,
            theta: photon.theta,
            phi: photon.phi,
            lambda: 0.0,
            initial_phi: photon.phi,
            sign_r: -1.0,      // Start moving inward (toward BH)
            sign_theta: 1.0,   // Start moving toward equator
        }
    }
}

// Single step of RK4 (4th order Runge-Kutta) integration
// 
// Physics: RK4 is a numerical method for solving ODEs (ordinary differential equations)
// It's more accurate than simple Euler method:
// - Euler: y_{n+1} = y_n + h*f(y_n)  [1st order, large errors]
// - RK4: Uses 4 evaluations per step for 4th order accuracy [much better!]
// 
// For our geodesic: dy/dλ = f(y), where y = (r, θ, φ)
fn rk4_step(
    state: &IntegrationState,
    energy: f64,
    angular_momentum: f64,
    carter_q: f64,
    m: f64,
    a: f64,
    step_size: f64,
) -> IntegrationState {
    let h = step_size;
    
    // k1 = f(y_n)
    let dr1 = geodesic_dr_dlambda(
        state.r, state.theta, energy, angular_momentum, carter_q, m, a, state.sign_r
    );
    let dtheta1 = geodesic_dtheta_dlambda(
        state.r, state.theta, energy, angular_momentum, carter_q, a, state.sign_theta
    );
    let dphi1 = geodesic_dphi_dlambda(
        state.r, state.theta, energy, angular_momentum, m, a
    );
    
    // k2 = f(y_n + h/2 * k1)
    let r2 = state.r + 0.5 * h * dr1;
    let theta2 = state.theta + 0.5 * h * dtheta1;
    let dr2 = geodesic_dr_dlambda(
        r2, theta2, energy, angular_momentum, carter_q, m, a, state.sign_r
    );
    let dtheta2 = geodesic_dtheta_dlambda(
        r2, theta2, energy, angular_momentum, carter_q, a, state.sign_theta
    );
    let dphi2 = geodesic_dphi_dlambda(
        r2, theta2, energy, angular_momentum, m, a
    );
    
    // k3 = f(y_n + h/2 * k2)
    let r3 = state.r + 0.5 * h * dr2;
    let theta3 = state.theta + 0.5 * h * dtheta2;
    let dr3 = geodesic_dr_dlambda(
        r3, theta3, energy, angular_momentum, carter_q, m, a, state.sign_r
    );
    let dtheta3 = geodesic_dtheta_dlambda(
        r3, theta3, energy, angular_momentum, carter_q, a, state.sign_theta
    );
    let dphi3 = geodesic_dphi_dlambda(
        r3, theta3, energy, angular_momentum, m, a
    );
    
    // k4 = f(y_n + h * k3)
    let r4 = state.r + h * dr3;
    let theta4 = state.theta + h * dtheta3;
    let dr4 = geodesic_dr_dlambda(
        r4, theta4, energy, angular_momentum, carter_q, m, a, state.sign_r
    );
    let dtheta4 = geodesic_dtheta_dlambda(
        r4, theta4, energy, angular_momentum, carter_q, a, state.sign_theta
    );
    let dphi4 = geodesic_dphi_dlambda(
        r4, theta4, energy, angular_momentum, m, a
    );
    
    // Combine: y_{n+1} = y_n + h/6 * (k1 + 2*k2 + 2*k3 + k4)
    let new_r = state.r + (h / 6.0) * (dr1 + 2.0*dr2 + 2.0*dr3 + dr4);
    let new_theta = state.theta + (h / 6.0) * (dtheta1 + 2.0*dtheta2 + 2.0*dtheta3 + dtheta4);
    let new_phi = state.phi + (h / 6.0) * (dphi1 + 2.0*dphi2 + 2.0*dphi3 + dphi4);
    
    // Check for turning points and flip signs if needed
    let mut new_sign_r = state.sign_r;
    let mut new_sign_theta = state.sign_theta;
    
    // Radial turning point: if dr/dλ changes sign or goes to zero
    if dr1 * dr4 < 0.0 || dr4.abs() < 1e-10 {
        new_sign_r = -state.sign_r;
    }
    
    // Polar turning point: if dθ/dλ changes sign (bounce off poles or equator)
    if dtheta1 * dtheta4 < 0.0 || dtheta4.abs() < 1e-10 {
        new_sign_theta = -state.sign_theta;
    }
    
    IntegrationState {
        r: new_r,
        theta: new_theta,
        phi: new_phi,
        lambda: state.lambda + h,
        initial_phi: state.initial_phi,
        sign_r: new_sign_r,
        sign_theta: new_sign_theta,
    }
}

// Calculate Keplerian angular velocity at radius r
// For a Keplerian disc orbiting a Kerr black hole
fn keplerian_omega(r: f64, m: f64, a: f64) -> f64 {
    // Ω = M^(1/2) / (r^(3/2) + a M^(1/2))
    // This is the angular velocity for a circular orbit in the equatorial plane
    let sqrt_m = m.sqrt();
    let r_3_2 = r.powf(1.5);
    sqrt_m / (r_3_2 + a * sqrt_m)
}

// Adaptive step size control
// 
// Physics: Near the black hole or disc, geodesics curve sharply
// → Need smaller steps for accuracy
// Far from BH, geodesics are nearly straight
// → Can use larger steps for efficiency
fn adaptive_step_size(r: f64, theta: f64, base_step: f64) -> f64 {
    // Smaller steps near horizon and near equator (where disc is)
    let r_factor = (r / 2.0).max(0.1);  // Smaller for small r
    let theta_factor = (theta - PI/2.0).abs().max(0.1);  // Smaller near equator
    
    base_step * r_factor * theta_factor
}

// ============================================================================
// DISC INTERSECTION DETECTION
// ============================================================================

// Check if ray intersects the accretion disc at current position
// 
// Physics: The disc is a thin structure in the equatorial plane (θ = π/2)
// It extends from r_inner (ISCO) to r_outer (e.g., 20M)
fn check_disc_intersection(
    r: f64,
    theta: f64,
    prev_theta: f64,
    r_inner: f64,
    r_outer: f64,
) -> bool {
    // The accretion disc is a geometrically thin, optically thick structure
    // centered on the equatorial plane (θ = π/2)
    // 
    // Physical disc model (Shakura-Sunyaev):
    // - Scale height H(r) = (H/R) × r, where H/R ≈ 0.01-0.1 for thin discs
    // - Disc exists in volume: |z| < H(r), or equivalently |θ - π/2| < H/r
    // 
    // We use H/R = 0.05 (realistic for geometrically thin disc)
    
    let equator = PI / 2.0;
    
    // Physical disc thickness: H/R = 0.05
    // At radius r, the disc extends from θ = π/2 - H/r to θ = π/2 + H/r
    const HEIGHT_TO_RADIUS: f64 = 0.05;
    let disc_half_thickness = HEIGHT_TO_RADIUS;  // In radians, approximately H/r for r >> H
    
    // Check if ray crosses the equatorial plane
    let crossed = (prev_theta - equator) * (theta - equator) < 0.0;
    
    // Check if ray is within disc thickness (emission volume)
    let theta_dist = (theta - equator).abs();
    let in_disc_volume = theta_dist < disc_half_thickness;
    
    // Check if we're within the disc's radial extent
    let in_disc_radius = r >= r_inner && r <= r_outer;
    
    (crossed || in_disc_volume) && in_disc_radius
}

// ============================================================================
// MAIN GEODESIC INTEGRATION
// ============================================================================

// Integrate a single geodesic and collect ALL disc crossings up to max_orders
// 
// This is more efficient than tracing separately per order - we do one integration
// and collect multiple crossings along the way.
// 
// Physics: Photons can cross the disc plane multiple times due to extreme
// gravitational lensing. Each crossing creates a different "order" image:
// - Order 0: Primary image (direct view)
// - Order 1: Secondary image (photon ring - wraps ~180-360° around BH)
// - Order 2+: Higher-order images (increasingly faint subrings)
// 
// Returns: Vec of results, one per order (0..max_orders)
//          Non-hits are marked as Escaped/Captured
pub fn integrate_geodesic_multi_order(
    photon: PhotonState,
    black_hole: &BlackHole,
    r_disc_inner: f64,
    r_disc_outer: f64,
    max_orders: u8,
    max_steps: usize,
) -> Vec<GeodesicResult> {
    let m = black_hole.mass;
    let a = black_hole.spin;
    let r_horizon = black_hole.horizon_radius();
    
    let mut state = IntegrationState::new(&photon);
    let mut results = vec![GeodesicResult::Escaped; max_orders as usize];
    let mut disc_crossings = 0u8;
    
    let base_step = 0.05;
    
    // Integration loop
    for _ in 0..max_steps {
        let h = adaptive_step_size(state.r, state.theta, base_step);
        let prev_theta = state.theta;
        
        // Take one RK4 step
        state = rk4_step(
            &state,
            photon.energy,
            photon.angular_momentum,
            photon.carter_q,
            m,
            a,
            h,
        );
        
        // Check stopping conditions
        
        // 1. Crossed horizon → mark remaining as captured
        if state.r < r_horizon * 1.01 {
            for i in disc_crossings..max_orders {
                results[i as usize] = GeodesicResult::Captured;
            }
            return results;
        }
        
        // 2. Escaped to infinity → remaining already marked as Escaped
        if state.r > 1000.0 {
            return results;
        }
        
        // 3. Check for disc intersection
        if check_disc_intersection(state.r, state.theta, prev_theta, r_disc_inner, r_disc_outer) {
            // Store this crossing if we haven't collected all orders yet
            if disc_crossings < max_orders {
                // Calculate all derived quantities
                let impact_param = photon.angular_momentum / photon.energy;
                let omega = keplerian_omega(state.r, m, a);
                let redshift = compute_redshift_factor(
                    state.r, state.theta, photon.energy, photon.angular_momentum, m, a, omega
                );
                let null_error = compute_null_invariant(
                    state.r, state.theta, photon.energy, photon.angular_momentum, photon.carter_q, m, a
                );
                
                // Calculate phi wraps (total phi travel / 2π)
                let phi_total = (state.phi - state.initial_phi).abs();
                let phi_wraps = phi_total / (2.0 * PI);
                
                results[disc_crossings as usize] = GeodesicResult::DiscHit {
                    r: state.r,
                    theta: state.theta,
                    phi: state.phi,
                    energy: photon.energy,
                    angular_momentum: photon.angular_momentum,
                    carter_q: photon.carter_q,
                    impact_parameter: impact_param,
                    redshift_factor: redshift,
                    affine_parameter: state.lambda,
                    phi_wraps,
                    order: disc_crossings,
                    null_invariant_error: null_error,
                };
            }
            
            disc_crossings += 1;
            
            // Early termination: if we have all orders, stop integrating
            if disc_crossings >= max_orders {
                return results;
            }
            
            // Otherwise continue to find next order
        }
        
        // Safety checks
        if state.theta < 0.0 || state.theta > PI {
            state.theta = state.theta.rem_euclid(PI);
        }
        
        state.phi = state.phi.rem_euclid(2.0 * PI);
    }
    
    // Max steps reached - return what we have
    results
}

// Legacy single-order function (for backward compatibility)
// 
// This wraps the multi-order function to maintain API compatibility
pub fn integrate_geodesic(
    photon: PhotonState,
    black_hole: &BlackHole,
    r_disc_inner: f64,   // ISCO radius
    r_disc_outer: f64,   // Outer edge of disc
    max_steps: usize,    // Safety limit to prevent infinite loops
) -> GeodesicResult {
    let results = integrate_geodesic_multi_order(
        photon,
        black_hole,
        r_disc_inner,
        r_disc_outer,
        1,  // Only get order 0
        max_steps,
    );
    results[0]
}

