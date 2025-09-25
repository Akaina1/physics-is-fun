"use client";

import { useEffect, useState } from "react";

export default function Starfield() {
  const [isDarkMode, setIsDarkMode] = useState(false);
  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    // Mark as mounted to avoid hydration mismatch
    setMounted(true);
    
    // Check initial theme
    const checkTheme = () => {
      setIsDarkMode(document.documentElement.classList.contains('dark'));
    };
    
    checkTheme();
    
    // Watch for theme changes with throttling
    let timeoutId: NodeJS.Timeout;
    const observer = new MutationObserver(() => {
      clearTimeout(timeoutId);
      timeoutId = setTimeout(checkTheme, 50); // Throttle to reduce updates
    });
    
    observer.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ['class'],
    });

    return () => {
      observer.disconnect();
      clearTimeout(timeoutId);
    };
  }, []);

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
