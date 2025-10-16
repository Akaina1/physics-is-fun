// Main rendering function for transfer maps

use crate::types::{BlackHole, Camera, RenderConfig, OrbitDirection};
use crate::transfer_maps::{TransferMaps, Manifest};
use crate::integration::integrate_geodesic_multi_order;
use crate::disc_model::generate_flux_lut;
use crate::validation::{check_theta_potential, check_r_potential};
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
    let mut maps = TransferMaps::new(
        config.width, 
        config.height, 
        temp_manifest, 
        flux_samples,
        config.max_orders,
        export_high_precision
    );
    
    // Generate emissivity lookup table
    maps.flux_r32f = generate_flux_lut(r_inner, r_outer, r_inner, flux_samples);
    
    let max_steps = 10000;
    
    // Thread-safe counters
    let disc_hits = AtomicUsize::new(0);
    let pixels_processed = AtomicU64::new(0);
    
    // Diagnostic: Print sample ray to verify LNRF initialization
    // Sample from center pixel to check our formulas
    let center_x = config.width / 2;
    let center_y = config.height / 2;
    let sample_ray = camera.generate_ray(center_x, center_y, config);
    let (sample_photon, sample_sign_theta) = sample_ray.to_photon_state(black_hole);
    
    // Calculate initial null invariant for this ray
    let init_ni = crate::geodesic::compute_null_invariant(
        sample_photon.r,
        sample_photon.theta,
        sample_photon.energy,
        sample_photon.angular_momentum,
        sample_photon.carter_q,
        black_hole.mass,
        black_hole.spin,
        -1.0,  // sign_r: moving inward initially
        1.0,   // sign_theta: moving toward equator initially
    );
    
    // Calculate K (non-negative Carter quantity)
    let k_nonneg = sample_photon.carter_q + (sample_photon.angular_momentum - black_hole.spin * sample_photon.energy).powi(2);
    
    // Test a few more pixels to check sign_theta distribution
    let top_ray = camera.generate_ray(config.width / 2, 0, config);
    let (top_photon, top_sign) = top_ray.to_photon_state(black_hole);
    
    let bottom_ray = camera.generate_ray(config.width / 2, config.height - 1, config);
    let (bottom_photon, bottom_sign) = bottom_ray.to_photon_state(black_hole);
    
    println!("\n🔬 Diagnostic: Sample Rays");
    println!("Center (x={}, y={}):", config.width / 2, config.height / 2);
    println!("  θ={:.6} rad, sign_theta={}, p_θ direction={}", 
        sample_photon.theta, sample_sign_theta, if sample_sign_theta > 0.0 { "increasing (toward equator)" } else { "decreasing (away from equator)" });
    println!("Top (x={}, y=0):", config.width / 2);
    println!("  θ={:.6} rad, sign_theta={}, p_θ direction={}", 
        top_photon.theta, top_sign, if top_sign > 0.0 { "increasing (toward equator)" } else { "decreasing (away from equator)" });
    println!("Bottom (x={}, y={}):", config.width / 2, config.height - 1);
    println!("  θ={:.6} rad, sign_theta={}, p_θ direction={}", 
        bottom_photon.theta, bottom_sign, if bottom_sign > 0.0 { "increasing (toward equator)" } else { "decreasing (away from equator)" });
    println!();
    println!("Center pixel:");
    println!("  Conserved:   E={:.6}, L_z={:.6}, Q={:.6}", 
        sample_photon.energy, sample_photon.angular_momentum, sample_photon.carter_q);
    println!("  K (Q+(Lz-aE)²): {:.6}", k_nonneg);
    println!("  Initial NI:  {:.3e}", init_ni);
    
    // Status indicators
    if k_nonneg >= -1e-10 {
        println!("  ✓ K ≥ 0: Physical geodesic");
    } else {
        println!("  ✗ K < 0: UNPHYSICAL (K={:.3e})", k_nonneg);
    }
    
    if init_ni < 1e-12 {
        println!("  ✓ Initial NI < 1e-12: Excellent initialization");
    } else if init_ni < 1e-9 {
        println!("  ⚠ Initial NI = {:.3e}: Good but not perfect", init_ni);
    } else if init_ni < 1e-6 {
        println!("  ⚠ Initial NI = {:.3e}: Marginal - check initialization", init_ni);
    } else {
        println!("  ✗ Initial NI = {:.3e}: BAD initialization!", init_ni);
    }
    println!();

    // Diagnostics: sample grid of starting potentials
    {
        let grid = 16u32; // 16x16 coarse grid
        let mut theta_ok = 0usize;
        let mut r_ok = 0usize;
        let mut total = 0usize;
        for gy in 0..grid {
            let y = gy * (config.height - 1) / (grid - 1).max(1);
            for gx in 0..grid {
                let x = gx * (config.width - 1) / (grid - 1).max(1);
                let ray = camera.generate_ray(x, y, config);
                let (photon, _sign_th) = ray.to_photon_state(black_hole);
                let (_th_pot, th_ok) = check_theta_potential(&photon, black_hole.spin);
                let (_r_pot, r_can) = check_r_potential(&photon, black_hole.mass, black_hole.spin);
                if th_ok { theta_ok += 1; }
                if r_can { r_ok += 1; }
                total += 1;
            }
        }
        let th_frac = theta_ok as f64 / total as f64 * 100.0;
        let r_frac = r_ok as f64 / total as f64 * 100.0;
        println!("🧪 Start potentials: Θ>0 for {:.1}% of sampled rays, R>0 for {:.1}%", th_frac, r_frac);
        if th_frac < 1.0 {
            println!("  ⚠ Almost all rays have Θ(θ0) ≤ 0 → cannot move toward equator (check camera initialization).");
        }
    }
    
    // Parallel loop over all pixels using Rayon
    // Process rows in parallel for better cache locality
    (0..config.height).into_par_iter().for_each(|y| {
        for x in 0..config.width {
            // Generate ray for this pixel
            let ray = camera.generate_ray(x, y, config);
            
            // Convert to photon state
            let (photon, sign_theta_init) = ray.to_photon_state(black_hole);
            
            // Get all orders in a single integration pass
            let results = integrate_geodesic_multi_order(
                photon,
                sign_theta_init,
                black_hole,
                r_inner,
                r_outer,
                config.max_orders,
                max_steps,
            );
            
            // Count disc hits (only order 0 to avoid double-counting)
            if results[0].is_hit() {
                disc_hits.fetch_add(1, Ordering::Relaxed);
            }
            
            // Pack all order results
            maps.pack_pixel_multi_order(x, y, &results);
            
            // Update progress (thread-safe atomic increment)
            let count = pixels_processed.fetch_add(1, Ordering::Relaxed) + 1;
            progress_callback(count);
        }
    });
    
    // Update manifest with actual disc_hits count (load atomic value)
    maps.manifest.disc_hits = disc_hits.load(Ordering::Relaxed);
    
    maps
}

