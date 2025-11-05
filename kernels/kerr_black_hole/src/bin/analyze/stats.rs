// Statistical infrastructure for robust outlier detection
//
// This module provides statistical functions optimized for heavy-tailed distributions
// commonly found in geodesic data (e.g., null invariant errors)

use std::collections::HashMap;

// ============================================================================
// BASIC STATISTICS
// ============================================================================

/// Compute median of a dataset
pub fn compute_median(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    
    let mid = sorted.len() / 2;
    if sorted.len() % 2 == 0 {
        (sorted[mid - 1] + sorted[mid]) / 2.0
    } else {
        sorted[mid]
    }
}

/// Compute Median Absolute Deviation (MAD) - more robust than standard deviation
/// 
/// MAD is defined as: MAD = median(|X_i - median(X)|)
/// 
/// Returns: (median, mad)
pub fn compute_mad(values: &[f64]) -> (f64, f64) {
    if values.is_empty() {
        return (0.0, 0.0);
    }
    
    let median = compute_median(values);
    
    let deviations: Vec<f64> = values.iter()
        .map(|v| (v - median).abs())
        .collect();
    
    let mad = compute_median(&deviations);
    
    (median, mad)
}

/// Compute MAD-based z-score (robust outlier detection)
/// 
/// The MAD z-score is more robust to outliers than standard z-score.
/// Scale factor 1.4826 makes MAD consistent with standard deviation for normal distribution.
pub fn mad_zscore(value: f64, median: f64, mad: f64) -> f64 {
    const MAD_SCALE: f64 = 1.4826; // Consistency constant for normal distribution
    
    if mad < 1e-300 {
        return 0.0; // Avoid division by zero
    }
    
    (value - median) / (MAD_SCALE * mad)
}

// ============================================================================
// LOG-SPACE STATISTICS (for heavy-tailed distributions)
// ============================================================================

/// Statistics computed in log space for heavy-tailed distributions
#[derive(Debug, Clone, Copy)]
pub struct LogStats {
    pub median_log: f64,
    pub mad_log: f64,
    pub count: usize,
}

/// Compute statistics in log space for null invariant (heavy-tailed distribution)
/// 
/// Null invariant errors span many orders of magnitude, so log-space is more appropriate.
pub fn compute_log_stats(values: &[f64]) -> LogStats {
    // Filter positive values and take log10
    let log_values: Vec<f64> = values.iter()
        .filter(|v| **v > 0.0)
        .map(|v| v.log10())
        .collect();
    
    if log_values.is_empty() {
        return LogStats {
            median_log: 0.0,
            mad_log: 0.0,
            count: 0,
        };
    }
    
    let (median, mad) = compute_mad(&log_values);
    
    LogStats {
        median_log: median,
        mad_log: mad,
        count: log_values.len(),
    }
}

/// Compute log-space MAD z-score for null invariant
pub fn log_mad_zscore(value: f64, log_stats: &LogStats) -> f64 {
    if value <= 0.0 {
        return 0.0;
    }
    
    let log_value = value.log10();
    mad_zscore(log_value, log_stats.median_log, log_stats.mad_log)
}

// ============================================================================
// PERCENTILE COMPUTATION
// ============================================================================

/// Compute percentile (0.0 to 1.0) from sorted data
pub fn compute_percentile(sorted_values: &[f64], percentile: f64) -> f64 {
    if sorted_values.is_empty() {
        return 0.0;
    }
    
    let idx = ((sorted_values.len() - 1) as f64 * percentile) as usize;
    sorted_values[idx.min(sorted_values.len() - 1)]
}

/// Compute multiple percentiles efficiently from sorted data
pub fn compute_percentiles(sorted_values: &[f64], percentiles: &[f64]) -> Vec<f64> {
    percentiles.iter()
        .map(|&p| compute_percentile(sorted_values, p))
        .collect()
}

// ============================================================================
// PER-ORDER STATISTICS
// ============================================================================

/// Statistics for a specific geodesic order
/// 
/// Computing per-order statistics is critical to avoid false positives,
/// as different orders have fundamentally different physical behavior.
#[derive(Debug, Clone)]
pub struct OrderStats {
    pub order: u8,
    
    // Null invariant (log space for heavy tail)
    pub ni_log_median: f64,
    pub ni_log_mad: f64,
    
    // Affine parameter (linear space)
    pub lambda_median: f64,
    pub lambda_mad: f64,
    
    // Redshift factor (linear space)
    pub g_median: f64,
    pub g_mad: f64,
    
    // Phi wraps (99th percentile for outlier threshold)
    pub wraps_p99: f64,
    
    // Turning points (99th percentile)
    pub turns_r_p99: u8,
    pub turns_theta_p99: u8,
    
    // Sample size
    pub count: usize,
}

/// Record structure compatible with HpRecord from main.rs
pub trait GeodesicRecord {
    fn null_invariant_error(&self) -> f64;
    fn affine_parameter(&self) -> f64;
    fn redshift_factor(&self) -> f64;
    fn phi_wraps(&self) -> f64;
    fn turns_r(&self) -> u8;
    fn turns_theta(&self) -> u8;
    fn order(&self) -> u8;
}

/// Compute per-order statistics from hit records
pub fn compute_order_stats<T: GeodesicRecord>(hits: &[&T], target_order: u8) -> OrderStats {
    // Filter to specific order
    let order_hits: Vec<_> = hits.iter()
        .filter(|h| h.order() == target_order)
        .collect();
    
    if order_hits.is_empty() {
        return OrderStats {
            order: target_order,
            ni_log_median: 0.0,
            ni_log_mad: 0.0,
            lambda_median: 0.0,
            lambda_mad: 0.0,
            g_median: 0.0,
            g_mad: 0.0,
            wraps_p99: 0.0,
            turns_r_p99: 0,
            turns_theta_p99: 0,
            count: 0,
        };
    }
    
    // Null invariant (log space)
    let ni_values: Vec<f64> = order_hits.iter()
        .map(|h| h.null_invariant_error())
        .collect();
    let ni_log_stats = compute_log_stats(&ni_values);
    
    // Affine parameter (linear space)
    let lambda_values: Vec<f64> = order_hits.iter()
        .map(|h| h.affine_parameter())
        .collect();
    let (lambda_median, lambda_mad) = compute_mad(&lambda_values);
    
    // Redshift factor (linear space)
    let g_values: Vec<f64> = order_hits.iter()
        .map(|h| h.redshift_factor())
        .collect();
    let (g_median, g_mad) = compute_mad(&g_values);
    
    // Phi wraps (need 99th percentile)
    let mut wraps_values: Vec<f64> = order_hits.iter()
        .map(|h| h.phi_wraps())
        .collect();
    wraps_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let wraps_p99 = compute_percentile(&wraps_values, 0.99);
    
    // Turning points (need 99th percentile)
    let mut turns_r_values: Vec<u8> = order_hits.iter()
        .map(|h| h.turns_r())
        .collect();
    turns_r_values.sort();
    let turns_r_p99 = turns_r_values[(turns_r_values.len() as f64 * 0.99) as usize];
    
    let mut turns_theta_values: Vec<u8> = order_hits.iter()
        .map(|h| h.turns_theta())
        .collect();
    turns_theta_values.sort();
    let turns_theta_p99 = turns_theta_values[(turns_theta_values.len() as f64 * 0.99) as usize];
    
    OrderStats {
        order: target_order,
        ni_log_median: ni_log_stats.median_log,
        ni_log_mad: ni_log_stats.mad_log,
        lambda_median,
        lambda_mad,
        g_median,
        g_mad,
        wraps_p99,
        turns_r_p99,
        turns_theta_p99,
        count: order_hits.len(),
    }
}

/// Compute statistics for all orders present in the data
pub fn compute_all_order_stats<T: GeodesicRecord>(hits: &[&T]) -> HashMap<u8, OrderStats> {
    // Find unique orders
    let mut orders: Vec<u8> = hits.iter().map(|h| h.order()).collect();
    orders.sort();
    orders.dedup();
    
    // Compute stats for each order
    orders.iter()
        .map(|&order| (order, compute_order_stats(hits, order)))
        .collect()
}

// ============================================================================
// SPATIAL INDEX GRID (for neighbor lookups)
// ============================================================================

/// Spatial index for O(1) neighbor lookups
/// 
/// Maps (x,y) pixel coordinates to indices in the positions array
pub struct IndexGrid {
    grid: Vec<Vec<Option<usize>>>,  // grid[y][x] -> index in positions
    width: usize,
    height: usize,
}

impl IndexGrid {
    /// Create spatial index from position data
    /// 
    /// Assumes positions are stored in order-major format:
    /// [pixel0_order0, pixel0_order1, ..., pixel1_order0, pixel1_order1, ...]
    pub fn new(width: usize, height: usize) -> Self {
        let grid = vec![vec![None; width]; height];
        Self { grid, width, height }
    }
    
    /// Set the index for a specific pixel
    pub fn set(&mut self, x: u32, y: u32, index: usize) {
        let x = x as usize;
        let y = y as usize;
        if x < self.width && y < self.height {
            self.grid[y][x] = Some(index);
        }
    }
    
    /// Get the index for a specific pixel
    pub fn get(&self, x: u32, y: u32) -> Option<usize> {
        let x = x as usize;
        let y = y as usize;
        if x < self.width && y < self.height {
            self.grid[y][x]
        } else {
            None
        }
    }
    
    /// Get 4-neighbor indices (N, S, E, W) that have same order
    /// 
    /// Used for spatial discontinuity detection
    pub fn get_neighbors_same_order<T: GeodesicRecord>(
        &self,
        x: u32,
        y: u32,
        order: u8,
        positions: &[T],
    ) -> Vec<usize> {
        let mut neighbors = Vec::new();
        
        // North (y-1)
        if y > 0 {
            if let Some(idx) = self.get(x, y - 1) {
                if idx < positions.len() && positions[idx].order() == order {
                    neighbors.push(idx);
                }
            }
        }
        
        // South (y+1)
        if let Some(idx) = self.get(x, y + 1) {
            if idx < positions.len() && positions[idx].order() == order {
                neighbors.push(idx);
            }
        }
        
        // West (x-1)
        if x > 0 {
            if let Some(idx) = self.get(x, y) {
                if idx < positions.len() && positions[idx].order() == order {
                    neighbors.push(idx);
                }
            }
        }
        
        // East (x+1)
        if let Some(idx) = self.get(x + 1, y) {
            if idx < positions.len() && positions[idx].order() == order {
                neighbors.push(idx);
            }
        }
        
        neighbors
    }
    
    /// Get 8-neighbor indices (N, S, E, W, NE, NW, SE, SW) that have same order
    /// 
    /// Used for more thorough spatial analysis
    pub fn get_neighbors_8_same_order<T: GeodesicRecord>(
        &self,
        x: u32,
        y: u32,
        order: u8,
        positions: &[T],
    ) -> Vec<usize> {
        let mut neighbors = Vec::new();
        
        let offsets = [
            (0, -1),  // N
            (0, 1),   // S
            (-1, 0),  // W
            (1, 0),   // E
            (-1, -1), // NW
            (1, -1),  // NE
            (-1, 1),  // SW
            (1, 1),   // SE
        ];
        
        for (dx, dy) in offsets.iter() {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            
            if nx >= 0 && ny >= 0 {
                if let Some(idx) = self.get(nx as u32, ny as u32) {
                    if idx < positions.len() && positions[idx].order() == order {
                        neighbors.push(idx);
                    }
                }
            }
        }
        
        neighbors
    }
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Compute mean (for reference, though median is more robust)
pub fn compute_mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<f64>() / values.len() as f64
}

/// Compute standard deviation (for reference, though MAD is more robust)
pub fn compute_std_dev(values: &[f64], mean: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    
    let variance = values.iter()
        .map(|v| (v - mean).powi(2))
        .sum::<f64>() / values.len() as f64;
    
    variance.sqrt()
}

/// Check if value is finite (not NaN or infinite)
pub fn is_finite(value: f64) -> bool {
    value.is_finite()
}

/// Check if all values in slice are finite
pub fn all_finite(values: &[f64]) -> bool {
    values.iter().all(|v| v.is_finite())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_median_odd() {
        let values = vec![1.0, 3.0, 2.0, 5.0, 4.0];
        assert_eq!(compute_median(&values), 3.0);
    }
    
    #[test]
    fn test_median_even() {
        let values = vec![1.0, 2.0, 3.0, 4.0];
        assert_eq!(compute_median(&values), 2.5);
    }
    
    #[test]
    fn test_mad() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let (median, mad) = compute_mad(&values);
        assert_eq!(median, 3.0);
        assert_eq!(mad, 1.0); // |1-3|=2, |2-3|=1, |3-3|=0, |4-3|=1, |5-3|=2 -> median([2,1,0,1,2]) = 1
    }
    
    #[test]
    fn test_mad_zscore() {
        let zscore = mad_zscore(6.0, 3.0, 1.0);
        assert!((zscore - 2.024).abs() < 0.01); // (6-3)/(1.4826*1) ≈ 2.024
    }
    
    #[test]
    fn test_log_stats() {
        let values = vec![1e-15, 1e-12, 1e-10, 1e-8];
        let stats = compute_log_stats(&values);
        assert!(stats.median_log < -10.0);
        assert!(stats.count == 4);
    }
}

// ============================================================================
// TIER 3: CRITICAL CURVE EXTRACTION & ELLIPSE FITTING
// ============================================================================

/// Parameters describing a fitted ellipse
#[derive(Debug, Clone)]
pub struct EllipseParams {
    pub center_x: f64,
    pub center_y: f64,
    pub semi_major: f64,
    pub semi_minor: f64,
    pub rotation: f64,  // radians
    pub axis_ratio: f64, // b/a
}

/// Trait for records that can be used for critical curve extraction
pub trait CriticalCurveRecord {
    fn pixel_x(&self) -> u32;
    fn pixel_y(&self) -> u32;
    fn is_captured(&self) -> bool;
}

/// Extract boundary pixels between captured and escaped rays
/// Returns pixel coordinates (x, y) that lie on the critical curve
pub fn extract_critical_curve<T: CriticalCurveRecord>(
    records: &[T],
    width: usize,
    height: usize,
) -> Vec<(u32, u32)> {
    // Build binary capture grid: 1 = captured, 0 = escaped/hit
    let mut capture_grid = vec![vec![0u8; width]; height];
    
    for rec in records {
        let x = rec.pixel_x() as usize;
        let y = rec.pixel_y() as usize;
        
        if x < width && y < height {
            // Captured rays: those that were captured (not escaped or hit)
            if rec.is_captured() {
                capture_grid[y][x] = 1;
            }
        }
    }
    
    // Edge detection: 4-neighbor difference
    let mut boundary_pixels = Vec::new();
    
    for y in 1..(height - 1) {
        for x in 1..(width - 1) {
            let center = capture_grid[y][x];
            let up = capture_grid[y - 1][x];
            let down = capture_grid[y + 1][x];
            let left = capture_grid[y][x - 1];
            let right = capture_grid[y][x + 1];
            
            // Boundary pixel if center differs from any neighbor
            if center != up || center != down || center != left || center != right {
                boundary_pixels.push((x as u32, y as u32));
            }
        }
    }
    
    boundary_pixels
}

/// Fit an ellipse to a set of 2D points using least-squares
/// Uses algebraic distance minimization
pub fn fit_ellipse(points: &[(u32, u32)]) -> Option<EllipseParams> {
    if points.len() < 6 {
        return None; // Need at least 6 points to fit an ellipse
    }
    
    // Convert to f64 and center the data
    let mean_x = points.iter().map(|(x, _)| *x as f64).sum::<f64>() / points.len() as f64;
    let mean_y = points.iter().map(|(_, y)| *y as f64).sum::<f64>() / points.len() as f64;
    
    let centered: Vec<(f64, f64)> = points.iter()
        .map(|(x, y)| (*x as f64 - mean_x, *y as f64 - mean_y))
        .collect();
    
    // Compute second moment matrix for simple ellipse fitting
    // This is a simplified algebraic fit (not full conic fitting)
    let mut mxx = 0.0;
    let mut mxy = 0.0;
    let mut myy = 0.0;
    
    for (x, y) in &centered {
        mxx += x * x;
        mxy += x * y;
        myy += y * y;
    }
    
    let n = points.len() as f64;
    mxx /= n;
    mxy /= n;
    myy /= n;
    
    // Compute eigenvalues to get semi-axes
    let trace = mxx + myy;
    let det = mxx * myy - mxy * mxy;
    let disc = ((trace * trace) / 4.0 - det).max(0.0).sqrt();
    
    let lambda1 = trace / 2.0 + disc;
    let lambda2 = trace / 2.0 - disc;
    
    if lambda1 <= 0.0 || lambda2 <= 0.0 {
        return None; // Invalid ellipse
    }
    
    // Semi-axes (scaled by sqrt for radius)
    let semi_major = lambda1.sqrt() * 2.0; // Factor of 2 for better scale matching
    let semi_minor = lambda2.sqrt() * 2.0;
    
    // Rotation angle from eigenvector
    let rotation = if mxy.abs() < 1e-10 {
        0.0
    } else {
        ((lambda1 - mxx) / mxy).atan()
    };
    
    let axis_ratio = semi_minor / semi_major;
    
    Some(EllipseParams {
        center_x: mean_x,
        center_y: mean_y,
        semi_major,
        semi_minor,
        rotation,
        axis_ratio,
    })
}

