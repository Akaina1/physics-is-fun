// Transfer map packing and storage

use crate::geodesic::GeodesicResult;
use serde::Serialize;

// ============================================================================
// MANIFEST METADATA
// ============================================================================

// Manifest metadata for a rendered preset
// This gets serialized to JSON for the frontend
#[derive(Debug, Clone, Serialize)]
pub struct Manifest {
    pub width: u32,
    pub height: u32,
    pub preset: String,
    pub inclination: f64,
    pub spin: f64,
    pub orders: u8,
    pub r_in: f64,
    pub r_out: f64,
    pub t1_url: String,
    pub t2_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub t3_url: Option<String>,  // Order 1 positions (if orders > 1)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub t4_url: Option<String>,  // Order 1 physics (if orders > 1)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub t5_url: Option<String>,  // Order 2+ positions (if orders > 2)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub t6_url: Option<String>,  // Order 2+ physics (if orders > 2)
    pub flux_url: String,
    pub disc_hits: usize,
    
    // NEW: Tier 1.4 - Provenance tracking
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_sha: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rustc_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build_timestamp: Option<String>,
}

impl Manifest {
    pub fn new(
        width: u32,
        height: u32,
        preset: String,
        inclination: f64,
        spin: f64,
        orders: u8,
        r_in: f64,
        r_out: f64,
        disc_hits: usize,
    ) -> Self {
        Self::with_provenance(
            width,
            height,
            preset,
            inclination,
            spin,
            orders,
            r_in,
            r_out,
            disc_hits,
            None,
            None,
            None,
        )
    }
    
    pub fn with_provenance(
        width: u32,
        height: u32,
        preset: String,
        inclination: f64,
        spin: f64,
        orders: u8,
        r_in: f64,
        r_out: f64,
        disc_hits: usize,
        git_sha: Option<String>,
        rustc_version: Option<String>,
        build_timestamp: Option<String>,
    ) -> Self {
        let base_url = format!("/blackhole/{}", preset);
        
        Self {
            width,
            height,
            preset: preset.clone(),
            inclination,
            spin,
            orders,
            r_in,
            r_out,
            t1_url: format!("{}/t1_rgba32f.bin", base_url),
            t2_url: format!("{}/t2_rgba32f.bin", base_url),
            t3_url: if orders > 1 { Some(format!("{}/t3_rgba32f.bin", base_url)) } else { None },
            t4_url: if orders > 1 { Some(format!("{}/t4_rgba32f.bin", base_url)) } else { None },
            t5_url: if orders > 2 { Some(format!("{}/t5_rgba32f.bin", base_url)) } else { None },
            t6_url: if orders > 2 { Some(format!("{}/t6_rgba32f.bin", base_url)) } else { None },
            flux_url: format!("{}/flux_r32f.bin", base_url),
            disc_hits,
            
            // NEW: Tier 1.4 - Provenance from build environment
            git_sha,
            rustc_version,
            build_timestamp,
        }
    }
}

// ============================================================================
// TRANSFER MAP PACKING
// ============================================================================

// Pack geodesic result into transfer map format
// 
// Multi-order support: Separate textures for each order
// T1/T2: Order 0 (primary image)
// T3/T4: Order 1 (photon ring/secondary image)
// T5/T6: Order 2+ (higher-order images)
// 
// Each texture pair stores:
// - Position texture (RGBA32F): (r, sin(φ), cos(φ), weight/mask)
// - Physics texture (RGBA32F): (-k_t, k_φ, order, pad)
// 
// Flux (R32F): 1D emissivity lookup table (shared across all orders)
// 
// Optional: High-precision f64 data for analysis/documentation
pub struct TransferMaps {
    pub t1_rgba32f: Vec<f32>,  // Order 0 positions
    pub t2_rgba32f: Vec<f32>,  // Order 0 physics
    pub t3_rgba32f: Vec<f32>,  // Order 1 positions
    pub t4_rgba32f: Vec<f32>,  // Order 1 physics
    pub t5_rgba32f: Vec<f32>,  // Order 2+ positions
    pub t6_rgba32f: Vec<f32>,  // Order 2+ physics
    pub flux_r32f: Vec<f32>,   // 1D emissivity LUT
    pub manifest: Manifest,
    pub width: u32,
    pub height: u32,
    pub max_orders: u8,
    
    // Optional: Original f64 precision data for analysis/documentation
    // Enable with export_high_precision = true
    pub high_precision_data: Option<HighPrecisionData>,
}

// SAFETY: TransferMaps can be safely shared across threads because:
// - pack_pixel() ensures each thread writes to disjoint indices
// - All fields are either immutable after construction or written to unique locations
unsafe impl Sync for TransferMaps {}

// High-precision f64 data export for documentation/analysis
// This preserves the original computation accuracy for reference
#[derive(Debug, Clone)]
pub struct HighPrecisionData {
    pub positions: Vec<PositionData>,  // Per-pixel f64 positions
}

#[derive(Debug, Clone, Copy, Default)]
pub struct PositionData {
    // Pixel coordinates (NEW for spatial analysis)
    pub pixel_x: u32,        // X coordinate in image
    pub pixel_y: u32,        // Y coordinate in image
    
    // Position at disc intersection
    pub r: f64,              // Radial coordinate
    pub theta: f64,          // Polar angle at disc intersection
    pub phi: f64,            // Azimuthal angle
    
    // Conserved quantities (complete set)
    pub energy: f64,         // Conserved energy E
    pub angular_momentum: f64, // Conserved angular momentum L_z
    pub carter_q: f64,       // Carter constant Q
    
    // Derived quantities
    pub impact_parameter: f64,  // b = L_z/E (classical impact parameter)
    pub redshift_factor: f64,   // Gravitational redshift g-factor
    
    // Path information
    pub affine_parameter: f64,  // Affine parameter λ at disc hit
    pub phi_wraps: f64,        // Number of azimuthal wraps (φ_total / 2π)
    
    // Order/status
    pub order: u8,           // Geodesic order (0=primary, 1=secondary, etc.)
    pub hit: bool,           // Did it hit the disc?
    
    // Validation metrics
    pub null_invariant_error: f64, // |g_μν k^μ k^ν| (should be ~0 for null geodesic)
    
    // NEW: Miss classification
    pub escaped: bool,       // r → ∞ (> 1000M)
    pub captured: bool,      // r → r_horizon (< r_h + 0.01M)
    pub aborted: bool,       // Numerical failure (NaN, step limit)
    
    // NEW: Geodesic complexity
    pub turns_r: u8,         // Total radial turning points
    pub turns_theta: u8,     // Total polar turning points
}

impl HighPrecisionData {
    // Export high-precision data to JSON for documentation/analysis
    // Simple manual JSON generation (will use serde in CLI later)
    pub fn to_json(&self) -> String {
        let mut json = String::from("{\n  \"positions\": [\n");
        
        // Filter out uninitialized slots (pixel_x = 0 && pixel_y = 0 && all flags false)
        let valid_positions: Vec<&PositionData> = self.positions.iter()
            .filter(|pos| {
                // Keep if it's a hit, or if it's a miss with at least one flag set
                pos.hit || pos.escaped || pos.captured || pos.aborted
            })
            .collect();
        
        for (i, pos) in valid_positions.iter().enumerate() {
            if pos.hit {
                json.push_str(&format!(
                    "    {{\"pixel_x\": {}, \"pixel_y\": {}, \"r\": {:.15}, \"theta\": {:.15}, \"phi\": {:.15}, \"energy\": {:.15}, \"angular_momentum\": {:.15}, \"carter_q\": {:.15}, \"impact_parameter\": {:.15}, \"redshift_factor\": {:.15}, \"affine_parameter\": {:.15}, \"phi_wraps\": {:.15}, \"order\": {}, \"null_invariant_error\": {:.15}, \"turns_r\": {}, \"turns_theta\": {}, \"hit\": true}}",
                    pos.pixel_x, pos.pixel_y, pos.r, pos.theta, pos.phi, pos.energy, pos.angular_momentum, pos.carter_q, 
                    pos.impact_parameter, pos.redshift_factor, pos.affine_parameter, pos.phi_wraps,
                    pos.order, pos.null_invariant_error, pos.turns_r, pos.turns_theta
                ));
            } else {
                json.push_str(&format!(
                    "    {{\"pixel_x\": {}, \"pixel_y\": {}, \"hit\": false, \"escaped\": {}, \"captured\": {}, \"aborted\": {}}}",
                    pos.pixel_x, pos.pixel_y, pos.escaped, pos.captured, pos.aborted
                ));
            }
            
            if i < valid_positions.len() - 1 {
                json.push_str(",\n");
            }
        }
        
        json.push_str("\n  ]\n}");
        json
    }
    
    // Get statistics about the data for MDX content
    pub fn statistics(&self) -> DataStatistics {
        let hit_pixels: Vec<_> = self.positions.iter().filter(|p| p.hit).collect();
        
        if hit_pixels.is_empty() {
            return DataStatistics::default();
        }
        
        // Separate by order
        let order_0: Vec<_> = hit_pixels.iter().filter(|p| p.order == 0).collect();
        let order_1: Vec<_> = hit_pixels.iter().filter(|p| p.order == 1).collect();
        let order_2_plus: Vec<_> = hit_pixels.iter().filter(|p| p.order >= 2).collect();
        
        // Collect all values
        let r_values: Vec<f64> = hit_pixels.iter().map(|p| p.r).collect();
        let l_z_values: Vec<f64> = hit_pixels.iter().map(|p| p.angular_momentum).collect();
        let energy_values: Vec<f64> = hit_pixels.iter().map(|p| p.energy).collect();
        let q_values: Vec<f64> = hit_pixels.iter().map(|p| p.carter_q).collect();
        let impact_values: Vec<f64> = hit_pixels.iter().map(|p| p.impact_parameter).collect();
        let phi_wraps: Vec<f64> = hit_pixels.iter().map(|p| p.phi_wraps).collect();
        let null_errors: Vec<f64> = hit_pixels.iter().map(|p| p.null_invariant_error).collect();
        
        // Helper functions
        let mean = |vals: &[f64]| vals.iter().sum::<f64>() / vals.len() as f64;
        let std_dev = |vals: &[f64], mean_val: f64| {
            (vals.iter().map(|v| (v - mean_val).powi(2)).sum::<f64>() / vals.len() as f64).sqrt()
        };
        let median = |vals: &mut Vec<f64>| {
            vals.sort_by(|a, b| a.partial_cmp(b).unwrap());
            vals[vals.len() / 2]
        };
        
        // Calculate means
        let mean_r = mean(&r_values);
        let mean_l_z = mean(&l_z_values);
        let mean_energy = mean(&energy_values);
        let mean_q = mean(&q_values);
        let mean_impact = mean(&impact_values);
        
        // Median radius
        let mut r_sorted = r_values.clone();
        let median_r = median(&mut r_sorted);
        
        // Per-order affine parameters
        let mean_affine_0 = if !order_0.is_empty() {
            order_0.iter().map(|p| p.affine_parameter).sum::<f64>() / order_0.len() as f64
        } else { 0.0 };
        
        let mean_affine_1 = if !order_1.is_empty() {
            order_1.iter().map(|p| p.affine_parameter).sum::<f64>() / order_1.len() as f64
        } else { 0.0 };
        
        let mean_affine_2 = if !order_2_plus.is_empty() {
            order_2_plus.iter().map(|p| p.affine_parameter).sum::<f64>() / order_2_plus.len() as f64
        } else { 0.0 };
        
        DataStatistics {
            // Overall counts
            total_pixels: self.positions.len(),
            total_hits: hit_pixels.len(),
            
            // Per-order breakdown
            order_0_hits: order_0.len(),
            order_1_hits: order_1.len(),
            order_2_plus_hits: order_2_plus.len(),
            
            // Radius statistics
            min_r: r_values.iter().cloned().fold(f64::INFINITY, f64::min),
            max_r: r_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
            mean_r,
            median_r,
            std_r: std_dev(&r_values, mean_r),
            
            // Angular momentum statistics
            min_l_z: l_z_values.iter().cloned().fold(f64::INFINITY, f64::min),
            max_l_z: l_z_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
            mean_l_z,
            std_l_z: std_dev(&l_z_values, mean_l_z),
            
            // Energy statistics
            mean_energy,
            std_energy: std_dev(&energy_values, mean_energy),
            
            // Carter constant statistics
            min_q: q_values.iter().cloned().fold(f64::INFINITY, f64::min),
            max_q: q_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
            mean_q,
            std_q: std_dev(&q_values, mean_q),
            
            // Impact parameter statistics
            min_impact_param: impact_values.iter().cloned().fold(f64::INFINITY, f64::min),
            max_impact_param: impact_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
            mean_impact_param: mean_impact,
            
            // Path statistics
            mean_affine_param_order_0: mean_affine_0,
            mean_affine_param_order_1: mean_affine_1,
            mean_affine_param_order_2: mean_affine_2,
            max_phi_wraps: phi_wraps.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
            mean_phi_wraps: mean(&phi_wraps),
            
            // Validation metrics
            max_null_invariant_error: null_errors.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
            mean_null_invariant_error: mean(&null_errors),
        }
    }
}

// Statistics for documentation in MDX
#[derive(Debug, Clone, Default, Serialize)]
pub struct DataStatistics {
    // Overall counts
    pub total_pixels: usize,
    pub total_hits: usize,
    
    // Per-order breakdown
    pub order_0_hits: usize,
    pub order_1_hits: usize,
    pub order_2_plus_hits: usize,
    
    // Radius statistics
    pub min_r: f64,
    pub max_r: f64,
    pub mean_r: f64,
    pub median_r: f64,
    pub std_r: f64,
    
    // Angular momentum statistics
    pub min_l_z: f64,
    pub max_l_z: f64,
    pub mean_l_z: f64,
    pub std_l_z: f64,
    
    // Energy statistics
    pub mean_energy: f64,
    pub std_energy: f64,
    
    // Carter constant statistics
    pub min_q: f64,
    pub max_q: f64,
    pub mean_q: f64,
    pub std_q: f64,
    
    // Impact parameter statistics
    pub min_impact_param: f64,
    pub max_impact_param: f64,
    pub mean_impact_param: f64,
    
    // Path statistics
    pub mean_affine_param_order_0: f64,
    pub mean_affine_param_order_1: f64,
    pub mean_affine_param_order_2: f64,
    pub max_phi_wraps: f64,
    pub mean_phi_wraps: f64,
    
    // Validation metrics
    pub max_null_invariant_error: f64,
    pub mean_null_invariant_error: f64,
}

impl TransferMaps {
    pub fn new(
        width: u32, 
        height: u32, 
        manifest: Manifest, 
        flux_samples: usize,
        max_orders: u8,
        export_high_precision: bool
    ) -> Self {
        let pixel_count = (width * height) as usize;
        
        let high_precision_data = if export_high_precision {
            Some(HighPrecisionData {
                positions: vec![PositionData::default(); pixel_count * max_orders as usize],
            })
        } else {
            None
        };
        
        Self {
            t1_rgba32f: vec![0.0f32; pixel_count * 4],
            t2_rgba32f: vec![0.0f32; pixel_count * 4],
            t3_rgba32f: vec![0.0f32; pixel_count * 4],
            t4_rgba32f: vec![0.0f32; pixel_count * 4],
            t5_rgba32f: vec![0.0f32; pixel_count * 4],
            t6_rgba32f: vec![0.0f32; pixel_count * 4],
            flux_r32f: vec![0.0f32; flux_samples],
            manifest,
            width,
            height,
            max_orders,
            high_precision_data,
        }
    }
    
    // Thread-safe pixel packing (legacy single-order support)
    // SAFETY: Each thread writes to a unique pixel index, so no data races occur.
    // We cast away the shared reference constraint because we know indices are disjoint.
    pub fn pack_pixel(&self, x: u32, y: u32, result: &GeodesicResult) {
        let idx = (y * self.width + x) as usize;
        let t1_idx = idx * 4;
        let t2_idx = idx * 4;
        
        // SAFETY: Each thread writes to disjoint pixel indices
        unsafe {
            let t1_ptr = self.t1_rgba32f.as_ptr() as *mut f32;
            let t2_ptr = self.t2_rgba32f.as_ptr() as *mut f32;
            
            match result {
                GeodesicResult::DiscHit { 
                    r, phi, energy, angular_momentum, 
                    theta, carter_q, impact_parameter, redshift_factor, 
                    affine_parameter, phi_wraps, order, null_invariant_error,
                    turns_r, turns_theta
                } => {
                    // T1: (r, sin(φ), cos(φ), mask=1)
                    *t1_ptr.add(t1_idx) = *r as f32;
                    *t1_ptr.add(t1_idx + 1) = phi.sin() as f32;
                    *t1_ptr.add(t1_idx + 2) = phi.cos() as f32;
                    *t1_ptr.add(t1_idx + 3) = 1.0;
                    
                    // T2: (-k_t, k_φ, order, pad)
                    *t2_ptr.add(t2_idx) = *energy as f32;
                    *t2_ptr.add(t2_idx + 1) = *angular_momentum as f32;
                    *t2_ptr.add(t2_idx + 2) = *order as f32;
                    *t2_ptr.add(t2_idx + 3) = 0.0;
                    
                    // High-precision data (if enabled)
                    if let Some(ref hp) = self.high_precision_data {
                        let hp_ptr = hp.positions.as_ptr() as *mut PositionData;
                        *hp_ptr.add(idx) = PositionData {
                            pixel_x: x,
                            pixel_y: y,
                            r: *r,
                            theta: *theta,
                            phi: *phi,
                            energy: *energy,
                            angular_momentum: *angular_momentum,
                            carter_q: *carter_q,
                            impact_parameter: *impact_parameter,
                            redshift_factor: *redshift_factor,
                            affine_parameter: *affine_parameter,
                            phi_wraps: *phi_wraps,
                            order: *order,
                            hit: true,
                            null_invariant_error: *null_invariant_error,
                            escaped: false,
                            captured: false,
                            aborted: false,
                            turns_r: *turns_r,
                            turns_theta: *turns_theta,
                        };
                    }
                }
                GeodesicResult::Escaped { turns_r, turns_theta } => {
                    if let Some(ref hp) = self.high_precision_data {
                        let hp_ptr = hp.positions.as_ptr() as *mut PositionData;
                        *hp_ptr.add(idx) = PositionData {
                            pixel_x: x,
                            pixel_y: y,
                            hit: false,
                            escaped: true,
                            captured: false,
                            aborted: false,
                            turns_r: *turns_r,
                            turns_theta: *turns_theta,
                            ..Default::default()
                        };
                    }
                }
                GeodesicResult::Captured { turns_r, turns_theta } => {
                    if let Some(ref hp) = self.high_precision_data {
                        let hp_ptr = hp.positions.as_ptr() as *mut PositionData;
                        *hp_ptr.add(idx) = PositionData {
                            pixel_x: x,
                            pixel_y: y,
                            hit: false,
                            escaped: false,
                            captured: true,
                            aborted: false,
                            turns_r: *turns_r,
                            turns_theta: *turns_theta,
                            ..Default::default()
                        };
                    }
                }
                GeodesicResult::Aborted { turns_r, turns_theta } => {
                    if let Some(ref hp) = self.high_precision_data {
                        let hp_ptr = hp.positions.as_ptr() as *mut PositionData;
                        *hp_ptr.add(idx) = PositionData {
                            pixel_x: x,
                            pixel_y: y,
                            hit: false,
                            escaped: false,
                            captured: false,
                            aborted: true,
                            turns_r: *turns_r,
                            turns_theta: *turns_theta,
                            ..Default::default()
                        };
                    }
                }
            }
        }
    }
    
    // Pack multiple order results for a single pixel
    // 
    // This is the primary packing function for multi-order geodesic tracing.
    // Each pixel can have up to max_orders disc crossings, representing different
    // gravitational lensing paths.
    pub fn pack_pixel_multi_order(&self, x: u32, y: u32, results: &[GeodesicResult]) {
        let idx = (y * self.width + x) as usize;
        
        unsafe {
            // Get pointers to all textures
            let t1_ptr = self.t1_rgba32f.as_ptr() as *mut f32;
            let t2_ptr = self.t2_rgba32f.as_ptr() as *mut f32;
            let t3_ptr = self.t3_rgba32f.as_ptr() as *mut f32;
            let t4_ptr = self.t4_rgba32f.as_ptr() as *mut f32;
            let t5_ptr = self.t5_rgba32f.as_ptr() as *mut f32;
            let t6_ptr = self.t6_rgba32f.as_ptr() as *mut f32;
            
            for (order, result) in results.iter().enumerate() {
                let tex_idx = idx * 4;
                
                // Determine which textures to write to based on order
                let (pos_ptr, phys_ptr, weight) = match order {
                    0 => (t1_ptr, t2_ptr, 1.0),      // Primary: full brightness
                    1 => (t3_ptr, t4_ptr, 0.3),      // Secondary: 30% (photon ring)
                    _ => (t5_ptr, t6_ptr, 0.1),      // Tertiary+: 10% (very faint)
                };
                
                if let GeodesicResult::DiscHit { 
                    r, phi, energy, angular_momentum, order: ord, turns_r: _, turns_theta: _, .. 
                } = result {
                    // Position texture: (r, sin(φ), cos(φ), weight)
                    *pos_ptr.add(tex_idx) = *r as f32;
                    *pos_ptr.add(tex_idx + 1) = phi.sin() as f32;
                    *pos_ptr.add(tex_idx + 2) = phi.cos() as f32;
                    *pos_ptr.add(tex_idx + 3) = weight;
                    
                    // Physics texture: (-k_t, k_φ, order, pad)
                    *phys_ptr.add(tex_idx) = *energy as f32;
                    *phys_ptr.add(tex_idx + 1) = *angular_momentum as f32;
                    *phys_ptr.add(tex_idx + 2) = *ord as f32;
                    *phys_ptr.add(tex_idx + 3) = 0.0;
                }
                
                // High-precision data
                if let Some(ref hp) = self.high_precision_data {
                    let hp_ptr = hp.positions.as_ptr() as *mut PositionData;
                    
                    match result {
                        GeodesicResult::DiscHit { 
                            r, theta, phi, energy, angular_momentum, carter_q,
                            impact_parameter, redshift_factor, affine_parameter, phi_wraps,
                            order, null_invariant_error, turns_r, turns_theta
                        } => {
                            // Write hit record at its order slot
                            let hp_idx = idx * self.max_orders as usize + (*order as usize);
                            *hp_ptr.add(hp_idx) = PositionData {
                                pixel_x: x,
                                pixel_y: y,
                                r: *r,
                                theta: *theta,
                                phi: *phi,
                                energy: *energy,
                                angular_momentum: *angular_momentum,
                                carter_q: *carter_q,
                                impact_parameter: *impact_parameter,
                                redshift_factor: *redshift_factor,
                                affine_parameter: *affine_parameter,
                                phi_wraps: *phi_wraps,
                                order: *order,
                                hit: true,
                                null_invariant_error: *null_invariant_error,
                                escaped: false,
                                captured: false,
                                aborted: false,
                                turns_r: *turns_r,
                                turns_theta: *turns_theta,
                            };
                        }
                        GeodesicResult::Escaped { turns_r, turns_theta } => {
                            // Miss record: write ONLY ONCE at order 0 slot
                            if order == 0 {
                                let hp_idx = idx * self.max_orders as usize;
                                *hp_ptr.add(hp_idx) = PositionData {
                                    pixel_x: x,
                                    pixel_y: y,
                                    hit: false,
                                    escaped: true,
                                    captured: false,
                                    aborted: false,
                                    turns_r: *turns_r,
                                    turns_theta: *turns_theta,
                                    ..Default::default()
                                };
                            }
                        },
                        GeodesicResult::Captured { turns_r, turns_theta } => {
                            // Miss record: write ONLY ONCE at order 0 slot
                            if order == 0 {
                                let hp_idx = idx * self.max_orders as usize;
                                *hp_ptr.add(hp_idx) = PositionData {
                                    pixel_x: x,
                                    pixel_y: y,
                                    hit: false,
                                    escaped: false,
                                    captured: true,
                                    aborted: false,
                                    turns_r: *turns_r,
                                    turns_theta: *turns_theta,
                                    ..Default::default()
                                };
                            }
                        },
                        GeodesicResult::Aborted { turns_r, turns_theta } => {
                            // Miss record: write ONLY ONCE at order 0 slot
                            if order == 0 {
                                let hp_idx = idx * self.max_orders as usize;
                                *hp_ptr.add(hp_idx) = PositionData {
                                    pixel_x: x,
                                    pixel_y: y,
                                    hit: false,
                                    escaped: false,
                                    captured: false,
                                    aborted: true,
                                    turns_r: *turns_r,
                                    turns_theta: *turns_theta,
                                    ..Default::default()
                                };
                            }
                        },
                    }
                }
            }
        }
    }
}

