// Proper Mahalanobis Distance Computation for Multivariate Outlier Detection
//
// Implements true Mahalanobis distance in 5D phase space:
// - Energy (E)
// - Angular momentum (Lz)
// - Carter constant (Q)
// - Affine parameter (λ)
// - log₁₀(Null Invariant)
//
// Uses proper covariance matrix computation and inversion.

use super::stats::{Outlier, OutlierCategory, OutlierDetectionRecord};
use std::collections::HashMap;

// ============================================================================
// DATA STRUCTURES
// ============================================================================

/// Statistics for Mahalanobis distance computation (computed per geodesic order)
#[derive(Debug, Clone)]
pub struct MahalanobisStats {
    pub mean: [f64; 5],           // Mean vector [E, Lz, Q, λ, log_NI]
    pub cov_inv: [[f64; 5]; 5],   // Inverse covariance matrix (5×5)
}

// ============================================================================
// COVARIANCE MATRIX COMPUTATION
// ============================================================================

/// Compute mean vector for 5D phase space
fn compute_mean<T: OutlierDetectionRecord>(records: &[&T]) -> [f64; 5] {
    let n = records.len() as f64;
    let mut sum = [0.0; 5];
    
    for rec in records {
        sum[0] += rec.energy();
        sum[1] += rec.angular_momentum();
        sum[2] += rec.carter_q();
        sum[3] += rec.affine_parameter();
        sum[4] += rec.null_invariant_error().max(1e-16).log10();
    }
    
    [
        sum[0] / n,
        sum[1] / n,
        sum[2] / n,
        sum[3] / n,
        sum[4] / n,
    ]
}

/// Compute 5×5 covariance matrix
fn compute_covariance<T: OutlierDetectionRecord>(
    records: &[&T],
    mean: &[f64; 5],
) -> [[f64; 5]; 5] {
    let n = records.len() as f64;
    let mut cov = [[0.0; 5]; 5];
    
    // Compute covariance: Cov(i,j) = E[(Xi - μi)(Xj - μj)]
    for rec in records {
        let x = [
            rec.energy() - mean[0],
            rec.angular_momentum() - mean[1],
            rec.carter_q() - mean[2],
            rec.affine_parameter() - mean[3],
            rec.null_invariant_error().max(1e-16).log10() - mean[4],
        ];
        
        for i in 0..5 {
            for j in 0..5 {
                cov[i][j] += x[i] * x[j];
            }
        }
    }
    
    // Normalize by (n-1) for unbiased estimator
    let norm = (n - 1.0).max(1.0);
    for i in 0..5 {
        for j in 0..5 {
            cov[i][j] /= norm;
        }
    }
    
    cov
}

// ============================================================================
// MATRIX INVERSION (5×5 via LU Decomposition)
// ============================================================================

/// Invert a 5×5 matrix using LU decomposition with partial pivoting
/// 
/// Returns None if matrix is singular (determinant ≈ 0)
fn invert_5x5(matrix: &[[f64; 5]; 5]) -> Option<[[f64; 5]; 5]> {
    // Create augmented matrix [A | I]
    let mut aug = [[0.0; 10]; 5];
    for i in 0..5 {
        for j in 0..5 {
            aug[i][j] = matrix[i][j];
        }
        aug[i][i + 5] = 1.0; // Identity on right side
    }
    
    // Forward elimination with partial pivoting
    for col in 0..5 {
        // Find pivot (largest absolute value in column)
        let mut max_row = col;
        let mut max_val = aug[col][col].abs();
        for row in (col + 1)..5 {
            let val = aug[row][col].abs();
            if val > max_val {
                max_val = val;
                max_row = row;
            }
        }
        
        // Check for singularity
        if max_val < 1e-12 {
            return None; // Matrix is singular
        }
        
        // Swap rows if needed
        if max_row != col {
            for j in 0..10 {
                let tmp = aug[col][j];
                aug[col][j] = aug[max_row][j];
                aug[max_row][j] = tmp;
            }
        }
        
        // Eliminate column below pivot
        let pivot = aug[col][col];
        for row in (col + 1)..5 {
            let factor = aug[row][col] / pivot;
            for j in col..10 {
                aug[row][j] -= factor * aug[col][j];
            }
        }
    }
    
    // Back substitution
    for col in (0..5).rev() {
        let pivot = aug[col][col];
        
        // Normalize pivot row
        for j in col..10 {
            aug[col][j] /= pivot;
        }
        
        // Eliminate column above pivot
        for row in 0..col {
            let factor = aug[row][col];
            for j in col..10 {
                aug[row][j] -= factor * aug[col][j];
            }
        }
    }
    
    // Extract inverse from right side of augmented matrix
    let mut inv = [[0.0; 5]; 5];
    for i in 0..5 {
        for j in 0..5 {
            inv[i][j] = aug[i][j + 5];
        }
    }
    
    Some(inv)
}

// ============================================================================
// MAHALANOBIS DISTANCE COMPUTATION
// ============================================================================

/// Compute Mahalanobis statistics for a set of records
/// 
/// Returns None if insufficient data (< 10 samples) or covariance is singular
pub fn compute_mahalanobis_stats<T: OutlierDetectionRecord>(
    records: &[&T],
) -> Option<MahalanobisStats> {
    // Need at least 10 samples for reliable covariance estimation
    if records.len() < 10 {
        return None;
    }
    
    // Compute mean vector
    let mean = compute_mean(records);
    
    // Compute covariance matrix
    let cov = compute_covariance(records, &mean);
    
    // Invert covariance matrix
    let cov_inv = invert_5x5(&cov)?;
    
    Some(MahalanobisStats {
        mean,
        cov_inv,
    })
}

/// Compute true Mahalanobis distance for a single record
/// 
/// Formula: D² = (x - μ)ᵀ Σ⁻¹ (x - μ)
pub fn compute_distance<T: OutlierDetectionRecord>(
    record: &T,
    stats: &MahalanobisStats,
) -> f64 {
    // Extract feature vector
    let x = [
        record.energy(),
        record.angular_momentum(),
        record.carter_q(),
        record.affine_parameter(),
        record.null_invariant_error().max(1e-16).log10(),
    ];
    
    // Compute deviation: (x - μ)
    let mut dev = [0.0; 5];
    for i in 0..5 {
        dev[i] = x[i] - stats.mean[i];
    }
    
    // Compute Σ⁻¹ * (x - μ)
    let mut temp = [0.0; 5];
    for i in 0..5 {
        for j in 0..5 {
            temp[i] += stats.cov_inv[i][j] * dev[j];
        }
    }
    
    // Compute (x - μ)ᵀ * temp = D²
    let mut d_squared = 0.0;
    for i in 0..5 {
        d_squared += dev[i] * temp[i];
    }
    
    // Return D (not D²)
    d_squared.max(0.0).sqrt()
}

// ============================================================================
// OUTLIER DETECTION
// ============================================================================

/// Detect phase-space edge cases using proper Mahalanobis distance
/// 
/// Flags geodesics with unusual (but valid) combinations in 5D phase space.
/// These are NOT errors - just statistically rare configurations.
/// 
/// Threshold selection:
/// - For 5D Gaussian data, Mahalanobis distance² follows χ²(5) distribution
/// - χ²(5, 0.999) ≈ 20.5 → D ≈ 4.5 (99.9th percentile)
/// - χ²(5, 0.9999) ≈ 26.1 → D ≈ 5.1 (99.99th percentile)
/// 
/// We use D > 5.0 as threshold (between 99.9% and 99.99%)
/// Marked as INFO severity since these are rare but physically valid.
pub fn detect_outliers<T: OutlierDetectionRecord>(
    records: &[T],
    order_stats_map: &HashMap<u8, MahalanobisStats>,
    threshold: f64,
) -> Vec<Outlier> {
    let mut outliers = Vec::new();
    
    for rec in records {
        let order = rec.order();
        
        // Get stats for this order
        let stats = match order_stats_map.get(&order) {
            Some(s) => s,
            None => continue, // No stats for this order
        };
        
        // Compute Mahalanobis distance
        let d_m = compute_distance(rec, stats);
        
        // Flag if exceeds threshold (INFO severity - rare but valid)
        if d_m > threshold {
            outliers.push(Outlier::new(
                rec.pixel_x(),
                rec.pixel_y(),
                order,
                OutlierCategory::PhaseSpaceEdgeCase,
                d_m,
                format!("Mahalanobis distance={:.1} in 5D phase space (E,Lz,Q,λ,logNI)", d_m),
            ));
        }
    }
    
    outliers
}

/// Compute Mahalanobis statistics for all orders
pub fn compute_all_order_stats<T: OutlierDetectionRecord>(
    records: &[T],
) -> HashMap<u8, MahalanobisStats> {
    // Group records by order
    let mut by_order: HashMap<u8, Vec<&T>> = HashMap::new();
    for rec in records {
        by_order.entry(rec.order())
            .or_insert_with(Vec::new)
            .push(rec);
    }
    
    // Compute stats for each order
    by_order.into_iter()
        .filter_map(|(order, order_records)| {
            compute_mahalanobis_stats(&order_records)
                .map(|stats| (order, stats))
        })
        .collect()
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_invert_identity() {
        let identity = [
            [1.0, 0.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 0.0, 1.0],
        ];
        
        let inv = invert_5x5(&identity).expect("Should invert identity");
        
        // Check result is identity
        for i in 0..5 {
            for j in 0..5 {
                let expected = if i == j { 1.0 } else { 0.0 };
                assert!((inv[i][j] - expected).abs() < 1e-10);
            }
        }
    }
    
    #[test]
    fn test_invert_diagonal() {
        let diag = [
            [2.0, 0.0, 0.0, 0.0, 0.0],
            [0.0, 3.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 4.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 5.0, 0.0],
            [0.0, 0.0, 0.0, 0.0, 6.0],
        ];
        
        let inv = invert_5x5(&diag).expect("Should invert diagonal");
        
        // Check diagonal elements are reciprocals
        assert!((inv[0][0] - 0.5).abs() < 1e-10);
        assert!((inv[1][1] - 1.0/3.0).abs() < 1e-10);
        assert!((inv[2][2] - 0.25).abs() < 1e-10);
        assert!((inv[3][3] - 0.2).abs() < 1e-10);
        assert!((inv[4][4] - 1.0/6.0).abs() < 1e-10);
    }
    
    #[test]
    fn test_singular_matrix() {
        let singular = [
            [1.0, 2.0, 3.0, 4.0, 5.0],
            [2.0, 4.0, 6.0, 8.0, 10.0], // 2× first row
            [0.0, 0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 0.0, 1.0],
        ];
        
        let result = invert_5x5(&singular);
        assert!(result.is_none(), "Should not invert singular matrix");
    }
}

