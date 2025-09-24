use wasm_bindgen::prelude::*;
use wasm_bindgen::Clamped;

/// Safe sinc: sin(x)/x with x→0 limit = 1
#[inline]
fn sinc(x: f64) -> f64 {
    if x.abs() < 1e-12 { 1.0 } else { x.sin() / x }
}

/// Render Fraunhofer double-slit intensity to an RGBA buffer.
/// Returns a Uint8ClampedArray (via wasm_bindgen) of length width*height*4.
#[wasm_bindgen]
pub fn render_fraunhofer_rgba(
    width: u32,
    height: u32,
    a: f64,        // slit width (m)
    d: f64,        // slit separation (m)
    lambda: f64,   // wavelength (m)
    l: f64,        // screen distance (m)
    scale: f64,    // half-width of physical x-domain mapped to NDC [-1,1] (m)
    gamma: f32     // display gamma; <=1.0 brightens fringes (e.g., 0.6)
) -> Clamped<Vec<u8>> {
    let w = width as usize;
    let h = height as usize;
    let mut out = vec![0u8; w * h * 4];

    // Constants
    let pi = std::f64::consts::PI;

    for yi in 0..h {
        for xi in 0..w {
            // Map pixel to NDC [-1,1] horizontally
            let ndc_x = (xi as f64 / (w as f64 - 1.0)) * 2.0 - 1.0;

            // Physical x (metres) on the screen plane
            let x = ndc_x * scale;

            // Analytic Fraunhofer intensity:
            // I(x) ∝ cos^2(π d x / (λ L)) * sinc^2(π a x / (λ L))
            let arg_cos  = pi * d * x / (lambda * l);
            let arg_sinc = pi * a * x / (lambda * l);

            let envelope = sinc(arg_sinc);
            let carrier  = arg_cos.cos();

            // Bound is in [0,1]
            let mut i_val = (carrier * carrier) * (envelope * envelope);

            // Apply simple display gamma
            let gamma_f64 = gamma as f64;
            if gamma_f64 > 0.0 {
                i_val = i_val.clamp(0.0, 1.0).powf(gamma_f64);
            } else {
                i_val = i_val.clamp(0.0, 1.0);
            }

            let g = (i_val * 255.0).round() as u8;
            let idx = (yi * w + xi) * 4;

            // Greyscale RGBA; let the page/container theme surround it
            out[idx + 0] = g;
            out[idx + 1] = g;
            out[idx + 2] = g;
            out[idx + 3] = 255;
        }
    }

    Clamped(out)
}
