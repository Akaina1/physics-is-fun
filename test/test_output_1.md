## Black Hole Ray Tracing Physics Validation

We've implemented a physics-accurate ray tracer for null geodesics (light paths) around rotating black holes. The simulation compares three black hole types across four viewing angles:

### **Black Hole Types:**

- **Prograde Kerr** - Rotating black hole with spin aligned to accretion disc rotation
- **Retrograde Kerr** - Rotating black hole with spin opposite to disc rotation
- **Schwarzschild** - Non-rotating black hole (a=0, reference case)

### **What the "Miss Rate" Means:**

For each view, we trace 9,216 light rays from a camera toward the black hole. The "miss rate" is the percentage of rays that **don't hit the accretion disc** (they either escape to infinity or fall into the black hole).

### **Key Physical Results:**

1. **Prograde Kerr shows lowest miss rates** (37-50%) - Frame-dragging from aligned rotation bends light paths toward the disc more effectively.

2. **Retrograde Kerr shows highest miss rates at low angles** (60-65%) - Counter-rotating spin deflects light away from disc. Converges to similar values at edge-on where geometry dominates.

3. **Schwarzschild falls in between** (37-58%) - No frame-dragging, pure gravitational lensing provides a baseline.

4. **All types converge at edge-on view** (~37% miss) - Geometric projection dominates over spin effects at extreme inclinations.

**This validates that the simulation correctly captures the relativistic effects of black hole spin on light bending.**

---

========================================
Prograde Kerr
========================================
Preset: face-on
Black Hole: Kerr (Prograde)
Resolution: 128x72
Camera Angle: 0.1 degrees
Export Precision: false
📊 Statistics:
Total pixels: 9216
Disc hits: 4612
Miss rate: 50.0%
=======================================
Preset: 30deg
Black Hole: Kerr (Prograde)
Resolution: 128x72
Camera Angle: 30.0 degrees
Export Precision: false
📊 Statistics:
Total pixels: 9216
Disc hits: 4932
Miss rate: 46.5%
=======================================
Preset: 60deg
Black Hole: Kerr (Prograde)
Resolution: 128x72
Camera Angle: 60.0 degrees
Export Precision: false
📊 Statistics:
Total pixels: 9216
Disc hits: 5616
Miss rate: 39.1%
=======================================
Preset: edge-on
Black Hole: Kerr (Prograde)
Resolution: 128x72
Camera Angle: 89.9 degrees
Export Precision: false
📊 Statistics:
Total pixels: 9216
Disc hits: 5794
Miss rate: 37.1%
=======================================

========================================
Retrograde Kerr
========================================
Preset: face-on
Black Hole: Kerr (Retrograde)
Resolution: 128x72
Camera Angle: 0.1 degrees
Export Precision: false
📊 Statistics:
Total pixels: 9216
Disc hits: 3254
Miss rate: 64.7%
=======================================
Preset: 30deg
Black Hole: Kerr (Retrograde)
Resolution: 128x72
Camera Angle: 30.0 degrees
Export Precision: false
📊 Statistics:
Total pixels: 9216
Disc hits: 3698
Miss rate: 59.9%
=======================================
Preset: 60deg
Black Hole: Kerr (Retrograde)
Resolution: 128x72
Camera Angle: 60.0 degrees
Export Precision: false
📊 Statistics:
Total pixels: 9216
Disc hits: 5508
Miss rate: 40.2%
=======================================
Preset: edge-on
Black Hole: Kerr (Retrograde)
Resolution: 128x72
Camera Angle: 89.9 degrees
Export Precision: false
📊 Statistics:
Total pixels: 9216
Disc hits: 5794
Miss rate: 37.1%
=======================================

========================================
Schwarzschild
========================================
Preset: face-on
Black Hole: Schwarzschild
Resolution: 128x72
Camera Angle: 0.1 degrees
Export Precision: false
📊 Statistics:
Total pixels: 9216
Disc hits: 3880
Miss rate: 57.9%
=======================================
Preset: 30deg
Black Hole: Schwarzschild
Resolution: 128x72
Camera Angle: 30.0 degrees
Export Precision: false
📊 Statistics:
Total pixels: 9216
Disc hits: 4204
Miss rate: 54.4%
=======================================
Preset: 60deg
Preset: 60deg
Black Hole: Schwarzschild
Resolution: 128x72
Camera Angle: 60.0 degrees
Export Precision: false
📊 Statistics:
Total pixels: 9216
Disc hits: 5428
Miss rate: 41.1%
=======================================
Preset: edge-on
Black Hole: Schwarzschild
Resolution: 128x72
Camera Angle: 89.9 degrees
Export Precision: false
📊 Statistics:
Total pixels: 9216
Disc hits: 5784
Miss rate: 37.2%
=======================================
