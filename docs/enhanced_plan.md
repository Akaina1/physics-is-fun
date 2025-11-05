# Enhanced Diagnostics & Outlier Detection Implementation

## Overview

Enhance the high-precision geodesic data analyzer with comprehensive diagnostic tools across 4 implementation tiers. Each tier builds on the previous, adding increasingly sophisticated analysis capabilities for research-grade output.

## Phase 1: Data Collection (Physics Layer)

### 1.1 Expand `PositionData` Structure

**File**: `kernels/kerr_black_hole/src/transfer_maps.rs`

Add new fields to track miss classification and geodesic complexity:

```rust
pub struct PositionData {
    // ... existing 15 fields ...

    // Miss classification (NEW)
    pub escaped: bool,      // r → ∞ (> 1000M)
    pub captured: bool,     // r → r_horizon
    pub aborted: bool,      // Numerical failure

    // Geodesic complexity (NEW)
    pub turns_r: u8,        // Total radial turning points
    pub turns_theta: u8,    // Total polar turning points
}
```

Update `to_json()` method to serialize new fields.

### 1.2 Track Turning Points

**File**: `kernels/kerr_black_hole/src/integration.rs`

In `integrate_geodesic_multi_order()`:

- Initialize counters: `let mut turns_r = 0u8; let mut turns_theta = 0u8;`
- Track sign flips throughout entire geodesic (all orders)
- Use `saturating_add(1)` to prevent overflow at 255
- Return counts with each `GeodesicResult::DiscHit`

### 1.3 Classify Miss Reasons

**File**: `kernels/kerr_black_hole/src/integration.rs`

Add classification logic at integration termination:

- `escaped = true` if `state.r > 1000.0`
- `captured = true` if `state.r < r_horizon * 1.01`
- `aborted = true` otherwise (step limit, NaN)
- Store in all non-hit results returned from `integrate_geodesic_multi_order()`

### 1.4 Update Packing Functions

**File**: `kernels/kerr_black_hole/src/transfer_maps.rs`

Modify `pack_pixel_multi_order()` to store new fields in `HighPrecisionData`:

- Extract miss flags and turning point counts from `GeodesicResult`
- Populate fields in `PositionData` struct during unsafe writes

---

## Phase 2: Statistical Infrastructure

### 2.1 Create Stats Module

**File**: `kernels/kerr_black_hole/src/bin/analyze/stats.rs` (NEW)

Implement robust statistical functions:

```rust
// Median computation
pub fn compute_median(values: &[f64]) -> f64

// MAD (Median Absolute Deviation) - more robust than StdDev
pub fn compute_mad(values: &[f64]) -> (f64, f64)  // Returns (median, mad)

// MAD-based z-score (robust outlier detection)
pub fn mad_zscore(value: f64, median: f64, mad: f64) -> f64

// Log-space statistics for heavy-tailed distributions (null invariant)
pub fn compute_log_stats(values: &[f64]) -> LogStats {
    median_log: f64,
    mad_log: f64,
}

// Per-order statistics to avoid false positives
pub struct OrderStats {
    pub ni_log_median: f64,
    pub ni_log_mad: f64,
    pub lambda_median: f64,
    pub lambda_mad: f64,
    pub g_median: f64,
    pub g_mad: f64,
    pub wraps_p99: f64,
    pub turns_r_p99: u8,
    pub turns_theta_p99: u8,
}

pub fn compute_order_stats(hits: &[&HpRecord], order: u8) -> OrderStats
```

### 2.2 Spatial Index Grid

**File**: `kernels/kerr_black_hole/src/bin/analyze/stats.rs`

Efficient O(1) neighbor lookups:

```rust
pub struct IndexGrid {
    grid: Vec<Vec<Option<usize>>>,  // (x,y) → positions index
    width: usize,
    height: usize,
}

impl IndexGrid {
    pub fn from_hp_data(data: &HpData, width: usize, height: usize) -> Self;
    pub fn get_neighbors_same_order(&self, x: usize, y: usize, order: u8, data: &HpData) -> Vec<usize>;
}
```

---

## Tier 1: Quick Wins (~2 hours)

### T1.1 Hit Taxonomy with Pie Chart

**Files**: `src/bin/analyze/main.rs`, `src/bin/analyze/charts.rs`

**Data Collection** (`compute_statistics`):

```rust
let escaped_count = hp_data.positions.iter().filter(|p| !p.hit && p.escaped).count();
let captured_count = hp_data.positions.iter().filter(|p| !p.hit && p.captured).count();
let aborted_count = hp_data.positions.iter().filter(|p| !p.hit && p.aborted).count();
```

**Chart Generation** (`charts.rs`):

```rust
pub fn generate_miss_taxonomy_pie(
    escaped: usize, captured: usize, aborted: usize, total: usize
) -> String {
    // SVG pie chart: 3 segments
    // Colors: #3B82F6 (blue/escaped), #EF4444 (red/captured), #6B7280 (gray/aborted)
    // Include percentages in labels
}
```

### T1.2 Order Mask Thumbnails

**File**: `src/bin/analyze/charts.rs`

Generate binary thumbnails showing which pixels hit at each order:

```rust
pub fn generate_order_thumbnail_svg(
    width: u32,          // Downsampled (e.g., 200px)
    height: u32,         // Downsampled (e.g., 112px)
    pixel_orders: &[Option<u8>],
    target_order: u8,
) -> String {
    // Downsample by binning (8×8 blocks)
    // SVG rects: white if order matches, black otherwise
    // Add title and border
}
```

Create 3 thumbnails: Order 0, Order 1, Order 2+

### T1.3 Basic Outlier Spotlight

**File**: `src/bin/analyze/main.rs`

Sort and display top-N outliers by null invariant:

```rust
// Top 10 worst null invariant errors
let mut ni_outliers: Vec<_> = hits.iter().collect();
ni_outliers.sort_by(|a, b| b.null_invariant_error.partial_cmp(&a.null_invariant_error).unwrap());
let top_10_ni = ni_outliers.into_iter().take(10).collect::<Vec<_>>();
```

Display in HTML table with pixel coordinates, value, and order.

### T1.4 Provenance Block

**Files**: `src/transfer_maps.rs`, `src/bin/generate.rs`

Add to `Manifest`:

```rust
pub git_sha: String,
pub rustc_version: String,
pub build_timestamp: String,  // ISO 8601
```

Populate during generation using `std::process::Command` for git SHA and `rustc --version`. Display in HTML footer.

---

## Tier 2: Research Metrics (~4 hours)

### T2.1 K Validation Heatmap

**File**: `src/bin/analyze/charts.rs`

Visualize Carter constant K = Q + (L_z - aE)²:

```rust
pub fn generate_k_heatmap_svg(
    positions: &[PositionData],
    manifest: &Manifest,
    downsample: (usize, usize),  // e.g., (400, 225)
) -> String {
    // Compute K for all pixels
    // Downsample by averaging blocks
    // Color scale: green (K>0), yellow (K≈0), red (K<0)
    // Log scale for better visibility
}
```

### T2.2 Transfer Function 2D Histogram

**File**: `src/bin/analyze/charts.rs`

Ray-disc transfer mapping (image radius → emission radius):

```rust
pub fn generate_transfer_function_svg(
    hits: &[&HpRecord],
    manifest: &Manifest,
    order: u8,
) -> String {
    // Compute image_radius = distance from image center
    // Bin by (image_r, emission_r) into 50×50 grid
    // Heatmap with log color scale
    // Vertical line at ISCO radius
}
```

Generate 3 separate heatmaps (orders 0, 1, 2+).

### T2.3 Asymmetry Quantification

**File**: `src/bin/analyze/main.rs`

Frame-dragging causes φ asymmetry:

```rust
let approaching: Vec<_> = hits.iter()
    .filter(|h| h.phi.rem_euclid(2.0 * PI) < PI)
    .collect();
let receding: Vec<_> = hits.iter()
    .filter(|h| h.phi.rem_euclid(2.0 * PI) >= PI)
    .collect();
let asymmetry_ratio = approaching.len() as f64 / receding.len() as f64;
```

Stacked bar chart showing approaching vs receding side hit counts.

### T2.4 Time Delay Map

**File**: `src/bin/analyze/charts.rs`

Visualize relative light travel times via affine parameter:

```rust
pub fn generate_time_delay_heatmap_svg(
    positions: &[PositionData],
    manifest: &Manifest,
    downsample: (usize, usize),
) -> String {
    // Normalize: λ_delay = λ - λ_min
    // Downsample and render heatmap
    // Warm colors = later arrival
}
```

---

## Tier 3: Publication Quality (~4.5 hours)

### T3.1 Critical Curve Extraction

**File**: `src/bin/analyze/stats.rs`

Find boundary between captured and escaped rays:

```rust
pub fn extract_critical_curve(positions: &[PositionData], width: usize, height: usize) -> Vec<(u32, u32)> {
    // Build binary capture_grid
    // Edge detection: 4-neighbor difference
    // Return boundary pixel coordinates
}

pub fn fit_ellipse(points: &[(u32, u32)]) -> EllipseParams {
    // Least-squares ellipse fitting
    // Returns: center, semi_major, semi_minor, rotation, axis_ratio
}
```

Report metrics in HTML: ellipse parameters, shadow size, theoretical comparison.

### T3.2 Turning-Point Histograms

**File**: `src/bin/analyze/charts.rs`

Visualize geodesic complexity:

```rust
pub fn generate_turning_points_histogram_svg(
    hits: &[&HpRecord],
) -> String {
    // Histogram turns_r: bins 0-10+
    // Histogram turns_theta: bins 0-10+
    // Dual bar charts side-by-side
    // Per-order coloring
}
```

### T3.3 Wrap-Angle vs Impact Parameter

**File**: `src/bin/analyze/charts.rs`

Test theoretical predictions:

```rust
pub fn generate_wraps_vs_impact_scatter_svg(
    hits: &[&HpRecord],
    manifest: &Manifest,
) -> String {
    // x-axis: b = L_z/E (impact parameter)
    // y-axis: φ-wraps
    // Color by order
    // Vertical line at b_photon_sphere = 3√3 M (a=0) or computed for Kerr
}
```

---

## Tier 4: Enhanced Outlier Detection (~8 hours)

### T4.1 Outlier Categories

**File**: `src/bin/analyze/stats.rs`

Define comprehensive outlier taxonomy:

```rust
pub enum OutlierCategory {
    // CRITICAL (physics violations)
    NegativeK,              // K < -1e-12
    LargeNullInvariant,     // NI > 1e-9
    InvalidState,           // NaN, ±∞
    InvalidRadius,          // r < r_h or r > 1000
    InvalidTheta,           // θ ∉ [0,π]

    // HIGH (statistical anomalies, top 0.1%)
    ExtremeNI,              // NI log z-score > 3.5 AND top 0.1%
    MahalanobisOutlier,     // D_M > 4.5 in (E, L_z, Q, λ, log NI) space
    SpatialDiscontinuity,   // Local MAD z-score > 6 from neighbors

    // MEDIUM (interesting edge cases)
    ExtremeAffine,          // λ MAD z-score > 3.5
    ExtremeWraps,           // φ-wraps > p99
    RapidTurningPoints,     // turns_r or turns_theta > 6

    // INFO
    RoundoffK,              // |K| < 1e-12
}

pub struct Outlier {
    pub pixel: (u32, u32),
    pub category: OutlierCategory,
    pub severity: OutlierSeverity,
    pub value: f64,
    pub details: String,
}
```

### T4.2 Detection Pipeline (2-Pass)

**File**: `src/bin/analyze/stats.rs`

```rust
pub fn detect_outliers(
    hp_data: &HpData,
    manifest: &Manifest,
    order_stats: &HashMap<u8, OrderStats>,
    index_grid: &IndexGrid,
) -> Vec<Outlier> {
    // PASS 1: Critical violations (short-circuit)
    classify_physical_outliers()

    // PASS 2: Statistical & spatial (order-aware)
    detect_statistical_outliers()
    detect_spatial_outliers()
    detect_mahalanobis_outliers()

    // Sort by severity then pixel
}
```

Implement each detection function with appropriate thresholds from plan.

### T4.3 Mahalanobis Distance

**File**: `src/bin/analyze/stats.rs`

Multivariate outlier detection in (E, L_z, Q, λ, log NI) space:

```rust
pub fn compute_mahalanobis_distance(
    record: &HpRecord,
    order_stats: &OrderStats,
) -> f64 {
    // Standardize each dimension by MAD
    // Compute Euclidean distance in standardized space
    // Threshold: D_M > 4.5
}
```

### T4.4 Outlier Visualization

**File**: `src/bin/analyze/charts.rs`

Spatial distribution overlay heatmap:

```rust
pub fn generate_outlier_overlay_svg(
    outliers: &[Outlier],
    width: u32,
    height: u32,
) -> String {
    // Downsample to manageable size
    // Color by severity: red (critical), orange (high), yellow (medium)
    // Opacity by count in cell
}
```

---

## HTML Report Integration

### Update `generate_summary.rs`

Add new sections:

1. **Miss Taxonomy Section**: Pie chart + table with escaped/captured/aborted breakdown
2. **Order Thumbnails Section**: 3 thumbnails side-by-side
3. **Outlier Summary Section**: Severity badges + collapsible tables per severity level
4. **Validation Section**: K heatmap, turning points histograms
5. **Transfer Analysis Section**: 2D histograms per order, asymmetry bar chart, time delay map
6. **Critical Curves Section**: Boundary visualization, ellipse fit parameters
7. **Advanced Plots Section**: Wraps vs impact parameter scatter
8. **Enhanced Footer**: Provenance block in `<details>` element

---

## Testing & Validation

After each tier:

1. Regenerate data for one preset: `cargo run --release --bin generate -- --preset balanced --export-precision`
2. Run analyzer: `cargo run --release --bin analyze -- -i public/blackhole/prograde/balanced/high_precision.json`
3. Verify HTML output visually
4. Check console for statistics matching expectations

Success Criteria:

- All new fields present in HP JSON
- Miss taxonomy sums to 100%
- K heatmap shows no red pixels (K<0)
- Outlier detection flags <1% critical
- Turning point histograms show expected distributions
- HTML renders correctly, loads in <2s

---

## Implementation Order

1. **Session 1** (3h): Phase 1 (data collection) + regenerate all presets
2. **Session 2** (2h): Phase 2 (stats infrastructure) + Tier 1
3. **Session 3** (4h): Tier 2 (research metrics)
4. **Session 4** (4.5h): Tier 3 (publication quality)
5. **Session 5** (8h): Tier 4 (enhanced outlier detection)
6. **Session 6** (1h): Polish, validation, documentation
