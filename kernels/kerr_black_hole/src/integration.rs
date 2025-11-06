// Geodesic integration using RK8(7) adaptive method (DOPRI8) with bisection at disc crossings

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
    fn new(photon: &PhotonState, initial_sign_theta: f64) -> Self {
        Self {
            r: photon.r,
            theta: photon.theta,
            phi: photon.phi,
            lambda: 0.0,
            initial_phi: photon.phi,
            sign_r: -1.0,                     // Start moving inward (toward BH)
            sign_theta: initial_sign_theta,   // Use actual ray direction
        }
    }
}

// ---------------------------------------------------------------------------
// DOPRI8(7) helpers and stepper
// ---------------------------------------------------------------------------

#[inline]
fn kerr_helpers(r: f64, th: f64, a: f64) -> (f64, f64, f64, f64, f64, f64) {
    let ct = th.cos();
    let st = th.sin().abs().max(1e-300);
    let r2 = r * r;
    let a2 = a * a;
    let rho2 = r2 + a2 * ct * ct;
    let delta = r2 - 2.0 * r + a2;
    (ct, st, r2, a2, rho2, delta)
}

#[inline]
fn carter_potentials(
    r: f64,
    th: f64,
    a: f64,
    e: f64,
    lz: f64,
    q: f64,
) -> (f64, f64) {
    let (ct, st, r2, a2, _rho2, delta) = kerr_helpers(r, th, a);
    let p = e * (r2 + a2) - a * lz;
    let k = q + (lz - a * e) * (lz - a * e);
    let mut big_r = p * p - delta * k;
    let cot = ct / st;
    let mut theta_pot = q + a2 * e * e * ct * ct - (lz * lz) * cot * cot;

    if big_r < 0.0 && big_r > -1e-24 { big_r = 0.0; }
    if theta_pot < 0.0 && theta_pot > -1e-24 { theta_pot = 0.0; }
    (big_r.max(0.0), theta_pot.max(0.0))
}

struct DormandPrince8Coefficients {
    a: [[f64; 12]; 12],
    b8: [f64; 13],
    b7: [f64; 13],
}

impl DormandPrince8Coefficients {
    fn new() -> Self {
        Self {
            a: [
                [1.0/18.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
                [1.0/48.0, 1.0/16.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
                [1.0/32.0, 0.0, 3.0/32.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
                [5.0/16.0, 0.0, -75.0/64.0, 75.0/64.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
                [3.0/80.0, 0.0, 0.0, 3.0/16.0, 3.0/20.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
                [29443841.0/614563906.0, 0.0, 0.0, 77736538.0/692538347.0, -28693883.0/1125000000.0, 23124283.0/1800000000.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
                [16016141.0/946692911.0, 0.0, 0.0, 61564180.0/158732637.0, 22789713.0/633445777.0, 545815736.0/2771057229.0, -180193667.0/1043307555.0, 0.0, 0.0, 0.0, 0.0, 0.0],
                [39632708.0/573591083.0, 0.0, 0.0, -433636366.0/683701615.0, -421739975.0/2616292301.0, 100302831.0/723423059.0, 790204164.0/839813087.0, 800635310.0/3783071287.0, 0.0, 0.0, 0.0, 0.0],
                [246121993.0/1340847787.0, 0.0, 0.0, -37695042795.0/15268766246.0, -309121744.0/1061227803.0, -12992083.0/490766935.0, 6005943493.0/2108947869.0, 393006217.0/1396673457.0, 123872331.0/1001029789.0, 0.0, 0.0, 0.0],
                [-1028468189.0/846180014.0, 0.0, 0.0, 8478235783.0/508512852.0, 1311729495.0/1432422823.0, -10304129995.0/1701304382.0, -48777925059.0/3047939560.0, 15336726248.0/1032824649.0, -45442868181.0/3398467696.0, 3065993473.0/597172653.0, 0.0, 0.0],
                [185892177.0/718116043.0, 0.0, 0.0, -3185094517.0/667107341.0, -477755414.0/1098053517.0, -703635378.0/230739211.0, 5731566787.0/1027545527.0, 5232866602.0/850066563.0, -4093664535.0/808688257.0, 3962137247.0/1805957418.0, 65686358.0/487910083.0, 0.0],
                [403863854.0/491063109.0, 0.0, 0.0, -5068492393.0/434740067.0, -411421997.0/543043805.0, 652783627.0/914296604.0, 11173962825.0/925320556.0, -13158990841.0/6184727034.0, 3936647629.0/1978049680.0, -160528059.0/685178525.0, 248638103.0/1413531060.0, 0.0],
            ],
            b8: [
                14005451.0/335480064.0, 0.0, 0.0, 0.0, 0.0, -59238493.0/1068277825.0,
                181606767.0/758867731.0, 561292985.0/797845732.0, -1041891430.0/1371343529.0,
                760417239.0/1151165299.0, 118820643.0/751138087.0, -528747749.0/2220607170.0, 1.0/4.0,
            ],
            b7: [
                13451932.0/455176623.0, 0.0, 0.0, 0.0, 0.0, -808719846.0/976000145.0,
                1757004468.0/5645159321.0, 656045339.0/265891186.0, -3867574721.0/1518517206.0,
                465885868.0/322736535.0, 53011238.0/667516719.0, 2.0/45.0, 0.0,
            ],
        }
    }
}

fn dopri8_step(
    state: &IntegrationState,
    energy: f64,
    angular_momentum: f64,
    carter_q: f64,
    m: f64,
    a: f64,
    step_size: f64,
    coeff: &DormandPrince8Coefficients,
) -> (IntegrationState, f64) {
    let h = step_size;

    let mut k_r = [0.0f64; 13];
    let mut k_theta = [0.0f64; 13];
    let mut k_phi = [0.0f64; 13];

    k_r[0] = geodesic_dr_dlambda(state.r, state.theta, energy, angular_momentum, carter_q, m, a, state.sign_r);
    k_theta[0] = geodesic_dtheta_dlambda(state.r, state.theta, energy, angular_momentum, carter_q, a, state.sign_theta);
    k_phi[0] = geodesic_dphi_dlambda(state.r, state.theta, energy, angular_momentum, m, a);

    for i in 1..13 {
        let mut r_temp = state.r;
        let mut theta_temp = state.theta;
        for j in 0..i { r_temp += h * coeff.a[i-1][j] * k_r[j]; theta_temp += h * coeff.a[i-1][j] * k_theta[j]; }

        let (r_pot, theta_pot) = carter_potentials(r_temp, theta_temp, a, energy, angular_momentum, carter_q);
        let local_sign_r = if (r_temp - state.r) >= 0.0 { 1.0 } else { -1.0 };
        let local_sign_theta = if (theta_temp - state.theta) >= 0.0 { 1.0 } else { -1.0 };
        let adj_sign_r = if r_pot <= 0.0 { -local_sign_r } else { local_sign_r };
        let adj_sign_theta = if theta_pot <= 0.0 { -local_sign_theta } else { local_sign_theta };

        k_r[i] = geodesic_dr_dlambda(r_temp, theta_temp, energy, angular_momentum, carter_q, m, a, adj_sign_r);
        k_theta[i] = geodesic_dtheta_dlambda(r_temp, theta_temp, energy, angular_momentum, carter_q, a, adj_sign_theta);
        k_phi[i] = geodesic_dphi_dlambda(r_temp, theta_temp, energy, angular_momentum, m, a);
    }

    let mut new_r = state.r;
    let mut new_theta = state.theta;
    let mut new_phi = state.phi;
    for i in 0..13 { new_r += h * coeff.b8[i] * k_r[i]; new_theta += h * coeff.b8[i] * k_theta[i]; new_phi += h * coeff.b8[i] * k_phi[i]; }

    let mut r7 = state.r; let mut theta7 = state.theta; let mut phi7 = state.phi;
    for i in 0..13 { r7 += h * coeff.b7[i] * k_r[i]; theta7 += h * coeff.b7[i] * k_theta[i]; phi7 += h * coeff.b7[i] * k_phi[i]; }

    let error_r = (new_r - r7).abs();
    let error_theta = (new_theta - theta7).abs();
    let error_phi = (new_phi - phi7).abs();
    let error_estimate = error_r.max(error_theta).max(error_phi);

    let (r_pot_before, theta_pot_before) = carter_potentials(state.r, state.theta, a, energy, angular_momentum, carter_q);
    let (r_pot_after, theta_pot_after) = carter_potentials(new_r, new_theta, a, energy, angular_momentum, carter_q);

    let mut new_sign_r = state.sign_r;
    let mut new_sign_theta = state.sign_theta;
    if r_pot_before > 0.0 && r_pot_after == 0.0 { new_sign_r = -new_sign_r; }
    if theta_pot_before > 0.0 && theta_pot_after == 0.0 { new_sign_theta = -new_sign_theta; }
    if r_pot_before * r_pot_after < 0.0 { new_sign_r = -new_sign_r; }
    if theta_pot_before * theta_pot_after < 0.0 { new_sign_theta = -new_sign_theta; }

    let new_state = IntegrationState { r: new_r, theta: new_theta, phi: new_phi, lambda: state.lambda + h, initial_phi: state.initial_phi, sign_r: new_sign_r, sign_theta: new_sign_theta };
    (new_state, error_estimate)
}

fn compute_next_step_size(current_h: f64, error: f64, tolerance: f64, safety_factor: f64) -> f64 {
    if error < 1e-14 { return current_h * 5.0; }
    let factor = (tolerance / error).powf(1.0 / 8.0) * safety_factor;
    let factor_clamped = factor.max(0.2).min(5.0);
    current_h * factor_clamped
}

fn bisect_disc_intersection(
    state_before: &IntegrationState,
    state_after: &IntegrationState,
    energy: f64,
    angular_momentum: f64,
    carter_q: f64,
    m: f64,
    a: f64,
    coeff: &DormandPrince8Coefficients,
    max_iterations: usize,
) -> IntegrationState {
    let equator = PI / 2.0;
    let tolerance = 1e-12;

    let mut lambda_left = state_before.lambda;
    let mut lambda_right = state_after.lambda;
    let mut theta_left = state_before.theta;
    let mut theta_right = state_after.theta;

    if (theta_left - equator) * (theta_right - equator) >= 0.0 {
        return if (theta_left - equator).abs() < (theta_right - equator).abs() { *state_before } else { *state_after };
    }

    for _ in 0..max_iterations {
        let frac = (equator - theta_left) / (theta_right - theta_left);
        let lambda_mid = lambda_left + frac * (lambda_right - lambda_left);
        let step_size = lambda_mid - state_before.lambda;
        let (mid_state, _) = dopri8_step(state_before, energy, angular_momentum, carter_q, m, a, step_size, coeff);
        let theta_mid = mid_state.theta;
        if (theta_mid - equator).abs() < tolerance { return mid_state; }
        if (theta_mid - equator) * (theta_right - equator) < 0.0 {
            lambda_left = lambda_mid; theta_left = theta_mid;
        } else {
            lambda_right = lambda_mid; theta_right = theta_mid;
        }
        if (lambda_right - lambda_left).abs() < 1e-15 {
            let final_step = (lambda_left + lambda_right) * 0.5 - state_before.lambda;
            let (final_state, _) = dopri8_step(state_before, energy, angular_momentum, carter_q, m, a, final_step, coeff);
            return final_state;
        }
    }
    let final_step = (lambda_left + lambda_right) * 0.5 - state_before.lambda;
    let (final_state, _) = dopri8_step(state_before, energy, angular_momentum, carter_q, m, a, final_step, coeff);
    final_state
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
// (legacy helper retained if needed)
#[allow(dead_code)]
fn adaptive_step_size(r: f64, theta: f64, base_step: f64) -> f64 {
    let r_factor = (r / 2.0).max(0.1);
    let theta_factor = (theta - PI/2.0).abs().max(0.1);
    base_step * r_factor * theta_factor
}

// ============================================================================
// DISC INTERSECTION DETECTION
// ============================================================================

// Check if ray intersects the accretion disc at current position
// 
// Physics: The disc is a thin structure in the equatorial plane (θ = π/2)
// It extends from r_inner (ISCO) to r_outer (e.g., 20M)
#[allow(dead_code)]
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
    initial_sign_theta: f64,
    black_hole: &BlackHole,
    r_disc_inner: f64,
    r_disc_outer: f64,
    max_orders: u8,
    max_steps: usize,
) -> Vec<GeodesicResult> {
    let m = black_hole.mass;
    let a = black_hole.spin;
    let r_horizon = black_hole.horizon_radius();
    
    let mut state = IntegrationState::new(&photon, initial_sign_theta);
    let mut results = vec![GeodesicResult::Escaped { turns_r: 0, turns_theta: 0 }; max_orders as usize];
    let mut disc_crossings = 0u8;
    
    // Track turning points throughout entire geodesic
    let mut turns_r = 0u8;
    let mut turns_theta = 0u8;
    let mut prev_sign_r = state.sign_r;
    let mut prev_sign_theta = state.sign_theta;

    let tolerance = 1e-12;  // Tightened from 1e-10 for better NI accuracy
    let safety_factor = 0.9;
    let mut h: f64 = 0.05;
    let h_min: f64 = 1e-8;
    let h_max: f64 = 0.5;

    let coeff = DormandPrince8Coefficients::new();
    let mut steps_taken = 0usize;
    let mut _steps_rejected = 0usize;

    while steps_taken < max_steps {
        // Step size will be adapted by the PI controller based on error; no pre-limiter

        let (trial_state, error) = dopri8_step(&state, photon.energy, photon.angular_momentum, photon.carter_q, m, a, h, &coeff);

        if error <= tolerance || h <= h_min {
            // accept step
            let prev = state;
            state = trial_state;
            steps_taken += 1;

            // Update signs from continuity (primary) and turning points (secondary)
            let dr = state.r - prev.r;
            let dth = state.theta - prev.theta;

            // Primary: continuity by finite difference
            if dr.abs() > 1e-12 {
                state.sign_r = if dr >= 0.0 { 1.0 } else { -1.0 };
            }
            if dth.abs() > 1e-12 {
                state.sign_theta = if dth >= 0.0 { 1.0 } else { -1.0 };
            }

            // Secondary: robust flip at turning points using potentials
            let (r_pot_before, th_pot_before) = carter_potentials(prev.r, prev.theta, a, photon.energy, photon.angular_momentum, photon.carter_q);
            let (r_pot_after, th_pot_after) = carter_potentials(state.r, state.theta, a, photon.energy, photon.angular_momentum, photon.carter_q);

            if r_pot_before > 0.0 && r_pot_after == 0.0 {
                state.sign_r = -state.sign_r;
            }
            if th_pot_before > 0.0 && th_pot_after == 0.0 {
                state.sign_theta = -state.sign_theta;
            }
            if r_pot_before * r_pot_after < 0.0 {
                state.sign_r = -state.sign_r;
            }
            if th_pot_before * th_pot_after < 0.0 {
                state.sign_theta = -state.sign_theta;
            }
            
            // Track turning points (sign flips with saturation)
            if state.sign_r != prev_sign_r {
                turns_r = turns_r.saturating_add(1);
                prev_sign_r = state.sign_r;
            }
            if state.sign_theta != prev_sign_theta {
                turns_theta = turns_theta.saturating_add(1);
                prev_sign_theta = state.sign_theta;
            }

            // Stopping conditions with miss classification
            if state.r < r_horizon * 1.01 {
                // Captured by horizon
                for i in disc_crossings..max_orders { 
                    results[i as usize] = GeodesicResult::Captured { turns_r, turns_theta }; 
                }
                return results;
            }
            if state.r > 1000.0 { 
                // Escaped to infinity
                for i in disc_crossings..max_orders {
                    results[i as usize] = GeodesicResult::Escaped { turns_r, turns_theta };
                }
                return results; 
            }

            // Disc crossing detection with bisection refinement
            let equator = PI / 2.0;
            let crossed = (prev.theta - equator) * (state.theta - equator) < 0.0;
            if crossed {
                let exact_state = bisect_disc_intersection(&prev, &state, photon.energy, photon.angular_momentum, photon.carter_q, m, a, &coeff, 20);
                let within_radius = exact_state.r >= r_disc_inner && exact_state.r <= r_disc_outer;
                if within_radius {
                    if disc_crossings < max_orders {
                        let impact_param = photon.angular_momentum / photon.energy;
                        let omega = keplerian_omega(exact_state.r, m, a);
                        let redshift = compute_redshift_factor(exact_state.r, exact_state.theta, photon.energy, photon.angular_momentum, m, a, omega);
                        let null_error = compute_null_invariant(exact_state.r, exact_state.theta, photon.energy, photon.angular_momentum, photon.carter_q, m, a, exact_state.sign_r, exact_state.sign_theta);
                        let phi_total = (exact_state.phi - exact_state.initial_phi).abs();
                        let phi_wraps = phi_total / (2.0 * PI);
                        results[disc_crossings as usize] = GeodesicResult::DiscHit {
                            r: exact_state.r,
                            theta: exact_state.theta,
                            phi: exact_state.phi,
                            energy: photon.energy,
                            angular_momentum: photon.angular_momentum,
                            carter_q: photon.carter_q,
                            impact_parameter: impact_param,
                            redshift_factor: redshift,
                            affine_parameter: exact_state.lambda,
                            phi_wraps,
                            order: disc_crossings,
                            null_invariant_error: null_error,
                            turns_r,
                            turns_theta,
                        };
                    }
                    disc_crossings += 1;
                    if disc_crossings >= max_orders { return results; }
                }
            }

            if state.theta < 0.0 || state.theta > PI { state.theta = state.theta.rem_euclid(PI); }
            state.phi = state.phi.rem_euclid(2.0 * PI);

            h = compute_next_step_size(h, error.max(1e-16), tolerance, safety_factor);
            h = h.max(h_min).min(h_max);
        } else {
            _steps_rejected += 1;
            h = compute_next_step_size(h, error, tolerance, safety_factor);
            h = h.max(h_min).min(h_max);
        }
    }

    // If we reached max steps without hitting all orders or stopping conditions
    // Classify based on final state (position + velocity) rather than blindly aborting
    if disc_crossings < max_orders {
        // Determine likely fate from final position and radial velocity
        let likely_fate = if state.r < 10.0 {
            // Close to horizon (r < 10M), likely being captured
            GeodesicResult::Captured { turns_r, turns_theta }
        } else if state.r > 100.0 {
            // Far from BH (r > 100M), likely escaping (just needs more time)
            GeodesicResult::Escaped { turns_r, turns_theta }
        } else {
            // Intermediate region (10M < r < 100M): check radial velocity
            if state.sign_r < 0.0 {
                // Moving inward → will eventually be captured
                GeodesicResult::Captured { turns_r, turns_theta }
            } else {
                // Moving outward → will eventually escape
                GeodesicResult::Escaped { turns_r, turns_theta }
            }
        };
        
        // Apply classification to all remaining orders
        for i in disc_crossings..max_orders {
            results[i as usize] = likely_fate.clone();
        }
    }

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
        1.0,  // Default: moving toward equator (legacy behavior)
        black_hole,
        r_disc_inner,
        r_disc_outer,
        1,  // Only get order 0
        max_steps,
    );
    results[0]
}

