// Main rendering function for transfer maps

use crate::types::{BlackHole, Camera, RenderConfig, OrbitDirection};
use crate::transfer_maps::{TransferMaps, Manifest};
use crate::integration::integrate_geodesic;
use crate::disc_model::generate_flux_lut;
use rayon::prelude::*;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

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
pub fn render_transfer_maps<F>(
    black_hole: &BlackHole,
    camera: &Camera,
    config: &RenderConfig,
    orbit_direction: OrbitDirection,
    preset_name: String,
    export_high_precision: bool,  // Enable f64 export for documentation
    progress_callback: F,
) -> TransferMaps 
where
    F: Fn(u64) + Sync + Send,
{
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
    
    // Thread-safe counters
    let disc_hits = AtomicUsize::new(0);
    let pixels_processed = AtomicU64::new(0);
    
    // Parallel loop over all pixels using Rayon
    // Process rows in parallel for better cache locality
    (0..config.height).into_par_iter().for_each(|y| {
        for x in 0..config.width {
            // Generate ray for this pixel
            let ray = camera.generate_ray(x, y, config);
            
            // Convert to photon state
            let photon = ray.to_photon_state(black_hole);
            
            
            // Integrate geodesic
            let result = integrate_geodesic(
                photon,
                black_hole,
                r_inner,
                r_outer,
                max_steps,
            );
            
            // Count disc hits (thread-safe atomic increment)
            if matches!(result, crate::geodesic::GeodesicResult::DiscHit { .. }) {
                disc_hits.fetch_add(1, Ordering::Relaxed);
            }
            
            // Pack result (thread-safe: each pixel writes to unique index)
            maps.pack_pixel(x, y, &result);
            
            // Update progress (thread-safe atomic increment)
            let count = pixels_processed.fetch_add(1, Ordering::Relaxed) + 1;
            progress_callback(count);
        }
    });
    
    // Update manifest with actual disc_hits count (load atomic value)
    maps.manifest.disc_hits = disc_hits.load(Ordering::Relaxed);
    
    maps
}

