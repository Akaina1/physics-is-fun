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
  <text x="{}" y="20" text-anchor="middle" font-size="14" font-weight="600" fill="#374151">Angular Distribution (φ)</text>
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
