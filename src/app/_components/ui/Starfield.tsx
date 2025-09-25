"use client";

import { useStarfield } from "@/app/_hooks/useStarfield";

export default function Starfield() {
  const { isDarkMode, mounted } = useStarfield();

  return (
    <div 
      className={
        "starfield-container fixed inset-0 pointer-events-none z-[-1] overflow-hidden transition-opacity duration-1000 ease-in-out motion-reduce:transition-none " +
        (mounted && isDarkMode ? "opacity-100" : "opacity-0")
      }
      aria-hidden="true"
    >
      {/* Layer 1: Far stars (small, dim) */}
      <div className="starfield-layer-1 absolute inset-0" />
      
      {/* Layer 2: Medium stars */}
      <div className="starfield-layer-2 absolute inset-0" />
      
      {/* Layer 3: Close stars (large, bright) */}
      <div className="starfield-layer-3 absolute inset-0" />
      
      {/* Subtle nebula glow */}
      <div className="starfield-nebula absolute inset-0" />
    </div>
  );
}
