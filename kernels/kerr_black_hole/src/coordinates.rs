// Coordinate system utilities for Kerr spacetime

// ============================================================================
// COORDINATE UTILITIES
// ============================================================================

// Convert Cartesian coordinates (x, y, z) to Boyer-Lindquist (r, θ, φ)
// 
// Physics: Boyer-Lindquist coordinates are the natural spherical-like
// coordinates for Kerr spacetime. They generalize standard spherical coords.
// 
// Relations:
// - r²: From x² + y² + (z² - a²) = r² - a² (modified spherical radius)
// - θ: From z = r cos(θ)
// - φ: From tan(φ) = y/x
pub fn cartesian_to_bl(x: f64, y: f64, z: f64, a: f64) -> (f64, f64, f64) {
    // Calculate r from the modified Kerr relation
    // In Kerr: x² + y² + z² ≠ r² (unlike normal spherical coords!)
    // Instead: ρ² = r² - a²cos²θ and x² + y² = ρ²sin²θ
    // For Kerr-Schild: r² = ½[(x²+y²+z²-a²) + √((x²+y²+z²-a²)² + 4a²z²)]
    
    let rho2 = x * x + y * y;  // Distance from rotation axis
    let z2 = z * z;
    let a2 = a * a;
    
    // Quadratic formula solution for r
    let sum = rho2 + z2 - a2;
    let r2 = 0.5 * (sum + (sum * sum + 4.0 * a2 * z2).sqrt());
    let r = r2.sqrt();
    
    // Polar angle θ from z = r cos(θ)
    // Avoid division by zero
    let cos_theta = if r > 1e-10 { z / r } else { 0.0 };
    // Clamp theta to avoid numerical errors
    let theta = cos_theta.clamp(-1.0, 1.0).acos();  // acos = inverse cosine
    
    // Azimuthal angle φ from x, y
    let phi = y.atan2(x);  // atan2 handles quadrants correctly
    
    (r, theta, phi)
}

// Convert Boyer-Lindquist (r, θ, φ) to Cartesian (x, y, z)
// 
// Relations (simpler in this direction):
// - x = √(r² + a²) sin(θ) cos(φ)
// - y = √(r² + a²) sin(θ) sin(φ)  
// - z = r cos(θ)
pub fn bl_to_cartesian(r: f64, theta: f64, phi: f64, a: f64) -> (f64, f64, f64) {
    let a2 = a * a;
    let r2 = r * r;
    
    // In Kerr-Schild, the "cylindrical radius" includes frame-dragging
    let rho = (r2 + a2).sqrt();  // Modified radius in x-y plane
    
    let sin_theta = theta.sin();
    let cos_theta = theta.cos();
    let sin_phi = phi.sin();
    let cos_phi = phi.cos();
    
    let x = rho * sin_theta * cos_phi;
    let y = rho * sin_theta * sin_phi;
    let z = r * cos_theta;
    
    (x, y, z)
}

// ============================================================================
// KERR METRIC COMPONENTS
// ============================================================================

// Helper functions for Kerr metric in Boyer-Lindquist coordinates
// 
// The Kerr metric has form: ds² = g_μν dx^μ dx^ν
// These functions compute key metric components and derived quantities

// Σ (Sigma) = r² + a²cos²θ
// This appears everywhere in Kerr metric
#[inline]
pub fn sigma(r: f64, theta: f64, a: f64) -> f64 {
    let cos_theta = theta.cos();
    r * r + a * a * cos_theta * cos_theta
}

// Δ (Delta) = r² - 2Mr + a²
// Related to the horizon location (Δ=0 at horizon)
#[inline]
pub fn delta(r: f64, m: f64, a: f64) -> f64 {
    r * r - 2.0 * m * r + a * a
}

// A² = (r² + a²)² - a²Δsin²θ
// Appears in the metric's φ terms
#[inline]
pub fn a_squared(r: f64, theta: f64, m: f64, a: f64) -> f64 {
    let r2 = r * r;
    let a2 = a * a;
    let sin_theta = theta.sin();
    let sin2_theta = sin_theta * sin_theta;
    
    let term1 = (r2 + a2) * (r2 + a2);
    let term2 = a2 * delta(r, m, a) * sin2_theta;
    
    term1 - term2
}

