# Precompute all black hole transfer maps
# 3 black hole types × 4 viewing presets = 12 total generations

$ErrorActionPreference = "Stop"

# Configuration
$width = 128
$height = 72
$presets = @("face-on", "30deg", "60deg", "edge-on")
$bhTypes = @(
    @{name="Prograde Kerr"; type="prograde"},
    @{name="Retrograde Kerr"; type="retrograde"},
    @{name="Schwarzschild"; type="schwarzschild"}
)

Write-Host "`nKerr Black Hole Transfer Map Generator" -ForegroundColor Cyan
Write-Host "========================================`n" -ForegroundColor Cyan

$totalConfigs = $bhTypes.Count * $presets.Count
$currentConfig = 0

foreach ($bh in $bhTypes) {
    Write-Host "`n========================================" -ForegroundColor Yellow
    Write-Host "  $($bh.name)" -ForegroundColor Yellow
    Write-Host "========================================`n" -ForegroundColor Yellow
    
    foreach ($preset in $presets) {
        $currentConfig++
        Write-Host "[$currentConfig/$totalConfigs] Generating: $preset..." -ForegroundColor Green
        
        # Run cargo command (temporarily allow stderr without stopping)
        $prevErrorAction = $ErrorActionPreference
        $ErrorActionPreference = "Continue"
        
        cargo run --release --bin generate -p kerr_black_hole -- `
            --preset $preset `
            --black-hole-type $($bh.type) `
            --width $width `
            --height $height
        
        $ErrorActionPreference = $prevErrorAction
        
        if ($LASTEXITCODE -ne 0) {
            Write-Host "Error generating $($bh.name) - $preset" -ForegroundColor Red
            exit 1
        }
    }
}

Write-Host "`nAll presets generated successfully!" -ForegroundColor Green
Write-Host "Output: public/blackhole/[prograde,retrograde,schwarzschild]/[face-on,30deg,60deg,edge-on]/`n" -ForegroundColor Cyan

