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
    pub r: f64,              // Radial coordinate
    pub phi: f64,            // Azimuthal angle
    pub energy: f64,         // Conserved energy
    pub angular_momentum: f64, // Conserved L_z
    pub order: u8,           // Geodesic order
    pub hit: bool,           // Did it hit the disc?
}

impl HighPrecisionData {
    // Export high-precision data to JSON for documentation/analysis
    // Simple manual JSON generation (will use serde in CLI later)
    pub fn to_json(&self) -> String {
        let mut json = String::from("{\n  \"positions\": [\n");
        
        for (i, pos) in self.positions.iter().enumerate() {
            if pos.hit {
                json.push_str(&format!(
                    "    {{\"r\": {:.15}, \"phi\": {:.15}, \"energy\": {:.15}, \"angular_momentum\": {:.15}, \"order\": {}, \"hit\": true}}",
                    pos.r, pos.phi, pos.energy, pos.angular_momentum, pos.order
                ));
            } else {
                json.push_str("    {\"hit\": false}");
            }
            
            if i < self.positions.len() - 1 {
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
        
        let r_values: Vec<f64> = hit_pixels.iter().map(|p| p.r).collect();
        let energy_values: Vec<f64> = hit_pixels.iter().map(|p| p.energy).collect();
        let l_z_values: Vec<f64> = hit_pixels.iter().map(|p| p.angular_momentum).collect();
        
        DataStatistics {
            total_pixels: self.positions.len(),
            hit_pixels: hit_pixels.len(),
            min_r: r_values.iter().cloned().fold(f64::INFINITY, f64::min),
            max_r: r_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
            mean_energy: energy_values.iter().sum::<f64>() / energy_values.len() as f64,
            mean_l_z: l_z_values.iter().sum::<f64>() / l_z_values.len() as f64,
        }
    }
}

// Statistics for documentation in MDX
#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct DataStatistics {
    pub total_pixels: usize,
    pub hit_pixels: usize,
    pub min_r: f64,
    pub max_r: f64,
    pub mean_energy: f64,
    pub mean_l_z: f64,
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
                GeodesicResult::DiscHit { r, phi, energy, angular_momentum, order } => {
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
                            r: *r,
                            phi: *phi,
                            energy: *energy,
                            angular_momentum: *angular_momentum,
                            order: *order,
                            hit: true,
                        };
                    }
                }
                _ => {
                    // Non-hits: zeros already (skip writing T1/T2)
                    // High-precision data (if enabled)
                    if let Some(ref hp) = self.high_precision_data {
                        let hp_ptr = hp.positions.as_ptr() as *mut PositionData;
                        *hp_ptr.add(idx) = PositionData::default();
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
                
                if let GeodesicResult::DiscHit { r, phi, energy, angular_momentum, order: ord } = result {
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
                    let hp_idx = idx * self.max_orders as usize + order;
                    let hp_ptr = hp.positions.as_ptr() as *mut PositionData;
                    
                    *hp_ptr.add(hp_idx) = match result {
                        GeodesicResult::DiscHit { r, phi, energy, angular_momentum, order } => {
                            PositionData {
                                r: *r,
                                phi: *phi,
                                energy: *energy,
                                angular_momentum: *angular_momentum,
                                order: *order,
                                hit: true,
                            }
                        }
                        _ => PositionData::default(),
                    };
                }
            }
        }
    }
}

