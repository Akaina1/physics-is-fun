# Precompute all black hole transfer maps
# 3 black hole types × 5 scene presets = 15 total generations
#
# Scene presets bundle inclination, VFOV, distance, and orders for consistent look:
# - balanced: 45° incl, 45° VFOV, 75% fill, 3 orders - Hero shots & promo stills
# - context: 45° incl, 45° VFOV, 70% fill, 2 orders - Wide establishing shots
# - dramatic: 60° incl, 60° VFOV, 75% fill, 3 orders - Interstellar-ish look
# - edge-on: 75° incl, 75° VFOV, 75% fill, 3 orders - M87-style extreme view
# - detail: 45° incl, 45° VFOV, 85% fill, 3 orders - Close-up for texture tests

param(
    [int]$Width = 1920,
    [int]$Height = 1080,
    [switch]$ExportPrecision = $true,
    [switch]$QuickTest             # Use 720p for quick tests
)

$ErrorActionPreference = "Stop"

# Quick test mode: lower resolution
if ($QuickTest) {
    $Width = 1280
    $Height = 720
    Write-Host "Quick test mode: ${Width}x${Height}" -ForegroundColor Yellow
}

# Scene presets - each bundles all camera parameters
$presets = @("balanced", "context", "dramatic", "edge-on", "detail")

$bhTypes = @(
    @{name="Prograde Kerr"; type="prograde"},
    @{name="Retrograde Kerr"; type="retrograde"},
    @{name="Schwarzschild"; type="schwarzschild"}
)

Write-Host "`nKerr Black Hole Transfer Map Generator" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Resolution: ${Width}x${Height}" -ForegroundColor Cyan
Write-Host "Export Precision: $ExportPrecision" -ForegroundColor Cyan
Write-Host "========================================`n" -ForegroundColor Cyan

$totalConfigs = $bhTypes.Count * $presets.Count
$currentConfig = 0
$startTime = Get-Date

foreach ($bh in $bhTypes) {
    Write-Host "`n========================================" -ForegroundColor Yellow
    Write-Host "  $($bh.name)" -ForegroundColor Yellow
    Write-Host "========================================`n" -ForegroundColor Yellow
    
    foreach ($preset in $presets) {
        $currentConfig++
        
        Write-Host "[$currentConfig/$totalConfigs] $preset..." -ForegroundColor Green
        
        # Build cargo arguments (scene presets bundle orders automatically)
        $cargoArgs = @(
            "run", "--release", "--bin", "generate", "-p", "kerr_black_hole", "--",
            "--preset", $preset,
            "--black-hole-type", $bh.type,
            "--width", $Width,
            "--height", $Height
        )
        
        if ($ExportPrecision) {
            $cargoArgs += "--export-precision"
        }
        
        # Run cargo command (temporarily allow stderr without stopping)
        $prevErrorAction = $ErrorActionPreference
        $ErrorActionPreference = "Continue"
        
        & cargo $cargoArgs
        
        $ErrorActionPreference = $prevErrorAction
        
        if ($LASTEXITCODE -ne 0) {
            Write-Host "Error generating $($bh.name) - $preset" -ForegroundColor Red
            exit 1
        }
        
        Write-Host ""
    }
}

$endTime = Get-Date
$duration = $endTime - $startTime
$avgTime = [math]::Round($duration.TotalSeconds / $totalConfigs, 1)

Write-Host "`n========================================" -ForegroundColor Green
Write-Host "✓ All presets generated successfully!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
Write-Host "Total time: $($duration.ToString('mm\:ss'))" -ForegroundColor Cyan
Write-Host "Average per preset: ${avgTime}s" -ForegroundColor Cyan
Write-Host "Output: public/blackhole/[type]/[preset]/`n" -ForegroundColor Cyan

