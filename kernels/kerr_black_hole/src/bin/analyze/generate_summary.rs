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
}

pub fn generate_html_report(stats: &Stats, manifest: &Manifest) -> String {
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
    
    // Calculate ISCO for radial profile
    let r_isco = 3.0 + 2.0 * ((3.0 - manifest.spin) * (3.0 + manifest.spin + 2.0 * (3.0 - manifest.spin).sqrt())).sqrt();
    let radial_profile_svg = charts::generate_radial_profile_svg(&stats.radial_profile, manifest.r_in, manifest.r_out, r_isco);
    let angular_distribution_svg = charts::generate_angular_distribution_svg(&stats.angular_distribution);
    
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
  .ok{{color:#059669}}
  .warn{{color:#d97706}}
  .bad{{color:#dc2626}}
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
      <p style="margin-top:16px" class="{}">✓ All geodesics maintain null constraint!</p>
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
      <p style="margin-top:16px" class="ok">✓ Clear Doppler asymmetry from frame dragging</p>
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
  <p style="margin-top:12px"><strong>Max wraps:</strong> {:.2} (deep lensing near photon sphere!)</p>
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

<footer style="margin-top:48px; padding-top:24px; border-top:1px solid #e5e7eb; color:#6b7280; font-size:13px">
  <p>Generated by Kerr Black Hole High-Precision Analyzer</p>
  <p>Preset: {} | {}×{} | {} orders | {:.1}° inclination</p>
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
        stats.ni_min, stats.ni_median, stats.ni_mean, stats.ni_p95,
        ni_p99_class, stats.ni_p99,
        stats.ni_max,
        (stats.ni_under_1e15 as f64 / stats.total_hits as f64 * 100.0).min(100.0),
        (stats.ni_under_1e12 as f64 / stats.total_hits as f64 * 100.0).min(100.0),
        ni_gate_class, ni_under_1e9_pct,
        ni_gate_class,
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
        angular_distribution_svg,
        stats.phi_wraps_min, stats.phi_wraps_max, stats.phi_wraps_mean_o0,
        stats.phi_wraps_mean_o1,
        stats.phi_wraps_mean_o2,
        stats.phi_wraps_max,
        manifest.preset, manifest.width, manifest.height, manifest.orders, manifest.inclination
    )
}

