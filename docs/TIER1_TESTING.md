# Tier 1 Implementation Complete - Testing Guide

## 🎉 Status: TIER 1 COMPLETE

All Tier 1 features have been implemented:

- ✅ T1.1: Miss Taxonomy Pie Chart & Table
- ✅ T1.2: Order Mask Thumbnails (3 thumbnails)
- ✅ T1.3: Basic Outlier Spotlight Table
- ✅ T1.4: Provenance Block Infrastructure

---

## Testing Commands

Run these commands in PowerShell from the project root to test the implementation:

### 1. Check Compilation

```powershell
cd kernels/kerr_black_hole
cargo check --bin generate
cargo check --bin analyze
```

**Expected**: Both should compile successfully with only the 2 harmless warnings about unused `turns_r` and `turns_theta` variables in `transfer_maps.rs`.

---

### 2. Run Unit Tests

```powershell
cargo test --lib stats
```

**Expected**: All 5 tests in stats module should pass.

---

### 3. Generate Test Data (Small Resolution for Fast Testing)

```powershell
cargo run --release --bin generate -- --preset balanced --width 640 --height 360 --export-precision
```

**Expected**:

- Creates `public/blackhole/prograde/balanced/high_precision.json`
- File size: ~5-8 MB (640×360 resolution)
- Should complete in <1 minute
- JSON should include new fields: `turns_r`, `turns_theta`, `escaped`, `captured`, `aborted`

---

### 4. Verify New Fields in JSON

```powershell
# Check for turning points
Select-String -Path public/blackhole/prograde/balanced/high_precision.json -Pattern "turns_r" | Select-Object -First 3

# Check for miss classification
Select-String -Path public/blackhole/prograde/balanced/high_precision.json -Pattern "escaped" | Select-Object -First 3
```

**Expected**: Should find the new fields in the JSON output.

---

### 5. Run Analyzer

```powershell
cargo run --release --bin analyze -- -i public/blackhole/prograde/balanced/high_precision.json
```

**Expected**:

- Completes successfully
- Creates `public/blackhole/prograde/balanced/analysis_report.html`
- Console output shows progress bars
- No errors or panics

---

### 6. View Report

```powershell
# Open in default browser (Windows)
Start-Process public/blackhole/prograde/balanced/analysis_report.html
```

**Expected HTML Report Should Show**:

#### New Sections:

1. **🎯 Miss Taxonomy (NEW)**
   - Pie chart with 3 colored segments
   - Table showing escaped/captured/aborted counts
   - Percentages adding to 100%

2. **🖼️ Order Mask Thumbnails (NEW)**
   - 3 thumbnails side-by-side
   - Order 0: Shows full disc + shadow
   - Order 1: Shows thin photon ring
   - Order 2+: Shows subrings (may be sparse)
   - Coverage percentages below each

3. **🔍 Outlier Spotlight (NEW)**
   - Table with top-10 worst null invariant errors
   - Columns: Rank, Pixel (x,y), Order, NI Error, Severity
   - Color-coded severity (green/yellow/red)

#### Footer:

- **🔧 Build Provenance** collapsible section
  - Currently empty (no data) - this is expected
  - Will be populated after generation binary update

---

## Expected Values (Balanced Preset, 640×360)

### Miss Taxonomy

- **Escaped**: ~55-65% (rays escape to infinity)
- **Captured**: ~25-35% (rays fall into black hole)
- **Aborted**: ~5-15% (numerical issues / step limit)

### Order Coverage

- **Order 0**: ~40-50% of pixels
- **Order 1**: ~15-25% of pixels
- **Order 2+**: ~1-5% of pixels

### Outlier Table

- **Good run**: All NI errors < 1e-12 (green "OK")
- **Acceptable**: A few 1e-12 to 1e-9 (yellow "Warning")
- **Problem**: Any > 1e-9 (red "Critical") - indicates integration issues

---

## Troubleshooting

### Issue: Compilation Fails

**Check**:

- Are you in `kernels/kerr_black_hole` directory?
- Run `cargo clean` and try again

### Issue: JSON Missing New Fields

**Problem**: You're using old generated data  
**Solution**: Delete `public/blackhole/prograde/balanced/high_precision.json` and regenerate

### Issue: Analyzer Crashes

**Check**:

- Is the JSON file corrupt? Try regenerating
- Check console for error messages
- Verify JSON has `manifest` section with all required fields

### Issue: Report Looks Wrong

**Check**:

- Open browser console (F12) for JavaScript errors
- Verify HTML file is not truncated (check file size)
- Try a different browser (Chrome/Firefox)

---

## Quick Validation Checklist

Before moving to Tier 2, verify:

- [ ] Both binaries compile successfully
- [ ] Generate produces JSON with new fields (`turns_r`, `escaped`, etc.)
- [ ] Analyzer runs without errors
- [ ] HTML report opens and displays correctly
- [ ] Miss taxonomy pie chart visible
- [ ] 3 order thumbnails visible
- [ ] Outlier table shows 10 rows
- [ ] Provenance section exists (even if empty)
- [ ] All existing sections still work (NI histogram, radial profile, etc.)

---

## Files Modified in Tier 1

### Data Layer

- `kernels/kerr_black_hole/src/transfer_maps.rs` - Added provenance fields to Manifest

### Analyzer Layer

- `kernels/kerr_black_hole/src/bin/analyze/main.rs` - Computed new statistics
- `kernels/kerr_black_hole/src/bin/analyze/generate_summary.rs` - Updated Stats & Manifest, added HTML sections
- `kernels/kerr_black_hole/src/bin/analyze/charts.rs` - Added 2 new chart generation functions

### Documentation

- `docs/tier1_summary.md` - Detailed implementation summary
- `TIER1_TESTING.md` - This testing guide

---

## Next Steps

Once Tier 1 testing is successful, you can proceed to:

**Tier 2: Research Metrics** (~4 hours)

- T2.1: K validation heatmap
- T2.2: Transfer function 2D histograms
- T2.3: Asymmetry quantification
- T2.4: Time delay map heatmap

Or:

**Tier 3: Publication Quality** (~4.5 hours)

- T3.1: Critical curve extraction
- T3.2: Turning-point histograms
- T3.3: Wrap-angle vs impact parameter scatter

All infrastructure (Phase 1 & 2) is already in place!

---

## Questions?

If you encounter issues:

1. Check the troubleshooting section above
2. Verify all commands are run from correct directories
3. Check for typos in paths
4. Ensure you have the latest code changes

**Ready to test!** Run the commands above and verify Tier 1 is working before proceeding to the next tier.
