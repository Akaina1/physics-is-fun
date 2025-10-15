// Geodesic integration using RK4 method

use std::f64::consts::PI;
use crate::geodesic::{PhotonState, GeodesicResult, geodesic_dr_dlambda, geodesic_dtheta_dlambda, geodesic_dphi_dlambda};
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
    sign_r: f64,      // ±1: direction of radial motion
    sign_theta: f64,  // ±1: direction of polar motion
}

impl IntegrationState {
    fn new(photon: &PhotonState) -> Self {
        Self {
            r: photon.r,
            theta: photon.theta,
            phi: photon.phi,
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
        sign_r: new_sign_r,
        sign_theta: new_sign_theta,
    }
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
    // The disc lies in the equatorial plane (θ = π/2)
    // We need to detect TWO cases:
    // 1. Ray crosses the equator (prev and current theta on opposite sides)
    // 2. Ray is traveling along the equator (both very close to π/2)
    
    let equator = PI / 2.0;
    // For a thin disc in the equatorial plane, we need to check:
    // 1. Ray crosses the equatorial plane (θ = π/2)
    // 2. OR ray is close enough to the plane (for face-on views where rays don't cross)
    
    let crossed = (prev_theta - equator) * (theta - equator) < 0.0;
    
    // For face-on views, rays may not cross but can be close to the disc
    // Use a generous tolerance that works for all viewing angles
    let tolerance = 0.5;  // ~28.6° tolerance
    let theta_dist = (theta - equator).abs();
    let near_equator = theta_dist < tolerance;
    
    // Check if we're within the disc's radial extent
    let in_disc_radius = r >= r_inner && r <= r_outer;
    
    (crossed || near_equator) && in_disc_radius
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
                results[disc_crossings as usize] = GeodesicResult::DiscHit {
                    r: state.r,
                    phi: state.phi,
                    energy: photon.energy,
                    angular_momentum: photon.angular_momentum,
                    order: disc_crossings,
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

