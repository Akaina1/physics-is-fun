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
    /// Preset name (e.g., "face-on", "30deg", "60deg", "edge-on")
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

    /// Output directory for generated assets
    #[arg(short, long, default_value = "public/blackhole")]
    output: PathBuf,

    /// Export high-precision f64 data (large JSON file)
    #[arg(long, default_value_t = false)]
    export_precision: bool,
}


/// Parse the camera viewing angle from the preset name
fn parse_camera_angle(preset: &str) -> Result<f64, String> {
    match preset {
        "face-on" => Ok(0.1),        // 0.1° inclination (nearly top-down view, minimal pole issues)
        "30deg" => Ok(30.0),         // 30° inclination
        "60deg" => Ok(60.0),         // 60° inclination
        "edge-on" => Ok(89.9),       // ~90° inclination (edge-on, avoid exactly 90°)
        _ => Err(format!(
            "Invalid preset: '{}'. Must be one of: face-on, 30deg, 60deg, edge-on",
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
) -> std::io::Result<()> {
    // Create output directory if it doesn't exist
    fs::create_dir_all(output_dir)?;
    
    // Write T1 transfer map (RGBA32F: positions, angles, mask)
    let t1_path = output_dir.join("t1_rgba32f.bin");
    write_binary(&t1_path, &maps.t1_rgba32f)?;
    println!("  ✓ Wrote T1: {} ({:.2} MB)", 
        t1_path.display(), 
        (maps.t1_rgba32f.len() * 4) as f64 / 1_000_000.0
    );
    
    // Write T2 transfer map (RGBA32F: energy, momentum, order)
    let t2_path = output_dir.join("t2_rgba32f.bin");
    write_binary(&t2_path, &maps.t2_rgba32f)?;
    println!("  ✓ Wrote T2: {} ({:.2} MB)", 
        t2_path.display(), 
        (maps.t2_rgba32f.len() * 4) as f64 / 1_000_000.0
    );
    
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
        let hp_path = output_dir.join("high_precision.json.gz");
        let hp_json = hp_data.to_json();
        
        // Gzip compress the JSON
        let file = fs::File::create(&hp_path)?;
        let mut encoder = GzEncoder::new(file, Compression::default());
        encoder.write_all(hp_json.as_bytes())?;
        encoder.finish()?;
        
        let compressed_size = fs::metadata(&hp_path)?.len();
        println!("  ✓ Wrote High-Precision Data: {} ({:.2} MB compressed, {:.2} MB uncompressed)", 
            hp_path.display(),
            compressed_size as f64 / 1_000_000.0,
            hp_json.len() as f64 / 1_000_000.0
        );
    }
    
    Ok(())
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
        max_orders: 2,  // Track primary and secondary images
    };
    
    // Print configuration
    println!("\nKerr Black Hole Asset Generator");
    println!("=======================================");
    println!("  Preset: {}", args.preset);
    println!("  Black Hole: {}", bh_type.name());
    println!("  Resolution: {}x{}", args.width, args.height);
    println!("  Camera Angle: {:.1} degrees", camera_angle);
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
    save_transfer_maps(&maps, &output_dir)?;
    
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