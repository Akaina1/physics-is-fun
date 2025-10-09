# Kerr Black Hole Renderer - Implementation Plan

## Project Structure Alignment

This plan adapts the general design document to the existing `physics-is-fun` Next.js blog structure:

- **Rust physics:** `kernels/kerr_black_hole/` (follows existing `kernels/double_slit/` pattern)
- **Build CLI:** `kernels/kerr_black_hole/src/bin/generate.rs` (standalone Rust binary)
- **Generated assets:** `public/blackhole/<preset>/` (served statically via Next.js)
- **TypeScript utilities:** `src/app/_lib/blackhole/` (types, loaders, helpers)
- **React component:** `src/app/_components/sims/KerrBlackHole/KerrBlackHole.tsx`
- **Build orchestration:** `scripts/gen-blackhole-assets.mjs` (calls Rust CLI, similar to `build-wasm.mjs`)
- **MDX integration:** Expose component in `src/app/[slug]/page.tsx` for use in `content/posts/black-hole.mdx`

## Phase 1: Rust Physics Core (f64 Geodesics)

### 1.1 Create kernel structure

Create `kernels/kerr_black_hole/` with:

- `Cargo.toml` - library + binary crate (no wasm-bindgen, pure Rust CLI)
- `src/lib.rs` - physics core (geodesic integration, disc model, packing)
- `src/bin/generate.rs` - CLI that calls lib and writes binary assets

### 1.2 Implement geodesic integration

In `src/lib.rs`, implement:

**Core types:**

```rust
pub struct BlackHole {
    pub mass: f64,        // M in geometric units
    pub spin: f64,        // a/M ∈ [0,1), dimensionless
}

pub struct Camera {
    pub distance: f64,    // observer distance from BH (in M)
    pub inclination: f64, // viewing angle in degrees
    pub fov: f64,         // field of view in degrees
}

pub struct RenderConfig {
    pub width: u32,
    pub height: u32,
    pub max_orders: u8,   // 1=primary only, 2=primary+secondary
}
```

**Kerr geodesics (Kerr-Schild coordinates):**

- Implement adaptive RK integrator (Dormand-Prince 8(7) or similar)
- Track conserved quantities: energy \( E = -k*t \), angular momentum \( L_z = k*\phi \)
- Carter constant \( Q \) for full Kerr geodesics
- Hit-test with equatorial thin disc (z=0 plane, \( r*{ISCO} < r < r*{out} \))
- Detect horizon crossing or photon escape

**Validation invariants:**

- Check \( g\_{\mu\nu} k^\mu k^\nu = 0 \) (null geodesic)
- Log maximum error per ray for debugging

### 1.3 Disc model and emissivity

**Thin Novikov-Thorne disc:**

- ISCO calculation for Kerr: \( r\_{ISCO}(a) \) (spin-dependent)
- Emissivity profile \( F(r) \propto r^{-3} (1 - \sqrt{r\_{ISCO}/r}) \) or similar
- Generate 1D flux LUT: sample \( F(r) \) from \( r*{in} \) to \( r*{out} \) (e.g., 256 samples)

### 1.4 Asset packing

**Per-pixel transfer maps:**

```rust
pub struct TransferMaps {
    pub t1_rgba16f: Vec<u16>,  // (r, sin φ₀, cos φ₀, mask)
    pub t2_rgba32f: Vec<f32>,  // (-k_t, k_φ, order, pad)
    pub flux_r32f: Vec<f32>,   // 1D emissivity LUT
    pub manifest: Manifest,
}
```

**Packing details:**

- T1: pack f64 → f16 using half-precision helpers (or simple range mapping)
- T2: keep as f32 to avoid banding near photon ring
- Mask channel: 1.0 = hit disc, 0.0 = captured/escaped

**Manifest JSON:**

```rust
pub struct Manifest {
    pub width: u32,
    pub height: u32,
    pub preset: String,
    pub inclination: f64,
    pub spin: f64,
    pub orders: u8,
    pub r_in: f64,
    pub r_out: f64,
    pub t1_url: String,
    pub t2_url: String,
    pub flux_url: String,
}
```

## Phase 2: Build-Time CLI Generator

### 2.1 CLI binary (`src/bin/generate.rs`)

Accept arguments:

```
--preset <name>      # e.g., "face-on", "30deg", "60deg", "edge-on"
--width <u32>        # e.g., 1280 or 1920
--height <u32>       # e.g., 720 or 1080
--inclination <f64>  # degrees, e.g., 0, 30, 60, 80
--spin <f64>         # a/M, e.g., 0.9
--orders <u8>        # 1 or 2
--out <path>         # output directory
```

**Behavior:**

- Call `kerr_black_hole::render_maps(bh, camera, config)`
- Write `t1_rgba16f.bin`, `t2_rgba32f.bin`, `flux_r32f.bin`, `manifest.json` to `--out` directory
- Print progress/stats (time, max invariant error, etc.)

### 2.2 Node.js orchestration script

Create `scripts/gen-blackhole-assets.mjs`:

```js
import { spawn } from 'node:child_process';
import { mkdirSync } from 'node:fs';
import { join } from 'node:path';

const presets = [
  { name: 'face-on', inc: 0, spin: 0.9, width: 1280, height: 720, orders: 2 },
  { name: '30deg', inc: 30, spin: 0.9, width: 1280, height: 720, orders: 2 },
  { name: '60deg', inc: 60, spin: 0.9, width: 1280, height: 720, orders: 2 },
  { name: 'edge-on', inc: 80, spin: 0.9, width: 1280, height: 720, orders: 2 },
  // Optional: 1920x1080 variants for performance testing
];

for (const p of presets) {
  const outDir = join('public', 'blackhole', p.name);
  mkdirSync(outDir, { recursive: true });

  await run(
    'cargo',
    [
      'run',
      '--release',
      '--bin',
      'generate',
      '--',
      '--preset',
      p.name,
      '--width',
      String(p.width),
      '--height',
      String(p.height),
      '--inclination',
      String(p.inc),
      '--spin',
      String(p.spin),
      '--orders',
      String(p.orders),
      '--out',
      outDir,
    ],
    { cwd: 'kernels/kerr_black_hole' }
  );
}
```

### 2.3 Package.json integration

Update `package.json`:

```json
{
  "scripts": {
    "blackhole:generate": "node scripts/gen-blackhole-assets.mjs",
    "prebuild": "pnpm wasm:build && pnpm blackhole:generate",
    "build": "next build --turbopack"
  }
}
```

Update `scripts/vercel-build.sh` to ensure Rust CLI builds run before Next.js build.

## Phase 3: Runtime WebGL2 Renderer (regl)

### 3.1 TypeScript types and loaders

Create `src/app/_lib/blackhole/types.ts`:

```typescript
export interface Manifest {
  width: number;
  height: number;
  preset: string;
  inclination: number;
  spin: number;
  orders: number;
  rIn: number;
  rOut: number;
  t1Url: string;
  t2Url: string;
  fluxUrl: string;
}

export interface PresetAssets {
  manifest: Manifest;
  t1: Uint16Array;
  t2: Float32Array;
  flux: Float32Array;
}
```

Create `src/app/_lib/blackhole/loader.ts`:

```typescript
export async function loadPreset(presetName: string): Promise<PresetAssets> {
  const manifestUrl = `/blackhole/${presetName}/manifest.json`;
  const manifest: Manifest = await fetch(manifestUrl).then(r => r.json());

  const [t1, t2, flux] = await Promise.all([
    fetch(manifest.t1Url).then(r => r.arrayBuffer()).then(b => new Uint16Array(b)),
    fetch(manifest.t2Url).then(r => r.arrayBuffer()).then(b => new Float32Array(b)),
    fetch(manifest.fluxUrl).then(r => r.arrayBuffer()).then(b => new Float32Array(b)),
  ]);

  return { manifest, t1, t2, flux };
}
```

### 3.2 regl shader implementation

Create `src/app/_lib/blackhole/shader.ts` with GLSL fragment shader:

**Key shader computations:**

```glsl
// Decode transfer maps
vec4 t1 = texture2D(uT1, vUV); // (r, sin φ₀, cos φ₀, mask)
vec4 t2 = texture2D(uT2, vUV); // (-k_t, k_φ, order, pad)

float r = t1.r;
float sinPhi0 = t1.g;
float cosPhi0 = t1.b;
float mask = t1.a;

float kE = t2.r;   // -k_t
float kPhi = t2.g; // k_φ

// Animate disc rotation: φ(t) = φ₀ + Ω(r) * t
float omega = kerrOmega(r, uSpin);
float phi = atan(sinPhi0, cosPhi0) + omega * uTime;

// Circular orbit 4-velocity (approximate)
float ut, uphi;
kerrOrbit(r, uSpin, omega, ut, uphi);

// Redshift factor g = (-k_t) / [(-k_t) u^t + k_φ u^φ]
float denom = kE * ut + kPhi * uphi;
float g = kE / max(denom, 1e-6);

// Doppler + gravitational redshift
float intensity = lookupFlux(r) * g * g * g;

// Tone mapping
float exposure = uExposure;
float mapped = (intensity * exposure) / (1.0 + intensity * exposure);

gl_FragColor = vec4(vec3(mapped), mask);
```

**Helper functions:**

```glsl
float kerrOmega(float r, float a) {
  return 1.0 / (pow(r, 1.5) + a);
}

void kerrOrbit(float r, float a, float omega, out float ut, out float uphi) {
  float A = 1.0 - 3.0 / r + 2.0 * a * omega;
  ut = inversesqrt(max(A, 1e-6));
  uphi = omega * ut;
}
```

### 3.3 React component

Create `src/app/_components/sims/KerrBlackHole/KerrBlackHole.tsx`:

**Component structure:**

- State: current preset name, exposure slider value, animation time
- useEffect: load preset assets on mount/preset change
- useEffect: initialize regl, create textures (RGBA16F for T1, RGBA32F for T2, R32F for flux)
- useEffect: animation loop with requestAnimationFrame
- UI: preset selector tabs/buttons, exposure slider, captions explaining what to notice

**Pattern similar to `DoubleSlitFraunhofer.tsx`:**

- Canvas element for regl rendering
- Controls for preset switching and exposure adjustment
- Responsive sizing with aspect ratio preservation

## Phase 4: MDX Integration

### 4.1 Expose component in page router

Update `src/app/[slug]/page.tsx`:

```typescript
import KerrBlackHole from '@/app/_components/sims/KerrBlackHole/KerrBlackHole';

const customComponents = {
  DoubleSlitFraunhofer,
  KerrBlackHole, // <-- add this
};
```

### 4.2 Use in MDX content

In `content/posts/black-hole.mdx`:

```mdx
# Visualizing a Rotating Black Hole

...physics explanation...

<KerrBlackHole />

...more content...
```

## Phase 5: Validation & Testing

### 5.1 Schwarzschild validation (a=0 test)

Even though we're focusing on Kerr, temporarily test with a=0 to validate:

- Photon orbit at r=3M
- ISCO at r=6M
- Shadow diameter ≈ √27 M ≈ 5.2M

### 5.2 Numerical accuracy checks

- Log maximum invariant error during geodesic integration
- Verify conserved quantities remain stable
- Check for NaN/Inf in packed outputs

### 5.3 Visual QA

- Primary image: bright approaching side (Doppler boost)
- Secondary image: faint arc above/below primary
- Shadow: offset and asymmetric for spinning BH
- Test all 4 presets at different inclinations

### 5.4 Performance testing

- Measure asset load times (should be <1s for all presets)
- Test 1920×1080 rendering performance on typical hardware
- Ensure 60fps animation on desktop, 30fps acceptable on mobile

## Phase 6: Documentation & Polish

### 6.1 Code documentation

- Document Rust physics functions with references to GR equations
- Add inline comments explaining coordinate choices and approximations
- TSDoc comments for TypeScript utilities

### 6.2 Post content

Write `content/posts/black-hole.mdx` with:

- Introduction to black holes and general relativity
- Math behind Kerr metric and geodesics (LaTeX via KaTeX)
- Explanation of what readers see in each preset
- Doppler beaming, gravitational lensing, shadow formation
- Comparison notes for future Schwarzschild implementation

### 6.3 User experience

- Add loading states while assets fetch
- Error boundaries for failed loads
- Responsive design testing (mobile, tablet, desktop)
- Accessibility: keyboard controls, ARIA labels

## Implementation Notes

**Physics accuracy priorities:**

1. f64 throughout geodesic integration (no precision loss)
2. Proper Kerr-Schild coordinates (stable near horizon)
3. Validated invariants at every integration step
4. Spin-dependent ISCO calculation
5. Accurate redshift formula with circular orbit 4-velocities

**Performance optimization:**

1. All heavy compute at build time (no runtime geodesic tracing)
2. Precomputed transfer maps = instant load
3. GPU shading only (simple texture lookups + arithmetic)
4. Assets compressed and served via CDN (Vercel)

**Scope boundaries:**

- Start with 4 presets (0°, 30°, 60°, 80°) at 1280×720, orders=2
- Kerr only (a=0.9), no Schwarzschild comparison yet
- Fixed camera positions, no free-look
- Future: add Schwarzschild sim
