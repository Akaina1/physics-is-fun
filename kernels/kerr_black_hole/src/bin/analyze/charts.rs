// SVG Chart Generation for Analysis Report

use std::f64::consts::PI;

/// High-precision record (for chart computation)
#[derive(Debug)]
pub struct HpRecord {
    pub _pixel_x: u32,
    pub _pixel_y: u32,
    pub r: f64,
    pub phi: f64,
    pub redshift_factor: f64,
    pub _affine_parameter: f64,
    pub phi_wraps: f64,
    pub order: u8,
    pub _hit: bool,
    pub null_invariant_error: f64,
    // NEW: Tier 2 - Additional fields for K validation and transfer functions
    pub energy: f64,
    pub angular_momentum: f64,
    pub carter_q: f64,
    pub impact_parameter: f64,
    // NEW: Tier 3 - Turning points for histogram
    pub turns_r: u8,
    pub turns_theta: u8,
}

/// Chart data computation functions
pub fn compute_ni_histogram(hits: &[&HpRecord]) -> Vec<(f64, f64, usize)> {
    // Log-scale bins from 1e-18 to 1e-3
    let bin_edges: Vec<f64> = vec![
        0.0, 1e-18, 1e-17, 1e-16, 1e-15, 1e-14, 1e-13, 1e-12,
        1e-11, 1e-10, 1e-9, 1e-8, 1e-7, 1e-6, 1e-5, 1e-4, 1e-3, f64::INFINITY
    ];
    
    let mut histogram = Vec::new();
    for i in 0..bin_edges.len() - 1 {
        let start = bin_edges[i];
        let end = bin_edges[i + 1];
        let count = hits.iter()
            .filter(|r| r.null_invariant_error >= start && r.null_invariant_error < end)
            .count();
        histogram.push((start, end, count));
    }
    histogram
}

pub fn compute_radial_histogram(hits: &[&HpRecord], r_in: f64, r_out: f64) -> Vec<(f64, usize, usize, usize)> {
    let num_bins = 40;
    let dr = (r_out - r_in) / num_bins as f64;
    
    let mut histogram = Vec::new();
    for i in 0..num_bins {
        let r_start = r_in + i as f64 * dr;
        let r_end = r_start + dr;
        let r_center = (r_start + r_end) / 2.0;
        
        let order0 = hits.iter().filter(|r| r.order == 0 && r.r >= r_start && r.r < r_end).count();
        let order1 = hits.iter().filter(|r| r.order == 1 && r.r >= r_start && r.r < r_end).count();
        let order2p = hits.iter().filter(|r| r.order >= 2 && r.r >= r_start && r.r < r_end).count();
        
        histogram.push((r_center, order0, order1, order2p));
    }
    histogram
}

pub fn compute_radial_profile(hits: &[&HpRecord], r_in: f64, r_out: f64) -> Vec<(f64, usize, f64, f64)> {
    let num_bins = 30;
    let dr = (r_out - r_in) / num_bins as f64;
    
    let mut profile = Vec::new();
    for i in 0..num_bins {
        let r_start = r_in + i as f64 * dr;
        let r_end = r_start + dr;
        let r_center = (r_start + r_end) / 2.0;
        
        let bin_hits: Vec<_> = hits.iter().filter(|r| r.r >= r_start && r.r < r_end).collect();
        let count = bin_hits.len();
        
        let mean_g = if count > 0 {
            bin_hits.iter().map(|r| r.redshift_factor).sum::<f64>() / count as f64
        } else {
            0.0
        };
        
        let mean_phi = if count > 0 {
            bin_hits.iter().map(|r| r.phi_wraps).sum::<f64>() / count as f64
        } else {
            0.0
        };
        
        profile.push((r_center, count, mean_g, mean_phi));
    }
    profile
}

pub fn compute_angular_distribution(hits: &[&HpRecord]) -> Vec<(f64, usize)> {
    let num_bins = 36;  // 10° per bin
    let dphi = 2.0 * PI / num_bins as f64;
    
    let mut distribution = Vec::new();
    for i in 0..num_bins {
        let phi_start = i as f64 * dphi;
        let phi_end = phi_start + dphi;
        let phi_center = (phi_start + phi_end) / 2.0;
        
        // Count hits in this angular bin (normalize phi to [0, 2π])
        let count = hits.iter()
            .filter(|r| {
                let phi_norm = r.phi.rem_euclid(2.0 * PI);
                phi_norm >= phi_start && phi_norm < phi_end
            })
            .count();
        
        distribution.push((phi_center, count));
    }
    distribution
}

/// Generate NI error histogram (log-scale)
pub fn generate_ni_histogram_svg(histogram: &[(f64, f64, usize)]) -> String {
    let width = 680;
    let height = 250;
    let margin = 50;
    let chart_width = width - 2 * margin;
    let chart_height = height - 2 * margin;
    
    // Find max count for scaling
    let max_count = histogram.iter().map(|(_, _, c)| c).max().unwrap_or(&1);
    
    // Generate bars
    let bar_width = chart_width as f64 / histogram.len() as f64;
    let mut bars = String::new();
    
    for (i, (_start, end, count)) in histogram.iter().enumerate() {
        if *count == 0 {
            continue;
        }
        
        let x = margin as f64 + i as f64 * bar_width;
        let bar_height = (*count as f64 / *max_count as f64) * chart_height as f64;
        let y = margin as f64 + chart_height as f64 - bar_height;
        
        // Color by quality: green (<1e-12), yellow (<1e-9), red (>=1e-9)
        let color = if *end <= 1e-12 {
            "#059669"  // green
        } else if *end <= 1e-9 {
            "#d97706"  // yellow/orange
        } else {
            "#dc2626"  // red
        };
        
        bars.push_str(&format!(
            r##"<rect x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" fill="{}" opacity="0.8"/>"##,
            x, y, bar_width * 0.9, bar_height, color
        ));
    }
    
    // X-axis labels (log scale)
    let labels = vec!["0", "1e-18", "1e-15", "1e-12", "1e-9", "1e-6", "1e-3"];
    let label_positions = vec![0, 1, 4, 7, 10, 13, 16];
    let mut x_labels = String::new();
    
    for (label, &pos) in labels.iter().zip(label_positions.iter()) {
        let x = margin as f64 + pos as f64 * bar_width;
        x_labels.push_str(&format!(
            r##"<text x="{}" y="{}" text-anchor="middle" font-size="11" fill="#6b7280">{}</text>"##,
            x, height - margin + 20, label
        ));
    }
    
    // Y-axis and title
    let title = "Null Invariant Error Distribution (Log Scale)";
    
    format!(r##"<svg width="{}" height="{}" style="background:white; border-radius:8px">
  <text x="{}" y="20" font-size="14" font-weight="600" fill="#374151">{}</text>
  <text x="{}" y="{}" text-anchor="middle" font-size="12" fill="#6b7280">Null Invariant (NI)</text>
  <text x="15" y="{}" text-anchor="middle" font-size="12" fill="#6b7280" transform="rotate(-90, 15, {})">Count</text>
  <line x1="{}" y1="{}" x2="{}" y2="{}" stroke="#e5e7eb" stroke-width="2"/>
  <line x1="{}" y1="{}" x2="{}" y2="{}" stroke="#e5e7eb" stroke-width="2"/>
  {}
  {}
  <!-- Quality threshold lines -->
  <line x1="{}" y1="{}" x2="{}" y2="{}" stroke="#059669" stroke-width="1" stroke-dasharray="4,2" opacity="0.5"/>
  <text x="{}" y="{}" font-size="10" fill="#059669">1e-12</text>
  <line x1="{}" y1="{}" x2="{}" y2="{}" stroke="#d97706" stroke-width="1" stroke-dasharray="4,2" opacity="0.5"/>
  <text x="{}" y="{}" font-size="10" fill="#d97706">1e-9</text>
</svg>"##,
        width, height,
        width / 2, title,
        width / 2, height - 10,
        15, (height + margin) / 2,
        margin, height - margin, width - margin, height - margin,  // x-axis
        margin, margin, margin, height - margin,  // y-axis
        bars,
        x_labels,
        // 1e-12 threshold line (bin 7)
        margin as f64 + 7.0 * bar_width, margin, margin as f64 + 7.0 * bar_width, height - margin,
        margin as f64 + 7.0 * bar_width + 5.0, margin + 15,
        // 1e-9 threshold line (bin 10)
        margin as f64 + 10.0 * bar_width, margin, margin as f64 + 10.0 * bar_width, height - margin,
        margin as f64 + 10.0 * bar_width + 5.0, margin + 15,
    )
}

/// Generate radial hit distribution (stacked by order)
pub fn generate_radial_histogram_svg(histogram: &[(f64, usize, usize, usize)], r_in: f64, r_out: f64) -> String {
    let width = 680;
    let height = 250;
    let margin = 50;
    let chart_width = width - 2 * margin;
    let chart_height = height - 2 * margin;
    
    // Find max total count for scaling
    let max_count = histogram.iter()
        .map(|(_, o0, o1, o2)| o0 + o1 + o2)
        .max()
        .unwrap_or(1);
    
    let bar_width = chart_width as f64 / histogram.len() as f64;
    let mut bars = String::new();
    
    for (i, (_, order0, order1, order2p)) in histogram.iter().enumerate() {
        let x = margin as f64 + i as f64 * bar_width;
        let total = order0 + order1 + order2p;
        
        if total == 0 {
            continue;
        }
        
        // Stacked bars: bottom=order0, middle=order1, top=order2+
        let total_height = (total as f64 / max_count as f64) * chart_height as f64;
        
        let o0_height = (*order0 as f64 / total as f64) * total_height;
        let o1_height = (*order1 as f64 / total as f64) * total_height;
        let o2_height = (*order2p as f64 / total as f64) * total_height;
        
        let y_base = margin as f64 + chart_height as f64;
        
        // Order 0 (blue)
        if o0_height > 0.0 {
            bars.push_str(&format!(
                r##"<rect x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" fill="#3b82f6" opacity="0.8"/>"##,
                x, y_base - o0_height, bar_width * 0.9, o0_height
            ));
        }
        
        // Order 1 (orange)
        if o1_height > 0.0 {
            bars.push_str(&format!(
                r##"<rect x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" fill="#f59e0b" opacity="0.8"/>"##,
                x, y_base - o0_height - o1_height, bar_width * 0.9, o1_height
            ));
        }
        
        // Order 2+ (red)
        if o2_height > 0.0 {
            bars.push_str(&format!(
                r##"<rect x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" fill="#ef4444" opacity="0.8"/>"##,
                x, y_base - o0_height - o1_height - o2_height, bar_width * 0.9, o2_height
            ));
        }
    }
    
    // Legend
    let legend = format!(r##"
  <rect x="{}" y="30" width="12" height="12" fill="#3b82f6" opacity="0.8"/>
  <text x="{}" y="40" font-size="11" fill="#374151">Order 0</text>
  <rect x="{}" y="30" width="12" height="12" fill="#f59e0b" opacity="0.8"/>
  <text x="{}" y="40" font-size="11" fill="#374151">Order 1</text>
  <rect x="{}" y="30" width="12" height="12" fill="#ef4444" opacity="0.8"/>
  <text x="{}" y="40" font-size="11" fill="#374151">Order 2+</text>
"##,
        width - 220, width - 206,
        width - 150, width - 136,
        width - 80, width - 66
    );
    
    format!(r##"<svg width="{}" height="{}" style="background:white; border-radius:8px">
  <text x="{}" y="20" font-size="14" font-weight="600" fill="#374151">Radial Hit Distribution by Order</text>
  <text x="{}" y="{}" text-anchor="middle" font-size="12" fill="#6b7280">Radius (M)</text>
  <text x="15" y="{}" text-anchor="middle" font-size="12" fill="#6b7280" transform="rotate(-90, 15, {})">Hits</text>
  <line x1="{}" y1="{}" x2="{}" y2="{}" stroke="#e5e7eb" stroke-width="2"/>
  <line x1="{}" y1="{}" x2="{}" y2="{}" stroke="#e5e7eb" stroke-width="2"/>
  <text x="{}" y="{}" font-size="11" fill="#6b7280">{:.1}</text>
  <text x="{}" y="{}" font-size="11" fill="#6b7280" text-anchor="end">{:.1}</text>
  {}
  {}
</svg>"##,
        width, height,
        width / 2,
        width / 2, height - 10,
        15, (height + margin) / 2,
        margin, height - margin, width - margin, height - margin,  // x-axis
        margin, margin, margin, height - margin,  // y-axis
        margin, height - margin + 20, r_in,
        width - margin, height - margin + 20, r_out,
        bars,
        legend
    )
}

/// Generate radial profile plot (multi-line chart)
pub fn generate_radial_profile_svg(profile: &[(f64, usize, f64, f64)], r_in: f64, r_out: f64, r_isco: f64) -> String {
    let width = 680;
    let height = 300;
    let margin = 60;
    let chart_width = width - 2 * margin;
    let chart_height = height - 2 * margin;
    
    if profile.is_empty() {
        return String::from("<svg></svg>");
    }
    
    // Normalize data
    let max_hits = profile.iter().map(|(_, h, _, _)| h).max().unwrap_or(&1);
    let max_g = profile.iter().map(|(_, _, g, _)| g).fold(0.0f64, |a, &b| a.max(b)).max(1.0);
    let max_phi = profile.iter().map(|(_, _, _, p)| p).fold(0.0f64, |a, &b| a.max(b)).max(1.0);
    
    // Generate paths
    let mut hits_path = String::from("M");
    let mut g_path = String::from("M");
    let mut phi_path = String::from("M");
    
    for (i, (r, hits, g, phi)) in profile.iter().enumerate() {
        let x = (margin as f64 + ((*r - r_in) / (r_out - r_in)) * chart_width as f64).min(width as f64 - margin as f64 - 3.0);
        let y_hits = margin as f64 + chart_height as f64 - (*hits as f64 / *max_hits as f64) * chart_height as f64;
        let y_g = margin as f64 + chart_height as f64 - (*g / max_g) * chart_height as f64;
        let y_phi = margin as f64 + chart_height as f64 - (*phi / max_phi) * chart_height as f64;
        
        let cmd = if i == 0 { format!("{:.1},{:.1}", x, y_hits) } else { format!(" L{:.1},{:.1}", x, y_hits) };
        hits_path.push_str(&cmd);
        
        let cmd = if i == 0 { format!("{:.1},{:.1}", x, y_g) } else { format!(" L{:.1},{:.1}", x, y_g) };
        g_path.push_str(&cmd);
        
        let cmd = if i == 0 { format!("{:.1},{:.1}", x, y_phi) } else { format!(" L{:.1},{:.1}", x, y_phi) };
        phi_path.push_str(&cmd);
    }
    
    // ISCO marker
    let isco_x = margin as f64 + ((r_isco - r_in) / (r_out - r_in)) * chart_width as f64;
    let isco_line = format!(
        r##"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="#dc2626" stroke-width="1" stroke-dasharray="4,2" opacity="0.7"/>
  <text x="{}" y="{}" font-size="10" fill="#dc2626">ISCO</text>"##,
        isco_x, margin, isco_x, height - margin,
        isco_x + 5.0, margin + 15
    );
    
    // Legend
    let legend = format!(r##"
  <line x1="{}" y1="35" x2="{}" y2="35" stroke="#3b82f6" stroke-width="2"/>
  <text x="{}" y="40" font-size="11" fill="#374151">Hit Count</text>
  <line x1="{}" y1="35" x2="{}" y2="35" stroke="#10b981" stroke-width="2"/>
  <text x="{}" y="40" font-size="11" fill="#374151">Redshift g</text>
  <line x1="{}" y1="35" x2="{}" y2="35" stroke="#f59e0b" stroke-width="2"/>
  <text x="{}" y="40" font-size="11" fill="#374151">φ-wraps</text>
"##,
        width - 280, width - 265, width - 260,
        width - 180, width - 165, width - 160,
        width - 80, width - 65, width - 60
    );
    
    format!(r##"<svg width="{}" height="{}" style="background:white; border-radius:8px">
  <text x="{}" y="20" font-size="14" font-weight="600" fill="#374151">Radial Profile: Multi-Metric Analysis</text>
  <text x="{}" y="{}" text-anchor="middle" font-size="12" fill="#6b7280">Radius (M)</text>
  <text x="15" y="{}" text-anchor="middle" font-size="12" fill="#6b7280" transform="rotate(-90, 15, {})">Normalized</text>
  <line x1="{}" y1="{}" x2="{}" y2="{}" stroke="#e5e7eb" stroke-width="2"/>
  <line x1="{}" y1="{}" x2="{}" y2="{}" stroke="#e5e7eb" stroke-width="2"/>
  <text x="{}" y="{}" font-size="11" fill="#6b7280">{:.1}</text>
  <text x="{}" y="{}" font-size="11" fill="#6b7280" text-anchor="end">{:.1}</text>
  {}
  <path d="{}" fill="none" stroke="#3b82f6" stroke-width="2.5"/>
  <path d="{}" fill="none" stroke="#10b981" stroke-width="2"/>
  <path d="{}" fill="none" stroke="#f59e0b" stroke-width="2"/>
  {}
</svg>"##,
        width, height,
        width / 2,
        width / 2, height - 10,
        15, (height + margin) / 2,
        margin, height - margin, width - margin, height - margin,  // x-axis
        margin, margin, margin, height - margin,  // y-axis
        margin, height - margin + 20, r_in,
        width - margin, height - margin + 20, r_out,
        isco_line,
        hits_path,
        g_path,
        phi_path,
        legend
    )
}

/// Generate angular distribution polar chart
pub fn generate_angular_distribution_svg(distribution: &[(f64, usize)]) -> String {
    let size = 300;
    let center = size as f64 / 2.0;
    let max_radius = center - 40.0;
    
    if distribution.is_empty() {
        return String::from("<svg></svg>");
    }
    
    let max_count = distribution.iter().map(|(_, c)| c).max().unwrap_or(&1);
    
    // Generate polar bars
    let mut bars = String::new();
    let num_bins = distribution.len();
    let angle_step = 2.0 * PI / num_bins as f64;
    
    for (i, (_phi, count)) in distribution.iter().enumerate() {
        if *count == 0 {
            continue;
        }
        
        let radius = (*count as f64 / *max_count as f64) * max_radius;
        let angle1 = i as f64 * angle_step - PI / 2.0;  // Start at top
        let angle2 = angle1 + angle_step;
        
        // Arc path
        let x1 = center + radius * angle1.cos();
        let y1 = center + radius * angle1.sin();
        let x2 = center + radius * angle2.cos();
        let y2 = center + radius * angle2.sin();
        
        // Draw wedge
        bars.push_str(&format!(
            r##"<path d="M{},{} L{},{} A{},{} 0 0,1 {},{} Z" fill="#3b82f6" opacity="0.7" stroke="#fff" stroke-width="1"/>"##,
            center, center, x1, y1, radius, radius, x2, y2
        ));
    }
    
    // Circular grid lines
    let grid = format!(r##"
  <circle cx="{}" cy="{}" r="{}" fill="none" stroke="#e5e7eb" stroke-width="1"/>
  <circle cx="{}" cy="{}" r="{}" fill="none" stroke="#e5e7eb" stroke-width="1"/>
  <circle cx="{}" cy="{}" r="{}" fill="none" stroke="#e5e7eb" stroke-width="1"/>
"##,
        center, center, max_radius * 0.33,
        center, center, max_radius * 0.66,
        center, center, max_radius
    );
    
    // Cardinal direction labels
    let labels = format!(r##"
  <text x="{}" y="{}" text-anchor="middle" font-size="12" fill="#374151" font-weight="600">0°</text>
  <text x="{}" y="{}" text-anchor="start" font-size="12" fill="#374151" font-weight="600">90°</text>
  <text x="{}" y="{}" text-anchor="middle" font-size="12" fill="#374151" font-weight="600">180°</text>
  <text x="{}" y="{}" text-anchor="end" font-size="12" fill="#374151" font-weight="600">270°</text>
"##,
        center, center - max_radius - 15.0,
        center + max_radius + 10.0, center + 5.0,
        center, center + max_radius + 20.0,
        center - max_radius - 10.0, center + 5.0
    );
    
    format!(r##"<svg width="{}" height="{}" style="background:white; border-radius:8px">
  <text x="{}" y="10" text-anchor="middle" font-size="14" font-weight="600" fill="#374151">Angular Distribution (φ)</text>
  <text x="{}" y="{}" text-anchor="middle" font-size="11" fill="#6b7280">Frame-dragging asymmetry</text>
  {}
  {}
  {}
</svg>"##,
        size, size + 30,
        center,
        center, size + 15,
        grid,
        bars,
        labels
    )
}

// ============================================================================
// TIER 1: MISS TAXONOMY PIE CHART
// ============================================================================

/// Generate SVG pie chart for miss taxonomy (escaped/captured/aborted)
pub fn generate_miss_taxonomy_pie(
    escaped: usize,
    captured: usize,
    aborted: usize,
    total_miss: usize,
) -> String {
    if total_miss == 0 {
        return String::from("<svg width=\"300\" height=\"200\"><text x=\"150\" y=\"100\" text-anchor=\"middle\" fill=\"#6b7280\">No misses</text></svg>");
    }
    
    let size = 300;
    let center = size / 2;
    let radius = 100;
    
    // Calculate percentages and angles
    let escaped_pct = escaped as f64 / total_miss as f64;
    let captured_pct = captured as f64 / total_miss as f64;
    let aborted_pct = aborted as f64 / total_miss as f64;
    
    // Colors: blue (escaped), red (captured), gray (aborted)
    let colors = ["#3B82F6", "#EF4444", "#6B7280"];
    let labels = ["Escaped", "Captured", "Aborted"];
    let counts = [escaped, captured, aborted];
    let percentages = [escaped_pct, captured_pct, aborted_pct];
    
    // Generate pie slices
    let mut current_angle = 0.0; // Start at right (3 o'clock) for better label visibility
    let mut slices = String::new();
    let mut legend_items = String::new();
    
    for i in 0..3 {
        if counts[i] == 0 {
            continue;
        }
        
        let angle_deg = percentages[i] * 360.0;
        let end_angle = current_angle + angle_deg;
        
        // Convert to radians
        let start_rad = current_angle * PI / 180.0;
        let end_rad = end_angle * PI / 180.0;
        
        // Calculate path
        let x1 = center as f64 + radius as f64 * start_rad.cos();
        let y1 = center as f64 + radius as f64 * start_rad.sin();
        let x2 = center as f64 + radius as f64 * end_rad.cos();
        let y2 = center as f64 + radius as f64 * end_rad.sin();
        
        let large_arc = if angle_deg > 180.0 { 1 } else { 0 };
        
        slices.push_str(&format!(
            r##"<path d="M {} {} L {:.2} {:.2} A {} {} 0 {} 1 {:.2} {:.2} Z" fill="{}" stroke="white" stroke-width="2"/>"##,
            center, center, x1, y1, radius, radius, large_arc, x2, y2, colors[i]
        ));
        
        // Add percentage label in slice (only if slice is >8%)
        if percentages[i] > 0.08 {  // Only show label if slice is >8%
            let mid_angle = (start_rad + end_rad) / 2.0;
            let label_radius = if percentages[i] > 0.6 {
                // For very large slices, put label closer to center
                radius as f64 * 0.5
            } else {
                // For smaller slices, put label at 70% radius
                radius as f64 * 0.7
            };
            let label_x = center as f64 + label_radius * mid_angle.cos();
            let label_y = center as f64 + label_radius * mid_angle.sin();
            
            // Use contrasting colors: white text with black outline for visibility
            slices.push_str(&format!(
                r##"<text x="{:.1}" y="{:.1}" text-anchor="middle" dominant-baseline="middle" font-size="16" font-weight="700" fill="white" stroke="#000" stroke-width="2" paint-order="stroke">{:.1}%</text>"##,
                label_x, label_y, percentages[i] * 100.0
            ));
        }
        
        // Legend item
        let legend_y = 250 + i * 25;
        legend_items.push_str(&format!(
            r##"<rect x="20" y="{}" width="16" height="16" fill="{}" rx="2"/><text x="42" y="{}" font-size="13" fill="#374151">{}: {} ({:.1}%)</text>"##,
            legend_y, colors[i], legend_y + 12, labels[i], counts[i], percentages[i] * 100.0
        ));
        
        current_angle = end_angle;
    }
    
    format!(r##"<svg width="{}" height="340" xmlns="http://www.w3.org/2000/svg">
  <text x="{}" y="20" text-anchor="middle" font-size="14" font-weight="600" fill="#374151">Miss Taxonomy</text>
  <text x="{}" y="35" text-anchor="middle" font-size="11" fill="#6b7280">{} total misses</text>
  {}
  {}
</svg>"##,
        size, center, center, total_miss, slices, legend_items
    )
}

// ============================================================================
// TIER 1: ORDER MASK THUMBNAILS
// ============================================================================

/// Generate binary thumbnail showing which pixels hit at a specific order
/// Downsamples the image by binning (e.g., 8×8 blocks → 1 pixel)
pub fn generate_order_thumbnail_svg(
    width: u32,
    height: u32,
    pixel_orders: &[Option<u8>],  // None = miss, Some(order_mask) = bitmask of orders that hit
    target_order: u8,
    downsample_factor: u32,  // e.g., 8 means 8×8 → 1 pixel
) -> String {
    let thumb_width = (width + downsample_factor - 1) / downsample_factor;
    let thumb_height = (height + downsample_factor - 1) / downsample_factor;
    
    // Compute SVG size (constrain to reasonable dimensions)
    let svg_width = (thumb_width * 2).min(200);
    let svg_height = (thumb_height * 2).min(120);
    let pixel_size = svg_width as f64 / thumb_width as f64;
    
    let mut rects = String::new();
    let mut hit_count = 0;
    let mut total_count = 0;
    
    for ty in 0..thumb_height {
        for tx in 0..thumb_width {
            // Sample block: check if any pixel in this block has target_order
            let mut block_has_order = false;
            
            for dy in 0..downsample_factor {
                for dx in 0..downsample_factor {
                    let px = tx * downsample_factor + dx;
                    let py = ty * downsample_factor + dy;
                    
                    if px >= width || py >= height {
                        continue;
                    }
                    
                    let idx = (py * width + px) as usize;
                    if idx < pixel_orders.len() {
                        if let Some(order_mask) = pixel_orders[idx] {
                            // Check if the target order bit is set in the bitmask
                            if target_order == 2 {
                                // Special case: order 2+ means any bit >= 2 is set
                                if order_mask >= 0b100 {
                                    block_has_order = true;
                                    break;
                                }
                            } else {
                                // Check if the specific order bit is set
                                let bit = 1 << target_order;
                                if (order_mask & bit) != 0 {
                                    block_has_order = true;
                                    break;
                                }
                            }
                        }
                    }
                }
                if block_has_order {
                    break;
                }
            }
            
            total_count += 1;
            if block_has_order {
                hit_count += 1;
                // Draw white pixel for hit
                let x = tx as f64 * pixel_size;
                let y = ty as f64 * pixel_size;
                rects.push_str(&format!(
                    r#"<rect x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" fill="white"/>"#,
                    x, y, pixel_size, pixel_size
                ));
            }
        }
    }
    
    let title = if target_order == 2 {
        "Order 2+".to_string()
    } else {
        format!("Order {}", target_order)
    };
    
    let coverage = hit_count as f64 / total_count as f64 * 100.0;
    
    format!("<svg width=\"{}\" height=\"{}\" xmlns=\"http://www.w3.org/2000/svg\">
  <g transform=\"translate(0, 20)\">
    <rect width=\"{}\" height=\"{}\" fill=\"#111827\"/>
    {}
  </g>
  <text x=\"{}\" y=\"12\" text-anchor=\"middle\" font-size=\"11\" font-weight=\"600\" fill=\"#374151\">{}</text>
  <text x=\"{}\" y=\"{}\" text-anchor=\"middle\" font-size=\"10\" fill=\"#6b7280\">{:.1}% coverage</text>
</svg>",
        svg_width, svg_height + 50,
        svg_width, svg_height,
        rects,
        svg_width / 2, title,
        svg_width / 2, svg_height + 38, coverage
    )
}

// ============================================================================
// TIER 2: K VALIDATION HEATMAP
// ============================================================================

/// Generate K (Carter constant) validation heatmap
/// K = Q + (L_z - aE)² should always be ≥ 0 for physical geodesics
pub fn generate_k_heatmap_svg(
    hits: &[&HpRecord],
    spin: f64,
    width: u32,
    height: u32,
    downsample: (usize, usize),  // Target dimensions (e.g., 400, 225)
) -> String {
    let (target_w, target_h) = downsample;
    let block_w = (width as usize + target_w - 1) / target_w;
    let block_h = (height as usize + target_h - 1) / target_h;
    
    // Build grid: accumulate K values per block
    let mut grid: Vec<Vec<Vec<f64>>> = vec![vec![Vec::new(); target_w]; target_h];
    
    for hit in hits {
        let bx = (hit._pixel_x as usize / block_w).min(target_w - 1);
        let by = (hit._pixel_y as usize / block_h).min(target_h - 1);
        
        // Compute K = Q + (L_z - a*E)²
        let e = hit.energy;
        let lz = hit.angular_momentum;
        let q = hit.carter_q;
        let k = q + (lz - spin * e).powi(2);
        
        grid[by][bx].push(k);
    }
    
    // Compute average K per block and track min/max
    let mut k_min = f64::INFINITY;
    let mut k_max = f64::NEG_INFINITY;
    let mut k_grid: Vec<Vec<Option<f64>>> = vec![vec![None; target_w]; target_h];
    
    for by in 0..target_h {
        for bx in 0..target_w {
            if !grid[by][bx].is_empty() {
                let avg = grid[by][bx].iter().sum::<f64>() / grid[by][bx].len() as f64;
                k_grid[by][bx] = Some(avg);
                k_min = k_min.min(avg);
                k_max = k_max.max(avg);
            }
        }
    }
    
    // Generate SVG heatmap
    let svg_w = 600;
    let svg_h = (svg_w as f64 * target_h as f64 / target_w as f64) as usize;
    let pixel_w = svg_w as f64 / target_w as f64;
    let pixel_h = svg_h as f64 / target_h as f64;
    
    let mut rects = String::new();
    let mut negative_count = 0;
    let mut total_count = 0;
    
    for by in 0..target_h {
        for bx in 0..target_w {
            if let Some(k) = k_grid[by][bx] {
                total_count += 1;
                
                // Color scale: green (K>0), yellow (K≈0), red (K<0)
                let color = if k < -1e-12 {
                    negative_count += 1;
                    "#EF4444"  // Red: violation
                } else if k.abs() < 1e-12 {
                    "#FCD34D"  // Yellow: roundoff
                } else {
                    // Green gradient based on log(K)
                    let intensity = (k.log10().max(-15.0) / -15.0 * 200.0) as u8;
                    rects.push_str(&format!(
                        r#"<rect x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" fill="rgb(34,{},34)"/>"#,
                        bx as f64 * pixel_w, by as f64 * pixel_h, pixel_w, pixel_h, 197 - intensity
                    ));
                    continue;
                };
                
                rects.push_str(&format!(
                    r#"<rect x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" fill="{}"/>"#,
                    bx as f64 * pixel_w, by as f64 * pixel_h, pixel_w, pixel_h, color
                ));
            }
        }
    }
    
    let status = if negative_count == 0 {
        "✓ All geodesics physical (K ≥ 0)"
    } else {
        "⚠ Physics violations detected"
    };
    
    format!("<svg width=\"{}\" height=\"{}\" xmlns=\"http://www.w3.org/2000/svg\">
  <text x=\"{}\" y=\"20\" text-anchor=\"middle\" font-size=\"14\" font-weight=\"600\" fill=\"#374151\">Carter Constant (K) Validation</text>
  <text x=\"{}\" y=\"35\" text-anchor=\"middle\" font-size=\"11\" fill=\"#6b7280\">{} | {} blocks sampled | {} negative</text>
  <g transform=\"translate(0, 50)\">
    {}
  </g>
  <g transform=\"translate(20, {})\">
    <text x=\"0\" y=\"0\" font-size=\"11\" fill=\"#374151\">Legend:</text>
    <rect x=\"0\" y=\"5\" width=\"20\" height=\"10\" fill=\"#22C55E\" rx=\"2\"/>
    <text x=\"25\" y=\"14\" font-size=\"10\" fill=\"#6b7280\">K &gt; 0 (physical)</text>
    <rect x=\"120\" y=\"5\" width=\"20\" height=\"10\" fill=\"#FCD34D\" rx=\"2\"/>
    <text x=\"145\" y=\"14\" font-size=\"10\" fill=\"#6b7280\">K ≈ 0 (roundoff)</text>
    <rect x=\"240\" y=\"5\" width=\"20\" height=\"10\" fill=\"#EF4444\" rx=\"2\"/>
    <text x=\"265\" y=\"14\" font-size=\"10\" fill=\"#6b7280\">K &lt; 0 (violation)</text>
  </g>
</svg>",
        svg_w, svg_h + 100,
        svg_w / 2, svg_w / 2,
        status, total_count, negative_count,
        rects,
        svg_h + 65
    )
}

// ============================================================================
// TIER 2: TRANSFER FUNCTION 2D HISTOGRAM
// ============================================================================

/// Generate transfer function 2D histogram (image radius → emission radius)
/// Shows how rays map from observer screen to disc
pub fn generate_transfer_function_svg(
    hits: &[&HpRecord],
    order: u8,
    r_in: f64,
    r_out: f64,
    r_isco: f64,
    image_center: (f64, f64),  // Center of image in pixels
) -> String {
    // Filter by order (use >= 2 for "2+" category)
    let order_hits: Vec<_> = if order == 2 {
        hits.iter().filter(|h| h.order >= 2).copied().collect()
    } else {
        hits.iter().filter(|h| h.order == order).copied().collect()
    };
    
    if order_hits.is_empty() {
        return format!("<svg width=\"400\" height=\"400\"><text x=\"200\" y=\"200\" text-anchor=\"middle\" fill=\"#6b7280\">No hits for order {}</text></svg>", order);
    }
    
    // Compute image radius for each hit (distance from image center)
    let data: Vec<(f64, f64)> = order_hits.iter().map(|h| {
        let dx = h._pixel_x as f64 - image_center.0;
        let dy = h._pixel_y as f64 - image_center.1;
        let image_r = (dx * dx + dy * dy).sqrt();
        let emission_r = h.r;
        (image_r, emission_r)
    }).collect();
    
    // Determine image radius range
    let image_r_max = data.iter().map(|(ir, _)| *ir).fold(f64::NEG_INFINITY, f64::max);
    
    // Create 2D histogram: 50×50 bins
    let bins = 50;
    let mut grid: Vec<Vec<usize>> = vec![vec![0; bins]; bins];
    
    for (image_r, emission_r) in &data {
        let ix = ((*image_r / image_r_max) * (bins as f64)).floor() as usize;
        let iy = (((*emission_r - r_in) / (r_out - r_in)) * (bins as f64)).floor() as usize;
        
        if ix < bins && iy < bins {
            grid[bins - 1 - iy][ix] += 1;  // Flip y for bottom-to-top
        }
    }
    
    // Find max count for color scaling
    let max_count = grid.iter().flatten().max().copied().unwrap_or(1);
    
    // Generate heatmap SVG
    let svg_w = 400;
    let svg_h = 400;
    let margin = 50;
    let plot_w = svg_w - 2 * margin;
    let plot_h = svg_h - 2 * margin;
    let cell_w = plot_w as f64 / bins as f64;
    let cell_h = plot_h as f64 / bins as f64;
    
    let mut rects = String::new();
    
    for (iy, row) in grid.iter().enumerate() {
        for (ix, &count) in row.iter().enumerate() {
            if count > 0 {
                // Log-scale color intensity
                let intensity = ((count as f64).ln() / (max_count as f64).ln() * 200.0) as u8;
                let color = format!("rgb({},{},{})", 55 + intensity, 55 + intensity, 200);
                
                let x = margin + (ix as f64 * cell_w) as usize;
                let y = margin + (iy as f64 * cell_h) as usize;
                
                rects.push_str(&format!(
                    r#"<rect x="{}" y="{}" width="{:.1}" height="{:.1}" fill="{}" />"#,
                    x, y, cell_w, cell_h, color
                ));
            }
        }
    }
    
    // Add ISCO line if in range
    let isco_y = if r_isco >= r_in && r_isco <= r_out {
        let frac = (r_isco - r_in) / (r_out - r_in);
        Some(margin + plot_h - (frac * plot_h as f64) as usize)
    } else {
        None
    };
    
    let isco_line = if let Some(y) = isco_y {
        format!("<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"#EF4444\" stroke-width=\"2\" stroke-dasharray=\"5,5\"/><text x=\"{}\" y=\"{}\" font-size=\"10\" fill=\"#EF4444\">ISCO</text>",
            margin, y, margin + plot_w, y, margin + plot_w + 5, y + 4)
    } else {
        String::new()
    };
    
    let order_label = if order == 2 { "2+".to_string() } else { order.to_string() };
    
    format!("<svg width=\"{}\" height=\"{}\" xmlns=\"http://www.w3.org/2000/svg\">
  <text x=\"{}\" y=\"20\" text-anchor=\"middle\" font-size=\"14\" font-weight=\"600\" fill=\"#374151\">Transfer Function — Order {}</text>
  <text x=\"{}\" y=\"35\" text-anchor=\"middle\" font-size=\"11\" fill=\"#6b7280\">{} rays</text>
  
  <!-- Heatmap -->
  <rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"#111827\" />
  {}
  {}
  
  <!-- Axes -->
  <line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"#6b7280\" stroke-width=\"2\"/>
  <line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"#6b7280\" stroke-width=\"2\"/>
  
  <!-- Labels -->
  <text x=\"{}\" y=\"{}\" text-anchor=\"middle\" font-size=\"11\" fill=\"#374151\">Image Radius (px)</text>
  <text x=\"15\" y=\"{}\" text-anchor=\"middle\" font-size=\"11\" fill=\"#374151\" transform=\"rotate(-90, 15, {})\">Emission Radius (M)</text>
  
  <!-- Axis ticks -->
  <text x=\"{}\" y=\"{}\" text-anchor=\"middle\" font-size=\"9\" fill=\"#6b7280\">0</text>
  <text x=\"{}\" y=\"{}\" text-anchor=\"middle\" font-size=\"9\" fill=\"#6b7280\">{:.0}</text>
  <text x=\"{}\" y=\"{}\" text-anchor=\"end\" font-size=\"9\" fill=\"#6b7280\">{:.1}</text>
  <text x=\"{}\" y=\"{}\" text-anchor=\"end\" font-size=\"9\" fill=\"#6b7280\">{:.1}</text>
</svg>",
        svg_w, svg_h + 20,
        svg_w / 2, order_label, svg_w / 2, order_hits.len(),
        margin, margin, plot_w, plot_h,
        rects,
        isco_line,
        margin, margin + plot_h, margin + plot_w, margin + plot_h,
        margin, margin, margin, margin + plot_h,
        svg_w / 2, svg_h + 10,
        svg_h / 2, svg_h / 2,
        margin, margin + plot_h + 15,
        margin + plot_w, margin + plot_h + 15, image_r_max,
        margin - 5, margin + plot_h, r_out,
        margin - 5, margin, r_in
    )
}

// ============================================================================
// TIER 2: TIME DELAY MAP HEATMAP
// ============================================================================

/// Generate time delay map showing relative light travel times via affine parameter
pub fn generate_time_delay_heatmap_svg(
    hits: &[&HpRecord],
    width: u32,
    height: u32,
    downsample: (usize, usize),  // Target dimensions (e.g., 400, 225)
) -> String {
    let (target_w, target_h) = downsample;
    let block_w = (width as usize + target_w - 1) / target_w;
    let block_h = (height as usize + target_h - 1) / target_h;
    
    // Build grid: accumulate affine parameter values per block
    let mut grid: Vec<Vec<Vec<f64>>> = vec![vec![Vec::new(); target_w]; target_h];
    
    for hit in hits {
        let bx = (hit._pixel_x as usize / block_w).min(target_w - 1);
        let by = (hit._pixel_y as usize / block_h).min(target_h - 1);
        grid[by][bx].push(hit._affine_parameter);
    }
    
    // Compute average affine parameter per block
    let mut lambda_grid: Vec<Vec<Option<f64>>> = vec![vec![None; target_w]; target_h];
    let mut lambda_min = f64::INFINITY;
    let mut lambda_max = f64::NEG_INFINITY;
    
    for by in 0..target_h {
        for bx in 0..target_w {
            if !grid[by][bx].is_empty() {
                let avg = grid[by][bx].iter().sum::<f64>() / grid[by][bx].len() as f64;
                lambda_grid[by][bx] = Some(avg);
                lambda_min = lambda_min.min(avg);
                lambda_max = lambda_max.max(avg);
            }
        }
    }
    
    // Normalize: relative delay from minimum
    let svg_w = 600;
    let svg_h = (svg_w as f64 * target_h as f64 / target_w as f64) as usize;
    let pixel_w = svg_w as f64 / target_w as f64;
    let pixel_h = svg_h as f64 / target_h as f64;
    
    let mut rects = String::new();
    let range = lambda_max - lambda_min;
    
    for by in 0..target_h {
        for bx in 0..target_w {
            if let Some(lambda) = lambda_grid[by][bx] {
                // Normalize to [0, 1]
                let normalized = if range > 0.0 { (lambda - lambda_min) / range } else { 0.0 };
                
                // Warm colors: earlier arrival (blue) → later arrival (red/yellow)
                let (r, g, b) = if normalized < 0.5 {
                    // Blue to cyan to green
                    let t = normalized * 2.0;
                    let r = 0;
                    let g = (t * 180.0) as u8 + 75;
                    let b = ((1.0 - t) * 200.0) as u8 + 55;
                    (r, g, b)
                } else {
                    // Green to yellow to red
                    let t = (normalized - 0.5) * 2.0;
                    let r = (t * 200.0) as u8 + 55;
                    let g = ((1.0 - t) * 180.0) as u8 + 75;
                    let b = 0;
                    (r, g, b)
                };
                
                rects.push_str(&format!(
                    r#"<rect x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" fill="rgb({},{},{})"/>"#,
                    bx as f64 * pixel_w, by as f64 * pixel_h, pixel_w, pixel_h, r, g, b
                ));
            }
        }
    }
    
    format!("<svg width=\"{}\" height=\"{}\" xmlns=\"http://www.w3.org/2000/svg\">
  <text x=\"{}\" y=\"20\" text-anchor=\"middle\" font-size=\"14\" font-weight=\"600\" fill=\"#374151\">Relative Light Travel Time (Affine Parameter)</text>
  <text x=\"{}\" y=\"35\" text-anchor=\"middle\" font-size=\"11\" fill=\"#6b7280\">Δλ: {:.1}M — {:.1}M (range: {:.1}M)</text>
  <g transform=\"translate(0, 50)\">
    {}
  </g>
  <g transform=\"translate(20, {})\">
    <text x=\"0\" y=\"0\" font-size=\"11\" fill=\"#374151\">Color Scale:</text>
    <defs>
      <linearGradient id=\"timeGradient\" x1=\"0%\" y1=\"0%\" x2=\"100%\" y2=\"0%\">
        <stop offset=\"0%\" style=\"stop-color:rgb(0,75,255);stop-opacity:1\" />
        <stop offset=\"50%\" style=\"stop-color:rgb(0,255,0);stop-opacity:1\" />
        <stop offset=\"100%\" style=\"stop-color:rgb(255,75,0);stop-opacity:1\" />
      </linearGradient>
    </defs>
    <rect x=\"0\" y=\"5\" width=\"200\" height=\"15\" fill=\"url(#timeGradient)\" rx=\"2\"/>
    <text x=\"0\" y=\"30\" font-size=\"10\" fill=\"#6b7280\">Earlier</text>
    <text x=\"200\" y=\"30\" text-anchor=\"end\" font-size=\"10\" fill=\"#6b7280\">Later</text>
  </g>
</svg>",
        svg_w, svg_h + 100,
        svg_w / 2, svg_w / 2,
        lambda_min, lambda_max, range,
        rects,
        svg_h + 65
    )
}

// ============================================================================
// TIER 3: TURNING-POINT HISTOGRAMS
// ============================================================================

pub fn generate_turning_points_histogram_svg(
    hits: &[&HpRecord],
) -> String {
    let width = 600;
    let height = 300;
    let margin = 60;
    let chart_w = (width - margin * 2) / 2 - 20;
    let chart_h = height - margin * 2;
    
    // Bin turning points: 0-10+
    let mut r_bins = vec![0usize; 11]; // bins 0-9, 10 is "10+"
    let mut theta_bins = vec![0usize; 11];
    
    for h in hits {
        let r_idx = (h.turns_r as usize).min(10);
        let theta_idx = (h.turns_theta as usize).min(10);
        r_bins[r_idx] += 1;
        theta_bins[theta_idx] += 1;
    }
    
    let max_r = r_bins.iter().copied().max().unwrap_or(1);
    let max_theta = theta_bins.iter().copied().max().unwrap_or(1);
    let max_count = max_r.max(max_theta).max(1);
    
    // Generate bars for turns_r
    let bar_w = chart_w as f64 / 11.0;
    let bar_width_ratio = 0.65; // Reduced from 0.85 to add more spacing (35% gap instead of 15%)
    let mut r_bars = String::new();
    for (i, &count) in r_bins.iter().enumerate() {
        if count > 0 {
            let bar_h = (count as f64 / max_count as f64) * chart_h as f64;
            let x = i as f64 * bar_w;
            let y = chart_h as f64 - bar_h;
            r_bars.push_str(&format!(
                "    <rect x=\"{:.1}\" y=\"{:.1}\" width=\"{:.1}\" height=\"{:.1}\" fill=\"#3B82F6\" opacity=\"0.8\" rx=\"2\"/>\n",
                x, y, bar_w * bar_width_ratio, bar_h
            ));
            r_bars.push_str(&format!(
                "    <text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" font-size=\"10\" fill=\"#374151\">{}</text>\n",
                x + bar_w * bar_width_ratio * 0.5, y - 5.0, count
            ));
        }
    }
    
    // Generate bars for turns_theta
    let mut theta_bars = String::new();
    for (i, &count) in theta_bins.iter().enumerate() {
        if count > 0 {
            let bar_h = (count as f64 / max_count as f64) * chart_h as f64;
            let x = i as f64 * bar_w;
            let y = chart_h as f64 - bar_h;
            theta_bars.push_str(&format!(
                "    <rect x=\"{:.1}\" y=\"{:.1}\" width=\"{:.1}\" height=\"{:.1}\" fill=\"#8B5CF6\" opacity=\"0.8\" rx=\"2\"/>\n",
                x, y, bar_w * bar_width_ratio, bar_h
            ));
            theta_bars.push_str(&format!(
                "    <text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" font-size=\"10\" fill=\"#374151\">{}</text>\n",
                x + bar_w * bar_width_ratio * 0.5, y - 5.0, count
            ));
        }
    }
    
    // X-axis labels
    let mut r_labels = String::new();
    let mut theta_labels = String::new();
    for i in 0..=10 {
        let x = i as f64 * bar_w + bar_w * bar_width_ratio * 0.5;
        let label = if i == 10 { "10+".to_string() } else { i.to_string() };
        r_labels.push_str(&format!(
            "    <text x=\"{:.1}\" y=\"{}\" text-anchor=\"middle\" font-size=\"10\" fill=\"#6b7280\">{}</text>\n",
            x, chart_h + 20, label
        ));
        theta_labels.push_str(&format!(
            "    <text x=\"{:.1}\" y=\"{}\" text-anchor=\"middle\" font-size=\"10\" fill=\"#6b7280\">{}</text>\n",
            x, chart_h + 20, label
        ));
    }
    
    format!("<svg width=\"{}\" height=\"{}\" xmlns=\"http://www.w3.org/2000/svg\">
  <text x=\"{}\" y=\"20\" text-anchor=\"middle\" font-size=\"14\" font-weight=\"600\" fill=\"#374151\">Turning Points Distribution</text>
  
  <!-- Radial Turning Points (turns_r) -->
  <g transform=\"translate({}, 50)\">
    <text x=\"{}\" y=\"0\" text-anchor=\"middle\" font-size=\"12\" fill=\"#3B82F6\" font-weight=\"500\">Radial (r)</text>
    <g transform=\"translate(0, 20)\">
{}
{}
      <line x1=\"0\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"#E5E7EB\" stroke-width=\"1\"/>
      <text x=\"{}\" y=\"{}\" text-anchor=\"middle\" font-size=\"10\" fill=\"#6b7280\">Turning points</text>
    </g>
  </g>
  
  <!-- Polar Turning Points (turns_theta) -->
  <g transform=\"translate({}, 50)\">
    <text x=\"{}\" y=\"0\" text-anchor=\"middle\" font-size=\"12\" fill=\"#8B5CF6\" font-weight=\"500\">Polar (θ)</text>
    <g transform=\"translate(0, 20)\">
{}
{}
      <line x1=\"0\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"#E5E7EB\" stroke-width=\"1\"/>
      <text x=\"{}\" y=\"{}\" text-anchor=\"middle\" font-size=\"10\" fill=\"#6b7280\">Turning points</text>
    </g>
  </g>
</svg>",
        width, height,
        width / 2,
        margin, chart_w as f64 / 2.0,
        r_bars, r_labels,
        chart_h, chart_w, chart_h,
        chart_w as f64 / 2.0, chart_h + 35,
        margin + chart_w + 40, chart_w as f64 / 2.0,
        theta_bars, theta_labels,
        chart_h, chart_w, chart_h,
        chart_w as f64 / 2.0, chart_h + 35
    )
}

// ============================================================================
// TIER 3: WRAP-ANGLE VS IMPACT PARAMETER SCATTER PLOT
// ============================================================================

pub fn generate_wraps_vs_impact_scatter_svg(
    hits: &[&HpRecord],
    spin: f64,
) -> String {
    let width = 600;
    let height = 400;
    let margin = 60;
    let chart_w = width - margin * 2;
    let chart_h = height - margin * 2;
    
    // Collect impact parameter b and phi-wraps for each hit
    let points: Vec<(f64, f64, u8)> = hits.iter()
        .filter_map(|h| {
            if h.impact_parameter.abs() > 1e-12 && h.phi_wraps.is_finite() {
                Some((h.impact_parameter, h.phi_wraps, h.order))
            } else {
                None
            }
        })
        .collect();
    
    if points.is_empty() {
        return format!("<svg width=\"{}\" height=\"{}\" xmlns=\"http://www.w3.org/2000/svg\">
  <text x=\"{}\" y=\"{}\" text-anchor=\"middle\" fill=\"#6b7280\">No valid data for wrap-angle analysis</text>
</svg>", width, height, width / 2, height / 2);
    }
    
    // Find data bounds
    let b_min = points.iter().map(|(b, _, _)| *b).fold(f64::INFINITY, f64::min);
    let b_max = points.iter().map(|(b, _, _)| *b).fold(f64::NEG_INFINITY, f64::max);
    let wraps_max = points.iter().map(|(_, w, _)| *w).fold(0.0, f64::max);
    
    // Add padding
    let b_range = (b_max - b_min).max(1.0);
    let b_min_plot = b_min - b_range * 0.1;
    let b_max_plot = b_max + b_range * 0.1;
    let wraps_range = wraps_max + 1.0;
    
    // Photon sphere radius for Kerr (approximate for visualization)
    // For a=0: r_ph = 3M, b_crit ≈ 3√3 M ≈ 5.196
    // For spinning: more complex, use simple approximation
    let b_photon_approx = 5.196 * (1.0 - 0.3 * spin.abs()); // Rough approximation
    
    // Generate scatter points
    let mut circles = String::new();
    for (b, wraps, order) in &points {
        let x = ((b - b_min_plot) / (b_max_plot - b_min_plot)) * chart_w as f64;
        let y_raw = chart_h as f64 - (wraps / wraps_range) * chart_h as f64;
        
        let (color, radius) = match order {
            0 => ("#22C55E", 2.5),
            1 => ("#3B82F6", 3.0),
            _ => ("#8B5CF6", 3.5),
        };
        
        // Clamp y to keep circles within chart bounds (account for radius)
        let y = y_raw.min(chart_h as f64 - radius).max(radius);
        
        circles.push_str(&format!(
            "    <circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"{:.1}\" fill=\"{}\" opacity=\"0.4\"/>\n",
            x, y, radius, color
        ));
    }
    
    // Photon sphere vertical line
    let x_photon = ((b_photon_approx - b_min_plot) / (b_max_plot - b_min_plot)) * chart_w as f64;
    let photon_line = if x_photon >= 0.0 && x_photon <= chart_w as f64 {
        format!("    <line x1=\"{:.1}\" y1=\"0\" x2=\"{:.1}\" y2=\"{}\" stroke=\"#EF4444\" stroke-width=\"2\" stroke-dasharray=\"5,5\" opacity=\"0.6\"/>\n    <text x=\"{:.1}\" y=\"-5\" text-anchor=\"middle\" font-size=\"10\" fill=\"#EF4444\">b_photon ≈ {:.2}</text>\n",
            x_photon, x_photon, chart_h, x_photon, b_photon_approx)
    } else {
        String::new()
    };
    
    // Axis labels
    let x_ticks = 5;
    let y_ticks = 5;
    let mut x_labels = String::new();
    let mut y_labels = String::new();
    
    for i in 0..=x_ticks {
        let b_val = b_min_plot + (b_max_plot - b_min_plot) * (i as f64 / x_ticks as f64);
        let x = (i as f64 / x_ticks as f64) * chart_w as f64;
        x_labels.push_str(&format!(
            "    <text x=\"{:.1}\" y=\"{}\" text-anchor=\"middle\" font-size=\"10\" fill=\"#6b7280\">{:.1}</text>\n",
            x, chart_h + 20, b_val
        ));
        x_labels.push_str(&format!(
            "    <line x1=\"{:.1}\" y1=\"{}\" x2=\"{:.1}\" y2=\"{}\" stroke=\"#E5E7EB\" stroke-width=\"1\"/>\n",
            x, chart_h, x, chart_h + 5
        ));
    }
    
    for i in 0..=y_ticks {
        let w_val = (wraps_range * (i as f64 / y_ticks as f64)).round();
        let y = chart_h as f64 - (i as f64 / y_ticks as f64) * chart_h as f64;
        y_labels.push_str(&format!(
            "    <text x=\"-10\" y=\"{:.1}\" text-anchor=\"end\" font-size=\"10\" fill=\"#6b7280\">{:.0}</text>\n",
            y + 4.0, w_val
        ));
        y_labels.push_str(&format!(
            "    <line x1=\"-5\" y1=\"{:.1}\" x2=\"0\" y2=\"{:.1}\" stroke=\"#E5E7EB\" stroke-width=\"1\"/>\n",
            y, y
        ));
    }
    
    format!("<svg width=\"{}\" height=\"{}\" xmlns=\"http://www.w3.org/2000/svg\">
  <text x=\"{}\" y=\"20\" text-anchor=\"middle\" font-size=\"14\" font-weight=\"600\" fill=\"#374151\">Wrap-Angle vs Impact Parameter</text>
  
  <g transform=\"translate({}, 50)\">
    <!-- Chart content -->
{}
{}
    
    <!-- Axes -->
    <line x1=\"0\" y1=\"0\" x2=\"0\" y2=\"{}\" stroke=\"#9CA3AF\" stroke-width=\"2\"/>
    <line x1=\"0\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"#9CA3AF\" stroke-width=\"2\"/>
    
    <!-- Labels -->
{}
{}
    
    <!-- Axis titles -->
    <text x=\"{}\" y=\"{}\" text-anchor=\"middle\" font-size=\"12\" fill=\"#374151\">Impact parameter b = L_z/E</text>
    <text x=\"-15\" y=\"{}\" text-anchor=\"middle\" font-size=\"12\" fill=\"#374151\" transform=\"rotate(-90, -15, {})\">φ-wraps</text>
  </g>
  
  <!-- Legend -->
  <g transform=\"translate(20, {})\">
    <circle cx=\"5\" cy=\"5\" r=\"2.5\" fill=\"#22C55E\" opacity=\"0.6\"/>
    <text x=\"15\" y=\"9\" font-size=\"10\" fill=\"#6b7280\">Order 0</text>
    <circle cx=\"70\" cy=\"5\" r=\"3\" fill=\"#3B82F6\" opacity=\"0.6\"/>
    <text x=\"80\" y=\"9\" font-size=\"10\" fill=\"#6b7280\">Order 1</text>
    <circle cx=\"135\" cy=\"5\" r=\"3.5\" fill=\"#8B5CF6\" opacity=\"0.6\"/>
    <text x=\"145\" y=\"9\" font-size=\"10\" fill=\"#6b7280\">Order 2+</text>
  </g>
</svg>",
        width, height,
        width / 2,
        margin,
        photon_line, circles,
        chart_h, chart_h, chart_w, chart_h,
        x_labels, y_labels,
        chart_w / 2, chart_h + 40,
        chart_h / 2, chart_h / 2,
        height - 25
    )
}

// ============================================================================
// TIER 4: OUTLIER SPATIAL OVERLAY
// ============================================================================

/// Generate spatial heatmap overlay showing outlier locations colored by severity
///
/// This downsamples the full-resolution outlier data into a manageable grid
/// and colors cells by the highest severity outlier found in that region.
pub fn generate_outlier_overlay_svg(
    outliers: &[crate::stats::Outlier],
    image_width: u32,
    image_height: u32,
    downsample_factor: u32,  // e.g., 4 = each cell is 4×4 pixels
) -> String {
    use crate::stats::OutlierSeverity;
    
    // Compute downsampled grid dimensions
    let grid_w = (image_width + downsample_factor - 1) / downsample_factor;
    let grid_h = (image_height + downsample_factor - 1) / downsample_factor;
    
    // Build grid: cell (gx, gy) → (count, max_severity)
    use std::collections::HashMap;
    let mut grid: HashMap<(u32, u32), (usize, OutlierSeverity)> = HashMap::new();
    
    for outlier in outliers {
        let gx = outlier.pixel_x / downsample_factor;
        let gy = outlier.pixel_y / downsample_factor;
        
        if gx < grid_w && gy < grid_h {
            let entry = grid.entry((gx, gy)).or_insert((0, OutlierSeverity::Info));
            entry.0 += 1; // Increment count
            
            // Update to highest severity
            if outlier.severity > entry.1 {
                entry.1 = outlier.severity;
            }
        }
    }
    
    // SVG dimensions (each grid cell = 2px for visibility)
    let cell_size = 2;
    let svg_w = grid_w * cell_size;
    let svg_h = grid_h * cell_size;
    let width = svg_w + 100; // Extra space for legend
    let height = svg_h + 60;
    
    let mut svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {}" width="{}" height="{}">
<style>
  .outlier-title {{ font: bold 14px sans-serif; fill: #1f2937; }}
  .outlier-label {{ font: 11px sans-serif; fill: #6b7280; }}
  .outlier-legend {{ font: 10px sans-serif; fill: #374151; }}
</style>
<rect width="100%" height="100%" fill="#f9fafb" />
"##,
        width, height, width, height
    );
    
    // Title
    svg.push_str(&format!(
        r#"<text x="{}" y="20" text-anchor="middle" class="outlier-title">Outlier Spatial Distribution</text>"#,
        width / 2
    ));
    svg.push_str(&format!(
        r#"<text x="{}" y="38" text-anchor="middle" class="outlier-label">{} outliers detected across {} regions</text>"#,
        width / 2, outliers.len(), grid.len()
    ));
    
    // Draw grid cells
    for ((gx, gy), (count, severity)) in &grid {
        let x = gx * cell_size;
        let y = 50 + gy * cell_size;
        
        // Opacity scales with count (more outliers = more opaque)
        let opacity = ((*count).min(10) as f64 / 10.0).max(0.3).min(1.0);
        
        svg.push_str(&format!(
            r#"<rect x="{}" y="{}" width="{}" height="{}" fill="{}" opacity="{:.2}"/>"#,
            x, y, cell_size, cell_size, severity.color(), opacity
        ));
    }
    
    // Legend
    let legend_x = svg_w + 10;
    let legend_y = 60;
    
    svg.push_str(&format!(
        r#"<text x="{}" y="{}" class="outlier-legend" font-weight="bold">Severity:</text>"#,
        legend_x, legend_y
    ));
    
    let severities = [
        (OutlierSeverity::Critical, "Critical"),
        (OutlierSeverity::High, "High"),
        (OutlierSeverity::Medium, "Medium"),
        (OutlierSeverity::Info, "Info"),
    ];
    
    for (i, (severity, label)) in severities.iter().enumerate() {
        let y = legend_y + 20 + i as u32 * 20;
        
        svg.push_str(&format!(
            r#"<rect x="{}" y="{}" width="12" height="12" fill="{}"/>"#,
            legend_x, y - 10, severity.color()
        ));
        svg.push_str(&format!(
            r#"<text x="{}" y="{}" class="outlier-legend">{}</text>"#,
            legend_x + 16, y, label
        ));
    }
    
    // Opacity legend
    svg.push_str(&format!(
        r#"<text x="{}" y="{}" class="outlier-legend" font-weight="bold">Density:</text>"#,
        legend_x, legend_y + 120
    ));
    svg.push_str(&format!(
        r#"<text x="{}" y="{}" class="outlier-legend">darker = more outliers</text>"#,
        legend_x, legend_y + 135
    ));
    
    svg.push_str("</svg>");
    svg
}