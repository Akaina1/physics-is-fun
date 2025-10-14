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
        Self {
            width,
            height,
            preset: preset.clone(),
            inclination,
            spin,
            orders,
            r_in,
            r_out,
            t1_url: format!("/blackhole/{}/t1_rgba32f.bin", preset),
            t2_url: format!("/blackhole/{}/t2_rgba32f.bin", preset),
            flux_url: format!("/blackhole/{}/flux_r32f.bin", preset),
            disc_hits,
        }
    }
}

// ============================================================================
// TRANSFER MAP PACKING
// ============================================================================

// Pack geodesic result into transfer map format
// 
// T1 (RGBA32F): (r, sin(φ₀), cos(φ₀), mask) - Full precision
// T2 (RGBA32F): (-k_t, k_φ, order, pad)
// Flux (R32F): 1D emissivity lookup table
// 
// Optional: High-precision f64 data for analysis/documentation
pub struct TransferMaps {
    pub t1_rgba32f: Vec<f32>,  // Position data (f32)
    pub t2_rgba32f: Vec<f32>,  // Physics data (f32)
    pub flux_r32f: Vec<f32>,   // 1D emissivity LUT
    pub manifest: Manifest,
    pub width: u32,
    pub height: u32,
    
    // Optional: Original f64 precision data for analysis/documentation
    // Enable with export_high_precision = true
    pub high_precision_data: Option<HighPrecisionData>,
}

// High-precision f64 data export for documentation/analysis
// This preserves the original computation accuracy for reference
#[derive(Debug, Clone)]
pub struct HighPrecisionData {
    pub positions: Vec<PositionData>,  // Per-pixel f64 positions
}

#[derive(Debug, Clone, Copy)]
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
    pub fn new(width: u32, height: u32, manifest: Manifest, flux_samples: usize, export_high_precision: bool) -> Self {
        let pixel_count = (width * height) as usize;
        
        let high_precision_data = if export_high_precision {
            Some(HighPrecisionData {
                positions: Vec::with_capacity(pixel_count),
            })
        } else {
            None
        };
        
        Self {
            t1_rgba32f: vec![0.0f32; pixel_count * 4],  // Now f32 instead of f16
            t2_rgba32f: vec![0.0f32; pixel_count * 4],
            flux_r32f: vec![0.0f32; flux_samples],
            manifest,
            width,
            height,
            high_precision_data,
        }
    }
    
    // Pack a single pixel's result
    pub fn pack_pixel(&mut self, x: u32, y: u32, result: &GeodesicResult) {
        let idx = (y * self.width + x) as usize;
        let t1_idx = idx * 4;
        let t2_idx = idx * 4;
        
        match result {
            GeodesicResult::DiscHit { r, phi, energy, angular_momentum, order } => {
                // T1: (r, sin(φ), cos(φ), mask=1) - Direct f64 → f32
                self.t1_rgba32f[t1_idx + 0] = *r as f32;
                self.t1_rgba32f[t1_idx + 1] = phi.sin() as f32;
                self.t1_rgba32f[t1_idx + 2] = phi.cos() as f32;
                self.t1_rgba32f[t1_idx + 3] = 1.0;  // Hit mask
                
                // T2: (-k_t, k_φ, order, pad)
                self.t2_rgba32f[t2_idx + 0] = *energy as f32;
                self.t2_rgba32f[t2_idx + 1] = *angular_momentum as f32;
                self.t2_rgba32f[t2_idx + 2] = *order as f32;
                self.t2_rgba32f[t2_idx + 3] = 0.0;  // Padding
                
                // Store high-precision f64 data if enabled
                if let Some(ref mut hp) = self.high_precision_data {
                    hp.positions.push(PositionData {
                        r: *r,
                        phi: *phi,
                        energy: *energy,
                        angular_momentum: *angular_momentum,
                        order: *order,
                        hit: true,
                    });
                }
            }
            _ => {
                // Captured or Escaped: leave as zeros (mask=0 means no hit)
                // Already initialized to zero
                
                // Store high-precision f64 data if enabled (non-hit case)
                if let Some(ref mut hp) = self.high_precision_data {
                    hp.positions.push(PositionData {
                        r: 0.0,
                        phi: 0.0,
                        energy: 0.0,
                        angular_momentum: 0.0,
                        order: 0,
                        hit: false,
                    });
                }
            }
        }
    }
}

