// High-Precision Geodesic Data Analyzer
//
// Generates a beautiful HTML report from high-precision JSON export

mod charts;
mod generate_summary;

use generate_summary::{Stats, Manifest, generate_html_report};

use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

/// CLI arguments for the analyzer
#[derive(Parser, Debug)]
#[command(name = "analyze")]
#[command(about = "Analyze high-precision geodesic data and generate HTML report", long_about = None)]
struct Args {
    /// Path to high_precision.json file
    #[arg(short, long)]
    input: PathBuf,
}

/// High-precision record (matches PositionData from transfer_maps.rs)
#[derive(Debug, Deserialize)]
struct HpRecord {
    pixel_x: u32,
    pixel_y: u32,
    #[serde(default)]
    r: f64,
    #[serde(default)]
    _theta: f64,
    #[serde(default)]
    phi: f64,  // Used for angular distribution
    #[serde(default)]
    _energy: f64,
    #[serde(default)]
    _angular_momentum: f64,
    #[serde(default)]
    _carter_q: f64,
    #[serde(default)]
    _impact_parameter: f64,
    #[serde(default)]
    redshift_factor: f64,
    #[serde(default)]
    affine_parameter: f64,
    #[serde(default)]
    phi_wraps: f64,
    #[serde(default)]
    order: u8,
    hit: bool,
    #[serde(default)]
    null_invariant_error: f64,
}

/// Wrapper for HP JSON structure
#[derive(Debug, Deserialize)]
struct HpData {
    positions: Vec<HpRecord>,
}

// Manifest is imported from generate_summary module

/// Per-pixel aggregation (for spatial analysis)
#[derive(Debug, Default, Clone)]
struct PixelAgg {
    any_hit: bool,
    order_mask: u8,  // Bitmask: bit 0 = order 0, bit 1 = order 1, etc.
    worst_ni: f64,
}

// Stats is imported from generate_summary module

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    println!("\n🔬 High-Precision Geodesic Data Analyzer");
    println!("========================================");
    
    // Load manifest from same directory
    let manifest_path = args.input.parent().unwrap().join("manifest.json");
    let manifest_str = fs::read_to_string(&manifest_path)?;
    let manifest: Manifest = serde_json::from_str(&manifest_str)?;
    
    println!("  Scene: {}", manifest.preset);
    println!("  Resolution: {}x{}", manifest.width, manifest.height);
    println!("  Orders: {}", manifest.orders);
    println!("========================================\n");
    
    // Progress: Loading HP data
    let pb = ProgressBar::new(4);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")?
            .progress_chars("█▓▒░ "),
    );
    
    pb.set_message("Loading high-precision data...");
    let file_size = fs::metadata(&args.input)?.len();
    let hp_str = fs::read_to_string(&args.input)?;
    pb.inc(1);
    
    pb.set_message("Parsing JSON...");
    let hp_data: HpData = serde_json::from_str(&hp_str)?;
    pb.inc(1);
    
    pb.set_message(format!("Computing statistics ({} records)...", hp_data.positions.len()));
    let stats = compute_statistics(&hp_data, &manifest);
    pb.inc(1);
    
    pb.set_message("Generating HTML report...");
    let html = generate_html_report(&stats, &manifest);
    pb.inc(1);
    
    pb.finish_with_message("✓ Analysis complete");
    
    // Write to same directory as input
    let output_path = args.input.parent().unwrap().join("analysis_report.html");
    fs::write(&output_path, &html)?;
    
    // Summary
    println!("\n📊 Summary:");
    println!("  Input file: {:.2} MB", file_size as f64 / 1_000_000.0);
    println!("  Records analyzed: {}", hp_data.positions.len());
    println!("  Hit pixels: {} ({:.1}%)", stats.total_hit_pixels, 
        stats.total_hit_pixels as f64 / stats.total_pixels as f64 * 100.0);
    println!("  Report size: {:.2} KB", html.len() as f64 / 1_000.0);
    
    println!("\n✨ HTML Report Generated!");
    println!("📄 {}\n", output_path.display());
    
    Ok(())
}

fn compute_statistics(hp_data: &HpData, manifest: &Manifest) -> Stats {
    let total_pixels = (manifest.width * manifest.height) as usize;
    
    // Initialize per-pixel aggregation
    let mut pixel_agg = vec![PixelAgg::default(); total_pixels];
    
    // Collect hit records
    let hits: Vec<_> = hp_data.positions.iter().filter(|r| r.hit).collect();
    
    // Aggregate per-pixel data
    for rec in &hp_data.positions {
        let idx = (rec.pixel_y * manifest.width + rec.pixel_x) as usize;
        if rec.hit {
            pixel_agg[idx].any_hit = true;
            pixel_agg[idx].order_mask |= 1 << rec.order;
            pixel_agg[idx].worst_ni = pixel_agg[idx].worst_ni.max(rec.null_invariant_error);
        }
    }
    
    // Count hit pixels
    let total_hit_pixels = pixel_agg.iter().filter(|p| p.any_hit).count();
    let miss_pixels = total_pixels - total_hit_pixels;
    
    // Order distribution per pixel
    let only_order_0 = pixel_agg.iter().filter(|p| p.order_mask == 0b001).count();
    let orders_0_and_1 = pixel_agg.iter().filter(|p| p.order_mask == 0b011).count();
    let orders_2_plus = pixel_agg.iter().filter(|p| p.order_mask >= 0b100).count();
    
    // Per-order hits
    let order_0_hits = hits.iter().filter(|r| r.order == 0).count();
    let order_1_hits = hits.iter().filter(|r| r.order == 1).count();
    let order_2_plus_hits = hits.iter().filter(|r| r.order >= 2).count();
    
    // Null invariant stats
    let mut ni_values: Vec<f64> = hits.iter().map(|r| r.null_invariant_error).collect();
    ni_values.sort_by(|a, b| a.partial_cmp(b).unwrap());
    
    let ni_min = ni_values.first().copied().unwrap_or(0.0);
    let ni_max = ni_values.last().copied().unwrap_or(0.0);
    let ni_mean = ni_values.iter().sum::<f64>() / ni_values.len().max(1) as f64;
    let ni_median = if !ni_values.is_empty() { ni_values[ni_values.len() / 2] } else { 0.0 };
    let ni_p95 = if !ni_values.is_empty() { ni_values[(ni_values.len() as f64 * 0.95) as usize] } else { 0.0 };
    let ni_p99 = if !ni_values.is_empty() { ni_values[(ni_values.len() as f64 * 0.99) as usize] } else { 0.0 };
    let ni_under_1e15 = ni_values.iter().filter(|&&v| v < 1e-15).count();
    let ni_under_1e12 = ni_values.iter().filter(|&&v| v < 1e-12).count();
    let ni_under_1e9 = ni_values.iter().filter(|&&v| v < 1e-9).count();
    
    // Redshift stats
    let g_values: Vec<f64> = hits.iter().map(|r| r.redshift_factor).collect();
    let g_min = g_values.iter().cloned().fold(f64::INFINITY, f64::min);
    let g_max = g_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let g_mean = g_values.iter().sum::<f64>() / g_values.len().max(1) as f64;
    let g_boosted_count = g_values.iter().filter(|&&v| v > 1.0).count();
    let g_dimmed_count = g_values.iter().filter(|&&v| v < 1.0).count();
    
    // Radial distribution (order 0 only)
    let order_0_records: Vec<_> = hits.iter().filter(|r| r.order == 0).collect();
    let r_in = manifest.r_in;
    let r_out = manifest.r_out;
    
    let bin_edges = vec![
        (r_in, 4.0, format!("[{:.1}-{:.1}M] Near ISCO", r_in, 4.0)),
        (4.0, 8.0, "[4.0-8.0M] Inner disc".to_string()),
        (8.0, 14.0, "[8.0-14.0M] Mid disc".to_string()),
        (14.0, r_out, format!("[{:.1}-{:.1}M] Outer disc", 14.0, r_out)),
    ];
    
    let r_bins: Vec<(String, usize)> = bin_edges
        .iter()
        .map(|(low, high, label)| {
            let count = order_0_records
                .iter()
                .filter(|r| r.r >= *low && r.r < *high)
                .count();
            (label.clone(), count)
        })
        .collect();
    
    // Phi wraps
    let phi_wraps_all: Vec<f64> = hits.iter().map(|r| r.phi_wraps).collect();
    let phi_wraps_min = phi_wraps_all.iter().cloned().fold(f64::INFINITY, f64::min);
    let phi_wraps_max = phi_wraps_all.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    
    let phi_wraps_o0: Vec<f64> = hits.iter().filter(|r| r.order == 0).map(|r| r.phi_wraps).collect();
    let phi_wraps_o1: Vec<f64> = hits.iter().filter(|r| r.order == 1).map(|r| r.phi_wraps).collect();
    let phi_wraps_o2: Vec<f64> = hits.iter().filter(|r| r.order >= 2).map(|r| r.phi_wraps).collect();
    
    let phi_wraps_mean_o0 = phi_wraps_o0.iter().sum::<f64>() / phi_wraps_o0.len().max(1) as f64;
    let phi_wraps_mean_o1 = phi_wraps_o1.iter().sum::<f64>() / phi_wraps_o1.len().max(1) as f64;
    let phi_wraps_mean_o2 = phi_wraps_o2.iter().sum::<f64>() / phi_wraps_o2.len().max(1) as f64;
    
    // Affine parameter
    let affine_o0: Vec<f64> = hits.iter().filter(|r| r.order == 0).map(|r| r.affine_parameter).collect();
    let affine_o1: Vec<f64> = hits.iter().filter(|r| r.order == 1).map(|r| r.affine_parameter).collect();
    let affine_o2: Vec<f64> = hits.iter().filter(|r| r.order >= 2).map(|r| r.affine_parameter).collect();
    
    let affine_mean_o0 = affine_o0.iter().sum::<f64>() / affine_o0.len().max(1) as f64;
    let affine_mean_o1 = affine_o1.iter().sum::<f64>() / affine_o1.len().max(1) as f64;
    let affine_mean_o2 = affine_o2.iter().sum::<f64>() / affine_o2.len().max(1) as f64;
    
    // Convert to chart HpRecord format for chart computation
    let chart_hits: Vec<charts::HpRecord> = hits.iter().map(|r| charts::HpRecord {
        _pixel_x: r.pixel_x,
        _pixel_y: r.pixel_y,
        r: r.r,
        phi: r.phi,
        redshift_factor: r.redshift_factor,
        _affine_parameter: r.affine_parameter,
        phi_wraps: r.phi_wraps,
        order: r.order,
        _hit: r.hit,
        null_invariant_error: r.null_invariant_error,
    }).collect();
    let chart_hit_refs: Vec<&charts::HpRecord> = chart_hits.iter().collect();
    
    // Chart data: NI histogram (log-scale bins)
    let ni_histogram = charts::compute_ni_histogram(&chart_hit_refs);
    
    // Chart data: Radial histogram (per-order)
    let radial_histogram = charts::compute_radial_histogram(&chart_hit_refs, r_in, r_out);
    
    // Chart data: Radial profile (multi-metric)
    let radial_profile = charts::compute_radial_profile(&chart_hit_refs, r_in, r_out);
    
    // Chart data: Angular distribution
    let angular_distribution = charts::compute_angular_distribution(&chart_hit_refs);
    
    Stats {
        total_pixels,
        total_hit_pixels,
        total_hits: hits.len(),
        miss_pixels,
        order_0_hits,
        order_1_hits,
        order_2_plus_hits,
        only_order_0,
        orders_0_and_1,
        orders_2_plus,
        ni_min,
        ni_max,
        ni_mean,
        ni_median,
        ni_p95,
        ni_p99,
        ni_under_1e15,
        ni_under_1e12,
        ni_under_1e9,
        g_min,
        g_max,
        g_mean,
        g_boosted_count,
        g_dimmed_count,
        r_bins,
        phi_wraps_min,
        phi_wraps_max,
        phi_wraps_mean_o0,
        phi_wraps_mean_o1,
        phi_wraps_mean_o2,
        affine_mean_o0,
        affine_mean_o1,
        affine_mean_o2,
        ni_histogram,
        radial_histogram,
        radial_profile,
        angular_distribution,
    }
}

// Chart computation and HTML generation are now in separate modules
