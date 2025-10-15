# Precompute all black hole transfer maps
# 3 black hole types × 4 viewing presets = 12 total generations

param(
    [int]$Width = 1920,
    [int]$Height = 1080,
    [int]$MaxOrders = 3,           # Default to 3 orders for good visual quality
    [switch]$ExportPrecision = $true,
    [switch]$QuickTest             # Use 720p for quick tests
)

$ErrorActionPreference = "Stop"

# Quick test mode: lower resolution
if ($QuickTest) {
    $Width = 1280
    $Height = 720
    $MaxOrders = 2
    Write-Host "Quick test mode: ${Width}x${Height}, orders=$MaxOrders" -ForegroundColor Yellow
}

# Configuration
$presets = @("face-on", "30deg", "60deg", "edge-on")

# You can override orders per preset if needed
$presetOrders = @{
    "face-on" = $MaxOrders
    "30deg" = $MaxOrders
    "60deg" = $MaxOrders
    "edge-on" = $MaxOrders
}

$bhTypes = @(
    @{name="Prograde Kerr"; type="prograde"},
    @{name="Retrograde Kerr"; type="retrograde"},
    @{name="Schwarzschild"; type="schwarzschild"}
)

Write-Host "`nKerr Black Hole Transfer Map Generator" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Resolution: ${Width}x${Height}" -ForegroundColor Cyan
Write-Host "Max Orders: $MaxOrders" -ForegroundColor Cyan
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
        $orders = $presetOrders[$preset]
        
        Write-Host "[$currentConfig/$totalConfigs] $preset (orders: $orders)..." -ForegroundColor Green
        
        # Build cargo arguments
        $cargoArgs = @(
            "run", "--release", "--bin", "generate", "-p", "kerr_black_hole", "--",
            "--preset", $preset,
            "--black-hole-type", $bh.type,
            "--width", $Width,
            "--height", $Height,
            "--max-orders", $orders
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

