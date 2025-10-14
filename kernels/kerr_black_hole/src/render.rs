// Main rendering function for transfer maps

use crate::types::{BlackHole, Camera, RenderConfig, OrbitDirection};
use crate::transfer_maps::{TransferMaps, Manifest};
use crate::integration::integrate_geodesic;
use crate::disc_model::generate_flux_lut;

// ============================================================================
// MAIN RENDER FUNCTION
// ============================================================================

// Render transfer maps for a black hole configuration
// 
// This is the main entry point that:
// 1. Generates rays for each pixel
// 2. Integrates geodesics
// 3. Packs results into transfer maps
// 4. Generates emissivity LUT
// 5. Optionally exports high-precision f64 data
pub fn render_transfer_maps(
    black_hole: &BlackHole,
    camera: &Camera,
    config: &RenderConfig,
    orbit_direction: OrbitDirection,
    preset_name: String,
    export_high_precision: bool,  // Enable f64 export for documentation
) -> TransferMaps {
    // Calculate disc bounds
    let r_inner = black_hole.isco_radius(orbit_direction);
    let r_outer = 20.0;  // Could be configurable
    
    // Number of samples for flux LUT (standard for texture lookups)
    let flux_samples = 256;
    
    // Temporary manifest for initialization (disc_hits will be updated later)
    let temp_manifest = Manifest::new(
        config.width,
        config.height,
        preset_name.clone(),
        camera.inclination,
        black_hole.spin,
        config.max_orders,
        r_inner,
        r_outer,
        0,  // disc_hits placeholder
    );
    
    // Initialize transfer maps with temp manifest
    let mut maps = TransferMaps::new(config.width, config.height, temp_manifest, flux_samples, export_high_precision);
    
    // Generate emissivity lookup table
    maps.flux_r32f = generate_flux_lut(r_inner, r_outer, r_inner, flux_samples);
    
    let max_steps = 10000;
    
    // Progress tracking (optional, useful for CLI)
    let total_pixels = config.pixel_count();
    let mut completed = 0;
    let mut disc_hits = 0;
    
    // Loop over all pixels
    for y in 0..config.height {
        for x in 0..config.width {
            // Generate ray for this pixel
            let ray = camera.generate_ray(x, y, config);
            
            // Convert to photon state
            let photon = ray.to_photon_state(black_hole);
            
            // Debug: Print first pixel info
            if x == config.width / 2 && y == config.height / 2 {
                eprintln!("\n=== CENTER PIXEL DEBUG ===");
                eprintln!("Pixel: ({}, {})", x, y);
                eprintln!("Camera pos: ({:.2}, {:.2}, {:.2})", ray.origin[0], ray.origin[1], ray.origin[2]);
                eprintln!("Ray dir: ({:.4}, {:.4}, {:.4})", ray.direction[0], ray.direction[1], ray.direction[2]);
                eprintln!("Photon BL coords: r={:.2}, θ={:.4} rad ({:.1}°), φ={:.4} rad", 
                    photon.r, photon.theta, photon.theta.to_degrees(), photon.phi);
                eprintln!("Disc range: {:.2}M to {:.2}M", r_disc_inner, r_disc_outer);
                eprintln!("========================\n");
            }
            
            // Integrate geodesic
            let result = integrate_geodesic(
                photon,
                black_hole,
                r_inner,
                r_outer,
                max_steps,
            );
            
            // Count disc hits
            if matches!(result, crate::geodesic::GeodesicResult::DiscHit { .. }) {
                disc_hits += 1;
            }
            
            // Pack result
            maps.pack_pixel(x, y, &result);
            
            completed += 1;
            
            // Optional: print progress every 1000 pixels
            if completed % 1000 == 0 {
                let progress = (completed as f64 / total_pixels as f64) * 100.0;
                println!("Progress: {:.1}%", progress);
            }
        }
    }
    
    // Update manifest with actual disc_hits count
    maps.manifest.disc_hits = disc_hits;
    
    println!("Rendering complete: {} pixels", total_pixels);
    maps
}

