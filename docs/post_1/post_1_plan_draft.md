Project Overview - Kerr Black Hole Renderer:

# What this post is about

- **Goal:** Build and explain a **scientifically faithful visual of a Kerr (rotating) black hole** with a thin accretion disc—linking **math → physics → code** so readers see how general relativity turns into pixels.
- **Audience:** Curious readers, programmers, and scientists. We’ll use plain-language intuition alongside correct equations (with LaTeX), and show how implementation choices preserve physical accuracy.

# What we’re building (deliverable)

- A **browser-based renderer** that displays a Kerr black hole and disc at **several preset viewing angles** (e.g., 0°, 30°, 60°, 80°) to answer:
  **“How does perspective change the visual appearance of a black hole?”**
- The image animates the **disc’s rotation** (physically with (\Omega(r))) and includes **primary** (and optionally **secondary**) lensed images, a **shadow**, and a visible **photon/lensing ring**.

# Accuracy-first design (key decisions)

- **Double precision for physics:** All geodesic and redshift-critical math runs in **Rust (f64)** to avoid the precision pitfalls near the photon ring and horizon.
- **GPU only shades:** The browser’s GPU (WebGL2/WebGPU) does lightweight work—texture lookups, (g)-factor, and tone mapping—in **f32**, which is safe once the physics is precomputed in f64.
- **Fixed perspectives:** We precompute a small set of camera inclinations to make the physics differences obvious without the complexity of free-look.

# Implementation, end to end

- **Physics core (Rust, f64):**
  - Integrate **null geodesics in Kerr** (Kerr–Schild recommended), detect disc intersections vs capture, and compute conserved quantities ((-k*t, k*\phi)).
  - Model a **thin Novikov–Thorne-like disc** with emissivity (F(r)); inner edge at **ISCO**, spin-dependent.
  - Output **per-pixel transfer maps**:
    - **T1 (RGBA16F):** ((r, \sin\phi_0, \cos\phi_0, \text{mask}))
    - **T2 (RGBA32F):** (({-k*t}, k*\phi, \text{order}, \text{pad}))
    - **Flux LUT (R16F/R32F 1D):** emissivity profile vs (r)

- **Build-time precompute (Rust CLI):**
  - Generate T1/T2/Flux + a `manifest.json` for each preset angle and resolution (e.g., 1280×720).
  - Assets are written under `public/blackhole/<preset>/` and served statically (instant load, mobile-friendly).

- **Runtime viewer (Next.js 15 / React 19):**
  - Minimal **WebGL2** (or WebGPU) full-screen pass (using `regl` or raw WebGL).
  - Fragment shader computes per-pixel brightness with
    [
    g = \frac{-k_t}{(-k_t)u^t + k_\phi u^\phi},\quad I_{\text{obs}} = g^3 I_{\text{em}}(r),
    ]
    advancing (\phi(t) = \phi_0 + \Omega(r)t) for animation.
  - Simple UI to switch **presets**; short captions point out what to notice (Doppler bright side, secondary image, shadow offset with spin).

# Why this works (and what readers will see)

- **Perspective matters:** Inclination increases **Doppler beaming** asymmetry, reveals the **far side** of the disc as a lensed arc, and subtly changes the **shadow’s shape/offset** in Kerr.
- **Physics is traceable:** Every pixel’s intensity can be tied back to geodesic constants and (g^3), making the render a didactic bridge between equations and imagery.

# Validation plan (scientific hygiene)

- **Schwarzschild checks:** shadow diameter (\approx \sqrt{27},M), photon orbit at (3M), ISCO at (6M).
- **Numerical sanity:** invariants/step-error logs during geodesics; clamp and report outliers.
- **Cross-checks:** spot-compare a few rays against a reference (e.g., GEOKERR/GYOTO) for hit radius and (g).

# Scope & extensions (kept in reserve)

- Start with **primary** + optional **secondary** images; add higher orders later if needed.
- Optional toggles: **Schwarzschild vs Kerr**, **exposure**, and **guide overlays** (approaching/receding labels; shadow boundary hint).
- Future work: free-look via on-demand precompute or a WebGPU FP64 backend if/when broadly available.

This plan keeps the blog post **fast to load**, **rigorous**, and **teachable**—a strong foundation to showcase both the physics and the engineering craft.

---

General Plan (not aligned to the current project structure):

Here’s a crisp, end-to-end plan to ship a **Kerr black hole** simulation in your Next.js project using **build-time physics + runtime GPU shading**. Tiny snippets only, as requested.

---

# 0) Outcomes & guardrails

- **Accuracy:** f64 geodesics (Rust) → packed float textures → GPU does (g^3) shading only.
- **Scope:** 3–4 preset inclinations (e.g., 0°, 30°, 60°, 80°), 1–2 image orders.
- **Delivery:** Next.js page/MDX block; no runtime heavy compute; instant load.

---

# 1) Repo layout (monorepo-friendly)

```
/crates/bh-core     # pure Rust (f64 geodesics, packing)
/crates/bh-cli      # build-time generator (calls bh-core)
/app/_lib/blackhole # TS types, loaders, small helpers
/public/blackhole   # generated assets: /<preset>/{t1.bin,t2.bin,flux.bin,manifest.json}
```

---

# 2) Physics core (Rust, f64) — `bh-core`

**Responsibilities**

- Kerr geodesics (prefer **Kerr–Schild**), hit test with thin equatorial disc, compute ((-k*t, k*\phi)), store ((r,\phi_0)).
- Support **orders 0–1** (primary, secondary).
- Pack per-pixel outputs to **T1/T2**.

**Key API (example signatures)**

```rust
pub struct Camera { pub fov: f64, pub inc: f64, pub yaw: f64, pub pitch: f64, pub dist: f64 }
pub struct BH { pub mass: f64, pub spin: f64 } // a in [0,M], use geometric units
pub struct Frame { pub width: u32, pub height: u32, pub orders: u8 }

pub fn render_maps(bh: &BH, cam: &Camera, fr: &Frame) -> Maps;
pub struct Maps {
  pub t1_rgba16f: Vec<u16>, // (r, sinφ0, cosφ0, mask)
  pub t2_rgba32f: Vec<f32>, // (-k_t, k_φ, order, pad)
  pub flux_r32f:  Vec<f32>, // 1D emissivity/flux LUT
  pub meta: Meta
}
```

**Notes**

- Use adaptive RK (e.g., DP8(7)) or semi-analytic integrals; detect turning points; stop at horizon or disc.
- Validate invariants each step; clamp edge cases before packing.

---

# 3) Build-time generator — `bh-cli`

**CLI behaviour**

- Accept `--preset <name> --width --height --orders --spin --inc --fov`.
- Write `t1_rgba16f.bin`, `t2_rgba32f.bin`, `flux_r32f.bin`, `manifest.json`.

**Snippet**

```rust
fn main() {
  let p = parse_args();
  let maps = bh_core::render_maps(&p.bh, &p.camera, &p.frame);
  write_bin("t1_rgba16f.bin", &maps.t1_rgba16f);
  write_bin("t2_rgba32f.bin", &maps.t2_rgba32f);
  write_bin("flux_r32f.bin",  &maps.flux_r32f);
  write_json("manifest.json", &maps.meta);
}
```

**NPM scripts**

```json
{
  "scripts": {
    "prebuild:blackhole": "node scripts/gen-maps.mjs",
    "build": "pnpm prebuild:blackhole && next build"
  }
}
```

**`scripts/gen-maps.mjs` (sketch)**

```js
await run('./target/release/bh-cli', [
  '--preset',
  '60deg',
  '--width',
  '1280',
  '--height',
  '720',
  '--orders',
  '2',
  '--spin',
  '0.9',
  '--inc',
  '60',
  '--fov',
  '35',
  '--out',
  'public/blackhole/60deg',
]);
```

---

# 4) Asset format (stable contract)

- **T1: RGBA16F** = `(r, sinφ0, cosφ0, mask)`
- **T2: RGBA32F** = `(-k_t, k_φ, order, pad)`
- **Flux: R32F** (length N)
- **manifest.json**:

```json
{
  "width": 1280,
  "height": 720,
  "orders": 2,
  "rIn": 1.0,
  "rOut": 50.0,
  "spin": 0.9,
  "t1Url": "/blackhole/60deg/t1_rgba16f.bin",
  "t2Url": "/blackhole/60deg/t2_rgba32f.bin",
  "fluxUrl": "/blackhole/60deg/flux_r32f.bin"
}
```

---

# 5) Runtime viewer (WebGL2 via **regl**)

**Tiny TS types**

```ts
export type Manifest = { width:number;height:number;orders:number;rIn:number;rOut:number;spin:number;
  t1Url:string;t2Url:string;fluxUrl:string };
```

**Loaders (sketch)**

```ts
const ab = (u:string) => fetch(u).then(r=>r.arrayBuffer());
const loadPreset = async (m:Manifest) => ({
  t1: new Uint16Array(await ab(m.t1Url)),
  t2: new Float32Array(await ab(m.t2Url)),
  flux: new Float32Array(await ab(m.fluxUrl))
});
```

**Shader (core few lines)**

```glsl
// g = (-k_t) / [ (-k_t) u^t + k_φ u^φ ]
float denom = kE * ut + kphi * uphi;
float g = kE / max(denom, 1e-6);
float I = jr * g * g * g;       // I_obs = g^3 * I_em
```

**Kerr circular orbit helper (approx. form)**

```glsl
void kerrOrbit(float r, float a, out float Omega, out float ut, out float uphi){
  Omega = 1.0 / (pow(r,1.5) + a);
  float A = 1.0 - 3.0/r + 2.0*a*Omega;
  ut = inversesqrt(max(A,1e-6));
  uphi = Omega * ut;
}
```

---

# 6) MDX integration (presets UX)

- Add a small MDX component with **tabs/thumbnails**: _Face-on_, _30°_, _60°_, _Edge-on_.
- On tab switch: fetch that preset’s manifest → create textures → render.
- Each preset shows a **caption**: what to notice (Doppler bright side, secondary image, shadow offset).

---

# 7) Validation (must-do)

- **Schwarzschild mode (a=0):** confirm shadow diameter (\approx \sqrt{27},M); photon orbit at (3M); ISCO at (6M).
- **Energy invariants:** check (|H|<\epsilon) per ray; log max error.
- **Spot-check vs reference:** a few rays vs GEOKERR/GYOTO outputs (radius hit, (g) value).
- **Visual QA:** compare primary vs secondary image morphology at (,i=60^\circ).

---

# 8) Performance & quality knobs

- Default **1280×720**, orders=1 for mobile; orders=2 for desktop.
- If banding near the ring: keep **T2 as RGBA32F** (do not downscale).
- Simple tone-map: `color = (I*exposure)/(1.0 + I*exposure)`; expose an **Exposure** slider.
- We will also test 1920x1080 rendering for the sim to test performance.

---

# 9) CI/CD (Vercel)

- Build step runs `prebuild:blackhole` to generate assets.
- Upload artefacts in `public/blackhole/**`; Vercel serves via CDN.
- Optional: nightly job regenerates assets if **modelVersion** changes.

---
