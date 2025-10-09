// kernels/kerr_black_hole/src/lib.rs

// Kerr Black Hole Ray Tracing Physics Core
// 
// This library implements null geodesic integration in Kerr spacetime.
// All computations use f64 for maximum precision near critical regions.

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
            Self::Kerr { spin, direction } => {
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