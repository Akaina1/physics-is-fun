"use client";

import { useStarfield } from "@/app/_hooks/useStarfield";

export default function Starfield() {
  const { isDarkMode, mounted } = useStarfield();

  // Don't render anything until mounted to avoid hydration mismatch
  if (!mounted) {
    return (
      <div 
        className="starfield-container fixed inset-0 pointer-events-none z-[-1] overflow-hidden opacity-0"
        aria-hidden="true"
      />
    );
  }

  return (
    <div 
      className={`starfield-container fixed inset-0 pointer-events-none z-[-1] overflow-hidden transition-opacity duration-400 ease-out ${
        isDarkMode ? 'opacity-100' : 'opacity-0'
      }`}
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
