// Kerr Black Hole Asset Generator CLI
//
// This binary generates transfer maps and flux LUTs at build time.
// It runs before deployment to precompute all physics data for WebGL rendering.

use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::path::PathBuf;
use std::env;
use std::io::Write;
use flate2::Compression;
use flate2::write::GzEncoder;

use kerr_black_hole::*;

/// CLI arguments for the asset generator
#[derive(Parser, Debug)]
#[command(name = "generate")]
#[command(about = "Generate Kerr black hole transfer maps and flux LUTs", long_about = None)]
struct Args {
    /// Preset name (e.g., "30deg", "45deg", "60deg", "75deg")
    #[arg(short, long)]
    preset: String,

    /// Black hole type (e.g., "prograde", "retrograde", "schwarzschild")
    #[arg(short, long, default_value = "prograde")]
    black_hole_type: String,

    /// Image width in pixels
    #[arg(short, long, default_value_t = 1280)]
    width: u32,

    /// Image height in pixels
    #[arg(short = 'H', long, default_value_t = 720)]
    height: u32,

    /// Maximum number of geodesic orders to trace (1-5)
    /// 1 = primary image only
    /// 2 = primary + photon ring (recommended)
    /// 3 = primary + photon ring + first subring
    /// 4-5 = higher order subrings (for scientific analysis)
    #[arg(short = 'O', long, default_value_t = 2, value_parser = clap::value_parser!(u8).range(1..=5))]
    max_orders: u8,

    /// Output directory for generated assets
    #[arg(short, long, default_value = "public/blackhole")]
    output: PathBuf,

    /// Export high-precision f64 data (large JSON file)
    #[arg(long, default_value_t = false)]
    export_precision: bool,

    /// Gzip compress the high-precision JSON output (creates .json.gz instead of .json)
    /// Only applies when --export-precision is enabled
    #[arg(long, default_value_t = false)]
    gzip: bool,
}


/// Parse the camera viewing angle from the preset name
/// 
/// Inclination angles chosen for optimal black hole visualization:
/// - 30° = Overhead view, clear photon ring, minimal Doppler asymmetry
/// - 45° = Balanced view, good photon ring + moderate Doppler shift
/// - 60° = Angled view, dramatic lensing + strong Doppler asymmetry (like Interstellar)
/// - 75° = Near edge-on, extreme Doppler shift + thin disc appearance (like M87*)
fn parse_camera_angle(preset: &str) -> Result<f64, String> {
    match preset {
        "30deg" => Ok(30.0),         // Overhead - clear photon ring
        "45deg" => Ok(45.0),         // Balanced - classic black hole view
        "60deg" => Ok(60.0),         // Angled - dramatic (Interstellar-like)
        "75deg" => Ok(75.0),         // Near edge-on - extreme effects (M87*-like)
        _ => Err(format!(
            "Invalid preset: '{}'. Must be one of: 30deg, 45deg, 60deg, 75deg",
            preset
        )),
    }
}

/// Parse the black hole type from the string
fn parse_black_hole_type(bh_type: &str) -> Result<BlackHoleType, String> {
    match bh_type {
        "prograde" => Ok(BlackHoleType::kerr_prograde(0.9)),     // Kerr with prograde orbit
        "retrograde" => Ok(BlackHoleType::kerr_retrograde(0.9)), // Kerr with retrograde orbit  
        "schwarzschild" => Ok(BlackHoleType::schwarzschild()),   // Non-rotating black hole
        _ => Err(format!(
            "Invalid black hole type: '{}'. Must be one of: prograde, retrograde, schwarzschild",
            bh_type
        )),
    }
}

/// Write binary data to a file
fn write_binary(path: &PathBuf, data: &[f32]) -> std::io::Result<()> {
    // Convert f32 slice to bytes
    let byte_data: Vec<u8> = data
        .iter()
        .flat_map(|&f| f.to_le_bytes())  // Convert each f32 to 4 little-endian bytes
        .collect();
    
    fs::write(path, byte_data)?;
    Ok(())
}

/// Write JSON data to a file
fn write_json(path: &PathBuf, json_str: &str) -> std::io::Result<()> {
    fs::write(path, json_str)?;
    Ok(())
}

/// Find the workspace root by looking for Cargo.toml
fn find_workspace_root() -> PathBuf {
    let mut current = env::current_dir().expect("Failed to get current directory");
    
    // Walk up the directory tree until we find workspace Cargo.toml
    loop {
        let cargo_toml = current.join("Cargo.toml");
        if cargo_toml.exists() {
            // Check if it's a workspace (has [workspace] section)
            if let Ok(contents) = fs::read_to_string(&cargo_toml) {
                if contents.contains("[workspace]") {
                    return current;
                }
            }
        }
        
        // Try parent directory
        if let Some(parent) = current.parent() {
            current = parent.to_path_buf();
        } else {
            // Couldn't find workspace root, use current dir
            return env::current_dir().expect("Failed to get current directory");
        }
    }
}

/// Save transfer maps to disk
fn save_transfer_maps(
    maps: &TransferMaps,
    output_dir: &PathBuf,
    gzip: bool,
) -> std::io::Result<()> {
    // Create output directory if it doesn't exist
    fs::create_dir_all(output_dir)?;
    
    // Write T1 transfer map (Order 0 positions)
    let t1_path = output_dir.join("t1_rgba32f.bin");
    write_binary(&t1_path, &maps.t1_rgba32f)?;
    println!("  ✓ Wrote T1 (Order 0 positions): {} ({:.2} MB)", 
        t1_path.display(), 
        (maps.t1_rgba32f.len() * 4) as f64 / 1_000_000.0
    );
    
    // Write T2 transfer map (Order 0 physics)
    let t2_path = output_dir.join("t2_rgba32f.bin");
    write_binary(&t2_path, &maps.t2_rgba32f)?;
    println!("  ✓ Wrote T2 (Order 0 physics): {} ({:.2} MB)", 
        t2_path.display(), 
        (maps.t2_rgba32f.len() * 4) as f64 / 1_000_000.0
    );
    
    // Write T3/T4 if max_orders > 1
    if maps.max_orders > 1 {
        let t3_path = output_dir.join("t3_rgba32f.bin");
        write_binary(&t3_path, &maps.t3_rgba32f)?;
        println!("  ✓ Wrote T3 (Order 1 positions): {} ({:.2} MB)", 
            t3_path.display(), 
            (maps.t3_rgba32f.len() * 4) as f64 / 1_000_000.0
        );
        
        let t4_path = output_dir.join("t4_rgba32f.bin");
        write_binary(&t4_path, &maps.t4_rgba32f)?;
        println!("  ✓ Wrote T4 (Order 1 physics): {} ({:.2} MB)", 
            t4_path.display(), 
            (maps.t4_rgba32f.len() * 4) as f64 / 1_000_000.0
        );
    }
    
    // Write T5/T6 if max_orders > 2
    if maps.max_orders > 2 {
        let t5_path = output_dir.join("t5_rgba32f.bin");
        write_binary(&t5_path, &maps.t5_rgba32f)?;
        println!("  ✓ Wrote T5 (Order 2+ positions): {} ({:.2} MB)", 
            t5_path.display(), 
            (maps.t5_rgba32f.len() * 4) as f64 / 1_000_000.0
        );
        
        let t6_path = output_dir.join("t6_rgba32f.bin");
        write_binary(&t6_path, &maps.t6_rgba32f)?;
        println!("  ✓ Wrote T6 (Order 2+ physics): {} ({:.2} MB)", 
            t6_path.display(), 
            (maps.t6_rgba32f.len() * 4) as f64 / 1_000_000.0
        );
    }
    
    // Write flux LUT (R32F: 1D emissivity lookup table)
    let flux_path = output_dir.join("flux_r32f.bin");
    write_binary(&flux_path, &maps.flux_r32f)?;
    println!("  ✓ Wrote Flux LUT: {} ({:.2} KB)", 
        flux_path.display(), 
        (maps.flux_r32f.len() * 4) as f64 / 1_000.0
    );
    
    // Write manifest (metadata)
    let manifest_path = output_dir.join("manifest.json");
    let manifest_json = serde_json::to_string_pretty(&maps.manifest)?;
    write_json(&manifest_path, &manifest_json)?;
    println!("  ✓ Wrote Manifest: {}", manifest_path.display());
    
    // Write high-precision data if available
    if let Some(ref hp_data) = maps.high_precision_data {
        let hp_json = hp_data.to_json();
        
        if gzip {
            // Write gzipped version
            let hp_path = output_dir.join("high_precision.json.gz");
            let file = fs::File::create(&hp_path)?;
            let mut encoder = GzEncoder::new(file, Compression::default());
            encoder.write_all(hp_json.as_bytes())?;
            encoder.finish()?;
            
            let compressed_size = fs::metadata(&hp_path)?.len();
            println!("  ✓ Wrote High-Precision Data (gzipped): {} ({:.2} MB compressed, {:.2} MB uncompressed)", 
                hp_path.display(),
                compressed_size as f64 / 1_000_000.0,
                hp_json.len() as f64 / 1_000_000.0
            );
        } else {
            // Write raw JSON
            let hp_path = output_dir.join("high_precision.json");
            write_json(&hp_path, &hp_json)?;
            println!("  ✓ Wrote High-Precision Data (JSON): {} ({:.2} MB)", 
                hp_path.display(),
                hp_json.len() as f64 / 1_000_000.0
            );
        }
    }
    
    Ok(())
}

/// Get description of order count for logging
fn order_description(orders: u8) -> &'static str {
    match orders {
        1 => "primary only",
        2 => "primary + photon ring",
        3 => "primary + photon ring + subring",
        4 => "up to 3rd order",
        5 => "up to 4th order",
        _ => "unknown",
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command-line arguments
    let args = Args::parse();
    
    // Find workspace root
    let workspace_root = find_workspace_root();
    
    // Parse black hole type
    let bh_type = parse_black_hole_type(&args.black_hole_type)
        .map_err(|e| e.to_string())?;
    
    // Parse camera angle from preset
    let camera_angle = parse_camera_angle(&args.preset)
        .map_err(|e| e.to_string())?;
    
    // Create black hole
    let black_hole = BlackHole::new(
        1.0,  // Mass M = 1 (geometric units)
        bh_type,
    );
    
    // Create camera with parsed angle
    let camera = Camera::new(
        50.0,          // Distance from black hole (50M, well outside disc)
        camera_angle,  // Inclination in degrees
        120.0,         // Wide field of view to capture disc
    );
    
    // Create render configuration
    let config = RenderConfig {
        width: args.width,
        height: args.height,
        max_orders: args.max_orders,
    };
    
    // Print configuration
    println!("\nKerr Black Hole Asset Generator");
    println!("=======================================");
    println!("  Preset: {}", args.preset);
    println!("  Black Hole: {}", bh_type.name());
    println!("  Resolution: {}x{}", args.width, args.height);
    println!("  Camera Angle: {:.1} degrees", camera_angle);
    println!("  Max Orders: {} ({})", args.max_orders, order_description(args.max_orders));
    println!("  Export Precision: {}", args.export_precision);
    println!("=======================================\n");
    
    // Create progress bar
    let total_pixels = (args.width * args.height) as u64;
    let pb = ProgressBar::new(total_pixels);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} pixels ({percent}%)")?
            .progress_chars("█▓▒░ "),
    );
    
    // Render transfer maps
    println!("Tracing geodesics...");
    
    // Extract orbit direction from black hole type
    let orbit_direction = match bh_type {
        BlackHoleType::Kerr { direction, .. } => direction,
        BlackHoleType::Schwarzschild => OrbitDirection::Prograde,  // Doesn't matter for Schwarzschild
    };
    
    let maps = render_transfer_maps(
        &black_hole,
        &camera,
        &config,
        orbit_direction,
        args.preset.clone(),
        args.export_precision,
        |pixels| {
            pb.set_position(pixels);
        },
    );
    
    pb.finish_with_message("✓ Geodesic integration complete");
    
    // Create output directory path (relative to workspace root)
    // Structure: public/blackhole/{bh_type}/{preset}/
    let output_dir = workspace_root.join(&args.output).join(&args.black_hole_type).join(&args.preset);
    
    // Save all files
    println!("\n💾 Writing files...");
    save_transfer_maps(&maps, &output_dir, args.gzip)?;
    
    // Print statistics
    println!("\n📊 Statistics:");
    println!("  Total pixels: {}", total_pixels);
    println!("  Disc hits: {}", maps.manifest.disc_hits);
    println!("  Miss rate: {:.1}%", 
        (total_pixels - maps.manifest.disc_hits as u64) as f64 / total_pixels as f64 * 100.0
    );
    
    if args.export_precision {
        if let Some(ref hp_data) = maps.high_precision_data {
            let stats = hp_data.statistics();
            println!("\nHigh-Precision Statistics:");
            println!("  Radius range: {:.6} - {:.6} M", stats.min_r, stats.max_r);
            println!("  Mean energy: {:.6}", stats.mean_energy);
            println!("  Mean L_z: {:.6}", stats.mean_l_z);
        }
    }
    
    println!("\n✨ Generation complete!");
    println!("📁 Output: {}\n", output_dir.display());
    
    Ok(())
}