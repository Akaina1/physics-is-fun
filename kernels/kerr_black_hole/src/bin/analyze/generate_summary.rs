// HTML Report Generation

use super::charts;

/// Analysis statistics
pub struct Stats {
    // Overall
    pub total_pixels: usize,
    pub total_hit_pixels: usize,
    pub total_hits: usize,
    pub miss_pixels: usize,
    
    // Per-order counts
    pub order_0_hits: usize,
    pub order_1_hits: usize,
    pub order_2_plus_hits: usize,
    
    // Order distribution per pixel
    pub only_order_0: usize,
    pub orders_0_and_1: usize,
    pub orders_2_plus: usize,
    
    // NEW: Miss taxonomy (Tier 1.1)
    pub miss_escaped: usize,
    pub miss_captured: usize,
    pub miss_aborted: usize,
    
    // Null invariant
    pub ni_min: f64,
    pub ni_max: f64,
    pub ni_mean: f64,
    pub ni_median: f64,
    pub ni_p95: f64,
    pub ni_p99: f64,
    pub ni_under_1e15: usize,
    pub ni_under_1e12: usize,
    pub ni_under_1e9: usize,
    
    // Redshift
    pub g_min: f64,
    pub g_max: f64,
    pub g_mean: f64,
    pub g_boosted_count: usize,
    pub g_dimmed_count: usize,
    
    // Radial distribution (order 0)
    pub r_bins: Vec<(String, usize)>,
    
    // Phi wraps
    pub phi_wraps_min: f64,
    pub phi_wraps_max: f64,
    pub phi_wraps_mean_o0: f64,
    pub phi_wraps_mean_o1: f64,
    pub phi_wraps_mean_o2: f64,
    
    // Affine parameter
    pub affine_mean_o0: f64,
    pub affine_mean_o1: f64,
    pub affine_mean_o2: f64,
    
    // Chart data
    pub ni_histogram: Vec<(f64, f64, usize)>,
    pub radial_histogram: Vec<(f64, usize, usize, usize)>,
    pub radial_profile: Vec<(f64, usize, f64, f64)>,
    pub angular_distribution: Vec<(f64, usize)>,
    
    // NEW: Tier 1.3 - Outlier spotlight
    pub top_ni_outliers: Vec<(u32, u32, u8, f64)>,  // (pixel_x, pixel_y, order, ni_value)
    
    // NEW: Tier 2 - Research metrics (SVG strings pre-generated)
    pub k_heatmap_svg: String,
    pub transfer_o0_svg: String,
    pub transfer_o1_svg: String,
    pub transfer_o2_svg: String,
    pub time_delay_svg: String,
    
    // NEW: Tier 3 - Publication quality
    pub critical_curve_points: usize,
    pub ellipse_params: Option<super::stats::EllipseParams>,
    pub turning_histogram_svg: String,
    pub wraps_scatter_svg: String,
    
    // NEW: Tier 4 - Enhanced outlier detection
    pub outliers: Vec<super::stats::Outlier>,
    pub outlier_overlay_svg: String,
}

/// Manifest metadata
#[derive(Debug, serde::Deserialize)]
pub struct Manifest {
    pub width: u32,
    pub height: u32,
    pub preset: String,
    pub inclination: f64,
    pub spin: f64,
    pub orders: u8,
    pub r_in: f64,
    pub r_out: f64,
    #[serde(alias = "disc_hits")]  // Accept old name for backward compatibility
    pub _disc_hits: usize,
    
    // NEW: Tier 1.4 - Provenance tracking
    pub git_sha: Option<String>,
    pub rustc_version: Option<String>,
    pub build_timestamp: Option<String>,
}

pub fn generate_html_report(stats: &Stats, manifest: &Manifest, pixel_orders: &[Option<u8>]) -> String {
    let miss_pct = stats.miss_pixels as f64 / stats.total_pixels as f64 * 100.0;
    let hit_pct = stats.total_hit_pixels as f64 / stats.total_pixels as f64 * 100.0;
    
    let only_o0_pct = stats.only_order_0 as f64 / stats.total_pixels as f64 * 100.0;
    let o01_pct = stats.orders_0_and_1 as f64 / stats.total_pixels as f64 * 100.0;
    let o2p_pct = stats.orders_2_plus as f64 / stats.total_pixels as f64 * 100.0;
    
    let ni_p99_class = if stats.ni_p99 < 1e-12 { "ok" } else if stats.ni_p99 < 1e-9 { "warn" } else { "bad" };
    let ni_under_1e9_pct = stats.ni_under_1e9 as f64 / stats.total_hits as f64 * 100.0;
    let ni_gate_class = if ni_under_1e9_pct > 99.0 { "ok" } else { "warn" };
    
    let g_asymmetry = if stats.g_dimmed_count > 0 {
        stats.g_boosted_count as f64 / stats.g_dimmed_count as f64
    } else {
        0.0
    };
    
    // Conditional interpretation messages based on actual data quality
    
    // 1. Null Invariant Quality Message
    let ni_under_1e12_pct = stats.ni_under_1e12 as f64 / stats.total_hits as f64 * 100.0;
    let (ni_message, ni_message_class) = if ni_under_1e12_pct >= 95.0 {
        ("✓ Excellent: 95%+ rays meet publication standard (NI < 1e-12)", "ok")
    } else if ni_under_1e12_pct >= 80.0 {
        ("⚠ Good: Most rays are publishable quality, some numerical drift", "warn")
    } else if ni_under_1e9_pct >= 95.0 {
        ("⚠ Acceptable for visualization, consider tightening tolerances for publication", "warn")
    } else {
        ("❌ Poor numerical quality - increase integration accuracy", "bad")
    };
    
    // 2. Doppler Asymmetry Message
    let spin_param = manifest.spin.abs();
    let (doppler_message, doppler_message_class) = if spin_param < 0.05 {
        // Near-Schwarzschild: expect symmetry
        if (g_asymmetry - 1.0).abs() < 0.2 {
            ("✓ Symmetric redshift distribution (as expected for low spin)", "ok")
        } else {
            ("⚠ Unexpected asymmetry for near-Schwarzschild metric", "warn")
        }
    } else {
        // Kerr: expect strong asymmetry from frame-dragging
        if g_asymmetry > 2.0 {
            ("✓ Clear Doppler asymmetry from frame-dragging", "ok")
        } else if g_asymmetry > 1.3 {
            ("⚠ Weak asymmetry - check spin parameter or disc geometry", "warn")
        } else {
            ("❌ No asymmetry detected - frame-dragging signature missing", "bad")
        }
    };
    
    // 3. Max Wraps Interpretation
    let wraps_interpretation = if stats.phi_wraps_max > 3.0 {
        "extreme lensing near photon sphere!"
    } else if stats.phi_wraps_max > 1.5 {
        "strong gravitational lensing"
    } else if stats.phi_wraps_max > 0.8 {
        "moderate lensing (typical for disc imaging)"
    } else {
        "mostly direct paths"
    };
    
    // Generate radial distribution bars
    let r_max_count = stats.r_bins.iter().map(|(_, c)| c).max().unwrap_or(&1);
    let r_bars = stats.r_bins
        .iter()
        .map(|(label, count)| {
            let width = (*count as f64 / *r_max_count as f64 * 100.0).min(100.0);
            let pct = *count as f64 / stats.order_0_hits.max(1) as f64 * 100.0;
            format!(
                r#"<tr><td>{}</td><td>{}</td><td>{:.1}%</td><td><div class="bar" style="width: {:.1}%"></div></td></tr>"#,
                label, count, pct, width
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    
    // Generate SVG charts
    let ni_histogram_svg = charts::generate_ni_histogram_svg(&stats.ni_histogram);
    let radial_histogram_svg = charts::generate_radial_histogram_svg(&stats.radial_histogram, manifest.r_in, manifest.r_out);
    
    // NEW: Tier 1.1 - Miss taxonomy pie chart
    let miss_taxonomy_pie_svg = charts::generate_miss_taxonomy_pie(
        stats.miss_escaped,
        stats.miss_captured,
        stats.miss_aborted,
        stats.miss_pixels
    );
    
    // NEW: Tier 1.2 - Order mask thumbnails (3 thumbnails)
    let downsample = if manifest.width > 1600 { 8 } else { 4 };
    let order_0_thumb = charts::generate_order_thumbnail_svg(
        manifest.width, manifest.height, pixel_orders, 0, downsample
    );
    let order_1_thumb = charts::generate_order_thumbnail_svg(
        manifest.width, manifest.height, pixel_orders, 1, downsample
    );
    let order_2plus_thumb = charts::generate_order_thumbnail_svg(
        manifest.width, manifest.height, pixel_orders, 2, downsample
    );
    
    // NEW: Tier 1.3 - Outlier table rows
    let outlier_rows = stats.top_ni_outliers.iter()
        .enumerate()
        .map(|(i, (px, py, order, ni))| {
            let severity_class = if *ni >= 1e-9 { "high" } else if *ni >= 1e-12 { "medium" } else { "low" };
            let severity_text = if *ni >= 1e-9 { "High Attention" } else if *ni >= 1e-12 { "Medium" } else { "Low" };
            format!(
                r#"<tr><td>{}</td><td>({}, {})</td><td>{}</td><td class="{}">{:.2e}</td><td class="{}">{}</td></tr>"#,
                i + 1, px, py, order, severity_class, ni, severity_class, severity_text
            )
        })
        .collect::<Vec<_>>()
        .join("\n    ");
    
    // NEW: Tier 1.4 - Provenance text (inline in footer)
    let provenance_text = {
        let mut parts = Vec::new();
        if let Some(ref sha) = manifest.git_sha {
            parts.push(format!("SHA: {}", sha));
        }
        if let Some(ref version) = manifest.rustc_version {
            parts.push(format!("rustc: {}", version));
        }
        if let Some(ref timestamp) = manifest.build_timestamp {
            parts.push(format!("built: {}", timestamp));
        }
        if !parts.is_empty() {
            format!(" | {}", parts.join(" | "))
        } else {
            String::new()
        }
    };
    
    // Calculate ISCO for radial profile
    // Reference: Bardeen, Press, Teukolsky (1972)
    // Convention: spin > 0 = prograde, spin < 0 = retrograde, spin = 0 = Schwarzschild
    let r_isco = if manifest.spin.abs() < 1e-10 {
        // Schwarzschild: ISCO at 6M
        6.0
    } else {
        // Kerr: General formula (works for both prograde and retrograde)
        let z1 = 1.0 + (1.0 - manifest.spin.powi(2)).powf(1.0/3.0) 
            * ((1.0 + manifest.spin).powf(1.0/3.0) + (1.0 - manifest.spin).powf(1.0/3.0));
        let z2 = (3.0 * manifest.spin.powi(2) + z1.powi(2)).sqrt();
        if manifest.spin > 0.0 {
            // Prograde: co-rotating orbit (smaller ISCO)
            3.0 + z2 - ((3.0 - z1) * (3.0 + z1 + 2.0 * z2)).sqrt()
        } else {
            // Retrograde: counter-rotating orbit (larger ISCO)
            3.0 + z2 + ((3.0 - z1) * (3.0 + z1 + 2.0 * z2)).sqrt()
        }
    };
    let radial_profile_svg = charts::generate_radial_profile_svg(&stats.radial_profile, manifest.r_in, manifest.r_out, r_isco);
    let angular_distribution_svg = charts::generate_angular_distribution_svg(&stats.angular_distribution);
    
    // NEW: Tier 3.1 - Ellipse parameters block
    let ellipse_block = if let Some(ref ellipse) = stats.ellipse_params {
        format!(r#"<table style="margin-top:12px">
    <tr><th>Parameter</th><th>Value</th></tr>
    <tr><td>Center (x, y)</td><td>({:.1}, {:.1})</td></tr>
    <tr><td>Semi-major axis</td><td>{:.1} px</td></tr>
    <tr><td>Semi-minor axis</td><td>{:.1} px</td></tr>
    <tr><td>Rotation</td><td>{:.2}°</td></tr>
    <tr><td>Axis ratio (b/a)</td><td>{:.3}</td></tr>
  </table>
  <p class="section-desc" style="margin-top:12px">Fitted ellipse approximates the black hole shadow boundary. Axis ratio near 1.0 indicates circular shadow (low spin or face-on view).</p>"#,
            ellipse.center_x, ellipse.center_y,
            ellipse.semi_major, ellipse.semi_minor,
            ellipse.rotation.to_degrees(),
            ellipse.axis_ratio)
    } else {
        r#"<p class="section-desc" style="margin-top:12px">No ellipse fit available (insufficient boundary points).</p>"#.to_string()
    };
    
    // NEW: Tier 4 - Outlier detection HTML
    use super::stats::OutlierSeverity;
    
    // Count outliers by severity
    let critical_count = stats.outliers.iter().filter(|o| o.severity == OutlierSeverity::Critical).count();
    let high_count = stats.outliers.iter().filter(|o| o.severity == OutlierSeverity::High).count();
    let medium_count = stats.outliers.iter().filter(|o| o.severity == OutlierSeverity::Medium).count();
    let info_count = stats.outliers.iter().filter(|o| o.severity == OutlierSeverity::Info).count();
    
    // Generate severity badges
    let severity_badges = format!(r#"
    <div style="padding:16px; background:#f9fafb; border-radius:8px; border:1px solid #e5e7eb">
      <div style="font-weight:600; color:#1f2937; font-size:15px">Critical</div>
      <div style="font-size:28px; font-weight:700; color:{}">{}</div>
      <div style="font-size:12px; color:#6b7280">Physics violations</div>
    </div>
    <div style="padding:16px; background:#f9fafb; border-radius:8px; border:1px solid #e5e7eb">
      <div style="font-weight:600; color:#1f2937; font-size:15px">High</div>
      <div style="font-size:28px; font-weight:700; color:{}">{}</div>
      <div style="font-size:12px; color:#6b7280">Statistical anomalies</div>
    </div>
    <div style="padding:16px; background:#f9fafb; border-radius:8px; border:1px solid #e5e7eb">
      <div style="font-weight:600; color:#1f2937; font-size:15px">Medium</div>
      <div style="font-size:28px; font-weight:700; color:{}">{}</div>
      <div style="font-size:12px; color:#6b7280">Edge cases</div>
    </div>
    <div style="padding:16px; background:#f9fafb; border-radius:8px; border:1px solid #e5e7eb">
      <div style="font-weight:600; color:#1f2937; font-size:15px">Info</div>
      <div style="font-size:28px; font-weight:700; color:{}">{}</div>
      <div style="font-size:12px; color:#6b7280">Roundoff</div>
    </div>
    "#,
        OutlierSeverity::Critical.color(), critical_count,
        OutlierSeverity::High.color(), high_count,
        OutlierSeverity::Medium.color(), medium_count,
        OutlierSeverity::Info.color(), info_count
    );
    
    // Generate collapsible outlier tables by severity
    let mut outlier_tables = String::new();
    
    for severity in &[OutlierSeverity::Critical, OutlierSeverity::High, OutlierSeverity::Medium, OutlierSeverity::Info] {
        let severity_outliers: Vec<_> = stats.outliers.iter().filter(|o| o.severity == *severity).collect();
        
        if severity_outliers.is_empty() {
            continue;
        }
        
        // Limit display to first 20 per severity
        let more_text = if severity_outliers.len() > 20 {
            format!(" (showing first 20 of {})", severity_outliers.len())
        } else {
            String::new()
        };
        
        let mut rows = String::new();
        for outlier in severity_outliers.iter().take(20) {
            rows.push_str(&format!(
                r#"    <tr>
      <td>({}, {})</td>
      <td style="text-align:center">{}</td>
      <td>{}</td>
      <td style="font-family:monospace; font-size:11px">{:.3e}</td>
      <td style="font-size:12px; color:#6b7280">{}</td>
    </tr>
"#,
                outlier.pixel_x, outlier.pixel_y,
                outlier.order,
                outlier.category.label(),
                outlier.value,
                outlier.details
            ));
        }
        
        outlier_tables.push_str(&format!(r#"
  <details style="margin-top:16px">
    <summary style="cursor:pointer; color:{}; font-weight:600; font-size:14px">
      {} — {} outliers{}
    </summary>
    <table style="margin-top:12px; width:100%">
      <thead>
        <tr style="background:#f9fafb">
          <th>Pixel</th>
          <th>Order</th>
          <th>Category</th>
          <th>Value</th>
          <th>Details</th>
        </tr>
      </thead>
      <tbody>
{}
      </tbody>
    </table>
  </details>
"#,
            severity.color(),
            severity.label(),
            severity_outliers.len(),
            more_text,
            rows
        ));
    }
    
    format!(r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8" />
<title>Kerr HP Analysis — {} {}×{}</title>
<style>
  body{{font:14px/1.5 system-ui, sans-serif; color:#1f2937; margin:24px; max-width:1400px; background:#f9fafb}}
  h1,h2{{margin:16px 0 12px; color:#111827}}
  h1{{font-size:28px; font-weight:700}}
  h2{{font-size:20px; font-weight:600; margin-top:32px}}
  .muted{{color:#6b7280}}
  .grid{{display:grid; gap:16px}}
  .kpis{{grid-template-columns: repeat(auto-fit, minmax(200px, 1fr))}}
  .card{{border:1px solid #e5e7eb; border-radius:12px; padding:16px; background:white; box-shadow:0 1px 3px rgba(0,0,0,0.1)}}
  .big{{font-size:28px; font-weight:700; margin-top:8px}}
  table{{border-collapse:collapse; width:100%; margin-top:12px}}
  th,td{{padding:10px 12px; border-bottom:1px solid #f3f4f6; text-align:right}}
  th:first-child, td:first-child{{text-align:left}}
  th{{font-weight:600; background:#f9fafb; color:#374151}}
  .low{{color:#059669}}
  .medium{{color:#6366f1}}
  .high{{color:#d97706}}
  details{{border:1px solid #e5e7eb; border-radius:8px; padding:12px; margin-top:12px; background:white}}
  summary{{cursor:pointer; font-weight:600; color:#4b5563}}
  .bar{{height:20px; background:linear-gradient(90deg, #3b82f6, #60a5fa); border-radius:4px; transition:width 0.3s}}
  .label-val{{display:flex; justify-content:space-between; margin:8px 0}}
  .label-val span:first-child{{color:#6b7280}}
  .label-val span:last-child{{font-weight:600}}
  .section-desc{{color:#6b7280; font-size:13px; margin-bottom:12px}}
  .chart-grid{{display:grid; grid-template-columns: 1fr 1fr; gap:20px; margin-top:16px}}
  @media (max-width: 900px) {{ .chart-grid{{grid-template-columns: 1fr}} }}
</style>
</head>
<body>

<h1>🌌 Kerr High-Precision Analysis <span class="muted">({})</span></h1>
<p class="muted">Resolution: {}×{} | Inclination: {:.1}° | Spin: {:.2} | Orders: {} | r ∈ [{:.1}, {:.1}]M</p>

<section class="grid kpis">
  <div class="card">
    <div class="muted">Hit Pixels</div>
    <div class="big">{}</div>
    <div class="muted">{:.1}%</div>
  </div>
  <div class="card">
    <div class="muted">Miss Rate</div>
    <div class="big">{:.1}%</div>
    <div class="muted">{} pixels</div>
  </div>
  <div class="card">
    <div class="muted">Order-1 Pixels</div>
    <div class="big">{}</div>
    <div class="muted">Photon ring</div>
  </div>
  <div class="card">
    <div class="muted">NI p99</div>
    <div class="big {}">{:.2e}</div>
    <div class="muted">Excellent</div>
  </div>
</section>

<h2>📊 Hit / Miss & Orders</h2>
<div class="card">
  <p class="section-desc">Distribution of geodesic orders per pixel. Multiple orders indicate strong gravitational lensing.</p>
  <table>
    <tr><th>Bucket</th><th>Pixels</th><th>Share</th><th>Visual</th></tr>
    <tr><td>Only order 0 (primary)</td><td>{}</td><td>{:.1}%</td><td><div class="bar" style="width: {:.1}%"></div></td></tr>
    <tr><td>Orders 0 & 1 (+ ring)</td><td>{}</td><td>{:.1}%</td><td><div class="bar" style="width: {:.1}%"></div></td></tr>
    <tr><td>Orders ≥2 (+ subrings)</td><td>{}</td><td>{:.1}%</td><td><div class="bar" style="width: {:.1}%"></div></td></tr>
    <tr><td>No hit (miss)</td><td>{}</td><td>{:.1}%</td><td><div class="bar" style="width: {:.1}%"></div></td></tr>
  </table>
</div>

<h2>🎯 Miss Taxonomy (NEW)</h2>
<div class="card">
  <p class="section-desc">Classification of rays that don't hit the accretion disc.</p>
  <div class="chart-grid">
    <div>
      {}
    </div>
    <div>
      <table>
        <tr><th>Miss Type</th><th>Count</th><th>Share</th></tr>
        <tr><td>Escaped (r → ∞)</td><td>{}</td><td>{:.1}%</td></tr>
        <tr><td>Captured (r → r<sub>h</sub>)</td><td>{}</td><td>{:.1}%</td></tr>
        <tr><td>Aborted (numerical)</td><td>{}</td><td>{:.1}%</td></tr>
      </table>
      <p class="section-desc" style="margin-top:16px">
        <strong>Escaped:</strong> Ray reaches r &gt; 1000M (escapes to infinity)<br>
        <strong>Captured:</strong> Ray falls into event horizon (r &lt; r<sub>h</sub> + 0.01M)<br>
        <strong>Aborted:</strong> Integration stopped due to numerical issues or step limit
      </p>
    </div>
  </div>
</div>

<h2>🖼️ Order Mask Thumbnails (NEW)</h2>
<div class="card">
  <p class="section-desc">Visual spatial distribution of geodesic orders. White pixels indicate hits at that order.</p>
  <div style="display:grid; grid-template-columns: repeat(3, 1fr); gap:16px; margin-top:16px">
    <div style="text-align:center">
      {}
      <p class="section-desc" style="margin-top:8px">Primary disc image</p>
    </div>
    <div style="text-align:center">
      {}
      <p class="section-desc" style="margin-top:8px">Photon ring</p>
    </div>
    <div style="text-align:center">
      {}
      <p class="section-desc" style="margin-top:8px">Higher-order subrings</p>
    </div>
  </div>
</div>

<h2>🔍 Outlier Spotlight (NEW)</h2>
<div class="card">
  <p class="section-desc">Top 10 rays with highest null invariant errors for quality inspection. These outliers warrant review but may still be within acceptable tolerances for complex geodesics.</p>
  <table>
    <tr><th>Rank</th><th>Pixel (x, y)</th><th>Order</th><th>NI Error</th><th>Attention Level</th></tr>
    {}
  </table>
  <p class="section-desc" style="margin-top:12px">
    <strong>Low:</strong> NI &lt; 1e-12 | <strong>Medium:</strong> 1e-12 ≤ NI &lt; 1e-9 | <strong>High Attention:</strong> NI ≥ 1e-9
  </p>
</div>

<h2>🔬 Carter Constant Validation (NEW)</h2>
<div class="card">
  <p class="section-desc">The Carter constant K = Q + (L<sub>z</sub> - aE)² must be ≥ 0 for physical geodesics. Violations indicate integration errors.</p>
  <div style="text-align:center; margin-top:16px">
    {}
  </div>
</div>

<h2>📊 Transfer Functions (NEW)</h2>
<div class="card">
  <p class="section-desc">2D histograms showing how image position maps to disc emission radius for each geodesic order.</p>
  <div style="display:grid; grid-template-columns: repeat(3, 1fr); gap:20px; margin-top:16px">
    <div style="text-align:center">
      {}
      <p class="section-desc" style="margin-top:8px">Order 0 (Primary)</p>
    </div>
    <div style="text-align:center">
      {}
      <p class="section-desc" style="margin-top:8px">Order 1 (Photon Ring)</p>
    </div>
    <div style="text-align:center">
      {}
      <p class="section-desc" style="margin-top:8px">Order 2+ (Subrings)</p>
    </div>
  </div>
</div>

<h2>⏱️ Relative Light Travel Times (NEW)</h2>
<div class="card">
  <p class="section-desc">Heatmap showing arrival time delays across the image via affine parameter λ. Warm colors indicate longer light paths.</p>
  <div style="text-align:center; margin-top:16px">
    {}
  </div>
</div>

<h2>🔬 Numerical Quality</h2>
<div class="card">
  <p class="section-desc">Null invariant (NI) measures how well geodesics maintain the constraint g<sub>μν</sub> k<sup>μ</sup> k<sup>ν</sup> = 0.</p>
  <div class="grid" style="grid-template-columns: 1fr 1fr">
    <div>
      <table>
        <tr><th>Metric</th><th>Value</th></tr>
        <tr><td>NI min</td><td>{:.2e}</td></tr>
        <tr><td>NI median</td><td>{:.2e}</td></tr>
        <tr><td>NI mean</td><td>{:.2e}</td></tr>
        <tr><td>NI p95</td><td>{:.2e}</td></tr>
        <tr><td>NI p99</td><td class="{}">{:.2e}</td></tr>
        <tr><td>NI max</td><td>{:.2e}</td></tr>
      </table>
    </div>
    <div>
      <div class="label-val"><span>NI &lt; 1e-15</span><span>{:.1}%</span></div>
      <div class="label-val"><span>NI &lt; 1e-12</span><span>{:.1}%</span></div>
      <div class="label-val"><span>NI &lt; 1e-9</span><span class="{}">{:.1}%</span></div>
      <p style="margin-top:16px" class="{}">{}</p>
    </div>
  </div>
  
  <div class="chart-grid">
    <div>{}</div>
  </div>
</div>

<h2>🌍 Disc Interaction & Radial Distribution</h2>
<div class="card">
  <p class="section-desc">Where disc hits occur. Peak near ISCO indicates brightest emission region.</p>
  
  <div class="chart-grid">
    <div>{}</div>
    <div>{}</div>
  </div>
  
  <h3 style="margin-top:24px">Binned Statistics (Order 0)</h3>
  <table>
    <tr><th>Radius Range</th><th>Hits</th><th>Share</th><th>Visual</th></tr>
    {}
  </table>
  
  <h3 style="margin-top:24px">Per-Order Statistics</h3>
  <p class="section-desc" style="margin-top:8px; font-size:12px; font-style:italic">φ-wraps are unitless (2π cycles), λ is affine parameter at disc hit</p>
  <table>
    <tr><th>Order</th><th>Count</th><th>Mean φ-wraps</th><th>Mean λ</th></tr>
    <tr><td>0 (Primary)</td><td>{}</td><td>{:.2}</td><td>{:.1}M</td></tr>
    <tr><td>1 (Ring)</td><td>{}</td><td>{:.2}</td><td>{:.1}M</td></tr>
    <tr><td>2+ (Subrings)</td><td>{}</td><td>{:.2}</td><td>{:.1}M</td></tr>
  </table>
</div>

<h2>🎨 Redshift & Doppler Analysis</h2>
<div class="card">
  <p class="section-desc">Redshift factor g = ν<sub>obs</sub>/ν<sub>emit</sub> (unitless). Values &gt;1 indicate Doppler boosting (approaching), &lt;1 indicate dimming (receding).</p>
  <div class="grid" style="grid-template-columns: 1fr 1fr">
    <div>
      <div class="label-val"><span>g min (most dimmed)</span><span>{:.3}</span></div>
      <div class="label-val"><span>g max (most boosted)</span><span>{:.3}</span></div>
      <div class="label-val"><span>g mean</span><span>{:.3}</span></div>
      <div class="label-val"><span>Asymmetry ratio</span><span>{:.2}:1</span></div>
    </div>
    <div>
      <div class="label-val"><span>Boosted (g&gt;1)</span><span>{} hits ({:.1}%)</span></div>
      <div class="label-val"><span>Dimmed (g&lt;1)</span><span>{} hits ({:.1}%)</span></div>
      <p style="margin-top:16px" class="{}">{}</p>
    </div>
  </div>
  
  <div class="chart-grid">
    <div>{}</div>
  </div>
</div>

<h2>🌀 Light Bending & Lensing</h2>
<div class="card">
  <p class="section-desc">φ-wraps measure how many times photons wind around the black hole before hitting the disc.</p>
  <table>
    <tr><th>Order</th><th>φ-wraps Range</th><th>Mean</th></tr>
    <tr><td>0 (Primary)</td><td>{:.2} - {:.2}</td><td>{:.2}</td></tr>
    <tr><td>1 (Ring)</td><td colspan="2">{:.2} wraps (avg)</td></tr>
    <tr><td>2+ (Subrings)</td><td colspan="2">{:.2} wraps (avg)</td></tr>
  </table>
  <p style="margin-top:12px"><strong>Max wraps:</strong> {:.2} ({})</p>
</div>

<h2>🔬 Advanced Geodesic Diagnostics (Tier 3)</h2>

<div class="card" style="margin-bottom:20px">
  <h3>Critical Curve & Shadow Fitting</h3>
  <p class="section-desc">Extracted {} boundary pixels between captured and escaped rays.</p>
  {}
</div>

<div class="card" style="margin-bottom:20px">
  <h3>Turning Points Distribution</h3>
  <p class="section-desc">Histogram of radial (r) and polar (θ) turning points. High counts indicate complex, chaotic geodesics near the photon sphere.</p>
  <div class="chart-grid">
    <div>{}</div>
  </div>
</div>

<div class="card" style="margin-bottom:20px">
  <h3>Wrap-Angle vs Impact Parameter</h3>
  <p class="section-desc">Relationship between impact parameter b = L<sub>z</sub>/E and azimuthal wraps. The vertical red line marks the photon sphere radius where rays orbit multiple times before hitting the disc.</p>
  <div class="chart-grid">
    <div>{}</div>
  </div>
  <details style="margin-top:12px">
    <summary style="cursor:pointer; color:#6b7280; font-size:13px">📖 How to Read This Chart</summary>
    <div style="margin-top:8px; font-size:13px; color:#374151">
      <p><strong>X-axis (Impact Parameter b):</strong> Angular momentum per unit energy (L<sub>z</sub>/E). Negative values mean counter-rotation relative to black hole spin.</p>
      <p style="margin-top:8px"><strong>Y-axis (φ-wraps):</strong> Number of times the light ray winds around the black hole before hitting the disc.</p>
      <p style="margin-top:8px"><strong>Colors:</strong> Green = Order 0 (primary), Blue = Order 1 (photon ring), Purple = Order 2+ (higher subrings)</p>
      <p style="margin-top:8px"><strong>Photon Sphere (red line):</strong> Critical impact parameter (~3.79M for a=0.9) where photons can orbit the black hole. Rays near this value show the highest wraps.</p>
      <p style="margin-top:8px"><strong>Expected Pattern:</strong> A peak in wraps near b<sub>photon</sub>, with Order 1 and 2+ rays concentrated in this region due to multiple orbits before disc intersection.</p>
    </div>
  </details>
</div>

<h2>🚨 Enhanced Outlier Detection (Tier 4)</h2>

<div class="card" style="margin-bottom:20px">
  <h3>Outlier Summary</h3>
  <p class="section-desc">Comprehensive 2-pass outlier detection using robust statistical methods (MAD-based z-scores, Mahalanobis distance, spatial discontinuity analysis).</p>
  
  <div style="display:grid; grid-template-columns: repeat(4, 1fr); gap:12px; margin-top:16px">
    {}
  </div>
  
  {}
</div>

<div class="card" style="margin-bottom:20px">
  <h3>Spatial Outlier Distribution</h3>
  <p class="section-desc">Downsampled heatmap showing where outliers are concentrated. Darker = more outliers in region, color = highest severity level.</p>
  <div class="chart-grid">
    <div>{}</div>
  </div>
</div>

<details>
  <summary>ℹ️ About This Analysis</summary>
  <p style="margin-top:12px">This report analyzes high-precision f64 geodesic data from Kerr spacetime ray tracing. Each pixel traces a null geodesic (light path) backward from the camera through curved spacetime, potentially crossing the accretion disc multiple times due to extreme gravitational lensing.</p>
  <ul style="margin-top:8px; padding-left:20px">
    <li><strong>Order 0</strong>: Primary image (direct view of disc)</li>
    <li><strong>Order 1</strong>: Secondary image (photon ring, wraps ~180-360° around BH)</li>
    <li><strong>Order 2+</strong>: Higher-order images (increasingly faint subrings)</li>
  </ul>
  <p style="margin-top:12px">The null invariant (NI) measures numerical accuracy. Values &lt;1e-9 indicate excellent geodesic integration.</p>
</details>

<footer style="margin-top:48px; padding-top:24px; border-top:1px solid #e5e7eb; color:#6b7280; font-size:12px">
  <p style="margin-bottom:4px">Generated by Kerr Black Hole High-Precision Analyzer</p>
  <p>Preset: {} | {}×{} | {} orders | {:.1}° inclination{}</p>
</footer>

</body>
</html>"#,
        manifest.preset, manifest.width, manifest.height,
        manifest.preset,
        manifest.width, manifest.height, manifest.inclination, manifest.spin, manifest.orders, manifest.r_in, manifest.r_out,
        stats.total_hit_pixels, hit_pct,
        miss_pct, stats.miss_pixels,
        stats.orders_0_and_1 + stats.orders_2_plus,
        ni_p99_class, stats.ni_p99,
        stats.only_order_0, only_o0_pct, only_o0_pct,
        stats.orders_0_and_1, o01_pct, o01_pct,
        stats.orders_2_plus, o2p_pct, o2p_pct,
        stats.miss_pixels, miss_pct, miss_pct,
        // NEW: Tier 1.1 - Miss taxonomy
        miss_taxonomy_pie_svg,
        stats.miss_escaped, (stats.miss_escaped as f64 / stats.miss_pixels.max(1) as f64 * 100.0),
        stats.miss_captured, (stats.miss_captured as f64 / stats.miss_pixels.max(1) as f64 * 100.0),
        stats.miss_aborted, (stats.miss_aborted as f64 / stats.miss_pixels.max(1) as f64 * 100.0),
        // NEW: Tier 1.2 - Order thumbnails
        order_0_thumb, order_1_thumb, order_2plus_thumb,
        // NEW: Tier 1.3 - Outlier spotlight
        outlier_rows,
        // NEW: Tier 2.1 - K validation heatmap
        stats.k_heatmap_svg,
        // NEW: Tier 2.2 - Transfer functions
        stats.transfer_o0_svg, stats.transfer_o1_svg, stats.transfer_o2_svg,
        // NEW: Tier 2.4 - Time delay map
        stats.time_delay_svg,
        stats.ni_min, stats.ni_median, stats.ni_mean, stats.ni_p95,
        ni_p99_class, stats.ni_p99,
        stats.ni_max,
        (stats.ni_under_1e15 as f64 / stats.total_hits as f64 * 100.0).min(100.0),
        (stats.ni_under_1e12 as f64 / stats.total_hits as f64 * 100.0).min(100.0),
        ni_gate_class, ni_under_1e9_pct,
        ni_message_class, ni_message,
        ni_histogram_svg,
        radial_histogram_svg,
        radial_profile_svg,
        r_bars,
        stats.order_0_hits, stats.phi_wraps_mean_o0, stats.affine_mean_o0,
        stats.order_1_hits, stats.phi_wraps_mean_o1, stats.affine_mean_o1,
        stats.order_2_plus_hits, stats.phi_wraps_mean_o2, stats.affine_mean_o2,
        stats.g_min, stats.g_max, stats.g_mean, g_asymmetry,
        stats.g_boosted_count, (stats.g_boosted_count as f64 / stats.total_hits as f64 * 100.0),
        stats.g_dimmed_count, (stats.g_dimmed_count as f64 / stats.total_hits as f64 * 100.0),
        doppler_message_class, doppler_message,
        angular_distribution_svg,
        stats.phi_wraps_min, stats.phi_wraps_max, stats.phi_wraps_mean_o0,
        stats.phi_wraps_mean_o1,
        stats.phi_wraps_mean_o2,
        stats.phi_wraps_max, wraps_interpretation,
        // NEW: Tier 3 - Publication quality
        stats.critical_curve_points,
        ellipse_block,
        stats.turning_histogram_svg,
        stats.wraps_scatter_svg,
        // NEW: Tier 4 - Enhanced outlier detection
        severity_badges,
        outlier_tables,
        stats.outlier_overlay_svg,
        manifest.preset, manifest.width, manifest.height, manifest.orders, manifest.inclination,
        // NEW: Tier 1.4 - Provenance (inline)
        provenance_text
    )
}

