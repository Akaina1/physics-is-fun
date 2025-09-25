"use client";

import { useEffect, useMemo, useRef, useState, memo } from "react";

// Small TS type for our params (in metres)
type Params = {
  a: number;       // slit width
  d: number;       // slit separation
  lambda: number;  // wavelength
  L: number;       // screen distance
  scale: number;   // half-width of physical x domain mapped to [-1,1]
  gamma: number;   // display gamma (<=1 brightens)
};

function DoubleSlitFraunhofer() {
  // Canvas size; you can make this responsive later
  const width = 960;
  const height = 540;

  // Reasonable defaults
  const [params] = useState<Params>({ // add setParams later when we have sliders
    a: 20e-6,        // 20 µm
    d: 200e-6,       // 200 µm
    lambda: 532e-9,  // 532 nm (green)
    L: 1.0,          // 1 metre
    scale: 0.02,     // +/- 2 cm across the screen plane
    gamma: 0.6,      // brighten fringes a bit
  });

  const canvasRef = useRef<HTMLCanvasElement>(null);

  // Draw function that calls WASM and paints
  const render = useMemo(() => {
    return async (p: Params) => {
      // Dynamic import of the wasm-pack JS wrapper
      // Tip: If TS complains about types, we can cast to any.
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const mod: any = await import("../../../../kernels/double_slit/pkg");

      // Some wasm-pack setups require calling the init function; others auto-init.
      // If the module exposes a default (init), call it once.
      if (typeof mod.default === "function") {
        await mod.default();
      }

      const clamped = mod.render_fraunhofer_rgba(
        width,
        height,
        p.a,
        p.d,
        p.lambda,
        p.L,
        p.scale,
        p.gamma
      );

      const canvas = canvasRef.current!;
      const ctx = canvas.getContext("2d", { alpha: true });
      if (!ctx) return;

      // clamped is a Uint8ClampedArray; create ImageData directly
      const img = new ImageData(clamped, width, height);
      ctx.putImageData(img, 0, 0);
    };
  }, [width, height]);

  useEffect(() => {
    render(params);
  }, [render, params]);

  return (
    <div className="rounded-xl border border-border bg-card p-3">
      <div className="mb-3 text-sm text-muted-foreground">
        Fraunhofer double-slit intensity (Rust→WASM → Canvas). a={formatSI(params.a)}m, d={formatSI(params.d)}m, λ={formatSI(params.lambda)}m, L={params.L}m
      </div>
      <canvas
        ref={canvasRef}
        width={width}
        height={height}
        className="w-full h-auto rounded-md bg-background"
        style={{ aspectRatio: `${width} / ${height}` }}
      />
      {/* Sliders can be added later; for now we render once with defaults */}
    </div>
  );
}

// Memoized export to prevent theme-triggered re-renders
export default memo(DoubleSlitFraunhofer);

// Tiny helper to show SI-ish numbers
function formatSI(x: number): string {
  const abs = Math.abs(x);
  if (abs >= 1) return x.toFixed(2);
  if (abs >= 1e-3) return (x * 1e3).toFixed(2) + "e-3";
  if (abs >= 1e-6) return (x * 1e6).toFixed(0) + "e-6";
  if (abs >= 1e-9) return (x * 1e9).toFixed(0) + "e-9";
  return x.toExponential(1);
}
