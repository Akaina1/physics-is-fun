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
        };
    }
    
    let (median, mad) = compute_mad(&log_values);
    
    LogStats {
        median_log: median,
        mad_log: mad,
    }
}

// ============================================================================
// PERCENTILE COMPUTATION
// ============================================================================

/// Compute percentile (0.0 to 1.0) from sorted data
fn compute_percentile(sorted_values: &[f64], percentile: f64) -> f64 {
    if sorted_values.is_empty() {
        return 0.0;
    }
    
    let idx = ((sorted_values.len() - 1) as f64 * percentile) as usize;
    sorted_values[idx.min(sorted_values.len() - 1)]
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
    // Null invariant (log space for heavy tail)
    pub ni_log_median: f64,
    pub ni_log_mad: f64,
    
    // Affine parameter (linear space)
    pub lambda_median: f64,
    pub lambda_mad: f64,
    
    // Phi wraps (99th percentile for outlier threshold)
    pub wraps_p99: f64,
    
    // Turning points (99th percentile)
    pub turns_r_p99: u8,
    pub turns_theta_p99: u8,
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
            ni_log_median: 0.0,
            ni_log_mad: 0.0,
            lambda_median: 0.0,
            lambda_mad: 0.0,
            wraps_p99: 0.0,
            turns_r_p99: 0,
            turns_theta_p99: 0,
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
        ni_log_median: ni_log_stats.median_log,
        ni_log_mad: ni_log_stats.mad_log,
        lambda_median,
        lambda_mad,
        wraps_p99,
        turns_r_p99,
        turns_theta_p99,
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

/// Check if value is finite (not NaN or infinite)
fn is_finite(value: f64) -> bool {
    value.is_finite()
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
        // Note: count field was removed from LogStats
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

// ============================================================================
// TIER 4: ENHANCED OUTLIER DETECTION
// ============================================================================

/// Outlier severity levels (highest to lowest priority)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum OutlierSeverity {
    Critical = 3,  // Physics violations - possible integration bugs
    High = 2,      // Statistical anomalies - rare but valid edge cases
    Medium = 1,    // Interesting edge cases - expected in complex geodesics
    Info = 0,      // Informational - roundoff at machine precision
}

impl OutlierSeverity {
    /// Get human-readable label
    pub fn label(&self) -> &'static str {
        match self {
            OutlierSeverity::Critical => "Critical",
            OutlierSeverity::High => "High",
            OutlierSeverity::Medium => "Medium",
            OutlierSeverity::Info => "Info",
        }
    }
    
    /// Get color for visualization
    pub fn color(&self) -> &'static str {
        match self {
            OutlierSeverity::Critical => "#DC2626", // red-600
            OutlierSeverity::High => "#EA580C",     // orange-600
            OutlierSeverity::Medium => "#CA8A04",   // yellow-600
            OutlierSeverity::Info => "#9CA3AF",     // gray-400
        }
    }
}

/// Comprehensive outlier taxonomy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutlierCategory {
    // CRITICAL: Physics violations (possible bugs)
    NegativeK,          // Carter constant K < -1e-12 (impossible)
    LargeNullInvariant, // NI > 1e-9 (severe integration error)
    InvalidState,       // NaN or ±∞ in any field
    InvalidRadius,      // r < r_horizon or r > 1000M at disc hit
    InvalidTheta,       // θ ∉ [0, π] at disc hit
    
    // HIGH: Statistical anomalies (rare but valid)
    ExtremeNI,              // NI log z-score > 3.5 AND top 0.1%
    SpatialDiscontinuity,   // Local MAD z-score > 6 from 8-neighbors
    
    // MEDIUM: Interesting edge cases
    ExtremeAffine,       // Affine parameter MAD z > 3.5 AND λ > 100M (highly bent trajectories)
    ExtremeWraps,        // φ-wraps > p99 for that order
    RapidTurningPoints,  // turns_r or turns_theta > 6
    
    // INFO: Diagnostics & edge cases (informational only)
    PhaseSpaceEdgeCase,     // Mahalanobis distance > 5.0 (rare but valid combination)
    SpatialGradient,        // MAD z > 6 but relative change < 5% (smooth gradient artifact)
    LongAffine,             // Affine parameter MAD z > 3.5 but λ < 100M (outer disc edge)
    RoundoffK,              // |K| < 1e-12 (perfect conservation at machine precision)
}

impl OutlierCategory {
    /// Get severity level for this category
    pub fn severity(&self) -> OutlierSeverity {
        match self {
            OutlierCategory::NegativeK
            | OutlierCategory::LargeNullInvariant
            | OutlierCategory::InvalidState
            | OutlierCategory::InvalidRadius
            | OutlierCategory::InvalidTheta => OutlierSeverity::Critical,
            
            OutlierCategory::ExtremeNI
            | OutlierCategory::SpatialDiscontinuity => OutlierSeverity::High,
            
            OutlierCategory::ExtremeAffine
            | OutlierCategory::ExtremeWraps
            | OutlierCategory::RapidTurningPoints => OutlierSeverity::Medium,
            
            OutlierCategory::PhaseSpaceEdgeCase
            | OutlierCategory::SpatialGradient
            | OutlierCategory::LongAffine
            | OutlierCategory::RoundoffK => OutlierSeverity::Info,
        }
    }
    
    /// Get human-readable label
    pub fn label(&self) -> &'static str {
        match self {
            OutlierCategory::NegativeK => "Negative Carter Constant",
            OutlierCategory::LargeNullInvariant => "Large Null Invariant Error",
            OutlierCategory::InvalidState => "Invalid State (NaN/∞)",
            OutlierCategory::InvalidRadius => "Invalid Radius",
            OutlierCategory::InvalidTheta => "Invalid Theta",
            OutlierCategory::ExtremeNI => "Extreme NI (Statistical)",
            OutlierCategory::SpatialDiscontinuity => "Spatial Discontinuity",
            OutlierCategory::ExtremeAffine => "Extreme Affine Parameter",
            OutlierCategory::ExtremeWraps => "Extreme φ-Wraps",
            OutlierCategory::RapidTurningPoints => "Rapid Turning Points",
            OutlierCategory::PhaseSpaceEdgeCase => "Phase-Space Edge Case",
            OutlierCategory::SpatialGradient => "Smooth Spatial Gradient",
            OutlierCategory::LongAffine => "Long Affine Parameter (Outer Disc)",
            OutlierCategory::RoundoffK => "K Roundoff (Machine Precision)",
        }
    }
}

/// Detected outlier with full context
#[derive(Debug, Clone)]
pub struct Outlier {
    pub pixel_x: u32,
    pub pixel_y: u32,
    pub order: u8,
    pub category: OutlierCategory,
    pub severity: OutlierSeverity,
    pub value: f64,     // The outlying value
    pub details: String, // Additional context
}

impl Outlier {
    /// Create a new outlier record
    pub fn new(
        pixel_x: u32,
        pixel_y: u32,
        order: u8,
        category: OutlierCategory,
        value: f64,
        details: String,
    ) -> Self {
        Self {
            pixel_x,
            pixel_y,
            order,
            severity: category.severity(),
            category,
            value,
            details,
        }
    }
}

// ============================================================================
// OUTLIER DETECTION: Extended GeodesicRecord trait
// ============================================================================

/// Extended trait for geodesic records used in outlier detection
pub trait OutlierDetectionRecord: GeodesicRecord {
    fn pixel_x(&self) -> u32;
    fn pixel_y(&self) -> u32;
    fn r(&self) -> f64;
    fn theta(&self) -> f64;
    fn energy(&self) -> f64;
    fn angular_momentum(&self) -> f64;
    fn carter_q(&self) -> f64;
}

// ============================================================================
// PASS 1: PHYSICAL VIOLATIONS (CRITICAL)
// ============================================================================

/// Detect critical physics violations
/// 
/// These represent impossible states that indicate bugs or severe numerical errors.
pub fn detect_physical_outliers<T: OutlierDetectionRecord>(
    records: &[T],
    spin: f64,
) -> Vec<Outlier> {
    let mut outliers = Vec::new();
    
    // Compute horizon radius
    let r_horizon = 1.0 + (1.0 - spin * spin).sqrt();
    
    for rec in records {
        let px = rec.pixel_x();
        let py = rec.pixel_y();
        let order = rec.order();
        
        // Check 1: Carter constant (K = Q + (L_z - aE)²)
        let energy = rec.energy();
        let l_z = rec.angular_momentum();
        let q = rec.carter_q();
        let k_carter = q + (l_z - spin * energy).powi(2);
        
        if k_carter < -1e-12 {
            outliers.push(Outlier::new(
                px, py, order,
                OutlierCategory::NegativeK,
                k_carter,
                format!("K={:.3e} (physically impossible)", k_carter),
            ));
        }
        
        // Check 2: Large null invariant (severe integration error)
        let ni = rec.null_invariant_error();
        if ni > 1e-9 {
            outliers.push(Outlier::new(
                px, py, order,
                OutlierCategory::LargeNullInvariant,
                ni,
                format!("NI={:.3e} (integration accuracy issue)", ni),
            ));
        }
        
        // Check 3: Invalid state (NaN or ±∞)
        let r = rec.r();
        let theta = rec.theta();
        let lambda = rec.affine_parameter();
        let g = rec.redshift_factor();
        
        if !is_finite(r) || !is_finite(theta) || !is_finite(lambda) || 
           !is_finite(g) || !is_finite(energy) || !is_finite(l_z) || !is_finite(q) {
            outliers.push(Outlier::new(
                px, py, order,
                OutlierCategory::InvalidState,
                f64::NAN,
                "NaN or ±∞ detected in geodesic state".to_string(),
            ));
        }
        
        // Check 4: Invalid radius
        if r < r_horizon {
            outliers.push(Outlier::new(
                px, py, order,
                OutlierCategory::InvalidRadius,
                r,
                format!("r={:.3}M < r_h={:.3}M", r, r_horizon),
            ));
        } else if r > 1000.0 {
            outliers.push(Outlier::new(
                px, py, order,
                OutlierCategory::InvalidRadius,
                r,
                format!("r={:.1}M > 1000M (escaped)", r),
            ));
        }
        
        // Check 5: Invalid theta (should be in [0, π])
        if theta < 0.0 || theta > std::f64::consts::PI {
            outliers.push(Outlier::new(
                px, py, order,
                OutlierCategory::InvalidTheta,
                theta,
                format!("θ={:.3} ∉ [0, π]", theta),
            ));
        }
    }
    
    outliers
}

// ============================================================================
// PASS 2A: STATISTICAL OUTLIERS (HIGH/MEDIUM)
// ============================================================================

/// Detect statistical outliers using per-order thresholds
/// 
/// Affine parameter uses dual classification:
/// - λ > 100M AND z > 3.5 → MEDIUM (highly bent trajectories, ~717 cases)
/// - λ < 100M AND z > 3.5 → INFO (outer disc edge, ~596 cases)
pub fn detect_statistical_outliers<T: OutlierDetectionRecord>(
    records: &[T],
    order_stats: &HashMap<u8, OrderStats>,
) -> Vec<Outlier> {
    let mut outliers = Vec::new();
    
    // Build sorted NI list for percentile calculation
    let mut ni_by_order: HashMap<u8, Vec<(usize, f64)>> = HashMap::new();
    for (idx, rec) in records.iter().enumerate() {
        let order = rec.order();
        ni_by_order.entry(order)
            .or_insert_with(Vec::new)
            .push((idx, rec.null_invariant_error()));
    }
    
    // Sort each order's NI values
    for values in ni_by_order.values_mut() {
        values.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    }
    
    for (idx, rec) in records.iter().enumerate() {
        let px = rec.pixel_x();
        let py = rec.pixel_y();
        let order = rec.order();
        
        let stats = match order_stats.get(&order) {
            Some(s) => s,
            None => continue, // Skip if no stats for this order
        };
        
        // Check 1: Extreme NI (log z-score > 3.5 AND top 0.1%)
        let ni = rec.null_invariant_error();
        if ni > 0.0 {
            let log_ni = ni.log10();
            let log_z = mad_zscore(log_ni, stats.ni_log_median, stats.ni_log_mad);
            
            // Check if in top 0.1%
            if let Some(order_list) = ni_by_order.get(&order) {
                let top_0_1_pct_idx = (order_list.len() as f64 * 0.001) as usize;
                let is_top_0_1_pct = order_list.iter()
                    .take(top_0_1_pct_idx.max(1))
                    .any(|(i, _)| *i == idx);
                
                if log_z.abs() > 3.5 && is_top_0_1_pct {
                    outliers.push(Outlier::new(
                        px, py, order,
                        OutlierCategory::ExtremeNI,
                        ni,
                        format!("NI={:.3e}, log z-score={:.1}, top 0.1%", ni, log_z),
                    ));
                }
            }
        }
        
        // Check 2: Extreme affine parameter (MAD z-score > 3.5)
        let lambda = rec.affine_parameter();
        let lambda_z = mad_zscore(lambda, stats.lambda_median, stats.lambda_mad);
        if lambda_z.abs() > 3.5 {
            if lambda > 100.0 {
                // Highly bent trajectories (λ > 100M)
                outliers.push(Outlier::new(
                    px, py, order,
                    OutlierCategory::ExtremeAffine,
                    lambda,
                    format!("λ={:.1}M (MAD z={:.1}, highly bent trajectory)", lambda, lambda_z),
                ));
            } else {
                // Outer disc edge (80M < λ < 100M)
                outliers.push(Outlier::new(
                    px, py, order,
                    OutlierCategory::LongAffine,
                    lambda,
                    format!("λ={:.1}M (MAD z={:.1}, outer disc edge)", lambda, lambda_z),
                ));
            }
        }
        
        // Check 3: Extreme wraps (> p99 for this order)
        let wraps = rec.phi_wraps();
        if wraps > stats.wraps_p99 && stats.wraps_p99 > 0.0 {
            outliers.push(Outlier::new(
                px, py, order,
                OutlierCategory::ExtremeWraps,
                wraps,
                format!("φ-wraps={:.2} > p99={:.2}", wraps, stats.wraps_p99),
            ));
        }
        
        // Check 4: Rapid turning points (> p99 for this order)
        let turns_r = rec.turns_r();
        let turns_theta = rec.turns_theta();
        if turns_r > stats.turns_r_p99 || turns_theta > stats.turns_theta_p99 {
            outliers.push(Outlier::new(
                px, py, order,
                OutlierCategory::RapidTurningPoints,
                (turns_r.max(turns_theta)) as f64,
                format!("turns_r={}, turns_θ={} (p99: r={}, θ={})", turns_r, turns_theta, stats.turns_r_p99, stats.turns_theta_p99),
            ));
        }
    }
    
    outliers
}

// ============================================================================
// PASS 2B: SPATIAL DISCONTINUITY DETECTION (HIGH) & GRADIENT ARTIFACTS (INFO)
// ============================================================================

/// Detect spatial discontinuities using 8-neighbor comparison
/// 
/// Uses dual-threshold classification:
/// - MAD z-score > 6.0 AND relative change > 5% → HIGH severity (true caustic/boundary)
/// - MAD z-score > 6.0 AND relative change ≤ 5% → INFO severity (smooth gradient artifact)
/// 
/// This separates genuine discontinuities (photons hitting wildly different disc locations)
/// from statistical artifacts (smooth gradients with low local variance).
pub fn detect_spatial_outliers<T: OutlierDetectionRecord>(
    records: &[T],
    index_grid: &IndexGrid,
) -> Vec<Outlier> {
    let mut outliers = Vec::new();
    
    for rec in records {
        let px = rec.pixel_x();
        let py = rec.pixel_y();
        let order = rec.order();
        
        // Get 8-neighbors with same order
        let neighbor_indices = index_grid.get_neighbors_8_same_order(px, py, order, records);
        
        if neighbor_indices.len() < 3 {
            continue; // Need at least 3 neighbors for robust statistics
        }
        
        // Collect neighbor values for key metrics
        let neighbor_ni: Vec<f64> = neighbor_indices.iter()
            .map(|&i| records[i].null_invariant_error().max(1e-16).log10())
            .collect();
        
        let neighbor_lambda: Vec<f64> = neighbor_indices.iter()
            .map(|&i| records[i].affine_parameter())
            .collect();
        
        let neighbor_g: Vec<f64> = neighbor_indices.iter()
            .map(|&i| records[i].redshift_factor())
            .collect();
        
        // Compute local MAD for each metric
        let (ni_median, ni_mad) = compute_mad(&neighbor_ni);
        let (lambda_median, lambda_mad) = compute_mad(&neighbor_lambda);
        let (g_median, g_mad) = compute_mad(&neighbor_g);
        
        // Check discontinuity in each metric
        let rec_ni_log = rec.null_invariant_error().max(1e-16).log10();
        let rec_lambda = rec.affine_parameter();
        let rec_g = rec.redshift_factor();
        
        let ni_z = mad_zscore(rec_ni_log, ni_median, ni_mad);
        let lambda_z = mad_zscore(rec_lambda, lambda_median, lambda_mad);
        let g_z = mad_zscore(rec_g, g_median, g_mad);
        
        // Flag if any metric has z-score > 6
        let max_z = ni_z.abs().max(lambda_z.abs()).max(g_z.abs());
        if max_z > 6.0 {
            // Determine which metric triggered and compute relative change
            let (metric, rec_value, neighbor_median) = if ni_z.abs() == max_z {
                ("NI", rec_ni_log, ni_median)
            } else if lambda_z.abs() == max_z {
                ("λ", rec_lambda, lambda_median)
            } else {
                ("g", rec_g, g_median)
            };
            
            // Compute relative change (for non-zero medians)
            let relative_change = if neighbor_median.abs() > 1e-10 {
                (rec_value - neighbor_median).abs() / neighbor_median.abs()
            } else {
                // If median is ~0, use absolute difference as proxy
                (rec_value - neighbor_median).abs()
            };
            
            // Classify based on relative change threshold
            if relative_change > 0.05 {
                // TRUE discontinuity: >5% change (likely caustic/boundary)
                outliers.push(Outlier::new(
                    px, py, order,
                    OutlierCategory::SpatialDiscontinuity,
                    max_z,
                    format!("{} differs from neighbors (local MAD z={:.1}, Δ={:.1}%)", 
                            metric, max_z, relative_change * 100.0),
                ));
            } else {
                // Smooth gradient artifact: high z-score but small absolute change
                outliers.push(Outlier::new(
                    px, py, order,
                    OutlierCategory::SpatialGradient,
                    max_z,
                    format!("{} smooth gradient (local MAD z={:.1}, Δ={:.2}%)", 
                            metric, max_z, relative_change * 100.0),
                ));
            }
        }
    }
    
    outliers
}

// ============================================================================
// INFO: PHASE-SPACE EDGE CASES (Mahalanobis Distance)
// ============================================================================
// NOTE: Actual implementation is in mahalanobis.rs module
// This is just a wrapper for integration into the detection pipeline

/// Detect phase-space edge cases using proper Mahalanobis distance
/// 
/// Flags geodesics with statistically rare (but physically valid) combinations
/// in 5D phase space (E, Lz, Q, λ, log NI).
/// 
/// Uses the new mahalanobis module with true covariance-based distance.
/// Threshold: D > 5.0 (between 99.9% and 99.99% for χ²(5) distribution)
/// 
/// Marked as INFO severity - these are NOT errors, just unusual configurations.
pub fn detect_phase_space_edge_cases<T: OutlierDetectionRecord>(
    records: &[T],
) -> Vec<Outlier> {
    // Compute Mahalanobis statistics for each order
    let order_stats = crate::mahalanobis::compute_all_order_stats(records);
    
    // Detect edge cases using threshold of 5.0 (INFO severity)
    crate::mahalanobis::detect_outliers(records, &order_stats, 5.0)
}

// ============================================================================
// INFO: ROUNDOFF K DETECTION
// ============================================================================

/// Detect excellent K conservation (informational only)
pub fn detect_roundoff_k<T: OutlierDetectionRecord>(
    records: &[T],
    spin: f64,
) -> Vec<Outlier> {
    let mut outliers = Vec::new();
    
    for rec in records {
        let energy = rec.energy();
        let l_z = rec.angular_momentum();
        let q = rec.carter_q();
        let k_carter = q + (l_z - spin * energy).powi(2);
        
        if k_carter.abs() < 1e-12 && k_carter >= 0.0 {
            outliers.push(Outlier::new(
                rec.pixel_x(),
                rec.pixel_y(),
                rec.order(),
                OutlierCategory::RoundoffK,
                k_carter,
                format!("K={:.3e} (machine precision)", k_carter),
            ));
        }
    }
    
    outliers
}

// ============================================================================
// MAIN DETECTION PIPELINE
// ============================================================================

/// Detect all outliers using 2-pass pipeline
/// 
/// Returns sorted list: Critical → High → Medium → Info, then by pixel
pub fn detect_outliers<T: OutlierDetectionRecord>(
    records: &[T],
    spin: f64,
    order_stats: &HashMap<u8, OrderStats>,
    index_grid: &IndexGrid,
) -> Vec<Outlier> {
    let mut all_outliers = Vec::new();
    
    // PASS 1: Critical violations (short-circuit)
    all_outliers.extend(detect_physical_outliers(records, spin));
    
    // PASS 2: Statistical & spatial (order-aware)
    all_outliers.extend(detect_statistical_outliers(records, order_stats));
    all_outliers.extend(detect_spatial_outliers(records, index_grid));
    
    // INFO: Phase-space edge cases & roundoff (informational only)
    all_outliers.extend(detect_phase_space_edge_cases(records));
    all_outliers.extend(detect_roundoff_k(records, spin));
    
    // Sort by severity (desc) then pixel (asc)
    all_outliers.sort_by(|a, b| {
        b.severity.cmp(&a.severity)
            .then_with(|| a.pixel_x.cmp(&b.pixel_x))
            .then_with(|| a.pixel_y.cmp(&b.pixel_y))
            .then_with(|| a.order.cmp(&b.order))
    });
    
    all_outliers
}

