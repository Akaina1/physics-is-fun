# Kerr Black Hole Analyzer: Enhanced Diagnostics & Outlier Detection Plan

## Project Goals

Enhance the high-precision geodesic data analyzer with comprehensive diagnostic tools to:

1. **Explain the high miss rate** through detailed miss taxonomy (escaped/captured/aborted)
2. **Validate numerical accuracy** with robust outlier detection and quality metrics
3. **Provide research-grade analysis** suitable for academic papers and technical blogs
4. **Visualize complex relativistic phenomena** (lensing, frame-dragging, caustics)

---

## Implementation Tiers

### Tier 1: Quick Wins (2 hours)

Core diagnostic features with immediate value

### Tier 2: Research Metrics (4 hours)

Advanced visualizations for deeper analysis

### Tier 3: Publication Quality (4.5 hours)

Academic-grade features for papers and presentations

### Enhanced: Robust Outlier Detection (8 hours)

Research-grade statistical validation integrated throughout all tiers

**Total Estimated Time:** ~18.5 hours across multiple sessions

---

## 🎯 **ADDITIONAL DATA POINTS TO ADD**:

**Tier 1 - Do these now (< 1 hour total):**

1. Hit taxonomy (escaped/captured/aborted) + pie chart
2. Order mask thumbnails
3. Outlier spotlight table
4. Provenance block

**Tier 2 - Next session (1-2 hours):** 5. K validation heatmap 6. Transfer function 2D histogram 7. Asymmetry quantification 8. Time delay map

**Tier 3 - Research paper quality (2-4 hours):** 9. Critical curve extraction 10. Turning-point histograms 11. Wrap-angle vs b plot

**Tier 4 - future additions if time permits:**

- Magnification/caustics (complex, need neighbor data)

---

## Phase 1: Data Collection (Physics Layer)

**Location:** `kernels/kerr_black_hole/src/`

### 1.1 Track Turning Points (`integration.rs`)

Already detecting sign flips in radial and polar motion; need to count and export them:

```rust
// In integrate_geodesic_multi_order
let mut turns_r = 0u8;
let mut turns_theta = 0u8;

// When potential crosses zero (already detecting):
if r_pot_before > 0.0 && r_pot_after == 0.0 {
    turns_r += 1;
}
if th_pot_before > 0.0 && th_pot_after == 0.0 {
    turns_theta += 1;
}

// Return with hit result
```

### 1.2 Classify Miss Reasons (`integration.rs`)

Determine why rays don't hit the disc:

```rust
enum MissReason {
    Escaped,    // r → ∞ (> 1000M)
    Captured,   // r → r_horizon (< r_h + 0.01M)
    Aborted,    // Numerical failure (NaN, step limit)
}

// At end of integration loop:
let (escaped, captured, aborted) = if !hit {
    if state.r > 1000.0 {
        (true, false, false)
    } else if state.r < r_horizon + 0.01 {
        (false, true, false)
    } else {
        (false, false, true)
    }
} else {
    (false, false, false)
};
```

### 1.3 Expand Data Structure (`transfer_maps.rs`)

Add new fields to `PositionData`:

```rust
pub struct PositionData {
    // ... existing 15 fields ...

    // NEW: Miss classification
    pub escaped: bool,
    pub captured: bool,
    pub aborted: bool,

    // NEW: Geodesic complexity
    pub turns_r: u8,
    pub turns_theta: u8,
}
```

**JSON Impact:** ~10 bytes/record → +5MB for 1080p (acceptable)

Update `to_json()` and `pack_pixel()` functions to include new fields.

---

## Phase 2: Statistical Infrastructure

**Location:** `kernels/kerr_black_hole/src/bin/analyze/stats.rs` (NEW FILE)

### 2.1 Robust Statistics Functions

```rust
/// Median Absolute Deviation (more robust than IQR for heavy-tailed data)
pub fn compute_mad(values: &[f64]) -> (f64, f64) {
    let median = compute_median(values);
    let deviations: Vec<f64> = values.iter()
        .map(|v| (v - median).abs())
        .collect();
    let mad = compute_median(&deviations);
    (median, mad)
}

/// MAD-based z-score (handles outliers better than standard deviation)
pub fn mad_zscore(value: f64, median: f64, mad: f64) -> f64 {
    const MAD_SCALE: f64 = 1.4826; // Consistency constant for normal dist
    (value - median) / (MAD_SCALE * mad)
}

/// Log-space statistics for null invariant (heavy-tailed)
pub fn compute_log_stats(values: &[f64]) -> LogStats {
    let log_values: Vec<f64> = values.iter()
        .filter(|v| **v > 0.0)
        .map(|v| v.log10())
        .collect();

    let (median, mad) = compute_mad(&log_values);
    // Returns statistics in log space
}
```

### 2.2 Per-Order Statistics

Critical for avoiding false positives (different physics per order):

```rust
pub struct OrderStats {
    pub ni_log_median: f64,    // Null invariant (log scale)
    pub ni_log_mad: f64,
    pub lambda_median: f64,    // Affine parameter
    pub lambda_mad: f64,
    pub g_median: f64,         // Redshift
    pub g_mad: f64,
    pub wraps_p99: f64,        // φ-wraps 99th percentile
    pub turns_r_p99: u8,       // Turning points
    pub turns_theta_p99: u8,
}

pub fn compute_order_stats(hits: &[&HpRecord], order: u8) -> OrderStats {
    // Filter to specific order, compute MAD for each metric
    // Use log space for NI, linear for others
}
```

### 2.3 Spatial Index Grid

For efficient neighbor lookups (O(1) instead of O(n)):

```rust
pub struct IndexGrid {
    grid: Vec<Vec<Option<usize>>>,  // Maps (x,y) → index in positions array
    width: usize,
    height: usize,
}

impl IndexGrid {
    pub fn from_hp_data(data: &HpData, width: usize, height: usize) -> Self;

    /// Get neighbors of same order (4-neighbor for edges, 8-neighbor interior)
    pub fn get_neighbors_same_order(&self, x: usize, y: usize, order: u8, data: &HpData) -> Vec<usize>;
}
```

---

## Phase 3: Tier 1 Implementation (2 hours)

**Location:** `kernels/kerr_black_hole/src/bin/analyze/`

### 3.1 Hit Taxonomy with Pie Chart (30 min)

Update `compute_statistics()` in `main.rs`:

```rust
// Count miss reasons
let escaped_count = hp_data.positions.iter()
    .filter(|p| !p.hit && p.escaped).count();
let captured_count = hp_data.positions.iter()
    .filter(|p| !p.hit && p.captured).count();
let aborted_count = hp_data.positions.iter()
    .filter(|p| !p.hit && p.aborted).count();

// Add to Stats struct
pub miss_escaped: usize,
pub miss_captured: usize,
pub miss_aborted: usize,
```

Create pie chart in `charts.rs`:

```rust
pub fn generate_miss_taxonomy_pie(
    escaped: usize,
    captured: usize,
    aborted: usize,
    total: usize
) -> String {
    // SVG pie chart with 3 segments
    // Colors: blue (escaped), red (captured), gray (aborted)
    // Include percentages in labels
}
```

### 3.2 Order Mask Thumbnails (45 min)

Generate small binary images showing which pixels hit at each order:

```rust
pub fn generate_order_thumbnail_svg(
    width: u32,  // e.g., 200 (downsampled from 1920)
    height: u32, // e.g., 112 (downsampled from 1080)
    pixel_orders: &[Option<u8>],  // Flattened grid
    target_order: u8,
) -> String {
    // Downsample: bin pixels into larger blocks
    // Render as SVG rects: white if order matches, black otherwise
    // Add border, title
}
```

Create 3 thumbnails (order 0, 1, 2+) side-by-side in HTML.

### 3.3 Enhanced Outlier Spotlight (2 hours)

**Now includes robust detection, not just top-N sorting.**

#### Outlier Categories:

```rust
pub enum OutlierCategory {
    // CRITICAL (physics violations)
    NegativeK,              // K < -1e-12
    LargeNullInvariant,     // NI > 1e-9
    InvalidState,           // NaN, ±∞
    InvalidRadius,          // r out of bounds
    InvalidTheta,           // θ ∉ [0,π]

    // HIGH (statistical anomalies, top 0.1%)
    ExtremeNI,              // NI log z-score > 3.5 AND top 0.1%
    MahalanobisOutlier,     // D_M > 4.5 in (E, L_z, Q, λ, log NI) space
    SpatialDiscontinuity,   // Local MAD z-score > 6 from neighbors

    // MEDIUM (interesting edge cases)
    ExtremeAffine,          // λ MAD z-score > 3.5
    ExtremeWraps,           // φ-wraps > p99 (not on ring band)
    RapidTurningPoints,     // N_r or N_θ > 6

    // INFO
    RoundoffK,              // |K| < 1e-12 (numerical zero)
}

pub struct Outlier {
    pub pixel: (u32, u32),
    pub category: OutlierCategory,
    pub severity: OutlierSeverity,  // Critical/High/Medium/Low
    pub value: f64,
    pub details: String,  // Human-readable explanation
}
```

#### Detection Pipeline (2-pass):

```rust
fn detect_outliers(
    hp_data: &HpData,
    manifest: &Manifest,
    order_stats: &HashMap<u8, OrderStats>,
    index_grid: &IndexGrid,
) -> Vec<Outlier> {
    let mut outliers = Vec::new();

    // PASS 1: Critical (short-circuit)
    for pos in &hp_data.positions {
        if let Some(crit) = classify_physical_outliers(pos, manifest.spin, r_h, r_max) {
            outliers.push(crit);
        }
    }

    // PASS 2: Statistical & Spatial (order-aware)
    outliers.extend(detect_statistical_outliers(&hits, &order_stats));
    outliers.extend(detect_spatial_outliers(hp_data, &index_grid));
    outliers.extend(detect_mahalanobis_outliers(&hits, order));

    // Sort by severity
    outliers.sort_by_key(|o| (o.severity as u8, o.pixel));
    outliers
}
```

### 3.4 Provenance Block (15 min)

Add to `Manifest` in `transfer_maps.rs`:

```rust
pub git_sha: String,         // From `git rev-parse HEAD`
pub rustc_version: String,   // From `rustc --version`
pub build_timestamp: String, // ISO 8601
```

Populate during generation in `bin/generate.rs`:

```rust
let git_sha = std::process::Command::new("git")
    .args(&["rev-parse", "HEAD"])
    .output()
    .ok()
    .and_then(|o| String::from_utf8(o.stdout).ok())
    .unwrap_or_else(|| "unknown".to_string());
```

Display in HTML footer.

---

## Phase 4: Tier 2 Implementation (4 hours)

### 4.1 K Validation Heatmap (1 hour)

Visualize Carter constant across image plane:

```rust
// Compute K for all pixels
let k_grid: Vec<f64> = hp_data.positions.iter().map(|p| {
    p.carter_q + (p.angular_momentum - a * p.energy).powi(2)
}).collect();

// Downsample to manageable size (e.g., 400×225)
let downsampled = downsample_grid(&k_grid, width, height, 400, 225);

// Render heatmap
pub fn generate_k_heatmap_svg(grid: &[f64], width: usize, height: usize) -> String {
    // Color scale:
    // - Green: K > 1e-10
    // - Yellow: |K| < 1e-10 (≈0)
    // - Red: K < -1e-10 (violation)
}
```

### 4.2 Transfer Function 2D Histogram (1.5 hours)

Shows ray-disc transfer mapping (where image pixels land on disc):

```rust
// For each order, bin by (image_radius, emission_radius)
let image_center = (width / 2, height / 2);

for rec in &hp_data.positions {
    if !rec.hit { continue; }

    let image_r = f64::hypot(
        rec.pixel_x as f64 - image_center.0 as f64,
        rec.pixel_y as f64 - image_center.1 as f64
    );
    let emission_r = rec.r;

    // Bin into 2D histogram
    bins[rec.order][image_r_bin][emission_r_bin] += 1;
}

// Generate 3 heatmaps (order 0, 1, 2+)
pub fn generate_transfer_function_svg(
    histogram: &[[usize; EMISSION_BINS]; IMAGE_BINS],
    order: u8,
) -> String;
```

### 4.3 Asymmetry Quantification (45 min)

Frame-dragging causes left/right asymmetry:

```rust
// Split by approaching (0 < φ < π) vs receding (π < φ < 2π)
let approaching: Vec<_> = hits.iter()
    .filter(|h| h.phi.rem_euclid(2.0 * PI) < PI)
    .collect();
let receding: Vec<_> = hits.iter()
    .filter(|h| h.phi.rem_euclid(2.0 * PI) >= PI)
    .collect();

let asymmetry_ratio = approaching.len() as f64 / receding.len() as f64;

// Stacked bar chart showing counts + percentages
pub fn generate_asymmetry_bar_chart(
    approaching_count: usize,
    receding_count: usize,
) -> String;
```

### 4.4 Time Delay Map (45 min)

Visualize relative light travel times:

```rust
// Compute min affine parameter (earliest arrival)
let min_lambda = hits.iter()
    .map(|h| h.affine_parameter)
    .min_by(|a, b| a.partial_cmp(b).unwrap())
    .unwrap_or(0.0);

// Build delay grid
let delay_grid: Vec<f64> = hp_data.positions.iter().map(|p| {
    if p.hit {
        p.affine_parameter - min_lambda
    } else {
        f64::NAN
    }
}).collect();

// Render as heatmap (warmer = later arrival)
pub fn generate_time_delay_heatmap_svg(
    grid: &[f64],
    width: usize,
    height: usize,
) -> String;
```

---

## Phase 5: Tier 3 Implementation (4.5 hours)

### 5.1 Critical Curve Extraction (2 hours)

Find boundary between captured and escaped rays (Kerr shadow):

```rust
// Build binary grid
let mut capture_grid = vec![vec![false; width]; height];
for pos in &hp_data.positions {
    capture_grid[pos.pixel_y as usize][pos.pixel_x as usize] = pos.captured;
}

// Edge detection (simple neighbor diff)
fn find_boundary_pixels(grid: &[Vec<bool>]) -> Vec<(u32, u32)> {
    let mut boundary = Vec::new();
    for y in 1..grid.len()-1 {
        for x in 1..grid[0].len()-1 {
            // If differs from any 4-neighbor, it's an edge
            let center = grid[y][x];
            if grid[y-1][x] != center || grid[y+1][x] != center ||
               grid[y][x-1] != center || grid[y][x+1] != center {
                boundary.push((x as u32, y as u32));
            }
        }
    }
    boundary
}

// Fit ellipse to boundary points (least-squares)
fn fit_ellipse(points: &[(u32, u32)]) -> EllipseParams {
    // Returns: center, semi_major, semi_minor, rotation, axis_ratio
}

// Report metrics in HTML
```

### 5.2 Turning-Point Histograms (1 hour)

Visualize geodesic complexity:

```rust
// Histogram turns_r and turns_theta
let turns_r_hist = compute_histogram(
    &hits.map(|h| h.turns_r),
    0..=10
);
let turns_theta_hist = compute_histogram(
    &hits.map(|h| h.turns_theta),
    0..=10
);

// Dual bar charts side-by-side
pub fn generate_turning_points_histogram_svg(
    r_hist: &[(u8, usize)],
    theta_hist: &[(u8, usize)],
) -> String;
```

### 5.3 Wrap-Angle vs Impact Parameter (1.5 hours)

Test against theoretical predictions:

```rust
// Compute impact parameter for each ray
let scatter_data: Vec<(f64, f64, u8)> = hits.iter().map(|h| {
    let b = h.angular_momentum / h.energy;  // Impact parameter
    (b, h.phi_wraps, h.order)
}).collect();

// Scatter plot with order-based coloring
pub fn generate_wraps_vs_impact_scatter_svg(
    data: &[(f64, f64, u8)],
    b_photon_sphere: f64,  // Theoretical critical value
) -> String {
    // x-axis: impact parameter b
    // y-axis: φ-wraps
    // Color by order
    // Add vertical line at b_crit (photon sphere)
}
```

---

## Phase 6: HTML Report Integration

### 6.1 Outlier Summary Section

```html
<h2>🚨 Outlier Detection & Validation</h2>
<div class="card">
  <div class="outlier-summary">
    <span class="badge critical">{critical_count} Critical</span>
    <span class="badge high">{high_count} High</span>
    <span class="badge medium">{medium_count} Medium</span>
  </div>

  <details open>
    <summary>Critical Outliers ({critical_count})</summary>
    <p class="section-desc">Physics violations requiring investigation</p>
    <table>
      <!-- Top 10 critical outliers with pixel coords, category, value -->
    </table>
  </details>

  <h3>Outlier Spatial Distribution</h3>
  <div class="chart-grid">
    <div>{outlier_overlay_svg}</div>
  </div>
</div>
```

### 6.2 Miss Taxonomy Section

```html
<h2>📊 Miss Taxonomy</h2>
<div class="card">
  <p class="section-desc">Why 83% of rays don't hit the disc</p>
  <div class="chart-grid">
    <div>{miss_pie_chart}</div>
    <div>
      <table>
        <tr>
          <td>Escaped to ∞</td>
          <td>{escaped_count}</td>
          <td>{escaped_pct}%</td>
        </tr>
        <tr>
          <td>Captured by horizon</td>
          <td>{captured_count}</td>
          <td>{captured_pct}%</td>
        </tr>
        <tr>
          <td>Numerical abort</td>
          <td>{aborted_count}</td>
          <td>{aborted_pct}%</td>
        </tr>
      </table>
    </div>
  </div>
</div>
```

### 6.3 Enhanced Footer

```html
<footer>
  <p>Generated by Kerr Black Hole High-Precision Analyzer</p>
  <p>
    Preset: {preset} | {width}×{height} | {orders} orders | {inclination}°
    inclination
  </p>
  <details>
    <summary>Provenance</summary>
    <ul>
      <li>Git SHA: {git_sha}</li>
      <li>Rust: {rustc_version}</li>
      <li>Generated: {timestamp}</li>
    </ul>
  </details>
</footer>
```

---

## Implementation Sessions

### Session 1: Data Collection & Infrastructure (3 hours)

1. Add fields to `PositionData` (`transfer_maps.rs`)
2. Track turning points (`integration.rs`)
3. Classify miss reasons (`integration.rs`)
4. Create `stats.rs` with MAD/log-space functions
5. Rebuild & regenerate test data
6. **Checkpoint:** Verify new fields in JSON output

### Session 2: Tier 1 Features (2 hours)

1. Hit taxonomy + pie chart
2. Order mask thumbnails
3. Basic outlier spotlight (top-N)
4. Provenance block
5. **Checkpoint:** Review HTML with Tier 1 features

### Session 3: Enhanced Outlier Detection (8 hours)

1. Implement physics violation checks
2. Per-order statistics infrastructure
3. Statistical outliers (MAD z-scores, log-space NI)
4. Spatial outliers (index grid, neighbor checks)
5. Mahalanobis distance (multivariate)
6. Outlier overlay heatmap
7. **Checkpoint:** Verify critical/high/medium classification

### Session 4: Tier 2 Features (4 hours)

1. K validation heatmap
2. Transfer function 2D histograms
3. Asymmetry quantification
4. Time delay map
5. **Checkpoint:** Review all Tier 2 visualizations

### Session 5: Tier 3 Features (4.5 hours)

1. Critical curve extraction & ellipse fitting
2. Turning-point histograms
3. Wrap-angle vs impact parameter scatter
4. **Checkpoint:** Full report review

### Session 6: Polish & Documentation (1 hour)

1. CSS refinements
2. Responsive layout testing
3. Add tooltips/explanations
4. Final validation run

---

## Success Criteria

- [ ] All 5 new fields exported correctly in HP JSON
- [ ] Miss taxonomy explains >95% of misses
- [ ] Outlier detection flags <1% of pixels as critical
- [ ] K validation shows zero red pixels (K<0)
- [ ] Transfer functions show expected ISCO peak
- [ ] Critical curve matches theoretical Kerr predictions
- [ ] Turning-point histograms peak at expected values
- [ ] HTML report loads in <2s, self-contained, print-ready

---

## Notes for Tomorrow

- No backward compatibility needed (regenerating all data)
- Use MAD instead of IQR throughout for robustness
- Always work in log-space for null invariant
- Per-order thresholds critical to avoid false positives
- Spatial checks require ≥3 same-order neighbors
- K epsilon = 1e-12 (roundoff tolerance)
- Mahalanobis threshold = 4.5 (stricter than 3.0)
- Local MAD z-score threshold = 6.0 for spatial outliers

---

## Additional Notes - EXTRA :

## 🔴 **Magnification / Caustic Analysis** (Complex, Needs Neighbor Data)

### What it does:

Compute the **Jacobian determinant** of the ray-disc mapping to find where gravitational lensing creates extreme brightness (caustics) and compute magnification factors.

### The math:

The magnification is:

```
μ = |∂(α,β)/∂(r,ϕ)|⁻¹
```

where `(α,β)` are image-plane coords and `(r,ϕ)` are disc hit coords.

To compute this, you need the **finite-difference Jacobian**:

```
∂r/∂α ≈ (r[i+1,j] - r[i-1,j]) / (2Δα)
∂r/∂β ≈ (r[i,j+1] - r[i,j-1]) / (2Δβ)
∂ϕ/∂α ≈ (ϕ[i+1,j] - ϕ[i-1,j]) / (2Δα)
∂ϕ/∂β ≈ (ϕ[i,j+1] - ϕ[i,j-1]) / (2Δβ)
```

Then:

```
μ = 1 / |∂r/∂α · ∂ϕ/∂β - ∂r/∂β · ∂ϕ/∂α|
```

### Why it's difficult:

1. **Needs neighbor pixel data** - For each pixel `[i,j]`, you need:
   - `r_hit` and `phi_hit` from neighbors: `[i±1, j]` and `[i, j±1]`
   - But your current data structure is **flat** (just a list of hits)
   - You'd need to either:
     - **Store neighbor indices** in each record: `neighbor_indices: [u32; 4]`
     - **Build a 2D lookup table** from the flat list (image[y][x] → record)

2. **Handling missing neighbors** - What if:
   - Pixel `[i,j]` hits the disc, but `[i+1,j]` misses?
   - Or they hit at **different orders**?
   - Or across a **caustic** where the mapping is discontinuous?
   - You need robust logic to skip/interpolate/flag these cases

3. **Disc coordinate wrapping** - φ wraps at 2π:
   - If `phi[i+1,j] = 0.1` and `phi[i-1,j] = 6.2`, the naive difference is wrong
   - Need `atan2`-style unwrapping: `Δφ = (φ₂ - φ₁ + π) % 2π - π`

4. **Order separation** - You should compute magnification **per order**:
   - Order 0 has its own caustic structure
   - Order 1 (photon ring) has a **super-high** magnification ridge
   - Mixing them gives garbage
   - So you need to filter: "only compute Jacobian for pixels where `[i,j]` and all 4 neighbors have `order == k`"

5. **Visualization complexity** - Caustics are:
   - **Ridges** of infinite magnification (1D curves in 2D)
   - Need edge-detection or gradient-based rendering
   - Often shown as log-scale heatmaps with manual color clipping

### Code complexity example:

```rust
// In transfer_maps.rs, add:
pub struct PositionData {
    // ... existing fields ...
    pub neighbor_up: Option<usize>,     // index in positions array
    pub neighbor_down: Option<usize>,
    pub neighbor_left: Option<usize>,
    pub neighbor_right: Option<usize>,
}

// In analyzer:
fn compute_magnification(data: &HpData, manifest: &Manifest) -> Vec<f64> {
    let mut magnification = vec![f64::NAN; data.positions.len()];

    for (idx, pos) in data.positions.iter().enumerate() {
        // Skip if any neighbor missing or different order
        let neighbors = [pos.neighbor_up, pos.neighbor_down,
                        pos.neighbor_left, pos.neighbor_right];

        if neighbors.iter().any(|n| n.is_none()) { continue; }

        let up = &data.positions[neighbors[0].unwrap()];
        let down = &data.positions[neighbors[1].unwrap()];
        let left = &data.positions[neighbors[2].unwrap()];
        let right = &data.positions[neighbors[3].unwrap()];

        if ![up, down, left, right].iter().all(|n| n.order == pos.order) {
            continue;  // Order mismatch
        }

        // Finite differences (with φ wrapping!)
        let dr_dx = (right.r - left.r) / (2.0 * pixel_size_x);
        let dr_dy = (up.r - down.r) / (2.0 * pixel_size_y);

        let dphi_dx = unwrap_phi_diff(right.phi - left.phi) / (2.0 * pixel_size_x);
        let dphi_dy = unwrap_phi_diff(up.phi - down.phi) / (2.0 * pixel_size_y);

        // Jacobian determinant
        let det = dr_dx * dphi_dy - dr_dy * dphi_dx;
        magnification[idx] = 1.0 / det.abs();

        // Clip caustic infinities
        if magnification[idx] > 1e6 { magnification[idx] = 1e6; }
    }

    magnification
}

fn unwrap_phi_diff(dphi: f64) -> f64 {
    // Handle 2π wrapping
    if dphi > PI { dphi - 2.0*PI }
    else if dphi < -PI { dphi + 2.0*PI }
    else { dphi }
}
```

**Then** you need to render it as a heatmap, which requires exporting a 2D grid back to the HTML...

---
