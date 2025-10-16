// Type definitions for Kerr black hole simulation

use std::f64::consts::PI;

// ============================================================================
// ORBITAL CONFIGURATION TYPES
// ============================================================================

// Direction of orbital motion relative to black hole spin
// 
// Physics: Material can orbit in the same direction as the black hole's spin
// (prograde) or opposite to it (retrograde). This dramatically affects the ISCO
// radius and the visual appearance of the accretion disc.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrbitDirection {
    // Prograde: orbiting in same direction as black hole spin
    // - Closer ISCO (e.g., ~2.3M for a=0.9)
    // - Most common in nature (~99% of systems)
    // - Hotter, brighter disc due to more energy extraction
    Prograde,
    
    // Retrograde: orbiting opposite to black hole spin
    // - Farther ISCO (e.g., ~9M for a=0.9)
    // - Rare in nature but interesting for comparison
    // - Cooler, dimmer disc
    Retrograde,
}

impl Default for OrbitDirection {
    fn default() -> Self {
        Self::Prograde
    }
}

// Black hole configuration for rendering comparisons
// 
// Purpose: Allows easy switching between different black hole types
// for educational comparisons in the blog post.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BlackHoleType {
    // Kerr black hole with specified spin and orbit direction
    // This is the main simulation focus
    Kerr { 
        // Spin parameter a/M ∈ (0, 1)
        spin: f64, 
        // Orbital direction of the accretion disc
        direction: OrbitDirection 
    },
    
    // Non-rotating Schwarzschild black hole (a=0)
    // Orbital direction doesn't matter since there's no frame-dragging
    // Useful baseline for comparison
    Schwarzschild,
}

impl BlackHoleType {
    // Create a Kerr black hole with prograde orbit (most common case)
    pub fn kerr_prograde(spin: f64) -> Self {
        Self::Kerr { 
            spin, 
            direction: OrbitDirection::Prograde 
        }
    }
    
    // Create a Kerr black hole with retrograde orbit (for comparison)
    pub fn kerr_retrograde(spin: f64) -> Self {
        Self::Kerr { 
            spin, 
            direction: OrbitDirection::Retrograde 
        }
    }
    
    // Create a Schwarzschild black hole (no spin)
    pub fn schwarzschild() -> Self {
        Self::Schwarzschild
    }
    
    // Get the spin value
    pub fn spin(&self) -> f64 {
        match self {
            Self::Kerr { spin, .. } => *spin,
            Self::Schwarzschild => 0.0,
        }
    }
    
    // Get the orbit direction (Schwarzschild defaults to Prograde for calculations)
    pub fn direction(&self) -> OrbitDirection {
        match self {
            Self::Kerr { direction, .. } => *direction,
            Self::Schwarzschild => OrbitDirection::Prograde, // Arbitrary, doesn't affect result
        }
    }
    
    // Get a human-readable name for this configuration
    pub fn name(&self) -> &'static str {
        match self {
            Self::Kerr { direction, .. } => {
                match direction {
                    OrbitDirection::Prograde => "Kerr (Prograde)",
                    OrbitDirection::Retrograde => "Kerr (Retrograde)",
                }
            }
            Self::Schwarzschild => "Schwarzschild",
        }
    }
}

// ============================================================================
// BLACK HOLE DEFINITION
// ============================================================================

// A Kerr black hole with mass M and spin parameter a
// 
// Physics concepts:
// - Mass (M): Sets the size scale. We use M=1 in "geometric units" where G=c=1
// - Spin (a): How fast the black hole rotates. Measured as a = J/M where J is 
//   angular momentum. Valid range is 0 ≤ a < M.
//   - a=0: Not rotating at all (called "Schwarzschild")
//   - a=0.9M: Rapidly rotating (what we'll use)
//   - a→M: Maximum possible spin (called "extremal")
#[derive(Debug, Clone, Copy)]
pub struct BlackHole {
    // Mass in geometric units (we typically use M=1 to make math simpler)
    pub mass: f64,
    
    // Spin parameter a ∈ [0, M)
    // This represents the angular momentum per unit mass
    pub spin: f64,
}

impl BlackHole {
    // Create a new black hole with given mass and type configuration
    pub fn new(mass: f64, bh_type: BlackHoleType) -> Self {
        assert!(mass > 0.0, "Mass must be positive");
        let spin = bh_type.spin();
        assert!(spin >= 0.0 && spin < mass, "Spin must be in [0, M)");
        Self { mass, spin }
    }
    
    // Create a Kerr black hole with prograde orbit (main use case)
    pub fn kerr_prograde(mass: f64, spin: f64) -> Self {
        Self::new(mass, BlackHoleType::kerr_prograde(spin))
    }
    
    // Create a Kerr black hole with retrograde orbit (for comparison)
    pub fn kerr_retrograde(mass: f64, spin: f64) -> Self {
        Self::new(mass, BlackHoleType::kerr_retrograde(spin))
    }
    
    // Create a Schwarzschild black hole (no rotation)
    pub fn schwarzschild(mass: f64) -> Self {
        Self::new(mass, BlackHoleType::schwarzschild())
    }
    
    // Get the spin parameter a
    #[inline]
    pub fn a(&self) -> f64 {
        self.spin
    }
    
    // Check if this is a Schwarzschild black hole (no spin)
    #[inline]
    pub fn is_schwarzschild(&self) -> bool {
        self.spin.abs() < 1e-10
    }
    
    // Calculate the event horizon radius r₊
    // 
    // Math: r₊ = M + √(M² - a²)
    // 
    // Physics: The event horizon is the "point of no return" - once you 
    // cross it, you can never escape, even traveling at light speed.
    // 
    // Spin dependence: 
    // - a=0 (Schwarzschild): r₊ = 2M
    // - a=0.9M (Kerr): r₊ ≈ 1.44M
    // - a→M (extremal): r₊ → M
    #[inline]
    pub fn horizon_radius(&self) -> f64 {
        let m = self.mass;
        let a = self.spin;
        m + (m * m - a * a).sqrt()
    }

    // Calculate the ISCO radius for a given orbital direction
    // 
    // Physics: The innermost stable circular orbit depends on:
    // 1. Black hole spin (a)
    // 2. Orbital direction (prograde vs retrograde)
    // 
    // Examples (all with M=1):
    // - Schwarzschild (a=0): r_ISCO = 6M (direction independent)
    // - Kerr prograde (a=0.9): r_ISCO ≈ 2.32M (very close!)
    // - Kerr retrograde (a=0.9): r_ISCO ≈ 8.95M (much farther)
    // 
    // This is where the accretion disc begins.
    // 
    // Math: Formula from Bardeen, Press & Teukolsky (1972):
    // - Z₁ = 1 + (1-a²/M²)^(1/3) × [(1+a/M)^(1/3) + (1-a/M)^(1/3)]
    // - Z₂ = √(3a²/M² + Z₁²)
    // - r_ISCO = M × (3 + Z₂ ± √[(3-Z₁)(3+Z₁+2Z₂)])
    //   where minus sign = prograde, plus sign = retrograde
    pub fn isco_radius(&self, direction: OrbitDirection) -> f64 {
        let m = self.mass;
        let a = self.spin;
        
        // Special case: Schwarzschild (no spin = direction independent)
        if self.is_schwarzschild() {
            return 6.0 * m;
        }
        
        // Normalize spin to a/M (dimensionless ratio)
        let a_norm = a / m;
        let a2 = a_norm * a_norm;
        
        // Calculate Z₁ term (combines spin effects)
        // Same for both prograde and retrograde
        let z1 = 1.0 + (1.0 - a2).powf(1.0 / 3.0)
            * ((1.0 + a_norm).powf(1.0 / 3.0)
            + (1.0 - a_norm).powf(1.0 / 3.0));
        
        // Calculate Z₂ term (orbital plane geometry)
        // Same for both prograde and retrograde
        let z2 = (3.0 * a2 + z1 * z1).sqrt();
        
        // The sign determines prograde (minus) vs retrograde (plus)
        // This is the ONLY difference between the two cases
        let sign = match direction {
            OrbitDirection::Prograde => -1.0,   // Minus: allows closer orbits
            OrbitDirection::Retrograde => 1.0,  // Plus: pushes orbits farther out
        };
        
        // Final ISCO calculation with direction-dependent sign
        let r_isco = m * (3.0 + z2 + sign * ((3.0 - z1) * (3.0 + z1 + 2.0 * z2)).sqrt());
        
        r_isco
    }
}

// ============================================================================
// CAMERA / OBSERVER CONFIGURATION
// ============================================================================

// Camera/Observer configuration
// 
// Physics: The observer is at a fixed position far from the black hole,
// looking toward it with a specific inclination angle. We assume the observer
// is stationary (not orbiting) and far enough away that spacetime is nearly flat.
#[derive(Debug, Clone, Copy)]
pub struct Camera {
    // Distance from black hole center (in units of M)
    // Typical value: 20-50M for good viewing distance
    // Too close: distorted FOV and risk of horizon
    // Too far: black hole appears tiny
    pub distance: f64,
    
    // Inclination angle (viewing angle from equatorial plane) in DEGREES
    // 0° = face-on (looking down at disc from north pole)
    // 90° = edge-on (looking at disc from within its plane)
    // This dramatically affects what we see due to gravitational lensing
    pub inclination: f64,
    
    // Field of view in DEGREES
    // Typical value: 30-40° for a good view of the black hole
    // Smaller FOV = zoomed in, larger FOV = wide angle
    pub fov: f64,
}

impl Camera {
    // Create a new camera configuration
    pub fn new(distance: f64, inclination: f64, fov: f64) -> Self {
        assert!(distance > 0.0, "Distance must be positive");
        assert!(inclination >= 0.0 && inclination <= 90.0, "Inclination must be in [0°, 90°]");
        assert!(fov > 0.0 && fov < 180.0, "FOV must be in (0°, 180°)");
        Self { distance, inclination, fov }
    }
    
    // Get inclination in radians (needed for math)
    // Most trig functions in Rust use radians, not degrees
    #[inline]
    pub fn inclination_rad(&self) -> f64 {
        self.inclination * PI / 180.0
    }
    
    // Get FOV in radians (needed for ray generation)
    #[inline]
    pub fn fov_rad(&self) -> f64 {
        self.fov * PI / 180.0
    }

    // Generate a ray for a specific pixel
    // 
    // Physics: Convert pixel coordinates to a direction in 3D space
    // The ray starts at the camera position and points toward the black hole
    // with a slight offset determined by the pixel position and FOV
    pub fn generate_ray(&self, pixel_x: u32, pixel_y: u32, config: &RenderConfig) -> Ray {
        // Convert pixel to normalized device coordinates [-1, 1]
        let ndc_x = (pixel_x as f64 / (config.width - 1) as f64) * 2.0 - 1.0;
        let ndc_y = (pixel_y as f64 / (config.height - 1) as f64) * 2.0 - 1.0;
        
        // Pinhole camera: apply tan to the angle (not linear!)
        let aspect = config.aspect_ratio();
        let half_fov = self.fov_rad() * 0.5;
        
        let ax = (ndc_x * half_fov).tan();  // Proper pinhole projection
        let ay = (ndc_y * half_fov).tan();
        
        // Screen coordinates with aspect ratio applied
        let screen_x = ax * aspect;
        let screen_y = ay;
        let screen_z = -1.0;  // Pinhole at z = -1
        
        // Camera is at distance D along +x axis, looking toward origin
        // Rotated by inclination angle around y-axis
        let inc = self.inclination_rad();
        
        // Camera position in Cartesian coords
        // Inclination rotates camera around the y-axis in the x-z plane
        // 0° = face-on (along +z axis, looking down at disc)
        // 90° = edge-on (along +x axis, looking at disc edge)
        let cam_x = self.distance * inc.sin();
        let cam_y = 0.0;
        let cam_z = self.distance * inc.cos();
        
        // Transform screen point to world coordinates
        let world_screen_x = screen_x * inc.cos() + screen_z * inc.sin() + cam_x;
        let world_screen_y = screen_y + cam_y;
        let world_screen_z = -screen_x * inc.sin() + screen_z * inc.cos() + cam_z;
        
        // Ray direction: from camera to screen point
        let (dir_x, dir_y, dir_z) = (
            world_screen_x - cam_x,
            world_screen_y - cam_y, 
            world_screen_z - cam_z
        );
        
        Ray::new(
            [cam_x, cam_y, cam_z],
            [dir_x, dir_y, dir_z],
        )
    }
}

// ============================================================================
// RENDER CONFIGURATION
// ============================================================================

// Rendering configuration
// 
// Defines the output image properties and quality settings
#[derive(Debug, Clone, Copy)]
pub struct RenderConfig {
    // Image width in pixels
    pub width: u32,
    
    // Image height in pixels
    pub height: u32,
    
    // Maximum number of disc crossings to track per ray
    // 1 = primary image only (direct view of disc)
    // 2 = primary + secondary (light that wraps around once)
    // 3+ = higher order images (usually very faint)
    // 
    // Higher orders = more realistic but slower to compute
    pub max_orders: u8,
}

impl RenderConfig {
    // Create a new render configuration
    pub fn new(width: u32, height: u32, max_orders: u8) -> Self {
        assert!(width > 0 && height > 0, "Dimensions must be positive");
        assert!(max_orders > 0 && max_orders <= 3, "Orders must be 1-3");
        Self { width, height, max_orders }
    }
    
    // Get total number of pixels to render
    #[inline]
    pub fn pixel_count(&self) -> usize {
        (self.width as usize) * (self.height as usize)
    }
    
    // Get aspect ratio (width / height)
    #[inline]
    pub fn aspect_ratio(&self) -> f64 {
        self.width as f64 / self.height as f64
    }
}

// ============================================================================
// RAY INITIAL CONDITIONS
// ============================================================================

// Initial ray from the camera (observer)
// 
// We trace rays BACKWARD from camera to black hole.
// This struct stores the starting conditions for integration.
#[derive(Debug, Clone, Copy)]
pub struct Ray {
    // Starting position in Cartesian-like coordinates
    // For a distant observer, this is approximately (distance, 0, 0)
    pub origin: [f64; 3],  // [x, y, z]
    
    // Initial direction (unit vector)
    // Determined by pixel position and camera FOV
    pub direction: [f64; 3],  // [dx, dy, dz]
}

impl Ray {
    // Create a new ray from origin pointing in direction
    pub fn new(origin: [f64; 3], direction: [f64; 3]) -> Self {
        // Normalize direction to unit vector
        let len = (direction[0] * direction[0] 
                 + direction[1] * direction[1] 
                 + direction[2] * direction[2]).sqrt();
        
        let normalized = [
            direction[0] / len,
            direction[1] / len,
            direction[2] / len,
        ];
        
        Self {
            origin,
            direction: normalized,
        }
    }
    
    // Convert to initial photon state with conserved quantities
    // This is where we calculate E, L_z, Q from the ray direction
    pub fn to_photon_state(&self, black_hole: &BlackHole) -> (PhotonState, f64) {
        let m = black_hole.mass;
        let a = black_hole.spin;
        
        // Calculate conserved quantities using LNRF/ZAMO tetrad at finite distance
        let (energy, angular_momentum, carter_q, initial_sign_theta) = calculate_conserved_quantities_lnrf(self, m, a);
        
        // Convert origin to Boyer-Lindquist coordinates
        let [x, y, z] = self.origin;
        let (r, theta, phi) = crate::coordinates::cartesian_to_bl(x, y, z, a);
        
        let photon = PhotonState::new(
            r,
            theta,
            phi,
            energy,
            angular_momentum,
            carter_q,
        );
        
        (photon, initial_sign_theta)
    }
}

// (legacy infinity-mapping initializer removed)

/// Build BL-aligned orthonormal triad in world/Cartesian space
/// Returns (ê_r, ê_θ, ê_φ) as world-space unit vectors at BL position (r, θ, φ)
/// These are the standard spherical-like coordinate directions evaluated at the camera point
#[inline]
fn bl_aligned_triad_world(th: f64, ph: f64) -> ([f64; 3], [f64; 3], [f64; 3]) {
    let (st, ct) = (th.sin(), th.cos());
    let (sph, cph) = (ph.sin(), ph.cos());

    // Unit vectors in Cartesian corresponding to BL coordinate lines
    // (standard spherical directions at this point)
    let e_r_cart = [st * cph, st * sph, ct];           // ∂x/∂r normalized (radial)
    let e_th_cart = [ct * cph, ct * sph, -st];         // (1/r) ∂x/∂θ direction (polar)
    let e_ph_cart = [-sph, cph, 0.0];                  // (1/(ρ sinθ)) ∂x/∂φ direction (azimuthal)

    // Orthonormalize defensively (they should already be orthonormal in flat limit)
    let norm = |v: [f64; 3]| {
        let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        [v[0] / l, v[1] / l, v[2] / l]
    };
    let e_r = norm(e_r_cart);
    let e_th = norm(e_th_cart);
    let e_ph = norm(e_ph_cart);

    (e_r, e_th, e_ph)
}

// LNRF/ZAMO tetrad-based initialization for finite-distance observer
// Returns conserved quantities (E, L_z, Q) and initial sign_theta from local orthonormal frame
fn calculate_conserved_quantities_lnrf(
    ray: &Ray,
    m: f64,
    a: f64,
) -> (f64, f64, f64, f64) {
    let [x, y, z] = ray.origin;
    let (r0, th0, ph0) = crate::coordinates::cartesian_to_bl(x, y, z, a);

    // BL-aligned orthonormal triad in world coords (standard spherical basis)
    let (e_r, e_th, e_ph) = bl_aligned_triad_world(th0, ph0);

    // Project the already-correct world ray direction into BL basis
    let n = ray.direction; // unit vector from generate_ray()
    let mut nr = n[0] * e_r[0] + n[1] * e_r[1] + n[2] * e_r[2];
    let mut nth = n[0] * e_th[0] + n[1] * e_th[1] + n[2] * e_th[2];
    let mut nph = n[0] * e_ph[0] + n[1] * e_ph[1] + n[2] * e_ph[2];

    // Renormalize to unit length (kill numerical drift)
    let nlen = (nr * nr + nth * nth + nph * nph).sqrt();
    nr = nr / nlen;
    nth = nth / nlen;
    nph = nph / nlen;

    // Tetrad mapping
    let (_e_t_t, e_t_phi, e_r_r, e_th_th, e_ph_phi) = lnrf_vector_tetrad_coeffs(r0, th0, m, a);
    let pr = e_r_r * nr;
    let pth = e_th_th * nth;
    let pphi = e_t_phi + e_ph_phi * nph;

    // Metric components and future-directed p^t root of null condition
    let (g_tt, g_tphi, g_rr, g_thth, g_phph) = kerr_metric_cov(r0, th0, m, a);
    let a_q = g_tt;
    let b_q = 2.0 * g_tphi * pphi;
    let c_q = g_rr*pr*pr + g_thth*pth*pth + g_phph*pphi*pphi;
    let disc = (b_q*b_q - 4.0*a_q*c_q).max(0.0);
    let sqrt_disc = disc.sqrt();
    let root1 = (-b_q + sqrt_disc) / (2.0 * a_q);
    let root2 = (-b_q - sqrt_disc) / (2.0 * a_q);
    let mut pt = if root1 > 0.0 && root2 > 0.0 { root1.max(root2) } else if root1 > 0.0 { root1 } else { root2 };
    if !(pt.is_finite() && pt > 0.0) { pt = root1.max(root2); }

    // Covariant components
    let p_t   = g_tt*pt + g_tphi*pphi;
    let p_phi = g_tphi*pt + g_phph*pphi;
    let p_th  = g_thth*pth;

    // Conserved quantities
    let energy = -p_t;
    let angular_momentum = p_phi;
    let ct = th0.cos();
    let st2 = th0.sin().powi(2).max(1e-300);
    let carter_q = p_th * p_th + ct * ct * (angular_momentum * angular_momentum / st2 - a * a * energy * energy);
    
    // Initial sign of theta motion from the actual momentum
    let initial_sign_theta = if p_th >= 0.0 { 1.0 } else { -1.0 };

    // Sanity checks (debug builds only)
    #[cfg(debug_assertions)]
    {
        // Θ₀ should equal p_θ² with our Q definition
        let a2 = a * a;
        let cos2 = th0.cos().powi(2);
        let sin2 = th0.sin().powi(2).max(1e-300);
        let theta_pot = carter_q + a2 * energy * energy * cos2 - (angular_momentum * angular_momentum / sin2) * cos2;
        debug_assert!(
            (theta_pot - (p_th * p_th)).abs() < 1e-10,
            "Theta potential mismatch at init: Θ={:.3e}, p_θ²={:.3e}",
            theta_pot,
            p_th * p_th
        );

        // K (non-negative Carter quantity) should be >= 0
        let k_nonneg = carter_q + (angular_momentum - a * energy).powi(2);
        debug_assert!(
            k_nonneg >= -1e-10,
            "K = Q + (Lz - aE)² should be non-negative: K={:.3e}",
            k_nonneg
        );

        // Null invariant should be tiny at init
        let inv0 = crate::geodesic::compute_null_invariant(r0, th0, energy, angular_momentum, carter_q, m, a, -1.0, initial_sign_theta);
        debug_assert!(
            inv0 < 1e-10,
            "Init null invariant too large: {:.3e}",
            inv0
        );
    }

    (energy, angular_momentum, carter_q, initial_sign_theta)
}

// LNRF contravariant tetrad coefficients e_(a)^μ at (r, θ)
fn lnrf_vector_tetrad_coeffs(r: f64, th: f64, m: f64, a: f64) -> (f64, f64, f64, f64, f64) {
    let (sigma, delta, big_a, st2, _ct2, omega) = kerr_scalars(r, th, m, a);
    let e_t_t = (big_a / (sigma * delta)).sqrt();
    let e_t_phi = omega * e_t_t;
    let e_r_r = (delta / sigma).sqrt();
    let e_th_th = 1.0 / sigma.sqrt();
    let e_ph_phi = (sigma / big_a).sqrt() / st2.sqrt().max(1e-300);
    (e_t_t, e_t_phi, e_r_r, e_th_th, e_ph_phi)
}

// Kerr metric covariant components at (r, θ)
fn kerr_metric_cov(r: f64, th: f64, m: f64, a: f64) -> (f64, f64, f64, f64, f64) {
    let (sigma, delta, big_a, st2, _ct2, _omega) = kerr_scalars(r, th, m, a);
    let g_tt = -(1.0 - 2.0 * m * r / sigma);
    let g_tphi = -2.0 * a * m * r * st2 / sigma;
    let g_rr = sigma / delta;
    let g_thth = sigma;
    let g_phph = big_a * st2 / sigma;
    (g_tt, g_tphi, g_rr, g_thth, g_phph)
}

// Kerr scalars at (r, θ)
fn kerr_scalars(r: f64, th: f64, m: f64, a: f64) -> (f64, f64, f64, f64, f64, f64) {
    let (st, ct) = (th.sin(), th.cos());
    let st2 = st * st;
    let ct2 = ct * ct;
    let r2 = r * r;
    let a2 = a * a;
    let sigma = r2 + a2 * ct2;
    let delta = r2 - 2.0 * m * r + a2;
    let big_a = (r2 + a2) * (r2 + a2) - a2 * delta * st2;
    let omega = 2.0 * a * m * r / big_a;
    (sigma, delta, big_a, st2, ct2, omega)
}

// Re-export PhotonState from geodesic module
pub use crate::geodesic::PhotonState;

