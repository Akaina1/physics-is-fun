// Accretion disc emissivity model

// ============================================================================
// NOVIKOV-THORNE DISC EMISSIVITY
// ============================================================================

// Calculate emissivity profile for a thin Novikov-Thorne accretion disc
// 
// Physics: The disc emissivity (flux) decreases with radius and goes to zero
// at the ISCO (innermost stable circular orbit). The standard profile is:
// 
// F(r) ∝ r^(-3) * (1 - sqrt(r_ISCO / r))
// 
// This captures:
// - r^(-3) falloff from gravitational binding energy
// - Boundary condition: F(r_ISCO) = 0 (no emission inside ISCO)
// - Normalization to make peak flux ~ 1.0
pub fn novikov_thorne_emissivity(r: f64, r_isco: f64) -> f64 {
    // No emission inside ISCO
    if r < r_isco {
        return 0.0;
    }
    
    // Standard Novikov-Thorne profile
    let r_ratio = r_isco / r;
    let falloff = r.powf(-3.0);
    let isco_term = 1.0 - r_ratio.sqrt();
    
    // Include a normalization factor so peak is around 1.0
    // Peak typically occurs near r ~ 1.5 * r_ISCO
    let normalization = r_isco.powf(3.0) * 2.0;
    
    falloff * isco_term * normalization
}

// Generate a 1D lookup table of emissivity values
// 
// This LUT will be sampled by the GPU shader to get disc brightness
// at any radius r
// 
// Parameters:
// - r_min: Inner edge (typically r_ISCO)
// - r_max: Outer edge (e.g., 20M)
// - samples: Number of samples (typically 256 or 512)
// 
// Returns: Vec<f32> of emissivity values from r_min to r_max
pub fn generate_flux_lut(r_min: f64, r_max: f64, r_isco: f64, samples: usize) -> Vec<f32> {
    let mut lut = Vec::with_capacity(samples);
    
    for i in 0..samples {
        // Linear sampling from r_min to r_max
        let t = i as f64 / (samples - 1) as f64;
        let r = r_min + t * (r_max - r_min);
        
        // Calculate emissivity at this radius
        let flux = novikov_thorne_emissivity(r, r_isco);
        
        lut.push(flux as f32);
    }
    
    lut
}

// Calculate peak emissivity for normalization/reference
// Useful for exposure calibration
pub fn peak_emissivity(r_isco: f64) -> f64 {
    // Peak occurs approximately at r ~ 1.4-1.5 * r_ISCO
    // We'll sample around that region to find the actual peak
    let r_peak_approx = 1.5 * r_isco;
    
    // Sample a small range around the approximate peak
    let mut peak = 0.0;
    for i in 0..100 {
        let r = r_peak_approx * (0.9 + 0.2 * i as f64 / 99.0);
        let flux = novikov_thorne_emissivity(r, r_isco);
        if flux > peak {
            peak = flux;
        }
    }
    
    peak
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_emissivity_at_isco() {
        let r_isco = 6.0;  // Schwarzschild ISCO
        let flux = novikov_thorne_emissivity(r_isco, r_isco);
        assert!(flux.abs() < 1e-10, "Flux should be zero at ISCO");
    }
    
    #[test]
    fn test_emissivity_inside_isco() {
        let r_isco = 6.0;
        let flux = novikov_thorne_emissivity(3.0, r_isco);
        assert_eq!(flux, 0.0, "Flux should be zero inside ISCO");
    }
    
    #[test]
    fn test_flux_lut_size() {
        let lut = generate_flux_lut(6.0, 20.0, 6.0, 256);
        assert_eq!(lut.len(), 256);
    }
    
    #[test]
    fn test_flux_lut_range() {
        let r_isco = 6.0;
        let lut = generate_flux_lut(r_isco, 20.0, r_isco, 256);
        
        // First value (at r_ISCO) should be near zero
        assert!(lut[0].abs() < 0.01, "Flux at ISCO should be near zero");
        
        // Last value (at r_max) should be positive but small
        assert!(lut[255] > 0.0, "Flux at outer edge should be positive");
        assert!(lut[255] < lut[128], "Flux should decrease with radius");
    }
}

